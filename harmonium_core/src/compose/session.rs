//! Habillage par graine : variantes de session, tirées une fois par graine, qui surchargent des
//! réglages globaux nommés (liste blanche) du profil (section `[session]`, optionnelle).
//!
//! Une graine ne changeait que le rythme du comping et des notes de passage : tempo, tonalité,
//! instruments, voicings, kit et forme étaient fixes par profil. Une **variante** porte un poids
//! et des surcharges ; elle se tire **une fois par graine de session**. Les variantes se
//! rangent en **groupes** : un tirage indépendant par groupe, dans l'ordre où les groupes
//! apparaissent, pour multiplier les combinaisons sans les énumérer (3 tempos × 3 tonalités × 3
//! comping × 2 basses × 2 kits × 3 voicings en 17 variantes). Deux groupes ne peuvent pas
//! régler le même champ (erreur de chargement) : l'ordre des groupes ne décide jamais.
//!
//! Surcharges : tempo (valeur ou sous-plage de `meter.min_bpm..max_bpm`), tonalité et mode (ceux
//! que le profil atteint), programmes par piste, preset Odin par piste, famille de voicings,
//! rapport de swing, registre par piste (octaves), tags de comping et de batterie (phrases et
//! relances comprises), arc nommé (`[session.arcs.<nom>]`), multiplicateurs d'intensité et de
//! densité de base. Algorithme générique : aucun nom de style ni valeur musicale ici.
//!
//! **Qui gagne** (hôte contre variante) :
//!
//! | Réglage | Sans commande de l'hôte | Commande de l'hôte |
//! |---|---|---|
//! | tempo | celui de la variante, base de la courbe `tempo` de `dynamics` | `SetBpm` (tempo direct) l'emporte tel quel ; la rampe de set et la variante n'y touchent plus |
//! | tonalité et mode | ceux de la variante, au premier air de la session | `[fixed_tonality]` (donc une grille utilisateur transposée par lui) l'emporte : la variante est ignorée |
//! | swing | celui de la variante | `SetSwing` l'emporte |
//! | programmes, voicings, registre, tags, arc, intensité, densité | ceux de la variante | aucune commande de l'hôte ne les touche ; un profil de l'hôte sans variante ne les tire pas |
//!
//! Le tirage vient du RNG injecté (deux entiers de 32 bits, **seulement si le profil a des
//! variantes** : sans elles le flux est celui d'avant), puis de flux ChaCha8 dérivés de cette
//! graine, un par groupe : le seek et le rejeu retrouvent la même variante.

use std::collections::{BTreeMap, HashMap, HashSet};

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use super::{
    accomp_profile::{ChordVoicing, require},
    arc::{ArcVariant, ChorusArc, TagWeight, validate_tags},
};
use crate::{
    algo::RngCore,
    odin::OdinTrackSpec,
    ornaments::mix,
    style_profile::{FixedTonality, ProfileError, ProfileInstruments, StyleProfile},
    timeline::{Measure, TrackId},
    tuning::AccompanimentParams,
};

/// Nombre maximal de variantes d'un profil.
pub const MAX_SESSION_VARIANTS: usize = 64;
/// Nombre maximal de groupes (tirages indépendants).
pub const MAX_SESSION_GROUPS: usize = 16;
/// Nombre maximal d'arcs nommés.
pub const MAX_SESSION_ARCS: usize = 16;
/// Décalage de registre maximal par piste, en octaves.
pub const MAX_REGISTER_OCTAVES: i8 = 1;

const GROUP_STREAM: u64 = 0x5345_4752;
const TEMPO_STREAM: u64 = 0x5345_5450;

const DEFAULT_GROUP: &str = "main";

fn default_group() -> String {
    DEFAULT_GROUP.to_owned()
}

const fn default_weight() -> u32 {
    1
}

/// Tempo d'une variante : une valeur, ou une sous-plage dont la graine tire un BPM entier.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SessionTempo {
    Fixed(f32),
    Range([f32; 2]),
}

