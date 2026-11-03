//! Données et validation des deux voix d'accompagnement.
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::{
    algo::RngCore,
    compose::{
        arc::{TagWeight, tagged_weight},
        dynamics::Curve,
        phrase::{Candidate, MAX_PHRASE_BARS, MAX_PHRASES, PhraseAt, Slot, Window, tile},
    },
    ornaments::mix,
    style_profile::ProfileError,
    timeline::TrackId,
    tuning::AccompanimentParams,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BassDegree {
    Root,
    Fifth,
    FifthIfHeld,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BassDoubling {
    Octave,
    Tenth,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BassHit {
    pub step: usize,
    pub degree: BassDegree,
    pub duration: usize,
    pub velocity: u8,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub doubling: Vec<BassDoubling>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BassReference {
    #[default]
    ChordRoot,
    Tonic,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BassMotifNote {
    pub step: usize,
    pub duration: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semitones: Option<i8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub degree: Option<i8>,
    #[serde(default)]
    pub octave: i8,
    pub velocity: u8,
    #[serde(default)]
    pub tie: bool,
    /// Legato monophonique : chauffe la note tenue du canal au lieu de la
    /// détacher (voir `Articulation::Legato`). Indépendant de `tie`, qui lie
    /// une même hauteur franchissant la barre.
    #[serde(default)]
    pub legato: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub doubling: Vec<BassDoubling>,
}

const fn default_one_bar() -> usize {
    1
}

const fn default_motif_weight() -> u32 {
    1
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BassMotif {
    #[serde(default = "default_one_bar")]
    pub bars: usize,
    #[serde(default = "default_motif_weight")]
    pub weight: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part: Option<String>,
    #[serde(default)]
    pub reference: BassReference,
    pub notes: Vec<BassMotifNote>,
}

/// Gabarit de basse : un rythme d'une mesure et une règle de hauteur, réalisés sur l'accord
/// courant au pas près. Alternative pondérée à côté des `motifs` écrits (même pool de tirage,
/// même portée `part`). Exclusif de `hits`.
///
/// - `cycle` : hauteurs jouées tour à tour par les attaques de `rhythm` d'une mesure à l'autre
///   (fondamentale/quinte, octaves) ; omis : la basse de l'accord.
/// - `ramp` : décalage ajouté par mesure du cycle (montée ou descente).
/// - `pickup` : relance, note courte de la basse de l'accord suivant avant le changement.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BassTemplate {
    #[serde(default = "default_motif_weight")]
    pub weight: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part: Option<String>,
    /// Mesures du cycle de `ramp` (`1..=16`) ; la montée repart au début du cycle.
    #[serde(default = "default_one_bar")]
    pub bars: usize,
    pub rhythm: Vec<TemplateHit>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cycle: Vec<TemplateNote>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ramp: Option<TemplateRamp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pickup: Option<TemplatePickup>,
}

/// Attaque d'un gabarit : pas de la mesure, durée, vélocité.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateHit {
    pub step: usize,
    pub duration: usize,
    pub velocity: u8,
}

/// Hauteur d'une attaque : degré de l'accord courant et octave relative.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateNote {
    pub degree: BassDegree,
    #[serde(default)]
    pub octave: i8,
}

/// Montée ou descente : `stride` unités de `motion` ajoutées à chaque mesure du cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateRamp {
    /// Pas par mesure, signé et non nul : positif monte, négatif descend.
    pub stride: i8,
    #[serde(default)]
    pub motion: RampMotion,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RampMotion {
    /// Demi-tons.
    #[default]
    Semitone,
    /// Notes de la gamme du mode actif.
    ScaleStep,
}

/// Relance : note de la basse de l'accord suivant (`degree`) jouée `lead` pas avant le
/// changement ; les attaques du rythme qu'elle recouvre sont retirées ou raccourcies.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplatePickup {
    pub lead: usize,
    pub duration: usize,
    pub velocity: u8,
    #[serde(default = "default_pickup_degree")]
    pub degree: BassDegree,
    #[serde(default)]
    pub when: PickupWhen,
}

const fn default_pickup_degree() -> BassDegree {
    BassDegree::Root
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PickupWhen {
    /// Avant chaque changement d'accord, y compris au passage à la mesure suivante.
    #[default]
    ChordChange,
    /// Avant chaque barre de mesure, même si l'accord ne change pas ; aucun changement
    /// interne à la mesure.
    BarEnd,
}

/// Marche de basse : une attaque par pulsation, arrivée sur la basse de l'accord à chaque
/// changement d'accord et au premier temps, approche de la cible suivante, notes de passage
/// tirées par poids. Source exclusive des coups (`hits`), des motifs (`motifs`) et des gabarits
/// (`templates`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BassWalking {
    pub velocity: u8,
    /// Surcroît de vélocité sur les arrivées (premier temps et changements d'accord).
    #[serde(default)]
    pub accent: u8,
    /// Durée de chaque note en pas, au plus une pulsation.
    pub duration: usize,
    /// Écart maximal en demi-tons entre deux notes de passage consécutives.
    pub max_leap: u8,
    /// Poids ajouté (à 1) à une note de passage qui se rapproche de la cible.
    #[serde(default = "default_toward_target_weight")]
    pub toward_target_weight: u32,
    /// Poids ajouté (à 1) à une note de passage qui poursuit le sens du mouvement précédent.
    #[serde(default = "default_keep_direction_weight")]
    pub keep_direction_weight: u32,
    pub approach: WalkingApproach,
    pub passing: WalkingPassing,
    /// Arrivée sur une autre note de l'accord que sa basse publiée (tierce, quinte, septième).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arrival: Option<WalkingArrival>,
    /// Croches de passage et notes fantômes entre deux pulsations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eighths: Option<WalkingEighths>,
    /// Deuxième attaque du feel à deux temps : la quinte plutôt qu'une note de passage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub two_feel: Option<WalkingTwoFeel>,
}

const fn default_toward_target_weight() -> u32 {
    3
}

const fn default_keep_direction_weight() -> u32 {
    2
}

/// Poids de l'approche de la pulsation qui précède une arrivée. Sans `side`, l'approche se pose
/// du côté de la note d'arrivée précédente : au-dessus de la cible si la ligne part plus haut,
/// sinon dessous.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WalkingApproach {
    /// Aucune approche : la pulsation est une note de passage ordinaire.
    #[serde(default)]
    pub none_weight: u32,
    /// Un demi-ton sous ou sur la cible.
    #[serde(default)]
    pub chromatic_weight: u32,
    /// La note de la gamme du mode voisine de la cible.
    #[serde(default)]
    pub diatonic_weight: u32,
    /// Côté de l'approche, tiré à chaque approche.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub side: Option<ApproachSide>,
}

/// Côté de l'approche d'une cible : poids de quatre manières de le choisir. Le côté retenu est
/// essayé d'abord ; la fenêtre de marche qui ne l'offre pas laisse l'autre côté.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApproachSide {
    /// Côté de l'arrivée précédente (au-dessus de la cible si la ligne part plus haut) : la règle
    /// historique.
    #[serde(default)]
    pub from_weight: u32,
    /// Côté opposé à l'arrivée précédente.
    #[serde(default)]
    pub other_weight: u32,
    /// Toujours au-dessus de la cible.
    #[serde(default)]
    pub above_weight: u32,
    /// Toujours sous la cible.
    #[serde(default)]
    pub below_weight: u32,
}

