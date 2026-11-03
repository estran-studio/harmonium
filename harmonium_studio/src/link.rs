//! Les liens d'ouverture du studio : `harmonium://open?profile=…&seed=…&bar=…`.
//!
//! Un lien décrit un état à entendre : un profil (distribué, ou dossier quelconque), une graine,
//! une mesure, une boucle, un tempo, un swing, des solos et des sourdines. [`OpenRequest`] le lit
//! ([`OpenRequest::parse`]), l'écrit ([`OpenRequest::to_link`]) et le traduit en intentions de
//! l'écran Live ([`OpenRequest::intents`]) ; [`Opening`] est une demande dont le profil est déjà
//! lu et validé.
//!
//! Les noms des paramètres sont ceux du REPL et du CLI (`--seed`, `--swing`, `--solo-track`,
//! `seek`, `loop`, `bpm`). Le module ne connaît ni egui ni la plate-forme : il vaut pour le canal
//! de commande natif, pour la ligne de commande et pour la requête d'URL du web.

use std::{fmt::Write as _, path::PathBuf};

use harmonium_core::{EngineCommand, StyleProfile, timeline::TrackId};

use crate::{
    live::{Intent, TRACKS},
    reload::ProfileWatch,
};

/// Schéma des liens.
pub const SCHEME: &str = "harmonium";

/// L'action du lien (`harmonium://open`).
const ACTION: &str = "open";

/// Tempo accepté (celui de `EngineCommand::SetBpm`).
pub const BPM_RANGE: (f32, f32) = (40.0, 300.0);

/// Swing accepté (celui de `EngineCommand::SetSwing`).
pub const SWING_RANGE: (f32, f32) = (0.5, 0.75);

/// Le profil désigné par un lien.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProfileRef {
    /// Identifiant d'un profil distribué (`reel_quebecois`). Natif : le dossier `profiles/<id>`
    /// du dépôt quand il existe, sinon le profil compilé.
    Id(String),
    /// Dossier d'un profil (chemin absolu) : profil d'un worktree, fixture…
    Folder(PathBuf),
}

impl ProfileRef {
    /// Lit la valeur de `profile=` : un chemin absolu (`/…`, `C:\…`) ou un identifiant.
    pub fn parse(value: &str) -> Result<Self, String> {
        if looks_absolute(value) {
            return Ok(Self::Folder(PathBuf::from(value)));
        }
        let id_like = !value.is_empty()
            && value.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_');
        if id_like {
            Ok(Self::Id(value.to_string()))
        } else if value.contains(['/', '\\']) {
            Err(format!("profile `{value}`: a folder must be an absolute path"))
        } else {
            Err(format!(
                "profile `{value}`: expected a profile id (a-z, 0-9, _) or an absolute path"
            ))
        }
    }

    fn text(&self) -> String {
        match self {
            Self::Id(id) => id.clone(),
            Self::Folder(path) => path.display().to_string(),
        }
    }
}

/// Un chemin absolu sur n'importe quelle plate-forme : un lien écrit sous Windows s'analyse aussi
/// sous macOS, et le web ne connaît pas les dossiers.
fn looks_absolute(value: &str) -> bool {
    let bytes = value.as_bytes();
    value.starts_with('/')
        || (bytes.len() >= 3
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && matches!(bytes[2], b'\\' | b'/'))
}

/// Ce qu'un lien demande. Un champ absent garde ce qui est en cours (le profil, la graine, le
/// tempo) ou prend la valeur neutre (mesure 1, pas de boucle, aucune piste coupée) ; `play` vaut
/// `false` : le studio reste en pause.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OpenRequest {
    pub profile: Option<ProfileRef>,
    pub seed: Option<u64>,
    /// Mesure où se placer (1-based) ; `Seek` : recomposée depuis cette mesure, même graine.
    pub bar: Option<usize>,
    /// Boucle (`début-fin`, mesures 1-based, `début <= fin`).
    pub loop_range: Option<(usize, usize)>,
    /// Tempo imposé (BPM) ; absent : celui du profil.
    pub bpm: Option<f32>,
    /// Swing imposé ; absent : celui du profil.
    pub swing: Option<f32>,
    pub solo: Option<TrackId>,
    pub mute: Vec<TrackId>,
    /// Lancer la lecture. Faux par défaut : aucun son tant que l'on n'appuie pas sur lecture.
    pub play: bool,
}

/// Une piste par son nom (`TrackId::name`, sans tenir compte de la casse ; `Counter` aussi).
pub fn parse_track(name: &str) -> Result<TrackId, String> {
    let name = name.trim();
    TrackId::ALL
        .into_iter()
        .find(|track| track.name().eq_ignore_ascii_case(name))
        .or_else(|| name.eq_ignore_ascii_case("counter").then_some(TrackId::Countermelody))
        .ok_or_else(|| {
            let names: Vec<&str> = TrackId::ALL.iter().map(|track| track.name()).collect();
            format!("unknown track `{name}` (one of {})", names.join(", "))
        })
}