/// Programmes qui remplacent ceux de `[instruments]` : seuls les champs présents changent.
/// Sert aux variantes de session (toute la session) et aux variantes de l'arc (un chorus).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstrumentOverride {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub melody_program: Option<u8>,
    /// Exige `instruments.countermelody_program` dans le profil.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub countermelody_program: Option<u8>,
    /// Exige `instruments.chords_program` dans le profil.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chords_program: Option<u8>,
    /// Exige `instruments.bass_program` dans le profil.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bass_program: Option<u8>,
    /// Banque du kit de percussion (`128` : kit GM).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub percussion_bank: Option<u16>,
    /// Programme du kit : baguettes, balais, kit électronique…
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub percussion_program: Option<u8>,
}

impl InstrumentOverride {
    /// `base` dont les champs présents sont remplacés.
    #[must_use]
    pub fn apply(&self, base: ProfileInstruments) -> ProfileInstruments {
        ProfileInstruments {
            melody_program: self.melody_program.unwrap_or(base.melody_program),
            countermelody_program: self.countermelody_program.or(base.countermelody_program),
            chords_program: self.chords_program.or(base.chords_program),
            bass_program: self.bass_program.or(base.bass_program),
            percussion_bank: self.percussion_bank.unwrap_or(base.percussion_bank),
            percussion_program: self.percussion_program.unwrap_or(base.percussion_program),
        }
    }

    /// Champs réglés (nom du champ), pour détecter deux groupes qui règlent le même.
    fn claims(&self) -> impl Iterator<Item = &'static str> {
        [
            ("melody_program", self.melody_program.is_some()),
            ("countermelody_program", self.countermelody_program.is_some()),
            ("chords_program", self.chords_program.is_some()),
            ("bass_program", self.bass_program.is_some()),
            ("percussion_bank", self.percussion_bank.is_some()),
            ("percussion_program", self.percussion_program.is_some()),
        ]
        .into_iter()
        .filter_map(|(name, set)| set.then_some(name))
    }

    /// Valide contre le profil ; `field` : chemin de la surcharge.
    pub(crate) fn validate(&self, profile: &StyleProfile, field: &str) -> Result<(), ProfileError> {
        let base = &profile.instruments;
        for (name, program, present) in [
            ("melody_program", self.melody_program, true),
            (
                "countermelody_program",
                self.countermelody_program,
                base.countermelody_program.is_some(),
            ),
            ("chords_program", self.chords_program, base.chords_program.is_some()),
            ("bass_program", self.bass_program, base.bass_program.is_some()),
            ("percussion_program", self.percussion_program, true),
        ] {
            let Some(program) = program else { continue };
            require(program <= 127, format!("{field}.{name}"), "programme MIDI dans 0..127")?;
            require(
                present,
                format!("{field}.{name}"),
                "le profil ne joue pas cette piste (aucun programme dans [instruments])",
            )?;
        }
        require(
            self.percussion_bank.is_none_or(|bank| bank <= 255),
            format!("{field}.percussion_bank"),
            "banque SF2 dans 0..255",
        )
    }
}

/// Une variante de session : poids et surcharges (chaque champ omis laisse le profil tel quel).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionVariant {
    /// Groupe : un tirage par groupe, les groupes se combinent.
    #[serde(default = "default_group")]
    pub group: String,
    /// Nom lisible, publié avec la variante tirée.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// Poids du tirage dans le groupe.
    #[serde(default = "default_weight")]
    pub weight: u32,
    /// Tempo : remplace `meter.default_bpm` comme base de la courbe `tempo` de `dynamics`,
    /// sauf tempo direct de l'hôte.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bpm: Option<SessionTempo>,
    /// Tonalité et mode du premier air ; ignorés sous `[fixed_tonality]`. Le mode doit être
    /// atteignable par le profil.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tonality: Option<FixedTonality>,
    /// Programmes SoundFont par piste, kit de percussion.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instruments: Option<InstrumentOverride>,
    /// Zones Odin qui remplacent celles de `[odin.<Piste>]` (jeux seulement).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub odin: BTreeMap<TrackId, OdinTrackSpec>,
    /// Famille de voicings du comping (`accompaniment.chords.voicing`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voicing: Option<ChordVoicing>,
    /// Rapport de swing (`0,5..=0,75`), sauf `SetSwing` de l'hôte.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub swing: Option<f32>,
    /// Décalage de registre par piste mélodique, en octaves (`-1..=1`).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub register_octaves: BTreeMap<TrackId, i8>,
    /// Tags de comping autorisés, pondérés (motifs et phrases) ; l'arc du chorus, s'il en
    /// déclare, passe devant.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub comping: Vec<TagWeight>,
    /// Tags de batterie autorisés, pondérés (patterns, phrases et relances) ; l'arc du chorus,
    /// s'il en déclare, passe devant.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub percussion: Vec<TagWeight>,
    /// Arc nommé (`[session.arcs.<nom>]`) qui remplace `[arc]` pour la session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arc: Option<String>,
    /// Multiplicateur de base de l'intensité lue par `dynamics` (comme celui de l'arc ; pas le
    /// tempo).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intensity: Option<f32>,
    /// Multiplicateur de base de la densité rythmique.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub density: Option<f32>,
}

