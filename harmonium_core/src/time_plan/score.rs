//! Plan partagé construit sans audio : graine + profil + commandes datées → [`TimePlan`].
//!
//! Chaque client de la simulation tient la même liste de [`DatedCommand`] (état
//! rollbacké du jeu) ; [`SharedTimePlan`] en dérive le plan, en rejouant le générateur
//! sans rendu. Les commandes s'appliquent avant la génération de leur mesure, dans un
//! ordre total ([`DatedCommand`] est `Ord`) : l'ordre d'arrivée ne compte pas. Une
//! commande qui touche une mesure déjà planifiée fait recalculer le plan depuis le point
//! de reprise qui précède cette mesure (rollback), jamais avant.

use std::{collections::BTreeSet, sync::Arc};

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use super::{BarTiming, Flicks, Tempo, TimePlan};
use crate::{
    mapper::GameControls, params::MusicalParams, trad::TradGenerator, tuning::TuningParams,
};

/// Écart entre deux points de reprise du générateur, en mesures.
pub const CHECKPOINT_BARS: usize = 8;

/// RNG d'une session après les deux tirages d'affichage du composer (tonalité et gamme de
/// session) : seule initialisation, partagée par le composer et le plan.
#[must_use]
pub fn session_rng(seed: u64) -> ChaCha8Rng {
    use rand::Rng;
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let _ = rng.gen_range(0..7usize);
    let _ = rng.gen_range(0..2usize);
    rng
}

/// Valeur entière en millièmes, convertie pour le générateur (`f32`).
#[allow(clippy::float_arithmetic)]
fn unit(milli: i32) -> f32 {
    milli as f32 / 1000.0
}

/// Commande de jeu qui touche le temps ou la génération. Entrées entières : la simulation
/// n'a jamais à fournir de flottant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ScoreCommand {
    /// Quatre axes émotionnels en millièmes (`valence` signée, -1000..=1000) ; active le mode
    /// émotion.
    Emotions { arousal: u16, valence: i16, density: u16, tension: u16 },
    /// Énergie du style, en millièmes.
    Intensity(u16),
    /// Tempo direct du jeu : désactive l'offset émotionnel et la rampe de set.
    Tempo(Tempo),
    /// Rend le tempo aux émotions et à la rampe de set.
    ReleaseTempo,
}

impl ScoreCommand {
    /// Applique la commande avec la sémantique du composer ([`GameControls`]).
    pub fn apply(
        self,
        controls: &mut GameControls,
        params: &mut MusicalParams,
        tuning: &TuningParams,
    ) {
        match self {
            Self::Emotions { arousal, valence, density, tension } => {
                controls.set_emotion_mode(true);
                controls.set_emotions(
                    [
                        unit(i32::from(arousal)),
                        unit(i32::from(valence)),
                        unit(i32::from(density)),
                        unit(i32::from(tension)),
                    ],
                    &tuning.profile,
                    params,
                );
            }
            Self::Intensity(value) => params.set_intensity(unit(i32::from(value))),
            Self::Tempo(tempo) => controls.set_bpm(tempo.bpm(), params),
            Self::ReleaseTempo => controls.reset_bpm(params),
        }
    }
}

/// Date d'une commande : une mesure, ou un instant du plan (la commande s'applique alors à
/// la première mesure qui commence à cet instant ou après).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CommandDate {
    Bar(usize),
    At(Flicks),
}

/// Commande datée. `order` est la clé d'ordre fixée par le jeu (frame d'émission puis
/// joueur, par exemple) : deux commandes de la même mesure s'appliquent dans l'ordre
/// `(order, command)`, quel que soit leur ordre d'arrivée.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DatedCommand {
    pub order: u64,
    pub command: ScoreCommand,
    pub date: CommandDate,
}

impl DatedCommand {
    /// Mesure à laquelle la commande s'applique, connaissant le début de la mesure `bar` et
    /// celui de la précédente (`None` pour la mesure 1).
    fn applies_at(&self, bar: usize, start: Flicks, previous_start: Option<Flicks>) -> bool {
        match self.date {
            CommandDate::Bar(target) => target.max(1) == bar,
            CommandDate::At(at) => at <= start && previous_start.is_none_or(|prev| at > prev),
        }
    }
}

/// État de génération avant une mesure : tout ce qui détermine la suite.
#[derive(Clone, Debug)]
struct Cursor {
    generator: TradGenerator,
    rng: ChaCha8Rng,
    params: MusicalParams,
    controls: GameControls,
}

/// Plan de temps partagé : même graine, même profil, mêmes commandes ⇒ même plan, sur
/// chaque client, sans audio.
#[derive(Clone, Debug)]
pub struct SharedTimePlan {
    tuning: Arc<TuningParams>,
    commands: BTreeSet<DatedCommand>,
    plan: TimePlan,
    cursor: Cursor,
    /// `checkpoints[i]` : état avant la mesure `1 + i × CHECKPOINT_BARS`.
    checkpoints: Vec<Cursor>,
}