impl OpenRequest {
    /// Lit un lien `harmonium://open?…`.
    pub fn parse(link: &str) -> Result<Self, String> {
        let link = link.trim();
        let rest = link
            .strip_prefix(SCHEME)
            .and_then(|rest| rest.strip_prefix("://"))
            .ok_or_else(|| format!("not a {SCHEME}:// link: {link}"))?;
        let (action, query) = rest.split_once('?').unwrap_or((rest, ""));
        if action.trim_end_matches('/') != ACTION {
            return Err(format!("unknown action `{action}` (only `{ACTION}`)"));
        }
        Self::from_query(query, &[])
    }

    /// Lit une requête `a=1&b=2` (celle d'un lien, ou la requête de l'URL de la page web).
    /// `ignored` : clés qui ne sont pas à nous (`buffer` sur le web) ; toute autre clé inconnue est
    /// une erreur, pour qu'une faute de frappe ne passe pas en silence.
    pub fn from_query(query: &str, ignored: &[&str]) -> Result<Self, String> {
        let mut request = Self::default();
        let mut seen: Vec<String> = Vec::new();
        for pair in query.trim_start_matches('?').split('&').filter(|pair| !pair.is_empty()) {
            let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
            let key = percent_decode(key)?;
            if ignored.contains(&key.as_str()) {
                continue;
            }
            let value = percent_decode(value)?;
            // `mute` peut se répéter ; les autres clés une fois.
            if key != "mute" {
                if seen.contains(&key) {
                    return Err(format!("`{key}` given twice"));
                }
                seen.push(key.clone());
            }
            request.set(&key, &value).map_err(|error| format!("{key}: {error}"))?;
        }
        Ok(request)
    }

    fn set(&mut self, key: &str, value: &str) -> Result<(), String> {
        let number = |text: &str| text.trim().parse::<usize>().ok().filter(|n| *n >= 1);
        match key {
            "profile" => self.profile = Some(ProfileRef::parse(value)?),
            "seed" => {
                self.seed =
                    Some(value.trim().parse().map_err(|_| format!("invalid seed `{value}`"))?);
            }
            "bar" => {
                self.bar = Some(
                    number(value).ok_or_else(|| format!("invalid bar `{value}` (1 or more)"))?,
                );
            }
            "loop" => {
                let range = value
                    .split_once('-')
                    .and_then(|(from, to)| Some((number(from)?, number(to)?)))
                    .filter(|(from, to)| from <= to)
                    .ok_or_else(|| {
                        format!("invalid loop `{value}` (expected `start-end`, 1 ≤ start ≤ end)")
                    })?;
                self.loop_range = Some(range);
            }
            "bpm" => self.bpm = Some(ranged(value, BPM_RANGE)?),
            "swing" => self.swing = Some(ranged(value, SWING_RANGE)?),
            "solo" => self.solo = Some(parse_track(value)?),
            "mute" => {
                for name in value.split(',').filter(|name| !name.trim().is_empty()) {
                    let track = parse_track(name)?;
                    if !self.mute.contains(&track) {
                        self.mute.push(track);
                    }
                }
            }
            "play" => {
                self.play = match value.trim().to_ascii_lowercase().as_str() {
                    "1" | "true" | "yes" => true,
                    "0" | "false" | "no" => false,
                    _ => return Err(format!("invalid value `{value}` (0 or 1)")),
                };
            }
            _ => return Err("unknown parameter".to_string()),
        }
        Ok(())
    }

    /// Le lien de cette demande : `harmonium://open?profile=…&seed=…&bar=…&loop=…`. `play=1`
    /// n'est écrit que s'il est demandé (pause par défaut).
    #[must_use]
    pub fn to_link(&self) -> String {
        format!("{SCHEME}://{ACTION}{}", self.to_query())
    }