impl Default for SessionVariant {
    /// Variante du groupe `main`, de poids 1, sans surcharge : de quoi construire un profil en
    /// mémoire (`SessionVariant { bpm: .., ..Default::default() }`).
    fn default() -> Self {
        Self {
            group: default_group(),
            name: String::new(),
            weight: default_weight(),
            bpm: None,
            tonality: None,
            instruments: None,
            odin: BTreeMap::new(),
            voicing: None,
            swing: None,
            register_octaves: BTreeMap::new(),
            comping: Vec::new(),
            percussion: Vec::new(),
            arc: None,
            intensity: None,
            density: None,
        }
    }
}

impl SessionVariant {
    /// Champs réglés, pour détecter deux groupes qui règlent le même.
    fn claims(&self) -> Vec<(String, String)> {
        let mut claims: Vec<(String, String)> = Vec::new();
        let mut claim = |field: &str, key: String| claims.push((field.to_owned(), key));
        if self.bpm.is_some() {
            claim("bpm", "bpm".into());
        }
        if self.tonality.is_some() {
            claim("tonality", "tonality".into());
        }
        if let Some(instruments) = &self.instruments {
            for name in instruments.claims() {
                claim(&format!("instruments.{name}"), format!("instruments.{name}"));
            }
        }
        for track in self.odin.keys() {
            claim(&format!("odin.{}", track.name()), format!("odin.{}", track.name()));
        }
        if self.voicing.is_some() {
            claim("voicing", "voicing".into());
        }
        if self.swing.is_some() {
            claim("swing", "swing".into());
        }
        for track in self.register_octaves.keys() {
            let key = format!("register_octaves.{}", track.name());
            claim(&key.clone(), key);
        }
        if !self.comping.is_empty() {
            claim("comping", "comping".into());
        }
        if !self.percussion.is_empty() {
            claim("percussion", "percussion".into());
        }
        if self.arc.is_some() {
            claim("arc", "arc".into());
        }
        if self.intensity.is_some() {
            claim("intensity", "intensity".into());
        }
        if self.density.is_some() {
            claim("density", "density".into());
        }
        claims
    }
}

/// Section `[session]` du profil.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileSession {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<SessionVariant>,
    /// Arcs que les variantes nomment (même schéma que `[arc]`).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub arcs: BTreeMap<String, ChorusArc>,
}

/// Variante tirée, telle que les snapshots la publient.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrawnVariant {
    pub group: String,
    /// Indice dans `[[session.variants]]`.
    pub index: usize,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
}

/// Base de session lue par l'arc : multiplicateurs et tags de repli (utilisés quand le chorus
/// n'en déclare pas), et variante neutre pour une session sans arc.
#[derive(Clone, Debug)]
pub(crate) struct SessionBase {
    pub(crate) intensity: f32,
    pub(crate) density: f32,
    pub(crate) comping: Vec<TagWeight>,
    pub(crate) percussion: Vec<TagWeight>,
    pub(crate) neutral: ArcVariant,
}

/// Résultat du tirage d'une graine : variantes retenues et surcharges résolues.
#[derive(Clone, Debug)]
pub struct SessionDraw {
    seed: u64,
    drawn: Vec<DrawnVariant>,
    tempo: Option<f32>,
    tonality: Option<FixedTonality>,
    instruments: Option<ProfileInstruments>,
    swing: Option<f32>,
    registers: Vec<(TrackId, i8)>,
    accompaniment: Option<AccompanimentParams>,
    odin: BTreeMap<TrackId, OdinTrackSpec>,
    arc: Option<String>,
    base: Option<SessionBase>,
}

