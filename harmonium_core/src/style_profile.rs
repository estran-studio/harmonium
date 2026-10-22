//! Profils de style validés : grille, forme et banques, embarqués ou chargés d'un dossier.
use std::{
    collections::HashSet,
    fmt,
    path::{Component, Path},
    sync::LazyLock,
};

use serde::{Deserialize, Serialize};

use crate::{
    params::{PulseSpec, TimeSignature},
    trad::{
        Mode, TradChord, arrangement::ReelArrangement, cells::CellBank, dynamics::StyleDynamics,
        feet::FeetTable, plan::ImposedRole,
    },
    tuning::{ReelFiddleParams, TradHarmonyParams},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileMeta {
    pub id: String,
    pub name: String,
    pub tradition: String,
    pub tags: Vec<String>,
    pub corpus: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileMeter {
    pub time_signature: TimeSignature,
    pub steps_per_bar: usize,
    pub steps_per_unit: usize,
    pub steps_per_pulse: usize,
    pub pulses_per_bar: usize,
    pub default_swing_ratio: f32,
    pub default_bpm: f32,
    pub min_bpm: f32,
    pub max_bpm: f32,
}
impl ProfileMeter {
    pub const fn pulse_spec(&self) -> PulseSpec {
        PulseSpec {
            steps_per_pulse: self.steps_per_pulse as u8,
            pulses_per_bar: self.pulses_per_bar as u8,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfilePart {
    pub name: String,
    pub roles: Vec<ImposedRole>,
    pub melody_center_offset: i8,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileForm {
    pub parts: Vec<ProfilePart>,
    pub sequence: Vec<usize>,
}
impl ProfileForm {
    pub fn bars_per_tour(&self) -> usize {
        self.sequence.iter().map(|&p| self.parts[p].roles.len()).sum()
    }
    pub fn unique_bars(&self) -> usize {
        self.parts.iter().map(|p| p.roles.len()).sum()
    }
    /// Partie, passe (1-based), mesure (0-based) pour une mesure de tour.
    pub fn position(&self, bar: usize) -> (usize, usize, usize) {
        let mut remaining = bar % self.bars_per_tour();
        for (index, &part) in self.sequence.iter().enumerate() {
            if remaining < self.parts[part].roles.len() {
                let pass = self.sequence[..=index].iter().filter(|&&p| p == part).count();
                return (part, pass, remaining);
            }
            remaining -= self.parts[part].roles.len();
        }
        unreachable!("validated nonempty form")
    }
    pub fn unique_offset(&self, part: usize) -> usize {
        self.parts[..part].iter().map(|p| p.roles.len()).sum()
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyModeWeight {
    pub key: u8,
    pub mode: Mode,
    pub weight: u32,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileMelody {
    pub cell_units: usize,
    pub cells: CellBank,
    pub params: ReelFiddleParams,
    pub key_modes: Vec<KeyModeWeight>,
    pub strong_units: Vec<usize>,
    pub weak_units: Vec<usize>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModeChords {
    pub mode: Mode,
    pub chords: Vec<TradChord>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileHarmony {
    pub model: String,
    pub steps_per_chord: usize,
    pub candidates: Vec<ModeChords>,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileInstruments {
    pub melody_program: u8,
    pub chords_program: u8,
    pub bass_program: u8,
    pub percussion_bank: u16,
    pub percussion_program: u8,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StyleProfile {
    pub meta: ProfileMeta,
    pub meter: ProfileMeter,
    pub form: ProfileForm,
    pub melody: ProfileMelody,
    pub harmony: ProfileHarmony,
    pub accompaniment: TradHarmonyParams,
    pub percussion: FeetTable,
    pub instruments: ProfileInstruments,
    pub arrangement: ReelArrangement,
    pub dynamics: StyleDynamics,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileError {
    pub field: String,
    pub message: String,
}
impl ProfileError {
    fn new(field: impl Into<String>, message: impl ToString) -> Self {
        Self { field: field.into(), message: message.to_string() }
    }
}
impl fmt::Display for ProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}
impl std::error::Error for ProfileError {}

const EMBEDDED: &str = include_str!("../../profiles/reel_quebecois/profile.toml");
const BANKS: [(&str, &str, &str); 4] = [
    ("cells", "melody.cells", include_str!("../../profiles/reel_quebecois/cells.toml")),
    ("feet", "percussion", include_str!("../../profiles/reel_quebecois/feet.toml")),
    ("arrangement", "arrangement", include_str!("../../profiles/reel_quebecois/arrangement.toml")),
    ("dynamics", "dynamics", include_str!("../../profiles/reel_quebecois/dynamics.toml")),
];
impl Default for StyleProfile {
    fn default() -> Self {
        Self::builtin().clone()
    }
}
impl StyleProfile {
    /// Profil embarqué partagé ; aucun accès au disque ni copie de banque.
    pub fn builtin() -> &'static Self {
        static DEFAULT: LazyLock<StyleProfile> =
            LazyLock::new(|| match StyleProfile::decode(EMBEDDED, None, true) {
                Ok(profile) => profile,
                Err(error) => unreachable!("profil embarqué invalide : {error}"),
            });
        &DEFAULT
    }
    /// Résolution des identifiants distribués avec le moteur (alias CLI inclus).
    pub fn named(id: &str) -> Result<Self, ProfileError> {
        if id.eq_ignore_ascii_case("reel") || id.eq_ignore_ascii_case(&Self::builtin().meta.id) {
            Ok(Self::default())
        } else {
            Err(ProfileError::new("meta.id", format!("profil inconnu : {id}")))
        }
    }
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ProfileError> {
        let path = path.as_ref();
        let file = if path.is_dir() { path.join("profile.toml") } else { path.to_path_buf() };
        let text = std::fs::read_to_string(&file)
            .map_err(|e| ProfileError::new(file.display().to_string(), e))?;
        Self::decode(&text, file.parent(), false)
    }
    /// Lecture d'un profil autonome (banques inline, sans accès au disque).
    pub fn from_toml(text: &str) -> Result<Self, ProfileError> {
        Self::decode(text, None, false)
    }
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }
    fn decode(text: &str, base: Option<&Path>, embedded: bool) -> Result<Self, ProfileError> {
        let mut root: toml::Value =
            toml::from_str(text).map_err(|e| ProfileError::new("profile.toml", e))?;
        let files = root.as_table_mut().and_then(|t| t.remove("files"));
        if let Some(files) = files {
            let files =
                files.as_table().ok_or_else(|| ProfileError::new("files", "table attendue"))?;
            for (key, filename) in files {
                let (_, target, builtin) = BANKS
                    .iter()
                    .find(|(k, _, _)| *k == key)
                    .ok_or_else(|| ProfileError::new(format!("files.{key}"), "banque inconnue"))?;
                let name = filename
                    .as_str()
                    .ok_or_else(|| ProfileError::new(format!("files.{key}"), "chemin attendu"))?;
                let relative = Path::new(name);
                if relative.components().any(|c| !matches!(c, Component::Normal(_))) {
                    return Err(ProfileError::new(
                        format!("files.{key}"),
                        "chemin relatif sans .. requis",
                    ));
                }
                let contents = if embedded {
                    builtin.to_string()
                } else {
                    let base = base.ok_or_else(|| {
                        ProfileError::new(
                            format!("files.{key}"),
                            "référence externe : utiliser load(dossier)",
                        )
                    })?;
                    std::fs::read_to_string(base.join(relative))
                        .map_err(|e| ProfileError::new(format!("files.{key}"), e))?
                };
                let value = toml::from_str(&contents)
                    .map_err(|e| ProfileError::new(format!("files.{key}"), e))?;
                if *target == "melody.cells" {
                    root.get_mut("melody")
                        .and_then(toml::Value::as_table_mut)
                        .ok_or_else(|| ProfileError::new("melody", "table requise"))?
                        .insert("cells".into(), value);
                } else {
                    root.as_table_mut()
                        .ok_or_else(|| ProfileError::new("profile.toml", "table requise"))?
                        .insert(target.to_string(), value);
                }
            }
        }
        if let Some(intro) = root.as_table_mut().and_then(|table| table.remove("intro")) {
            let arrangement = root
                .get_mut("arrangement")
                .and_then(toml::Value::as_table_mut)
                .ok_or_else(|| ProfileError::new("intro", "section arrangement requise"))?;
            if arrangement.insert("intro".into(), intro).is_some() {
                return Err(ProfileError::new("intro", "déjà défini dans arrangement.intro"));
            }
        }
        let profile: Self = serde_path_to_error::deserialize(root)
            .map_err(|e| ProfileError::new(e.path().to_string(), e.inner()))?;
        profile.validate()?;
        Ok(profile)
    }
    pub fn validate(&self) -> Result<(), ProfileError> {
        let check = |ok: bool, field: &str, message: &str| {
            if ok { Ok(()) } else { Err(ProfileError::new(field, message)) }
        };
        check(
            !self.meta.id.is_empty()
                && self
                    .meta
                    .id
                    .bytes()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_'),
            "meta.id",
            "identifiant ASCII minuscule requis",
        )?;
        for (name, value) in [
            ("name", &self.meta.name),
            ("tradition", &self.meta.tradition),
            ("corpus", &self.meta.corpus),
        ] {
            check(!value.trim().is_empty(), &format!("meta.{name}"), "texte non vide requis")?;
        }
        let m = &self.meter;
        check(
            (1..=64).contains(&m.time_signature.numerator)
                && m.time_signature.denominator <= 64
                && m.time_signature.denominator.is_power_of_two(),
            "meter.time_signature",
            "signature positive, dénominateur puissance de deux",
        )?;
        check((1..=4096).contains(&m.steps_per_bar), "meter.steps_per_bar", "attendu 1..4096")?;
        check(
            m.steps_per_unit > 0 && m.steps_per_unit <= m.steps_per_bar,
            "meter.steps_per_unit",
            "unité positive dans la mesure",
        )?;
        check(
            m.steps_per_pulse.checked_mul(m.pulses_per_bar) == Some(m.steps_per_bar)
                && m.steps_per_pulse > 0
                && m.steps_per_pulse <= 255
                && m.pulses_per_bar <= 255,
            "meter.steps_per_pulse",
            "pulse × nombre de pulses doit égaler steps_per_bar",
        )?;
        check(
            m.default_swing_ratio.is_finite() && (0.5..=0.75).contains(&m.default_swing_ratio),
            "meter.default_swing_ratio",
            "attendu 0.5..0.75",
        )?;
        check(
            m.min_bpm.is_finite()
                && m.max_bpm.is_finite()
                && (40.0..=300.0).contains(&m.min_bpm)
                && m.min_bpm <= m.max_bpm
                && m.max_bpm <= 300.0,
            "meter.min_bpm",
            "plage finie ordonnée dans 40..300",
        )?;
        check(
            m.default_bpm.is_finite() && (m.min_bpm..=m.max_bpm).contains(&m.default_bpm),
            "meter.default_bpm",
            "tempo hors plage",
        )?;
        check(
            !self.form.parts.is_empty() && self.form.parts.len() <= 32,
            "form.parts",
            "1..32 parties requises",
        )?;
        let mut names = HashSet::new();
        for (i, part) in self.form.parts.iter().enumerate() {
            let field = format!("form.parts[{i}]");
            check(
                !part.name.is_empty() && names.insert(&part.name),
                &format!("{field}.name"),
                "nom non vide et unique requis",
            )?;
            check(
                !part.roles.is_empty() && part.roles.len() <= 255,
                &format!("{field}.roles"),
                "1..255 mesures requises",
            )?;
            check(
                (-24..=24).contains(&part.melody_center_offset),
                &format!("{field}.melody_center_offset"),
                "attendu -24..24",
            )?;
            for (bar, role) in part.roles.iter().enumerate() {
                if let ImposedRole::CopyOf(source) = role {
                    check(
                        *source > 0 && usize::from(*source) <= bar,
                        &format!("{field}.roles[{bar}]"),
                        "la copie doit viser une mesure précédente (1-based)",
                    )?;
                }
            }
        }
        check(
            !self.form.sequence.is_empty() && self.form.sequence.len() <= 256,
            "form.sequence",
            "1..256 passes requises",
        )?;
        for (i, &part) in self.form.sequence.iter().enumerate() {
            check(part < self.form.parts.len(), &format!("form.sequence[{i}]"), "partie inconnue")?;
        }
        check(
            (0..self.form.parts.len()).all(|p| self.form.sequence.contains(&p)),
            "form.sequence",
            "chaque partie doit être jouée",
        )?;
        check(
            self.melody.cell_units.checked_mul(m.steps_per_unit) == Some(m.steps_per_bar),
            "melody.cell_units",
            "cell_units × steps_per_unit doit couvrir une mesure",
        )?;
        self.melody.cells.validate().map_err(|e| ProfileError::new("melody.cells", e))?;
        for (i, cell) in self.melody.cells.cells.iter().enumerate() {
            check(
                cell.notes.len() == self.melody.cell_units,
                &format!("melody.cells[{i}].notes"),
                "longueur différente de cell_units",
            )?;
        }
        for role in crate::trad::CellRole::ALL {
            let needed = self
                .form
                .parts
                .iter()
                .flat_map(|p| &p.roles)
                .any(|r| r.forced_cell_role() == Some(role))
                || matches!(
                    role,
                    crate::trad::CellRole::Run
                        | crate::trad::CellRole::Arp
                        | crate::trad::CellRole::Turn
                );
            check(
                !needed
                    || self
                        .melody
                        .cells
                        .cells
                        .iter()
                        .any(|c| c.role == role && c.respects_cadence()),
                &format!("melody.cells.{role:?}"),
                "aucune cellule éligible pour ce rôle",
            )?;
        }
        check(
            !self.melody.key_modes.is_empty() && self.melody.key_modes.iter().any(|k| k.weight > 0),
            "melody.key_modes",
            "distribution non vide et poids total positif requis",
        )?;
        check(
            self.melody.key_modes.iter().map(|k| u64::from(k.weight)).sum::<u64>()
                <= u64::from(u32::MAX),
            "melody.key_modes",
            "somme des poids trop grande",
        )?;
        for (i, key) in self.melody.key_modes.iter().enumerate() {
            check(key.key < 12, &format!("melody.key_modes[{i}].key"), "attendu 0..11")?;
        }
        for (name, units) in
            [("strong_units", &self.melody.strong_units), ("weak_units", &self.melody.weak_units)]
        {
            check(
                units.iter().all(|&u| u < self.melody.cell_units),
                &format!("melody.{name}"),
                "indice hors cellule",
            )?;
        }
        let p = &self.melody.params;
        check(
            p.tessitura_min <= p.tessitura_max
                && p.tessitura_max <= 127
                && p.tessitura_max - p.tessitura_min >= 12,
            "melody.params.tessitura_min",
            "tessiture MIDI d'au moins une octave requise",
        )?;
        check(
            (p.tessitura_min..=p.tessitura_max).contains(&p.melody_center_midi),
            "melody.params.melody_center_midi",
            "centre hors tessiture",
        )?;
        check(
            p.tonic_midi_min <= p.tonic_midi_max && p.tonic_midi_max <= 127,
            "melody.params.tonic_midi_min",
            "plage MIDI ordonnée requise",
        )?;
        check(
            p.continuity_max_retries > 0 && p.continuity_max_retries <= 10000,
            "melody.params.continuity_max_retries",
            "attendu 1..10000",
        )?;
        check(
            p.max_continuity_step_distance >= 0,
            "melody.params.max_continuity_step_distance",
            "distance positive requise",
        )?;
        check(
            (1..=127).contains(&p.base_velocity),
            "melody.params.base_velocity",
            "attendu 1..127",
        )?;
        check(
            u64::from(p.weight_role_run)
                + u64::from(p.weight_role_arp)
                + u64::from(p.weight_role_turn)
                <= u64::from(u32::MAX)
                && (p.weight_role_run > 0 || p.weight_role_arp > 0 || p.weight_role_turn > 0),
            "melody.params.weight_role_run",
            "poids libres : somme positive sans débordement",
        )?;
        check(
            self.harmony.model == "trad_functional",
            "harmony.model",
            "modèle disponible : trad_functional",
        )?;
        check(
            self.harmony.steps_per_chord > 0
                && m.steps_per_bar.is_multiple_of(self.harmony.steps_per_chord),
            "harmony.steps_per_chord",
            "doit diviser steps_per_bar",
        )?;
        let mut modes = HashSet::new();
        for (i, entry) in self.harmony.candidates.iter().enumerate() {
            check(
                modes.insert(entry.mode) && !entry.chords.is_empty(),
                &format!("harmony.candidates[{i}]"),
                "mode unique et accords non vides requis",
            )?;
            check(
                entry.chords.iter().any(|c| c.degree == crate::trad::ChordDegree::I)
                    && entry.chords.iter().any(|c| c.degree == crate::trad::ChordDegree::V),
                &format!("harmony.candidates[{i}].chords"),
                "tonique I et dominante V requises",
            )?;
        }
        check(
            Mode::ALL.iter().all(|mode| modes.contains(mode)),
            "harmony.candidates",
            "accords requis pour les quatre modes (dynamique parallèle)",
        )?;
        let a = &self.accompaniment;
        check(
            a.strong_weight > 0 && a.weak_weight > 0,
            "accompaniment.strong_weight",
            "poids de vote positifs requis",
        )?;
        for (name, steps, duration) in [
            ("bass_steps", &a.bass_steps, a.bass_duration_steps),
            ("chop_steps", &a.chop_steps, a.chop_duration_steps),
            ("strong_steps", &a.strong_steps, 0),
        ] {
            check(
                steps.iter().all(|&s| {
                    s.checked_add(duration)
                        .is_some_and(|end| s < m.steps_per_bar && end <= m.steps_per_bar)
                }),
                &format!("accompaniment.{name}"),
                "pas/durée hors mesure",
            )?;
        }
        check(
            a.bass_duration_steps > 0 && a.chop_duration_steps > 0,
            "accompaniment.bass_duration_steps",
            "durées positives requises",
        )?;
        check(
            (1..=127).contains(&a.bass_velocity) && (1..=127).contains(&a.chop_velocity),
            "accompaniment.bass_velocity",
            "vélocités dans 1..127",
        )?;
        check(
            a.bass_low <= a.bass_high && a.bass_high <= 127 && a.guitar_center <= 127,
            "accompaniment.bass_low",
            "registre MIDI invalide",
        )?;
        self.percussion.validate().map_err(|e| ProfileError::new("percussion", e))?;
        for (i, pattern) in self.percussion.patterns.iter().enumerate() {
            check(
                pattern.hits.iter().all(|h| h.step < m.steps_per_bar),
                &format!("percussion.patterns[{i}].hits"),
                "pas hors mesure",
            )?;
        }
        check(
            self.percussion.heel_note <= 127 && self.percussion.toe_note <= 127,
            "percussion.heel_note",
            "notes MIDI dans 0..127",
        )?;
        let i = &self.instruments;
        for (name, program) in [
            ("melody_program", i.melody_program),
            ("chords_program", i.chords_program),
            ("bass_program", i.bass_program),
            ("percussion_program", i.percussion_program),
        ] {
            check(program <= 127, &format!("instruments.{name}"), "programme MIDI dans 0..127")?;
        }
        check(i.percussion_bank <= 255, "instruments.percussion_bank", "banque SF2 dans 0..255")?;
        self.arrangement.validate().map_err(|error| {
            use crate::trad::ArrangementError as E;
            let field = match &error {
                E::ToursPerTuneOutOfRange(_) => "arrangement.tours_per_tune".into(),
                E::TourOutOfRange { track, .. } => format!("arrangement.entries.{track:?}.tour"),
                E::PassOutOfRange { track, .. } => format!("arrangement.entries.{track:?}.pass"),
                E::PickupEighthsOutOfRange(_) => "arrangement.intro.pickup_eighths".into(),
                E::IntroBarsOutOfRange { role, .. } | E::IntroWeightZero { role } => {
                    let index = self
                        .arrangement
                        .intro
                        .forms
                        .iter()
                        .position(|f| f.role == *role)
                        .unwrap_or(0);
                    let field =
                        if matches!(error, E::IntroWeightZero { .. }) { "weight" } else { "bars" };
                    format!("arrangement.intro.forms[{index}].{field}")
                }
                _ => "arrangement".into(),
            };
            ProfileError::new(field, error)
        })?;
        for track in crate::timeline::TrackId::ALL {
            let entry = self.arrangement.entry(track);
            let path = format!("arrangement.entries.{track:?}");
            check(
                entry.part.0 < self.form.parts.len(),
                &format!("{path}.part"),
                "partie inconnue",
            )?;
            check(
                entry.pass > 0
                    && self.form.sequence.iter().filter(|&&part| part == entry.part.0).count()
                        >= entry.pass,
                &format!("{path}.pass"),
                "passe absente du schéma joué",
            )?;
        }
        check(
            usize::from(self.arrangement.intro.pickup_eighths) <= self.melody.cell_units,
            "arrangement.intro.pickup_eighths",
            "la levée dépasse la mesure",
        )?;
        for (i, pattern) in self.dynamics.chop_patterns.iter().enumerate() {
            check(
                pattern.iter().all(|&step| {
                    step.checked_add(a.chop_duration_steps)
                        .is_some_and(|end| step < m.steps_per_bar && end <= m.steps_per_bar)
                }),
                &format!("dynamics.chop_patterns[{i}]"),
                "motif hors mesure",
            )?;
        }
        self.dynamics.validate().map_err(|error| {
            let field = error.split(':').next().unwrap_or("dynamics");
            ProfileError::new(format!("dynamics.{field}"), &error)
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn embedded_and_external_profiles_match() -> Result<(), Box<dyn std::error::Error>> {
        let embedded = StyleProfile::default();
        let external =
            StyleProfile::load(concat!(env!("CARGO_MANIFEST_DIR"), "/../profiles/reel_quebecois"))?;
        assert_eq!(embedded, external);
        assert_eq!(StyleProfile::from_toml(&embedded.to_toml()?)?, embedded);
        Ok(())
    }
    #[test]
    fn invalid_boundaries_name_the_field() {
        let mut p = StyleProfile::default();
        p.meter.steps_per_pulse = 0;
        assert_eq!(p.validate().unwrap_err().field, "meter.steps_per_pulse");
        p = StyleProfile::default();
        p.form.parts[0].roles[0] = ImposedRole::CopyOf(1);
        assert_eq!(p.validate().unwrap_err().field, "form.parts[0].roles[0]");
        p = StyleProfile::default();
        p.form.sequence.push(99);
        assert_eq!(p.validate().unwrap_err().field, "form.sequence[4]");
        p = StyleProfile::default();
        p.harmony.steps_per_chord = 0;
        assert_eq!(p.validate().unwrap_err().field, "harmony.steps_per_chord");
    }
}