impl SharedTimePlan {
    /// Plan vide d'une session : profil (défauts de mètre et de tempo) et graine.
    #[must_use]
    pub fn new(tuning: TuningParams, seed: u64) -> Self {
        let mut params = MusicalParams::default();
        params.apply_profile_meter(&tuning.profile);
        let controls = GameControls::new(tuning.profile.meter.default_bpm);
        let generator = TradGenerator::new(params.clone(), tuning.clone());
        Self {
            tuning: Arc::new(tuning),
            commands: BTreeSet::new(),
            plan: TimePlan::new(),
            cursor: Cursor { generator, rng: session_rng(seed), params, controls },
            checkpoints: Vec::new(),
        }
    }

    /// Commandes connues, dans leur ordre d'application.
    pub fn commands(&self) -> impl Iterator<Item = &DatedCommand> {
        self.commands.iter()
    }

    /// Plan calculé jusqu'ici.
    #[must_use]
    pub const fn plan(&self) -> &TimePlan {
        &self.plan
    }

    /// Ajoute une commande (idempotent). Si elle touche une mesure déjà planifiée, le plan
    /// est recalculé depuis le point de reprise qui la précède.
    pub fn insert(&mut self, command: DatedCommand) {
        if self.commands.insert(command) {
            self.invalidate_for(command);
        }
    }

    /// Retire une commande (entrée annulée par le rollback du jeu).
    pub fn remove(&mut self, command: &DatedCommand) {
        if self.commands.remove(command) {
            self.invalidate_for(*command);
        }
    }

    /// Remplace la liste par celle de la simulation (état rollbacké) ; seules les mesures
    /// touchées par la différence sont recalculées.
    pub fn sync(&mut self, commands: &[DatedCommand]) {
        let wanted: BTreeSet<DatedCommand> = commands.iter().copied().collect();
        let changed: Vec<DatedCommand> =
            self.commands.symmetric_difference(&wanted).copied().collect();
        self.commands = wanted;
        for command in changed {
            self.invalidate_for(command);
        }
    }

    /// Étend le plan jusqu'à la mesure `bar` incluse.
    pub fn extend_to_bar(&mut self, bar: usize) {
        while self.plan.bars() < bar {
            self.generate_next();
        }
    }

    /// Étend le plan jusqu'à couvrir l'instant `at`.
    pub fn extend_past(&mut self, at: Flicks) {
        while self.plan.end() <= at {
            self.generate_next();
        }
    }

    /// Position musicale à l'instant `at`, en étendant le plan si besoin.
    pub fn position_at(&mut self, at: Flicks) -> Option<super::PlanPosition> {
        self.extend_past(at);
        self.plan.position_at(at)
    }

    /// Première mesure que `command` touche, si elle est déjà planifiée.
    fn first_planned_bar(&self, command: DatedCommand) -> Option<usize> {
        match command.date {
            CommandDate::Bar(bar) => Some(bar.max(1)),
            CommandDate::At(at) => (1..=self.plan.bars())
                .find(|&bar| self.plan.bar_start(bar).is_some_and(|start| at <= start)),
        }
        .filter(|&bar| bar <= self.plan.bars())
    }

    fn invalidate_for(&mut self, command: DatedCommand) {
        if let Some(bar) = self.first_planned_bar(command) {
            self.rollback_to(bar);
        }
    }

    /// Reprend la génération au point de reprise qui précède la mesure `bar` ; ce point est
    /// retiré, la génération le reprend en repassant par sa mesure.
    fn rollback_to(&mut self, bar: usize) {
        let index = (bar - 1) / CHECKPOINT_BARS;
        let Some(cursor) = self.checkpoints.get(index).cloned() else { return };
        self.checkpoints.truncate(index);
        self.cursor = cursor;
        self.plan.truncate(index * CHECKPOINT_BARS);
    }

    /// Génère la mesure suivante sans rendu et ajoute son en-tête au plan.
    fn generate_next(&mut self) {
        let bar = self.plan.bars() + 1;
        if (bar - 1).is_multiple_of(CHECKPOINT_BARS) {
            self.checkpoints.push(self.cursor.clone());
        }
        let start = self.plan.end();
        let previous_start = self.plan.bar_start(bar - 1);
        let Cursor { generator, rng, params, controls } = &mut self.cursor;
        for command in &self.commands {
            if command.applies_at(bar, start, previous_start) {
                command.command.apply(controls, params, &self.tuning);
            }
        }
        generator.update_params(params.clone());
        let measure = generator.generate_measure(bar, rng);
        self.plan.push(BarTiming::of(&measure));
    }
}