fn is_zero(value: &u32) -> bool {
    *value == 0
}

/// Poids des notes de passage entre l'arrivée et l'approche.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WalkingPassing {
    /// Note de l'accord en cours (tensions comprises).
    #[serde(default)]
    pub chord_tone_weight: u32,
    /// Note de la gamme du mode, à un pas diatonique de la note précédente.
    #[serde(default)]
    pub scale_step_weight: u32,
    /// Chromatisme de passage : un demi-ton de la note précédente, hors de la gamme du mode et
    /// de l'accord.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub chromatic_weight: u32,
    /// Note répétée : la note précédente rejouée.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub repeat_weight: u32,
}

/// Arrivée sur une note de l'accord : poids de la basse publiée (la fondamentale, ou la basse
/// d'un accord slash) et des autres degrés, tirés par arrivée (changement d'accord et premier
/// temps). Un accord slash, une pédale ou un accord sans le degré demandé gardent la basse
/// publiée : la grille de l'utilisateur n'est jamais contredite (D4). La note choisie est
/// celle de l'arrivée et de l'approche qui la précède, d'une mesure à l'autre.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WalkingArrival {
    /// La basse de l'accord.
    #[serde(default)]
    pub root_weight: u32,
    #[serde(default)]
    pub third_weight: u32,
    #[serde(default)]
    pub fifth_weight: u32,
    #[serde(default)]
    pub seventh_weight: u32,
}

/// Croches entre deux pulsations de la marche : une pulsation peut recevoir, après sa note, une
/// croche de passage (approche de la note suivante, au demi-ton ou à la gamme) ou une note
/// fantôme (la même hauteur, légère). Seulement quand la note suivante tombe à la pulsation
/// suivante (feel de marche) et que la pulsation a un nombre pair de pas.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WalkingEighths {
    /// Pulsation sans croche.
    #[serde(default)]
    pub none_weight: u32,
    /// Croche de passage : approche chromatique de la note suivante.
    #[serde(default)]
    pub chromatic_weight: u32,
    /// Croche de passage : approche par la gamme du mode.
    #[serde(default)]
    pub diatonic_weight: u32,
    /// Note fantôme.
    #[serde(default)]
    pub ghost_weight: u32,
    /// Vélocité d'une croche de passage.
    pub velocity: u8,
    /// Vélocité d'une note fantôme.
    pub ghost_velocity: u8,
}

