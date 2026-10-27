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
    timeline::TrackId,
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
    /// Noms courts acceptés par `--style` / `style` (ex. `reel`, `gigue`). Uniques parmi les
    /// profils distribués : un nom court désigne toujours un seul style.
    #[serde(default)]
    pub aliases: Vec<String>,
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
/// Rôle fixe historique ou opération/tirage décrit dans le profil.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum ProfileRole {
    Choice {
        choose: Vec<WeightedRole>,
    },
    Sequence {
        #[serde(rename = "SequenceOf")]
        sequence: SequenceRole,
    },
    CrossPart {
        #[serde(rename = "CopyOfPart")]
        source: PartReference,
    },
    Fixed(ImposedRole),
}
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeightedRole {
    pub role: ProfileRole,
    pub weight: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DegreeShift {
    pub degrees: i8,
    pub weight: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SequenceRole {
    pub bar: u8,
    pub shift: Vec<DegreeShift>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartReference {
    pub part: String,
    pub bar: u8,
}
impl From<ImposedRole> for ProfileRole {
    fn from(role: ImposedRole) -> Self {
        Self::Fixed(role)
    }
}
impl ProfileRole {
    pub fn fixed(&self) -> Option<ImposedRole> {
        match self {
            Self::Fixed(role) => Some(*role),
            _ => None,
        }
    }
    fn needs_cell_role(&self, role: crate::trad::CellRole) -> bool {
        match self {
            Self::Fixed(fixed) => fixed.forced_cell_role() == Some(role),
            Self::Choice { choose } => choose.iter().any(|c| c.role.needs_cell_role(role)),
            _ => false,
        }
    }
    fn uses_varied_end(&self) -> bool {
        match self {
            Self::Fixed(ImposedRole::VariedEndOf(_)) => true,
            Self::Choice { choose } => choose.iter().any(|c| c.role.uses_varied_end()),
            _ => false,
        }
    }
    pub(crate) fn draw(
        &self,
        form: &ProfileForm,
        rng: &mut dyn crate::algo::RngCore,
    ) -> ImposedRole {
        match self {
            Self::Fixed(role) => *role,
            Self::Choice { choose } => {
                let index = draw_weight(choose.iter().map(|c| c.weight), rng);
                choose[index].role.draw(form, rng)
            }
            Self::Sequence { sequence } => {
                let index = draw_weight(sequence.shift.iter().map(|s| s.weight), rng);
                ImposedRole::SequenceOf { bar: sequence.bar, shift: sequence.shift[index].degrees }
            }
            Self::CrossPart { source } => ImposedRole::CopyOfPart {
                part: form.parts.iter().position(|p| p.name == source.part).unwrap_or(0),
                bar: source.bar,
            },
        }
    }
    fn validate(
        &self,
        form: &ProfileForm,
        part: usize,
        bar: usize,
        field: &str,
    ) -> Result<(), ProfileError> {
        let previous = |source: u8| {
            check(
                source > 0 && usize::from(source) <= bar,
                field,
                "référence antérieure 1-based requise",
            )
        };
        match self {
            Self::Choice { choose } => {
                check(!choose.is_empty() && choose.len() <= 64, field, "1..64 choix requis")?;
                check_weights(choose.iter().map(|c| c.weight), field)?;
                for (index, choice) in choose.iter().enumerate() {
                    check(
                        !matches!(choice.role, Self::Choice { .. }),
                        field,
                        "choix imbriqués interdits",
                    )?;
                    choice.role.validate(
                        form,
                        part,
                        bar,
                        &format!("{field}.choose[{index}].role"),
                    )?;
                }
                Ok(())
            }
            Self::Sequence { sequence } => {
                previous(sequence.bar)?;
                check(
                    !sequence.shift.is_empty() && sequence.shift.len() <= 64,
                    field,
                    "1..64 décalages requis",
                )?;
                check(
                    sequence.shift.iter().all(|s| (-14..=14).contains(&s.degrees)),
                    field,
                    "décalage attendu -14..14",
                )?;
                check_weights(sequence.shift.iter().map(|s| s.weight), field)
            }
            Self::CrossPart { source } => check(
                form.parts[..part].iter().enumerate().any(|(index, p)| {
                    p.name == source.part
                        && source.bar > 0
                        && usize::from(source.bar) <= p.roles.len()
                        && form.sequence.iter().position(|&p| p == index)
                            < form.sequence.iter().position(|&p| p == part)
                }),
                field,
                "partie antérieure et mesure existante requises",
            ),
            Self::Fixed(ImposedRole::CopyOf(source) | ImposedRole::VariedEndOf(source)) => {
                previous(*source)
            }
            Self::Fixed(ImposedRole::SequenceOf { .. } | ImposedRole::CopyOfPart { .. }) => Err(
                ProfileError::new(field, "opération de motif attendue avec poids ou nom de partie"),
            ),
            Self::Fixed(_) => Ok(()),
        }
    }
}
fn check(ok: bool, field: &str, message: &str) -> Result<(), ProfileError> {
    if ok { Ok(()) } else { Err(ProfileError::new(field, message)) }
}
fn check_weights(mut weights: impl Iterator<Item = u32>, field: &str) -> Result<(), ProfileError> {
    let total = weights.try_fold(0u32, |sum, weight| sum.checked_add(weight));
    check(total.is_some_and(|total| total > 0), field, "somme des poids positive dans u32 requise")
}
fn draw_weight(
    weights: impl Iterator<Item = u32> + Clone,
    rng: &mut dyn crate::algo::RngCore,
) -> usize {
    let total: u32 = weights.clone().sum();
    let mut pick = rng.next_range_usize(0..total as usize) as u32;
    for (index, weight) in weights.enumerate() {
        if pick < weight {
            return index;
        }
        pick -= weight;
    }
    unreachable!("validated positive weights")
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoleResolution {
    #[default]
    PerTune,
    PerPass,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VariationOperation {
    VariedEnd,
    RepeatedNotes,
    Octave,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RhythmPattern {
    pub durations: Vec<usize>,
    pub weight: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PositionRhythm {
    pub bar: usize,
    pub patterns: Vec<RhythmPattern>,
}
fn default_true() -> bool {
    true
}
fn default_opening_retries() -> usize {
    8
}
fn default_contour_window() -> f32 {
    4.0
}
fn default_register_tolerance() -> f32 {
    6.0
}
fn default_leap() -> i8 {
    4
}
fn default_variation_operations() -> Vec<VariationOperation> {
    vec![
        VariationOperation::VariedEnd,
        VariationOperation::RepeatedNotes,
        VariationOperation::Octave,
    ]
}
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfilePart {
    pub name: String,
    pub roles: Vec<ProfileRole>,
    pub melody_center_offset: i8,
    /// Variante de la dernière mesure, jouée à la dernière reprise de la partie.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub second_ending: Option<ProfileRole>,
    /// Frontières explicites 1-based ; vide : frontières portées par les rôles cadentiels.
    #[serde(default)]
    pub phrase_ends: Vec<usize>,
    #[serde(default = "default_true")]
    pub link_previous: bool,
    #[serde(default)]
    pub rhythm: Vec<PositionRhythm>,
    /// Mesures sans aucun ornement (cadences). Omission : protéger la dernière mesure ;
    /// [] : aucune protection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ornament_protected_bars: Option<Vec<usize>>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileForm {
    pub parts: Vec<ProfilePart>,
    pub sequence: Vec<usize>,
    #[serde(default)]
    pub resolve: RoleResolution,
    #[serde(default)]
    pub repeat_variation: f32,
    #[serde(default)]
    pub tour_variation: f32,
    #[serde(default = "default_variation_operations")]
    pub variation_operations: Vec<VariationOperation>,
    #[serde(default = "default_true")]
    pub avoid_adjacent: bool,
    #[serde(default = "default_opening_retries")]
    pub opening_retries: usize,
    #[serde(default = "default_contour_window")]
    pub contour_window_semitones: f32,
    #[serde(default = "default_register_tolerance")]
    pub register_tolerance_semitones: f32,
    #[serde(default = "default_leap")]
    pub max_leap_degrees: i8,
    #[serde(default = "default_true")]
    pub close_part_loop: bool,
    #[serde(default = "default_half")]
    pub varied_end_keep: f32,
    #[serde(default = "default_two_units")]
    pub cadence_min_units: usize,
    #[serde(default = "default_half")]
    pub cadence_search_from: f32,
}
impl ProfileForm {
    pub(crate) fn varied_end_split(&self, units: usize) -> usize {
        ((units as f32 * self.varied_end_keep) as usize).clamp(1, units.saturating_sub(1).max(1))
    }
    pub(crate) fn expanded_material(&self) -> bool {
        self.resolve == RoleResolution::PerPass
            || self.repeat_variation > 0.0
            || self.tour_variation > 0.0
    }
    pub(crate) fn phrase_end(&self, part: usize, offset: usize, role: ImposedRole) -> bool {
        let ends = &self.parts[part].phrase_ends;
        if ends.is_empty() {
            matches!(
                role,
                ImposedRole::CadHalf
                    | ImposedRole::CadFull
                    | ImposedRole::CadTonic
                    | ImposedRole::CadOpen
                    | ImposedRole::CadFifth
                    | ImposedRole::CadOther
            )
        } else {
            ends.contains(&(offset + 1))
        }
    }
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
fn default_half() -> f32 {
    0.5
}
fn default_two_units() -> usize {
    2
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub candidates: Vec<ModeChords>,
    /// Règles et poids du modèle jazz ; les profils trad ne les consultent pas.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<crate::models::jazz::RewriteRule>,
    #[serde(default)]
    pub max_recursion: u8,
    #[serde(default)]
    pub scale_guidance: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileInstruments {
    pub melody_program: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub countermelody_program: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chords_program: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bass_program: Option<u8>,
    pub percussion_bank: u16,
    pub percussion_program: u8,
}
impl ProfileInstruments {
    pub const fn program(&self, track: TrackId) -> Option<u8> {
        match track {
            TrackId::Melody => Some(self.melody_program),
            TrackId::Countermelody => self.countermelody_program,
            TrackId::Chords => self.chords_program,
            TrackId::Bass => self.bass_program,
            TrackId::Heel | TrackId::Toe => Some(self.percussion_program),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StyleProfile {
    pub meta: ProfileMeta,
    pub meter: ProfileMeter,
    pub form: ProfileForm,
    pub melody: ProfileMelody,
    pub harmony: ProfileHarmony,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accompaniment: Option<TradHarmonyParams>,
    pub percussion: FeetTable,
    pub instruments: ProfileInstruments,
    pub arrangement: ReelArrangement,
    pub dynamics: StyleDynamics,
    #[serde(default)]
    pub mix: crate::mix::ProfileMix,
    #[serde(default)]
    pub performance: crate::performance::Performance,
    #[serde(default)]
    pub ornaments: crate::ornaments::Ornaments,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heterophony: Option<crate::heterophony::Heterophony>,
    /// Forme longue : tonalités, raccords, clôture, rampe et mémoire d'airs (`set.toml`).
    #[serde(default)]
    pub set: crate::trad::set::ProfileSet,
}
impl StyleProfile {
    /// Vrai si le profil déclare et active la piste (non nulle, modèle harmonique cohérent).
    pub fn has_track(&self, track: TrackId) -> bool {
        match track {
            TrackId::Melody => true,
            TrackId::Countermelody => {
                self.heterophony.is_some()
                    && self.instruments.countermelody_program.is_some()
                    && self.arrangement.has_track(track)
            }
            TrackId::Chords => {
                self.harmony.model != "none"
                    && self.instruments.chords_program.is_some()
                    && self.arrangement.has_track(TrackId::Chords)
            }
            TrackId::Bass => {
                self.harmony.model != "none"
                    && self.instruments.bass_program.is_some()
                    && self.arrangement.has_track(TrackId::Bass)
            }
            TrackId::Heel => self.arrangement.has_track(TrackId::Heel),
            TrackId::Toe => self.arrangement.has_track(TrackId::Toe),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileError {
    pub field: String,
    pub message: String,
}
impl ProfileError {
    pub(crate) fn new(field: impl Into<String>, message: impl ToString) -> Self {
        Self { field: field.into(), message: message.to_string() }
    }
}
impl fmt::Display for ProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}
impl std::error::Error for ProfileError {}

#[derive(Deserialize)]
struct DistributedBundle {
    profiles: Vec<String>,
}

const DISTRIBUTED_BUNDLE_TOML: &str =
    include_str!(concat!(env!("OUT_DIR"), "/distributed_profiles.toml"));

impl Default for StyleProfile {
    fn default() -> Self {
        Self::builtin().clone()
    }
}
impl StyleProfile {
    /// Profils distribués embarqués avec le moteur.
    pub fn distributed() -> &'static [Self] {
        static PROFILES: LazyLock<Vec<StyleProfile>> = LazyLock::new(|| {
            let bundle: Result<DistributedBundle, _> = toml::from_str(DISTRIBUTED_BUNDLE_TOML);
            match bundle {
                Ok(b) => b
                    .profiles
                    .iter()
                    .filter_map(|text| StyleProfile::from_toml(text).ok())
                    .collect(),
                Err(_) => Vec::new(),
            }
        });
        &PROFILES
    }
    /// Profil embarqué partagé ; aucun accès au disque ni copie de banque.
    pub fn builtin() -> &'static Self {
        &Self::distributed()[0]
    }
    /// Résolution d'un profil distribué par identifiant exact, puis par alias déclaré dans
    /// `[meta] aliases`. Pas de déduction par préfixe ni par tag : avec plusieurs traditions,
    /// un nom court comme `gigue` serait ambigu.
    pub fn named(id: &str) -> Result<Self, ProfileError> {
        let distributed = Self::distributed();
        distributed
            .iter()
            .find(|p| p.meta.id.eq_ignore_ascii_case(id))
            .or_else(|| {
                distributed
                    .iter()
                    .find(|p| p.meta.aliases.iter().any(|a| a.eq_ignore_ascii_case(id)))
            })
            .cloned()
            .ok_or_else(|| ProfileError::new("meta.id", format!("profil inconnu : {id}")))
    }
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ProfileError> {
        let path = path.as_ref();
        let file = if path.is_dir() { path.join("profile.toml") } else { path.to_path_buf() };
        let text = std::fs::read_to_string(&file)
            .map_err(|e| ProfileError::new(file.display().to_string(), e))?;
        Self::decode(&text, file.parent())
    }
    /// Lecture d'un profil autonome (banques inline, sans accès au disque).
    pub fn from_toml(text: &str) -> Result<Self, ProfileError> {
        Self::decode(text, None)
    }
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }
    fn decode(text: &str, base: Option<&Path>) -> Result<Self, ProfileError> {
        let mut root: toml::Value =
            toml::from_str(text).map_err(|e| ProfileError::new("profile.toml", e))?;
        let files = root.as_table_mut().and_then(|t| t.remove("files"));
        if let Some(files) = files {
            let files =
                files.as_table().ok_or_else(|| ProfileError::new("files", "table attendue"))?;
            for (key, filename) in files {
                let target = match key.as_str() {
                    "cells" => "melody.cells",
                    "feet" => "percussion",
                    "arrangement" => "arrangement",
                    "dynamics" => "dynamics",
                    "set" => "set",
                    _ => return Err(ProfileError::new(format!("files.{key}"), "banque inconnue")),
                };
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
                let base = base.ok_or_else(|| {
                    ProfileError::new(
                        format!("files.{key}"),
                        "référence externe : utiliser load(dossier)",
                    )
                })?;
                let contents = std::fs::read_to_string(base.join(relative))
                    .map_err(|e| ProfileError::new(format!("files.{key}"), e))?;
                let value = toml::from_str(&contents)
                    .map_err(|e| ProfileError::new(format!("files.{key}"), e))?;
                if target == "melody.cells" {
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
        self.mix.validate()?;
        self.performance.validate(self.meter.steps_per_bar)?;
        self.ornaments.validate()?;
        if let Some(h) = &self.heterophony {
            h.validate()?;
            check(
                self.instruments.countermelody_program.is_some(),
                "instruments.countermelody_program",
                "programme requis avec heterophony",
            )?;
            check(
                self.arrangement.has_track(TrackId::Countermelody),
                "arrangement.entries.Countermelody",
                "entrée requise avec heterophony",
            )?;
        }
        self.set.validate()?;
        if let Some(ceiling) = self.set.ramp.as_ref().and_then(|ramp| ramp.ceiling_bpm) {
            check(
                ceiling >= self.meter.min_bpm,
                "set.ramp.ceiling_bpm",
                "plafond inférieur à meter.min_bpm",
            )?;
        }
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
        for (name, value) in [
            ("repeat_variation", self.form.repeat_variation),
            ("tour_variation", self.form.tour_variation),
        ] {
            check(
                value.is_finite() && (0.0..=1.0).contains(&value),
                &format!("form.{name}"),
                "probabilité finie attendue dans 0..1",
            )?;
        }
        check(
            !self.form.variation_operations.is_empty()
                && self.form.variation_operations.iter().collect::<HashSet<_>>().len()
                    == self.form.variation_operations.len(),
            "form.variation_operations",
            "opérations non vides et uniques requises",
        )?;
        check(self.form.opening_retries <= 10000, "form.opening_retries", "attendu 0..10000")?;
        for (name, value) in [
            ("contour_window_semitones", self.form.contour_window_semitones),
            ("register_tolerance_semitones", self.form.register_tolerance_semitones),
        ] {
            check(
                value.is_finite() && (0.0..=127.0).contains(&value),
                &format!("form.{name}"),
                "attendu 0..127 fini",
            )?;
        }
        check(
            (0..=28).contains(&self.form.max_leap_degrees),
            "form.max_leap_degrees",
            "attendu 0..28",
        )?;
        check(
            self.form.varied_end_keep.is_finite()
                && self.form.varied_end_keep > 0.0
                && self.form.varied_end_keep < 1.0,
            "form.varied_end_keep",
            "fraction finie strictement entre 0 et 1 requise",
        )?;
        check(
            self.form.cadence_min_units > 0
                && self.form.cadence_min_units <= self.melody.cell_units,
            "form.cadence_min_units",
            "durée positive au plus égale à melody.cell_units requise",
        )?;
        check(
            self.form.cadence_search_from.is_finite()
                && (0.0..=1.0).contains(&self.form.cadence_search_from),
            "form.cadence_search_from",
            "fraction finie attendue dans 0..1",
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
            check(
                part.phrase_ends.iter().all(|&bar| bar > 0 && bar <= part.roles.len())
                    && part.phrase_ends.windows(2).all(|w| w[0] < w[1]),
                &format!("{field}.phrase_ends"),
                "frontières 1-based strictement croissantes dans la partie",
            )?;
            if let Some(bars) = &part.ornament_protected_bars {
                check(
                    bars.iter().all(|&bar| bar > 0 && bar <= part.roles.len())
                        && bars.windows(2).all(|w| w[0] < w[1]),
                    &format!("{field}.ornament_protected_bars"),
                    "mesures protégées 1-based strictement croissantes dans la partie",
                )?;
            }
            let mut rhythm_bars = HashSet::new();
            for (r, rhythm) in part.rhythm.iter().enumerate() {
                let rhythm_field = format!("{field}.rhythm[{r}]");
                check(
                    rhythm.bar > 0
                        && rhythm.bar <= part.roles.len()
                        && rhythm_bars.insert(rhythm.bar),
                    &rhythm_field,
                    "position 1-based existante et unique requise",
                )?;
                check(
                    !rhythm.patterns.is_empty() && rhythm.patterns.len() <= 256,
                    &rhythm_field,
                    "1..256 motifs rythmiques requis",
                )?;
                check_weights(rhythm.patterns.iter().map(|p| p.weight), &rhythm_field)?;
                for pattern in &rhythm.patterns {
                    check(
                        !pattern.durations.is_empty()
                            && pattern.durations.iter().all(|&d| d > 0)
                            && pattern
                                .durations
                                .iter()
                                .try_fold(0usize, |sum, &d| sum.checked_add(d))
                                == Some(self.melody.cell_units),
                        &rhythm_field,
                        "durées positives en unités de cellule couvrant exactement la mesure",
                    )?;
                }
            }
            for (bar, role) in part.roles.iter().enumerate() {
                role.validate(&self.form, i, bar, &format!("{field}.roles[{bar}]"))?;
            }
            if let Some(ending) = &part.second_ending {
                check(
                    self.form.sequence.iter().filter(|&&part| part == i).count() >= 2,
                    &format!("{field}.second_ending"),
                    "une seconde fin exige au moins deux passes",
                )?;
                ending.validate(
                    &self.form,
                    i,
                    part.roles.len() - 1,
                    &format!("{field}.second_ending"),
                )?;
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
        if self
            .form
            .parts
            .iter()
            .flat_map(|p| p.roles.iter().chain(p.second_ending.iter()))
            .any(ProfileRole::uses_varied_end)
        {
            check(
                self.melody.cell_units >= 2,
                "melody.cell_units",
                "VariedEndOf exige au moins deux unités",
            )?;
            let split = self.form.varied_end_split(self.melody.cell_units);
            let endings = self
                .melody
                .cells
                .cells
                .iter()
                .filter(|cell| matches!(cell.notes[split], crate::trad::CellNote::Degree(_)));
            check(
                split > 0
                    && endings.clone().any(|a| {
                        endings.clone().any(|b| {
                            !crate::trad::fiddle::same_degree_classes(
                                &a.notes[split..],
                                &b.notes[split..],
                            )
                        })
                    }),
                "melody.cells",
                "VariedEndOf exige deux fins distinctes avec attaque à mi-mesure",
            )?;
        }
        for role in crate::trad::CellRole::ALL {
            let needed = self
                .form
                .parts
                .iter()
                .flat_map(|p| p.roles.iter().chain(p.second_ending.iter()))
                .any(|r| r.needs_cell_role(role))
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
            matches!(self.harmony.model.as_str(), "trad_functional" | "jazz_functional" | "none"),
            "harmony.model",
            "modèles disponibles : trad_functional, jazz_functional, none",
        )?;
        check(
            self.harmony.steps_per_chord > 0
                && m.steps_per_bar.is_multiple_of(self.harmony.steps_per_chord),
            "harmony.steps_per_chord",
            "doit diviser steps_per_bar",
        )?;
        if self.harmony.model == "jazz_functional" {
            check(self.harmony.max_recursion <= 16, "harmony.max_recursion", "attendu 0..16")?;
            check(
                !self.harmony.rules.is_empty() && self.harmony.rules.len() <= 256,
                "harmony.rules",
                "1..256 règles requises pour jazz_functional",
            )?;
            let mut total = 0.0f32;
            for (i, rule) in self.harmony.rules.iter().enumerate() {
                check(
                    rule.weight.is_finite()
                        && rule.weight >= 0.0
                        && !rule.rhs.is_empty()
                        && rule.rhs.len() <= 64
                        && rule.min_valence.is_finite()
                        && rule.max_valence.is_finite()
                        && (-1.0..=1.0).contains(&rule.min_valence)
                        && (rule.min_valence..=1.0).contains(&rule.max_valence),
                    &format!("harmony.rules[{i}]"),
                    "poids fini positif ou nul, 1..64 symboles, valences ordonnées dans -1..1",
                )?;
                total += rule.weight
                    * crate::models::jazz::GrammarStyle::Jazz.category_weight(rule.category);
            }
            check(
                total.is_finite() && total > 0.0,
                "harmony.rules",
                "somme des poids effectifs finie positive",
            )?;
        }
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
        // Modes atteignables : `key_modes` (valence neutre) et `color`
        // (ailleurs), fermés par les bascules parallèles dans les deux sens.
        // Eux seuls exigent accords candidats et ligne de bascule : un profil
        // qui n'emploie ni le phrygien ni le mineur harmonique reste valide
        // sans eux, et leurs notes ne changent pas.
        let mut reachable = std::collections::HashSet::new();
        reachable.extend(self.melody.key_modes.iter().map(|k| k.mode));
        reachable.extend(self.dynamics.color.iter().map(|c| c.mode));
        loop {
            let mut fringe = Vec::new();
            for row in &self.dynamics.parallel_modes {
                if reachable.contains(&row.bright) {
                    fringe.push(row.dark);
                }
                if reachable.contains(&row.dark) {
                    fringe.push(row.bright);
                }
            }
            if fringe.iter().all(|mode| reachable.contains(mode)) {
                break;
            }
            reachable.extend(fringe);
        }
        check(
            self.harmony.model != "trad_functional"
                || reachable.iter().all(|mode| modes.contains(mode)),
            "harmony.candidates",
            "accords requis pour chaque mode atteignable (key_modes, color, parallel_modes)",
        )?;
        for mode in &reachable {
            if mode.is_minor() {
                check(
                    self.dynamics.parallel_modes.iter().any(|c| c.dark == *mode),
                    "dynamics.parallel_modes",
                    "chaque mode sombre atteignable requiert sa bascule depuis un mode clair",
                )?;
            } else {
                check(
                    self.dynamics.parallel_modes.iter().any(|c| c.bright == *mode),
                    "dynamics.parallel_modes",
                    "chaque mode clair atteignable requiert sa bascule vers un mode sombre",
                )?;
            }
        }
        if let Some(a) = &self.accompaniment {
            a.validate_accompaniment(m.steps_per_bar, self.harmony.model == "jazz_functional")?;
        } else {
            check(
                self.harmony.model == "none",
                "accompaniment",
                "section accompaniment requise sauf pour le modèle d'harmonie none",
            )?;
        }
        self.percussion.validate().map_err(|e| ProfileError::new("percussion", e))?;
        for (i, pattern) in self
            .percussion
            .patterns
            .iter()
            .chain(self.percussion.cadence_patterns.iter())
            .enumerate()
        {
            check(
                pattern.euclidean_steps().is_none_or(|steps| steps == m.steps_per_bar),
                &format!("percussion.patterns[{i}].euclid.steps"),
                "doit égaler meter.steps_per_bar",
            )?;
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
        check(i.melody_program <= 127, "instruments.melody_program", "programme MIDI dans 0..127")?;
        if let Some(program) = i.countermelody_program {
            check(
                program <= 127,
                "instruments.countermelody_program",
                "programme MIDI dans 0..127",
            )?;
        }
        if let Some(chords) = i.chords_program {
            check(chords <= 127, "instruments.chords_program", "programme MIDI dans 0..127")?;
        }
        if let Some(bass) = i.bass_program {
            check(bass <= 127, "instruments.bass_program", "programme MIDI dans 0..127")?;
        }
        check(
            i.percussion_program <= 127,
            "instruments.percussion_program",
            "programme MIDI dans 0..127",
        )?;
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
            if let Some(entry) = self.arrangement.entry_opt(track) {
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
            } else {
                check(
                    !self.has_track(track),
                    &format!("arrangement.entries.{track:?}"),
                    "piste obligatoire manquante dans l'arrangement",
                )?;
            }
        }
        check(
            usize::from(self.arrangement.intro.pickup_eighths) <= self.melody.cell_units,
            "arrangement.intro.pickup_eighths",
            "la levée dépasse la mesure",
        )?;
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
        p.form.parts[0].roles[0] = ImposedRole::CopyOf(1).into();
        assert_eq!(p.validate().unwrap_err().field, "form.parts[0].roles[0]");
        p = StyleProfile::default();
        p.form.sequence.push(99);
        assert_eq!(p.validate().unwrap_err().field, "form.sequence[4]");
        p = StyleProfile::default();
        p.harmony.steps_per_chord = 0;
        assert_eq!(p.validate().unwrap_err().field, "harmony.steps_per_chord");
    }
}