impl SessionDraw {
    /// Variantes tirées, une par groupe, dans l'ordre des groupes.
    #[must_use]
    pub fn variants(&self) -> &[DrawnVariant] {
        &self.drawn
    }

    /// Graine de la session pour les flux dérivés.
    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    /// Tempo tiré (BPM), s'il est réglé par une variante.
    #[must_use]
    pub const fn tempo(&self) -> Option<f32> {
        self.tempo
    }

    /// Tonalité et mode tirés (ignorés sous `[fixed_tonality]`).
    #[must_use]
    pub const fn tonality(&self) -> Option<FixedTonality> {
        self.tonality
    }

    /// Programmes de la session, si une variante en règle.
    #[must_use]
    pub const fn instruments(&self) -> Option<ProfileInstruments> {
        self.instruments
    }

    /// Rapport de swing tiré.
    #[must_use]
    pub const fn swing(&self) -> Option<f32> {
        self.swing
    }

    /// Décalages de registre `(piste, octaves)`.
    #[must_use]
    pub fn registers(&self) -> &[(TrackId, i8)] {
        &self.registers
    }

    /// Nom de l'arc choisi, s'il y en a un.
    #[must_use]
    pub fn arc_name(&self) -> Option<&str> {
        self.arc.as_deref()
    }

    /// Spécification Odin tirée pour `track`, si une variante la remplace.
    #[must_use]
    pub fn odin(&self, track: TrackId) -> Option<&OdinTrackSpec> {
        self.odin.get(&track)
    }

    /// Zones Odin que la variante tirée met à la place de celles du profil, par piste.
    #[must_use]
    pub const fn odin_overrides(&self) -> &BTreeMap<TrackId, OdinTrackSpec> {
        &self.odin
    }

    /// Accompagnement avec la famille de voicings tirée, si une variante en règle une.
    pub(crate) fn accompaniment(&self) -> Option<&AccompanimentParams> {
        self.accompaniment.as_ref()
    }

    /// Base lue par l'arc ; `None` quand aucune variante ne règle intensité, densité ni tags.
    pub(crate) fn base(&self) -> Option<&SessionBase> {
        self.base.as_ref()
    }

    /// Décale les pistes mélodiques de leur registre tiré (octaves entières, hauteurs de la
    /// mesure ; une note qui sortirait de `0..=127` garde sa hauteur).
    pub(crate) fn shift_registers(&self, measure: &mut Measure) {
        for &(track, octaves) in &self.registers {
            let delta = i16::from(octaves) * 12;
            for note in measure.tracks.get_mut(&track).into_iter().flatten() {
                let pitch = i16::from(note.pitch) + delta;
                if (0..=127).contains(&pitch) {
                    note.pitch = pitch as u8;
                }
            }
        }
    }
}

/// Tirage pondéré d'un indice avec le flux dérivé `seed` ; `weights` est non vide (validé).
fn pick(weights: &[u32], seed: u64) -> usize {
    let total: u64 = weights.iter().map(|&w| u64::from(w)).sum();
    let mut pick = ChaCha8Rng::seed_from_u64(seed).next_range_u64(0..total.max(1));
    for (index, &weight) in weights.iter().enumerate() {
        if pick < u64::from(weight) {
            return index;
        }
        pick -= u64::from(weight);
    }
    weights.len() - 1
}