/// Feel à deux temps : poids de la quinte de l'accord (fondamentale-quinte) contre une note de
/// passage ordinaire à la deuxième attaque d'une étendue d'accord qui n'est pas une approche.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WalkingTwoFeel {
    #[serde(default)]
    pub passing_weight: u32,
    #[serde(default)]
    pub fifth_weight: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BassVoice {
    pub low: u8,
    pub high: u8,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hits: Vec<BassHit>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub motifs: Vec<BassMotif>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub templates: Vec<BassTemplate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub walk_low: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub walk_high: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub passing: Option<BassPassing>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub walking: Option<BassWalking>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChordHit {
    pub step: usize,
    pub duration: usize,
    pub velocity: u8,
    #[serde(default)]
    pub direction: AttackDirection,
    #[serde(default = "unit_multiplier")]
    pub probability: f32,
    /// Liaison : pas de nouvelle attaque si les hauteurs sonnent encore à
    /// l'arrivée (nappe). Sans coïncidence exacte, attaque normale.
    #[serde(default)]
    pub tie: bool,
    /// Attaque anticipée : joue l'accord en vigueur à la fin de sa durée (`step + duration`),
    /// donc le prochain accord quand un changement tombe à cette fin.
    #[serde(default, skip_serializing_if = "is_false")]
    pub anticipate: bool,
}

fn is_false(value: &bool) -> bool {
    !value
}

/// Rythme de comping d'une mesure, tiré par poids. Sans attaque : la mesure est silencieuse.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompingMotif {
    #[serde(default = "default_motif_weight")]
    pub weight: u32,
    #[serde(default)]
    pub hits: Vec<ChordHit>,
    /// Étiquette que `[arc]` nomme pour choisir parmi les motifs (`comping[].tag`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

fn is_any(at: &PhraseAt) -> bool {
    matches!(at, PhraseAt::Any)
}

/// Phrase de comping : une suite de motifs de mesure (indices dans `comping`) jouée sur
/// plusieurs mesures. Chaque passe de partie se découpe en phrases tirées par poids, sans état
/// d'une mesure à l'autre (voir `phrase.rs`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompingPhrase {
    #[serde(default = "default_motif_weight")]
    pub weight: u32,
    /// Portée optionnelle : la phrase ne concourt que pour cette partie (`form.parts[].name`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part: Option<String>,
    /// Étiquette que `[arc]` nomme pour choisir parmi les phrases.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// Emplacement dans la passe de la partie : `any`, `part_start`, `part_end` (mise en place
    /// avant un changement de section) ou `chorus_end`.
    #[serde(default, skip_serializing_if = "is_any")]
    pub at: PhraseAt,
    /// Un motif par mesure : indices dans `comping`. 1 à 8 mesures.
    pub motifs: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChordPattern {
    pub hits: Vec<ChordHit>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChordVoice {
    pub center: u8,
    pub max_drift: u8,
    pub default_pattern: usize,
    pub patterns: Vec<ChordPattern>,
    /// Rythmes tirés par mesure (poids) ; remplacent le choix du motif par intensité.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub comping: Vec<CompingMotif>,
    /// Phrases de plusieurs mesures, suites de motifs de `comping` ; remplacent le tirage d'un
    /// motif par mesure là où une phrase tient dans la passe de la partie.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub phrases: Vec<CompingPhrase>,
    #[serde(default = "triad_note_count")]
    pub note_count: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spread: Option<Spread>,
    #[serde(default)]
    pub voicing: ChordVoicing,
    #[serde(default = "fixed_low")]
    pub low: u8,
    #[serde(default = "fixed_high")]
    pub high: u8,
    /// Tension de couleur tirée par graine parmi celles que la qualité permet ; sans cette
    /// section, la tension de la qualité qui passe en premier selon la priorité historique.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<ColorTensions>,
    /// Dérive du registre d'une mesure à l'autre, bornée autour de `center`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub register_drift: Option<RegisterDrift>,
}

/// Poids de la tension de couleur d'un accord, tirée par graine (mesure et créneau d'accord) :
/// `none` : aucune tension ; `published` : une tension de la qualité publiée (priorité 9, 13, #11,
/// b9, #9, b13, 11) ; 9, 11 et 13 : une tension naturelle que la qualité permet **sans la
/// publier**. Une tension naturelle n'est tirée que si la qualité l'autorise (tableau de
/// `voicing.rs`), si elle ne double pas un son de l'accord, si la qualité n'altère pas déjà ce
/// degré (b9 ou #9 pour la 9, #11 pour la 11, b13 pour la 13) et si elle ne tombe pas un demi-ton
/// au-dessus d'un son de l'accord (note à éviter) : jamais en conflit avec la grille (D4). Un
/// poids inéligible vaut 0 ; sans poids éligible, la tension historique.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColorTensions {
    #[serde(default)]
    pub none_weight: u32,
    #[serde(default)]
    pub published_weight: u32,
    #[serde(default)]
    pub ninth_weight: u32,
    #[serde(default)]
    pub eleventh_weight: u32,
    #[serde(default)]
    pub thirteenth_weight: u32,
}

/// Dérive du registre : à chaque mesure, le centre des voicings se déplace de `step` demi-tons au
/// plus (haut, bas ou rien, tiré par graine) sans jamais s'éloigner de plus de `max` demi-tons de
/// `center`. La dérive repart de zéro au début de chaque passe de partie (les voicings sont
/// rejoués depuis ce point, d'où le rejeu à l'identique après un seek).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegisterDrift {
    pub step: u8,
    pub max: u8,
}

fn triad_note_count() -> u8 {
    3
}