    /// La requête (`?profile=…`), sans le schéma : celle d'une URL de page web. Vide si rien.
    #[must_use]
    pub fn to_query(&self) -> String {
        let mut pairs: Vec<(&str, String)> = Vec::new();
        if let Some(profile) = &self.profile {
            pairs.push(("profile", percent_encode(&profile.text())));
        }
        if let Some(seed) = self.seed {
            pairs.push(("seed", seed.to_string()));
        }
        if let Some(bar) = self.bar {
            pairs.push(("bar", bar.to_string()));
        }
        if let Some((from, to)) = self.loop_range {
            pairs.push(("loop", format!("{from}-{to}")));
        }
        if let Some(bpm) = self.bpm {
            pairs.push(("bpm", bpm.to_string()));
        }
        if let Some(swing) = self.swing {
            pairs.push(("swing", swing.to_string()));
        }
        if let Some(solo) = self.solo {
            pairs.push(("solo", solo.name().to_string()));
        }
        if !self.mute.is_empty() {
            let names: Vec<&str> = self.mute.iter().map(|track| track.name()).collect();
            pairs.push(("mute", names.join(",")));
        }
        if self.play {
            pairs.push(("play", "1".to_string()));
        }
        let mut query = String::new();
        for (index, (key, value)) in pairs.iter().enumerate() {
            let _ = write!(query, "{}{key}={value}", if index == 0 { '?' } else { '&' });
        }
        query
    }

    /// Ce que l'écran exécute, dans l'ordre : profil, graine, tempo et swing, pistes, mesure,
    /// boucle, lecture. Le profil (`None` : celui qui joue reste) vient déjà lu. Un lien est un
    /// état entier : sans `bar`, la mesure est celle du début de la boucle, sinon la mesure 1 ;
    /// sans `loop`, pas de boucle ; les pistes qu'il ne coupe pas sont rendues.
    #[must_use]
    pub fn intents(&self, profile: Option<StyleProfile>) -> Vec<Intent> {
        let mut intents = Vec::new();
        if let Some(profile) = profile {
            intents.push(Intent::Command(EngineCommand::SetStyleProfile(Box::new(profile))));
        }
        if let Some(seed) = self.seed {
            intents.push(Intent::Command(EngineCommand::SetSeed(seed)));
        }
        if let Some(bpm) = self.bpm {
            intents.push(Intent::Command(EngineCommand::SetBpm(bpm)));
        }
        if let Some(swing) = self.swing {
            intents.push(Intent::Command(EngineCommand::SetSwing(swing)));
        }
        for track in TRACKS {
            intents.push(Intent::Mute(track, self.mute.contains(&track)));
        }
        intents.push(Intent::Solo(self.solo));
        let bar = self.bar.or(self.loop_range.map(|(from, _)| from)).unwrap_or(1);
        intents.push(Intent::Command(EngineCommand::Seek(bar)));
        intents.push(match self.loop_range {
            Some((from, to)) => Intent::Loop(from, to),
            None => Intent::Command(EngineCommand::ClearLoop),
        });
        intents.push(Intent::Play(self.play));
        intents
    }
}

fn ranged(value: &str, (low, high): (f32, f32)) -> Result<f32, String> {
    value
        .trim()
        .parse::<f32>()
        .ok()
        .filter(|number| number.is_finite() && (low..=high).contains(number))
        .ok_or_else(|| format!("invalid value `{value}` ({low} to {high})"))
}

/// Un profil lu et validé. `watch` : le dossier d'où il vient (natif), pris **avant** la lecture,
/// pour que le rechargement à chaud voie une modification faite entre la lecture et l'écran ;
/// `None` : profil compilé.
#[derive(Clone, Debug)]
pub struct ResolvedProfile {
    pub profile: StyleProfile,
    pub watch: Option<ProfileWatch>,
}

/// Une demande prête à exécuter : le profil désigné est déjà lu, donc une erreur de profil est
/// connue de celui qui ouvre le lien, avant que le studio ne change quoi que ce soit.
#[derive(Clone, Debug)]
pub struct Opening {
    pub request: OpenRequest,
    pub profile: Option<ResolvedProfile>,
}

// === Pourcentage ===

/// Caractères écrits tels quels : non réservés, plus `/`, `,` et `:` qui ne gênent pas dans une
/// requête (chemins et listes restent lisibles).
fn is_plain(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~' | b'/' | b',' | b':')
}

/// Encode en pourcentage (UTF-8).
#[must_use]
pub fn percent_encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for &byte in text.as_bytes() {
        if is_plain(byte) {
            out.push(char::from(byte));
        } else {
            let _ = write!(out, "%{byte:02X}");
        }
    }
    out
}

/// Décode le pourcentage (UTF-8) ; `+` reste un `+` (un chemin peut en contenir).
pub fn percent_decode(text: &str) -> Result<String, String> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let digits = bytes
                .get(index + 1..index + 3)
                .filter(|pair| pair.iter().all(u8::is_ascii_hexdigit))
                .and_then(|pair| std::str::from_utf8(pair).ok())
                .and_then(|pair| u8::from_str_radix(pair, 16).ok())
                .ok_or_else(|| format!("invalid %-escape in `{text}`"))?;
            out.push(digits);
            index += 3;
        } else {
            out.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(out).map_err(|_| format!("`{text}` is not valid UTF-8"))
}