impl ProfileSession {
    /// Vrai sans variante ni arc nommé : le profil n'a pas d'habillage par graine.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.variants.is_empty() && self.arcs.is_empty()
    }

    /// Groupes dans l'ordre de première apparition.
    fn groups(&self) -> Vec<&str> {
        let mut groups: Vec<&str> = Vec::new();
        for variant in &self.variants {
            if !groups.contains(&variant.group.as_str()) {
                groups.push(&variant.group);
            }
        }
        groups
    }

    /// Tire une variante par groupe avec la graine de session `seed` et résout les surcharges.
    /// Profil validé : aucune erreur possible.
    #[must_use]
    pub fn draw(&self, profile: &StyleProfile, seed: u64) -> SessionDraw {
        let mut draw = SessionDraw {
            seed,
            drawn: Vec::new(),
            tempo: None,
            tonality: None,
            instruments: None,
            swing: None,
            registers: Vec::new(),
            accompaniment: None,
            odin: BTreeMap::new(),
            arc: None,
            base: None,
        };
        let (mut intensity, mut density) = (None, None);
        let (mut comping, mut percussion): (Vec<TagWeight>, Vec<TagWeight>) = Default::default();
        for (group_index, group) in self.groups().into_iter().enumerate() {
            let members: Vec<(usize, &SessionVariant)> = self
                .variants
                .iter()
                .enumerate()
                .filter(|(_, variant)| variant.group == group)
                .collect();
            let weights: Vec<u32> = members.iter().map(|(_, variant)| variant.weight).collect();
            let (index, variant) =
                members[pick(&weights, mix(mix(seed, GROUP_STREAM), group_index as u64))];
            draw.drawn.push(DrawnVariant {
                group: group.to_owned(),
                index,
                name: variant.name.clone(),
            });
            if let Some(tempo) = variant.bpm {
                draw.tempo = Some(match tempo {
                    SessionTempo::Fixed(bpm) => bpm,
                    SessionTempo::Range([low, high]) => {
                        let steps = (high - low).floor().max(0.0) as u64;
                        let mut rng = ChaCha8Rng::seed_from_u64(mix(
                            mix(seed, TEMPO_STREAM),
                            group_index as u64,
                        ));
                        low + rng.next_range_u64(0..steps + 1) as f32
                    }
                });
            }
            draw.tonality = variant.tonality.or(draw.tonality);
            if let Some(overrides) = &variant.instruments {
                draw.instruments =
                    Some(overrides.apply(draw.instruments.unwrap_or(profile.instruments)));
            }
            draw.odin.extend(variant.odin.iter().map(|(track, spec)| (*track, spec.clone())));
            if let (Some(voicing), Some(base)) = (variant.voicing, &profile.accompaniment) {
                let mut dressed = base.clone();
                dressed.chords.voicing = voicing;
                draw.accompaniment = Some(dressed);
            }
            draw.swing = variant.swing.or(draw.swing);
            draw.registers.extend(variant.register_octaves.iter().map(|(t, o)| (*t, *o)));
            if !variant.comping.is_empty() {
                comping.clone_from(&variant.comping);
            }
            if !variant.percussion.is_empty() {
                percussion.clone_from(&variant.percussion);
            }
            draw.arc = variant.arc.clone().or(draw.arc);
            intensity = variant.intensity.or(intensity);
            density = variant.density.or(density);
        }
        if intensity.is_some() || density.is_some() || !comping.is_empty() || !percussion.is_empty()
        {
            draw.base = Some(SessionBase {
                intensity: intensity.unwrap_or(1.0),
                density: density.unwrap_or(1.0),
                comping,
                percussion,
                neutral: ArcVariant::default(),
            });
        }
        draw
    }

    /// Valide la section contre le profil (chemins `session.variants[i].champ`).
    pub(crate) fn validate(&self, profile: &StyleProfile) -> Result<(), ProfileError> {
        require(
            self.variants.len() <= MAX_SESSION_VARIANTS,
            "session.variants",
            "au plus 64 variantes",
        )?;
        require(self.arcs.len() <= MAX_SESSION_ARCS, "session.arcs", "au plus 16 arcs nommés")?;
        for (name, arc) in &self.arcs {
            require(!name.is_empty(), "session.arcs", "nom d'arc non vide requis")?;
            arc.validate(profile, &format!("session.arcs.{name}"))?;
        }
        require(
            self.groups().len() <= MAX_SESSION_GROUPS,
            "session.variants",
            "au plus 16 groupes",
        )?;
        let meter = &profile.meter;
        let mut totals: HashMap<&str, u64> = HashMap::new();
        let mut claimed: HashMap<String, &str> = HashMap::new();
        for (v, variant) in self.variants.iter().enumerate() {
            let field = format!("session.variants[{v}]");
            require(!variant.group.is_empty(), format!("{field}.group"), "groupe non vide requis")?;
            require(variant.weight > 0, format!("{field}.weight"), "poids positif requis")?;
            let total = totals.entry(&variant.group).or_default();
            *total += u64::from(variant.weight);
            require(
                *total <= u64::from(u32::MAX),
                format!("{field}.weight"),
                "somme des poids du groupe trop grande (u32)",
            )?;
            if let Some(tempo) = variant.bpm {
                let (low, high) = match tempo {
                    SessionTempo::Fixed(bpm) => (bpm, bpm),
                    SessionTempo::Range([low, high]) => (low, high),
                };
                require(
                    low.is_finite()
                        && high.is_finite()
                        && low <= high
                        && low >= meter.min_bpm
                        && high <= meter.max_bpm,
                    format!("{field}.bpm"),
                    "tempo (ou plage ordonnée) fini dans meter.min_bpm..max_bpm requis",
                )?;
            }
            if let Some(tonality) = variant.tonality {
                require(tonality.key < 12, format!("{field}.tonality.key"), "attendu 0..11")?;
                require(
                    profile.fixed_tonality.is_some()
                        || profile.reachable_modes().contains(&tonality.mode),
                    format!("{field}.tonality.mode"),
                    "mode inatteignable par le profil (melody.key_modes, dynamics.color, \
                     dynamics.parallel_modes)",
                )?;
            }
            if let Some(instruments) = &variant.instruments {
                instruments.validate(profile, &format!("{field}.instruments"))?;
            }
            for (track, spec) in &variant.odin {
                profile.validate_odin_spec(
                    *track,
                    spec,
                    &format!("{field}.odin.{}", track.name()),
                )?;
            }
            if let Some(voicing) = variant.voicing {
                let Some(accompaniment) = &profile.accompaniment else {
                    return Err(ProfileError::new(
                        format!("{field}.voicing"),
                        "la famille de voicings demande une section accompaniment",
                    ));
                };
                let mut dressed = accompaniment.clone();
                dressed.chords.voicing = voicing;
                dressed
                    .validate_accompaniment(
                        meter.steps_per_bar,
                        meter.steps_per_pulse,
                        profile.harmony.steps_per_chord,
                        &profile.form.parts,
                    )
                    .map_err(|error| {
                        ProfileError::new(
                            format!("{field}.voicing"),
                            format!("famille refusée par le profil : {}", error.message),
                        )
                    })?;
            }
            if let Some(swing) = variant.swing {
                require(
                    swing.is_finite() && (0.5..=0.75).contains(&swing),
                    format!("{field}.swing"),
                    "rapport de swing fini dans 0,5..0,75 requis",
                )?;
            }
            for (track, octaves) in &variant.register_octaves {
                let field = format!("{field}.register_octaves.{}", track.name());
                require(
                    !matches!(track, TrackId::PercussionLow | TrackId::PercussionHigh),
                    &field,
                    "piste mélodique requise (pas la percussion)",
                )?;
                require(
                    (-MAX_REGISTER_OCTAVES..=MAX_REGISTER_OCTAVES).contains(octaves),
                    &field,
                    "décalage de -1 à 1 octave requis",
                )?;
            }
            validate_tags(profile, &field, "comping", &variant.comping)?;
            validate_tags(profile, &field, "percussion", &variant.percussion)?;
            if let Some(arc) = &variant.arc {
                require(
                    self.arcs.contains_key(arc),
                    format!("{field}.arc"),
                    "arc inconnu dans session.arcs",
                )?;
            }
            for (name, value) in [("intensity", variant.intensity), ("density", variant.density)] {
                require(
                    value.is_none_or(|value| value.is_finite() && (0.0..=2.0).contains(&value)),
                    format!("{field}.{name}"),
                    "multiplicateur fini dans 0..2 requis",
                )?;
            }
            let mut seen = HashSet::new();
            for (name, key) in variant.claims() {
                if !seen.insert(key.clone()) {
                    continue;
                }
                if let Some(other) = claimed.insert(key, &variant.group)
                    && other != variant.group
                {
                    return Err(ProfileError::new(
                        format!("{field}.{name}"),
                        format!("déjà réglé par le groupe « {other} » : un champ, un seul groupe"),
                    ));
                }
            }
        }
        Ok(())
    }
}