fn fixed_low() -> u8 {
    52
}
fn fixed_high() -> u8 {
    64
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChordVoicing {
    #[default]
    CloseTriad,
    FixedTriad,
    /// Notes guides : tierce et septième (ou sixte), plus une tension de la qualité si elle en a.
    GuideTones,
    /// Quatre sons sans fondamentale, types A (tierce en bas) et B (septième en bas) ;
    /// la conduite de voix choisit celui qui bouge le moins.
    Rootless,
    /// Type A seulement : tierce en bas.
    RootlessA,
    /// Type B seulement : septième en bas.
    RootlessB,
    /// Quatre sons en position serrée dont la deuxième voix depuis l'aigu descend d'une octave.
    Drop2,
    /// Quatre sons en position serrée dont la troisième voix depuis l'aigu descend d'une octave.
    Drop3,
    /// Quartes justes empilées (*So What*) : quatre sons, une pile de quartes dans les sons de
    /// la qualité qui garde sa tierce ou sa septième.
    Fourths,
    /// Structure supérieure : une triade (majeure ou mineure) bâtie sur un degré de la qualité,
    /// que la basse porte ; les sons de la triade sont ceux de la qualité et de sa tension de
    /// couleur.
    UpperStructure,
    /// *Shell* à deux voix : la tierce et la septième (ou sixte, quinte d'une triade).
    Shell,
    /// *Spread* : position serrée à quatre sons dont les deuxième et quatrième voix depuis
    /// l'aigu descendent d'une octave (drop 2 et 4).
    Open,
}

impl ChordVoicing {
    /// Voicings de triade : doublures `note_count` et fenêtre fixe possibles.
    pub(crate) const fn is_triad_kind(self) -> bool {
        matches!(self, Self::CloseTriad | Self::FixedTriad)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttackDirection {
    #[default]
    Block,
    Down,
    Up,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spread {
    pub down_gap_ms: f32,
    pub up_gap_ms: f32,
    pub up_note_count: u8,
    pub gap_jitter: f32,
    pub anchor: f32,
    pub tempo_exponent: f32,
    pub ref_bpm: f32,
    pub max_spread_ratio: f32,
    pub velocity_slope: u8,
    pub velocity_jitter: u8,
}

impl ChordVoice {
    pub(crate) fn pattern(&self, offset: f32) -> &ChordPattern {
        let index = if offset == 0.0 {
            self.default_pattern
        } else {
            (1.0 + offset).round().clamp(0.0, (self.patterns.len() - 1) as f32) as usize
        };
        &self.patterns[index]
    }
    pub(crate) fn needs_seed(&self) -> bool {
        !self.comping.is_empty()
            || self.patterns.iter().flat_map(|p| &p.hits).any(|h| {
                (h.probability > 0.0 && h.probability < 1.0)
                    || (h.probability > 0.0
                        && h.direction != AttackDirection::Block
                        && self
                            .spread
                            .as_ref()
                            .is_some_and(|s| s.gap_jitter > 0.0 || s.velocity_jitter > 0))
            })
    }

    /// Découpage de la passe `window` de la partie `part` en phrases de comping, restreint aux
    /// étiquettes `tags` de l'arc du chorus (vide : toutes). Vide : le profil n'a pas de phrase
    /// (ou aucune ne concourt) ; chaque mesure se tire alors seule (`bar_hits`).
    pub(crate) fn phrase_tiling(
        &self,
        window: Window,
        part: &str,
        tags: &[TagWeight],
    ) -> Vec<Option<Slot>> {
        if self.phrases.is_empty() {
            return Vec::new();
        }
        tile(window, self.phrases.len(), |index| {
            let phrase = &self.phrases[index];
            phrase.part.as_deref().is_none_or(|p| p == part).then(|| Candidate {
                bars: phrase.motifs.len(),
                weight: tagged_weight(phrase.weight, phrase.tag.as_deref(), tags),
                at: phrase.at,
            })
        })
    }

    /// Attaques de la mesure : celles du motif que `slot` (la phrase qui couvre la mesure)
    /// désigne ; sinon un motif de comping tiré par poids avec `seed` (une mesure se rejoue à
    /// l'identique), sinon le motif choisi par l'intensité (`offset`). `tags` (l'arc du chorus)
    /// restreint le tirage aux motifs qui portent un de ses tags, au poids du motif fois celui
    /// du tag ; aucun motif ne concourt, ou `tags` vide : le tirage du profil.
    pub(crate) fn bar_hits(
        &self,
        offset: f32,
        seed: u64,
        tags: &[TagWeight],
        slot: Option<Slot>,
    ) -> &[ChordHit] {
        if let Some(slot) = slot {
            return &self.comping[self.phrases[slot.index].motifs[slot.bar]].hits;
        }
        let weight = |motif: &CompingMotif| tagged_weight(motif.weight, motif.tag.as_deref(), tags);
        let total: u64 = self.comping.iter().map(weight).fold(0, u64::saturating_add);
        if total == 0 {
            return if tags.is_empty() {
                &self.pattern(offset).hits
            } else {
                self.bar_hits(offset, seed, &[], None)
            };
        }
        let mut pick = ChaCha8Rng::seed_from_u64(mix(seed, 0x434F_4D50)).next_range_u64(0..total);
        for motif in &self.comping {
            if pick < weight(motif) {
                return &motif.hits;
            }
            pick -= weight(motif);
        }
        unreachable!("le tirage reste sous la somme des poids")
    }
}

impl Spread {
    fn validate(&self, note_count: u8) -> Result<(), ProfileError> {
        let field = "accompaniment.chords.spread";
        for (name, value, low, high) in [
            ("down_gap_ms", self.down_gap_ms, 0.0, 40.0),
            ("up_gap_ms", self.up_gap_ms, 0.0, 40.0),
            ("gap_jitter", self.gap_jitter, 0.0, 0.8),
            ("anchor", self.anchor, 0.0, 1.0),
            ("tempo_exponent", self.tempo_exponent, 0.0, 1.0),
            ("ref_bpm", self.ref_bpm, 1.0, 1000.0),
            ("max_spread_ratio", self.max_spread_ratio, 0.0, 0.4),
        ] {
            require(
                value.is_finite() && (low..=high).contains(&value),
                format!("{field}.{name}"),
                &format!("valeur finie dans {low}..{high} requise"),
            )?;
        }
        require(
            (2..=note_count).contains(&self.up_note_count),
            format!("{field}.up_note_count"),
            "2..note_count requis",
        )?;
        require(self.velocity_slope <= 20, format!("{field}.velocity_slope"), "0..20 requis")?;
        require(self.velocity_jitter <= 20, format!("{field}.velocity_jitter"), "0..20 requis")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PassingWhen {
    NextTargetDiffers,
    ChordChangeInsideBar,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PassingMotion {
    ScaleStep,
    ScaleWalk,
    Chromatic,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PassingRule {
    pub when: PassingWhen,
    pub before: Vec<usize>,
    pub motion: PassingMotion,
    pub weight: u32,
    pub duration: usize,
    pub velocity_delta: i16,
    pub min_leap: u8,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BassPassing {
    pub none_weight: u32,
    pub density: Curve,
    pub phrase: Curve,
    pub avoid_melody_intervals: Vec<u8>,
    #[serde(default = "unit_multiplier")]
    pub arousal_low_multiplier: f32,
    #[serde(default = "unit_multiplier")]
    pub arousal_high_multiplier: f32,
    pub rules: Vec<PassingRule>,
}

fn unit_multiplier() -> f32 {
    1.0
}

impl BassPassing {
    pub fn is_inert(&self) -> bool {
        self.rules.iter().all(|r| r.weight == 0)
            || self.density.0.iter().all(|p| p[1] == 0.0)
            || self.phrase.0.iter().all(|p| p[1] == 0.0)
            || (self.arousal_low_multiplier == 0.0 && self.arousal_high_multiplier == 0.0)
    }
    fn validate(&self, steps: usize) -> Result<(), ProfileError> {
        let field = "accompaniment.bass.passing";
        for (name, curve) in [("density", &self.density), ("phrase", &self.phrase)] {
            curve
                .validate(name, 0.0, 4.0)
                .map_err(|e| ProfileError::new(format!("{field}.{name}"), e))?;
        }
        for (name, value) in [
            ("arousal_low_multiplier", self.arousal_low_multiplier),
            ("arousal_high_multiplier", self.arousal_high_multiplier),
        ] {
            require(
                value.is_finite() && (0.0..=4.0).contains(&value),
                format!("{field}.{name}"),
                "multiplicateur fini dans 0..4 requis",
            )?;
        }
        require(
            self.avoid_melody_intervals.iter().all(|&i| i < 12),
            format!("{field}.avoid_melody_intervals"),
            "intervalles modulo 12 requis",
        )?;
        require(self.rules.len() <= 64, format!("{field}.rules"), "au plus 64 règles")?;
        let mut sum = self.none_weight;
        for (i, rule) in self.rules.iter().enumerate() {
            let f = format!("{field}.rules[{i}]");
            sum = sum.checked_add(rule.weight).ok_or_else(|| {
                ProfileError::new(format!("{f}.weight"), "somme des poids hors u32")
            })?;
            require(
                !rule.before.is_empty()
                    && rule.before.len() <= steps
                    && rule.before.iter().all(|&b| b > 0 && b < steps)
                    && rule.before.windows(2).all(|w| w[0] > w[1]),
                format!("{f}.before"),
                "distances positives décroissantes dans la mesure requises",
            )?;
            require(
                rule.motion == PassingMotion::ScaleWalk || rule.before.len() == 1,
                format!("{f}.before"),
                "scale_step et chromatic demandent une seule note",
            )?;
            require(
                rule.duration > 0
                    && rule.before.last().is_some_and(|&b| rule.duration <= b)
                    && rule.before.windows(2).all(|w| rule.duration <= w[0] - w[1]),
                format!("{f}.duration"),
                "durée positive sans chevauchement de la cible ni des passages requise",
            )?;
            require(
                (-127..=127).contains(&rule.velocity_delta),
                format!("{f}.velocity_delta"),
                "delta dans -127..127 requis",
            )?;
            require(rule.min_leap <= 127, format!("{f}.min_leap"), "intervalle MIDI requis")?;
        }
        require(sum > 0, format!("{field}.none_weight"), "somme des poids positive requise")
    }
}

pub(crate) fn require(
    ok: bool,
    field: impl Into<String>,
    message: &str,
) -> Result<(), ProfileError> {
    if ok { Ok(()) } else { Err(ProfileError::new(field, message)) }
}

fn validate_hits(
    hits: impl Iterator<Item = (usize, usize, u8)>,
    steps: usize,
    field: &str,
) -> Result<(), ProfileError> {
    let mut previous = None;
    for (i, (step, duration, velocity)) in hits.enumerate() {
        let f = format!("{field}[{i}]");
        require(
            step < steps && previous.is_none_or(|p| step > p),
            format!("{f}.step"),
            "pas croissants uniques dans la mesure requis",
        )?;
        require(
            duration > 0 && step.checked_add(duration).is_some_and(|end| end <= steps),
            format!("{f}.duration"),
            "durée positive dans la mesure requise",
        )?;
        require(
            (1..=127).contains(&velocity),
            format!("{f}.velocity"),
            "vélocité dans 1..127 requise",
        )?;
        previous = Some(step);
    }
    Ok(())
}

/// Attaques d'accord : pas, durées et vélocités de la mesure, probabilités finies dans 0..1.
fn validate_chord_hits(hits: &[ChordHit], steps: usize, field: &str) -> Result<(), ProfileError> {
    validate_hits(hits.iter().map(|h| (h.step, h.duration, h.velocity)), steps, field)?;
    for (j, hit) in hits.iter().enumerate() {
        require(
            hit.probability.is_finite() && (0.0..=1.0).contains(&hit.probability),
            format!("{field}[{j}].probability"),
            "probabilité finie dans 0..1 requise",
        )?;
    }
    Ok(())
}

impl BassWalking {
    fn validate(
        &self,
        bass: &BassVoice,
        steps: usize,
        steps_per_pulse: usize,
        steps_per_chord: usize,
    ) -> Result<(), ProfileError> {
        let field = "accompaniment.bass.walking";
        require(
            bass.hits.is_empty()
                && bass.motifs.is_empty()
                && bass.templates.is_empty()
                && bass.passing.is_none(),
            field,
            "la marche est exclusive de hits, motifs, gabarits et passing (sources de la même voix)",
        )?;
        require(
            steps_per_pulse > 0
                && steps.is_multiple_of(steps_per_pulse)
                && steps_per_chord.is_multiple_of(steps_per_pulse),
            field,
            "un changement d'accord doit tomber sur une pulsation : \
             harmony.steps_per_chord multiple de meter.steps_per_pulse",
        )?;
        require(
            bass.high - bass.low >= 11,
            "accompaniment.bass.high",
            "la marche demande un registre qui couvre les douze classes de hauteur",
        )?;
        require(
            (1..=127).contains(&self.velocity)
                && u16::from(self.velocity) + u16::from(self.accent) <= 127,
            format!("{field}.velocity"),
            "vélocité dans 1..127, accent compris, requise",
        )?;
        require(
            (1..=steps_per_pulse).contains(&self.duration),
            format!("{field}.duration"),
            "durée de 1 pas à une pulsation requise",
        )?;
        require(
            (1..=12).contains(&self.max_leap),
            format!("{field}.max_leap"),
            "écart maximal dans 1..12 demi-tons requis",
        )?;
        for (name, weight) in [
            ("toward_target_weight", self.toward_target_weight),
            ("keep_direction_weight", self.keep_direction_weight),
        ] {
            require(weight <= 100, format!("{field}.{name}"), "poids dans 0..100 requis")?;
        }
        let approach = &self.approach;
        require(
            u64::from(approach.none_weight)
                + u64::from(approach.chromatic_weight)
                + u64::from(approach.diatonic_weight)
                > 0,
            format!("{field}.approach"),
            "somme des poids positive requise",
        )?;
        if let Some(side) = &approach.side {
            require(
                u64::from(side.from_weight)
                    + u64::from(side.other_weight)
                    + u64::from(side.above_weight)
                    + u64::from(side.below_weight)
                    > 0,
                format!("{field}.approach.side"),
                "somme des poids positive requise",
            )?;
        }
        let passing = &self.passing;
        require(
            u64::from(passing.chord_tone_weight) + u64::from(passing.scale_step_weight) > 0,
            format!("{field}.passing"),
            "somme des poids positive requise (note de l'accord ou pas de gamme)",
        )?;
        if let Some(arrival) = &self.arrival {
            require(
                u64::from(arrival.root_weight)
                    + u64::from(arrival.third_weight)
                    + u64::from(arrival.fifth_weight)
                    + u64::from(arrival.seventh_weight)
                    > 0,
                format!("{field}.arrival"),
                "somme des poids positive requise",
            )?;
        }
        if let Some(eighths) = &self.eighths {
            require(
                steps_per_pulse.is_multiple_of(2),
                format!("{field}.eighths"),
                "une croche demande un nombre pair de pas par pulsation",
            )?;
            require(
                u64::from(eighths.none_weight)
                    + u64::from(eighths.chromatic_weight)
                    + u64::from(eighths.diatonic_weight)
                    + u64::from(eighths.ghost_weight)
                    > 0,
                format!("{field}.eighths"),
                "somme des poids positive requise",
            )?;
            for (name, velocity) in
                [("velocity", eighths.velocity), ("ghost_velocity", eighths.ghost_velocity)]
            {
                require(
                    (1..=127).contains(&velocity),
                    format!("{field}.eighths.{name}"),
                    "vélocité dans 1..127 requise",
                )?;
            }
        }
        if let Some(two_feel) = &self.two_feel {
            require(
                u64::from(two_feel.passing_weight) + u64::from(two_feel.fifth_weight) > 0,
                format!("{field}.two_feel"),
                "somme des poids positive requise",
            )?;
        }
        Ok(())
    }
}

fn validate_doubling(
    doubling: &[BassDoubling],
    low: u8,
    high: u8,
    max_offset: i16,
    field: &str,
) -> Result<(), ProfileError> {
    require(
        doubling.len() <= 2 && doubling.iter().enumerate().all(|(i, d)| !doubling[..i].contains(d)),
        field,
        "doublures uniques octave/tenth requises",
    )?;
    let extra = if doubling.contains(&BassDoubling::Tenth) {
        16i16
    } else if doubling.is_empty() {
        0
    } else {
        12
    };
    require(
        (0..12).all(|pc| {
            let base = i16::from(super::accomp::place_in_range(pc, low, high));
            base + max_offset + extra <= 127
        }),
        field,
        "registre trop aigu pour les doublures MIDI demandées",
    )
}

/// Origine d'une mesure de basse tirée par plan : motif écrit ou gabarit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BassSource {
    Motif(usize),
    Template(usize),
}

/// Candidat du tirage pondéré : motifs écrits, puis gabarits.
pub(crate) struct BassCandidate<'a> {
    pub source: BassSource,
    pub part: Option<&'a str>,
    pub weight: u32,
    pub bars: usize,
}

impl BassVoice {
    /// Vrai si la voix tire ses mesures d'un plan (motifs ou gabarits) au lieu de `hits`.
    pub(crate) fn uses_plan(&self) -> bool {
        !self.motifs.is_empty() || !self.templates.is_empty()
    }

    /// Pool de tirage : les motifs dans l'ordre du profil, puis les gabarits. Sans gabarit,
    /// c'est exactement la liste des motifs, donc le même flux de tirages.
    pub(crate) fn sources(&self) -> impl Iterator<Item = BassCandidate<'_>> {
        let motifs = self.motifs.iter().enumerate().map(|(i, m)| BassCandidate {
            source: BassSource::Motif(i),
            part: m.part.as_deref(),
            weight: m.weight,
            bars: m.bars,
        });
        let templates = self.templates.iter().enumerate().map(|(i, t)| BassCandidate {
            source: BassSource::Template(i),
            part: t.part.as_deref(),
            weight: t.weight,
            bars: t.bars,
        });
        motifs.chain(templates)
    }

    /// Mesures d'une source.
    pub(crate) fn source_bars(&self, source: BassSource) -> usize {
        match source {
            BassSource::Motif(i) => self.motifs[i].bars,
            BassSource::Template(i) => self.templates[i].bars,
        }
    }
}

/// Décalages extrêmes (demi-tons) que la montée d'un gabarit ajoute à ses notes.
fn ramp_extremes(template: &BassTemplate) -> (i16, i16) {
    let Some(ramp) = template.ramp else {
        return (0, 0);
    };
    let cycles = i16::try_from(template.bars - 1).unwrap_or(0) * i16::from(ramp.stride);
    let (low, high) = match ramp.motion {
        RampMotion::Semitone => (cycles, cycles),
        // Un pas de gamme couvre 1 à 3 demi-tons (seconde augmentée comprise).
        RampMotion::ScaleStep if cycles >= 0 => (cycles, 3 * cycles),
        RampMotion::ScaleStep => (3 * cycles, cycles),
    };
    (low.min(0), high.max(0))
}

fn validate_templates(
    b: &BassVoice,
    steps: usize,
    steps_per_chord: usize,
    parts: &[crate::style_profile::ProfilePart],
    total_weight: &mut u32,
) -> Result<(), ProfileError> {
    require(
        b.templates.len() <= 256,
        "accompaniment.bass.templates",
        "au plus 256 gabarits de basse",
    )?;
    for (i, template) in b.templates.iter().enumerate() {
        let tf = format!("accompaniment.bass.templates[{i}]");
        *total_weight = total_weight
            .checked_add(template.weight)
            .ok_or_else(|| ProfileError::new(format!("{tf}.weight"), "somme des poids hors u32"))?;
        require(
            (1..=16).contains(&template.bars),
            format!("{tf}.bars"),
            "1..16 mesures par cycle requises",
        )?;
        if let Some(part_name) = &template.part {
            require(
                parts.iter().any(|p| &p.name == part_name),
                format!("{tf}.part"),
                "partie inconnue dans form.parts",
            )?;
            require(
                parts
                    .iter()
                    .all(|p| &p.name != part_name || !p.tracks_off.contains(&TrackId::Bass)),
                format!("{tf}.part"),
                "gabarit sans emploi : la partie tait la basse (tracks_off)",
            )?;
        }
        let rhythm = format!("{tf}.rhythm");
        require(!template.rhythm.is_empty(), &rhythm, "rythme non vide requis")?;
        validate_hits(
            template.rhythm.iter().map(|h| (h.step, h.duration, h.velocity)),
            steps,
            &rhythm,
        )?;
        for (j, pair) in template.rhythm.windows(2).enumerate() {
            require(
                pair[1].step >= pair[0].step + pair[0].duration,
                format!("{rhythm}[{}].step", j + 1),
                "les attaques du rythme ne doivent pas se chevaucher",
            )?;
        }
        require(template.cycle.len() <= 16, format!("{tf}.cycle"), "au plus 16 hauteurs")?;
        for (j, note) in template.cycle.iter().enumerate() {
            require(
                (-2..=2).contains(&note.octave),
                format!("{tf}.cycle[{j}].octave"),
                "octave relative dans -2..2 requise",
            )?;
        }
        if let Some(ramp) = template.ramp {
            require(
                ramp.stride != 0 && ramp.stride.unsigned_abs() <= 12,
                format!("{tf}.ramp.stride"),
                "pas par mesure non nul dans -12..12 requis",
            )?;
        }
        let (ramp_low, ramp_high) = ramp_extremes(template);
        let (octave_low, octave_high) = template
            .cycle
            .iter()
            .map(|n| i16::from(n.octave))
            .fold((0, 0), |(low, high), octave| (low.min(octave), high.max(octave)));
        require(
            (0..12).all(|pc| {
                let base = i16::from(super::accomp::place_in_range(pc, b.low, b.high));
                base + 12 * octave_low + ramp_low >= 0 && base + 12 * octave_high + ramp_high <= 127
            }),
            &tf,
            "hauteur résultante hors du domaine MIDI 0..127",
        )?;
        if let Some(pickup) = template.pickup {
            let field = format!("{tf}.pickup");
            require(
                (1..steps_per_chord).contains(&pickup.lead),
                format!("{field}.lead"),
                "1..harmony.steps_per_chord-1 pas requis",
            )?;
            require(
                (1..=pickup.lead).contains(&pickup.duration),
                format!("{field}.duration"),
                "durée de 1 pas à lead requise",
            )?;
            require(
                (1..=127).contains(&pickup.velocity),
                format!("{field}.velocity"),
                "vélocité dans 1..127 requise",
            )?;
            require(
                pickup.degree != BassDegree::FifthIfHeld,
                format!("{field}.degree"),
                "root ou fifth requis",
            )?;
        }
    }
    Ok(())
}

impl AccompanimentParams {
    pub(crate) fn validate_accompaniment(
        &self,
        steps: usize,
        steps_per_pulse: usize,
        steps_per_chord: usize,
        parts: &[crate::style_profile::ProfilePart],
    ) -> Result<(), ProfileError> {
        require(
            self.strong_weight > 0 && self.weak_weight > 0,
            "accompaniment.strong_weight",
            "poids de vote positifs requis",
        )?;
        require(
            self.strong_steps.iter().all(|&s| s < steps),
            "accompaniment.strong_steps",
            "pas hors mesure",
        )?;
        let b = &self.bass;
        require(
            b.low <= b.high && b.high <= 127,
            "accompaniment.bass.low",
            "registre MIDI ordonné requis",
        )?;
        require(
            b.walk_low.unwrap_or(b.low) <= b.low,
            "accompaniment.bass.walk_low",
            "doit être inférieur ou égal à low",
        )?;
        require(
            b.walk_high.unwrap_or(b.high) >= b.high && b.walk_high.unwrap_or(b.high) <= 127,
            "accompaniment.bass.walk_high",
            "doit être supérieur ou égal à high, dans MIDI",
        )?;
        if let Some(passing) = &b.passing {
            passing.validate(steps)?;
        }
        validate_hits(
            b.hits.iter().map(|h| (h.step, h.duration, h.velocity)),
            steps,
            "accompaniment.bass.hits",
        )?;
        for (i, hit) in b.hits.iter().enumerate() {
            let field = format!("accompaniment.bass.hits[{i}].doubling");
            validate_doubling(&hit.doubling, b.low, b.high, 0, &field)?;
        }
        if b.uses_plan() {
            require(
                b.hits.is_empty(),
                "accompaniment.bass.hits",
                "hits et motifs sont deux sources exclusives de la même voix",
            )?;
            require(
                b.motifs.len() <= 256,
                "accompaniment.bass.motifs",
                "1..256 motifs de basse requis",
            )?;
            let mut total_weight = 0u32;
            for (i, motif) in b.motifs.iter().enumerate() {
                let mf = format!("accompaniment.bass.motifs[{i}]");
                require(
                    (1..=4).contains(&motif.bars),
                    format!("{mf}.bars"),
                    "1..4 mesures par motif requises",
                )?;
                total_weight = total_weight.checked_add(motif.weight).ok_or_else(|| {
                    ProfileError::new(format!("{mf}.weight"), "somme des poids hors u32")
                })?;
                if let Some(part_name) = &motif.part {
                    require(
                        parts.iter().any(|p| &p.name == part_name),
                        format!("{mf}.part"),
                        "partie inconnue dans form.parts",
                    )?;
                    require(
                        parts.iter().all(|p| {
                            &p.name != part_name || !p.tracks_off.contains(&TrackId::Bass)
                        }),
                        format!("{mf}.part"),
                        "motif sans emploi : la partie tait la basse (tracks_off)",
                    )?;
                }
                let span = motif.bars * steps;
                let nf = format!("{mf}.notes");
                require(!motif.notes.is_empty(), &nf, "motif non vide requis")?;
                let mut previous = None;
                let mut previous_end = 0;
                for (j, note) in motif.notes.iter().enumerate() {
                    let f = format!("{nf}[{j}]");
                    require(
                        note.step < span && previous.is_none_or(|p| note.step > p),
                        format!("{f}.step"),
                        "pas croissants uniques dans le motif requis",
                    )?;
                    require(
                        note.step >= previous_end,
                        format!("{f}.step"),
                        "les notes principales du motif ne doivent pas se chevaucher",
                    )?;
                    require(
                        note.duration > 0
                            && note.step.checked_add(note.duration).is_some_and(|end| end <= span),
                        format!("{f}.duration"),
                        "durée positive dans le motif requise",
                    )?;
                    require(
                        (1..=127).contains(&note.velocity),
                        format!("{f}.velocity"),
                        "vélocité dans 1..127 requise",
                    )?;
                    require(
                        note.semitones.is_some() ^ note.degree.is_some(),
                        &f,
                        "exactement un champ parmi semitones ou degree requis",
                    )?;
                    if let Some(semitones) = note.semitones {
                        require(
                            (-24..=24).contains(&semitones),
                            format!("{f}.semitones"),
                            "intervalle dans -24..24 requis",
                        )?;
                    }
                    if let Some(degree) = note.degree {
                        require(
                            degree != 0 && (-14..=14).contains(&degree),
                            format!("{f}.degree"),
                            "degré non nul dans -14..14 requis",
                        )?;
                    }
                    require(
                        (-2..=2).contains(&note.octave),
                        format!("{f}.octave"),
                        "octave relative dans -2..2 requise",
                    )?;
                    // Un pas modal couvre 1..3 demi-tons (seconde augmentée comprise).
                    // Ces bornes couvrent aussi les références chromatiques hors du mode.
                    let (min_offset, max_offset) = if let Some(semitones) = note.semitones {
                        (i16::from(semitones), i16::from(semitones))
                    } else {
                        let degree = i16::from(note.degree.unwrap_or(1));
                        if degree > 0 {
                            (degree - 1, 3 * (degree - 1))
                        } else {
                            (3 * degree, degree)
                        }
                    };
                    let octave = 12 * i16::from(note.octave);
                    let (min_offset, max_offset) = (min_offset + octave, max_offset + octave);
                    require(
                        (0..12).all(|pc| {
                            let base = i16::from(super::accomp::place_in_range(pc, b.low, b.high));
                            base + min_offset >= 0 && base + max_offset <= 127
                        }),
                        &f,
                        "hauteur résultante hors du domaine MIDI 0..127",
                    )?;
                    validate_doubling(
                        &note.doubling,
                        b.low,
                        b.high,
                        max_offset,
                        &format!("{f}.doubling"),
                    )?;
                    previous = Some(note.step);
                    previous_end = note.step + note.duration;
                }
            }
            validate_templates(b, steps, steps_per_chord, parts, &mut total_weight)?;
            let pool = if b.motifs.is_empty() {
                "accompaniment.bass.templates"
            } else {
                "accompaniment.bass.motifs"
            };
            require(total_weight > 0, pool, "somme des poids positive requise")?;
            for part in parts {
                if part.tracks_off.contains(&TrackId::Bass) {
                    continue;
                }
                require(
                    b.sources().any(|source| {
                        source.weight > 0
                            && (source.part == Some(part.name.as_str()) || source.part.is_none())
                    }),
                    pool,
                    format!("aucun motif de poids positif pour la partie {}", part.name).as_str(),
                )?;
            }
        }
        if let Some(walking) = &b.walking {
            walking.validate(b, steps, steps_per_pulse, steps_per_chord)?;
        }
        let c = &self.chords;
        require(
            c.voicing.is_triad_kind() || c.high.saturating_sub(c.low) >= 12,
            "accompaniment.chords.high",
            "les voicings jazz demandent une fenêtre low..high d'au moins une octave",
        )?;
        require(
            c.comping.len() <= 256,
            "accompaniment.chords.comping",
            "au plus 256 motifs de comping",
        )?;
        let mut comping_weight = 0u32;
        for (i, motif) in c.comping.iter().enumerate() {
            let field = format!("accompaniment.chords.comping[{i}]");
            comping_weight = comping_weight.checked_add(motif.weight).ok_or_else(|| {
                ProfileError::new(format!("{field}.weight"), "somme des poids hors u32")
            })?;
            validate_chord_hits(&motif.hits, steps, &format!("{field}.hits"))?;
        }
        require(
            c.comping.is_empty() || comping_weight > 0,
            "accompaniment.chords.comping",
            "somme des poids positive requise",
        )?;
        require(
            c.phrases.len() <= MAX_PHRASES,
            "accompaniment.chords.phrases",
            "au plus 256 phrases de comping",
        )?;
        for (i, phrase) in c.phrases.iter().enumerate() {
            let field = format!("accompaniment.chords.phrases[{i}]");
            require(phrase.weight > 0, format!("{field}.weight"), "poids positif requis")?;
            require(
                (1..=MAX_PHRASE_BARS).contains(&phrase.motifs.len()),
                format!("{field}.motifs"),
                "1 à 8 mesures requises",
            )?;
            require(
                phrase.part.as_deref().is_none_or(|name| parts.iter().any(|p| p.name == name)),
                format!("{field}.part"),
                "partie inconnue dans form.parts",
            )?;
            for (j, &motif) in phrase.motifs.iter().enumerate() {
                require(
                    motif < c.comping.len(),
                    format!("{field}.motifs[{j}]"),
                    "motif inconnu dans accompaniment.chords.comping",
                )?;
            }
        }
        require(c.center <= 127, "accompaniment.chords.center", "registre MIDI requis")?;
        require(c.max_drift <= 127, "accompaniment.chords.max_drift", "registre MIDI requis")?;
        require(
            (3..=6).contains(&c.note_count),
            "accompaniment.chords.note_count",
            "3..6 notes requises",
        )?;
        require(
            c.low <= c.high && c.high <= 127,
            "accompaniment.chords.low",
            "fenêtre MIDI ordonnée requise",
        )?;
        require(
            c.voicing != ChordVoicing::FixedTriad || c.high - c.low >= 11,
            "accompaniment.chords.high",
            "fixed_triad demande douze hauteurs consécutives",
        )?;
        if let Some(spread) = &c.spread {
            spread.validate(c.note_count)?;
        }
        if let Some(color) = &c.color {
            require(
                u64::from(color.none_weight)
                    + u64::from(color.published_weight)
                    + u64::from(color.ninth_weight)
                    + u64::from(color.eleventh_weight)
                    + u64::from(color.thirteenth_weight)
                    > 0,
                "accompaniment.chords.color",
                "somme des poids positive requise",
            )?;
        }
        if let Some(drift) = &c.register_drift {
            require(
                (1..=12).contains(&drift.step) && (drift.step..=24).contains(&drift.max),
                "accompaniment.chords.register_drift",
                "step dans 1..12 et max dans step..24 requis",
            )?;
        }
        require(
            c.default_pattern < c.patterns.len(),
            "accompaniment.chords.default_pattern",
            "indice de motif existant requis",
        )?;
        for (i, pattern) in c.patterns.iter().enumerate() {
            let field = format!("accompaniment.chords.patterns[{i}].hits");
            require(!pattern.hits.is_empty(), &field, "motif non vide requis")?;
            validate_chord_hits(&pattern.hits, steps, &field)?;
        }
        Ok(())
    }
}
