//! MusicComposer - Main-thread music generation, decoupled from audio playback.
//!
//! Owns the generation side: writehead, generator, musical params,
//! emotion mapping, and measure snapshots. Callable directly (no audio stream needed).

use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use arrayvec::ArrayString;
use harmonium_core::{
    log,
    params::{MusicalParams, SessionConfig, TimeSignature},
    theory::chord::{Chord, ChordType},
    timeline::Writehead,
    trad::{BarRole, TradGenerator, TunePlan, pitch_class_name},
    tuning::TuningParams,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rust_music_theory::{note::PitchSymbol, scale::ScaleType};

use crate::{
    mapper::GameControls,
    playback::{TaggedMeasure, unpack},
};

/// Main-thread music composer. Generates measures synchronously.
///
/// This struct owns the generation pipeline: Generator, Writehead,
/// MusicalParams, GameControls, and the measure ring buffer producer.
/// It reads the packed transport position via an `Arc<AtomicU64>` shared
/// with PlaybackEngine.
pub struct MusicComposer {
    pub config: SessionConfig,

    // Generation
    generator: TradGenerator,
    writehead: Writehead,
    // Handoff: staged measures (composer → playback) and returned boxes
    // (playback → composer). The timeline is the source of truth; `published`
    // mirrors what playback is believed to hold (bar → epoch at stage time).
    measure_tx: rtrb::Producer<TaggedMeasure>,
    return_rx: rtrb::Consumer<TaggedMeasure>,
    published: BTreeMap<usize, u64>,
    // Generation epoch, bumped by every invalidation/seek/reset. Staged
    // boxes carry it; playback treats older boxes as stale.
    epoch: u64,
    // Smallest bar the playback engine must evict (>= bar), set by the last
    // invalidation/seek/reset. Drained by whoever owns the command ring
    // (NativeHandle, REPL) via `take_pending_drop`, as `DropFrom`.
    pending_drop: Option<usize>,
    // Pending GM Program Change commands `(channel, program)` queued on style change.
    pending_program_changes: Vec<(u8, u8)>,

    // State
    musical_params: MusicalParams,
    rng: ChaCha8Rng,
    sample_rate: f64,

    // Seed + init params (for deterministic seek replay)
    session_seed: u64,

    // Game controls: emotions, emotion mode, direct-tempo override (shared
    // semantics with the time plan's dated commands).
    controls: GameControls,

    // Report cache (chord info from last generated measure)
    last_chord_name: ArrayString<64>,
    last_chord_root_offset: i32,
    last_chord_is_minor: bool,

    // Pending measure snapshots for the frontend
    pending_measure_snapshots: Vec<harmonium_core::report::MeasureSnapshot>,
    /// Mesures entendues, collectées exclusivement sur le thread principal.
    score_recording: Option<(Vec<harmonium_core::timeline::Measure>, Option<usize>)>,

    // Shared transport position, packed (bar, step) — written by
    // PlaybackEngine, read here as the bar
    transport_position: Arc<AtomicU64>,

    // Font queue (shared with PlaybackEngine via NativeHandle)
    pub font_queue: crate::FontQueue,
}

impl MusicComposer {
    /// Create a new MusicComposer.
    pub fn new(
        sample_rate: f64,
        measure_tx: rtrb::Producer<TaggedMeasure>,
        return_rx: rtrb::Consumer<TaggedMeasure>,
        transport_position: Arc<AtomicU64>,
        font_queue: crate::FontQueue,
    ) -> Self {
        use rand::Rng;
        let session_seed: u64 = rand::thread_rng().r#gen();
        Self::new_with_seed(
            sample_rate,
            measure_tx,
            return_rx,
            transport_position,
            font_queue,
            session_seed,
        )
    }

    /// Create with explicit seed for deterministic output.
    pub fn new_with_seed(
        sample_rate: f64,
        measure_tx: rtrb::Producer<TaggedMeasure>,
        return_rx: rtrb::Consumer<TaggedMeasure>,
        transport_position: Arc<AtomicU64>,
        font_queue: crate::FontQueue,
        session_seed: u64,
    ) -> Self {
        use rand::Rng;
        let mut rng = ChaCha8Rng::seed_from_u64(session_seed);

        let bpm = 120.0;
        let steps = 16;
        let initial_pulses = 4;

        let keys = [
            PitchSymbol::C,
            PitchSymbol::D,
            PitchSymbol::E,
            PitchSymbol::F,
            PitchSymbol::G,
            PitchSymbol::A,
            PitchSymbol::B,
        ];
        let scales = [ScaleType::PentatonicMinor, ScaleType::PentatonicMajor];
        let random_key = keys[rng.gen_range(0..keys.len())];
        let random_scale = scales[rng.gen_range(0..scales.len())];

        let config = SessionConfig {
            bpm,
            key: format!("{}", random_key),
            scale: format!("{:?}", random_scale),
            pulses: initial_pulses,
            steps,
        };

        log::info(&format!(
            "MusicComposer - Session: {} {} | BPM: {:.1}",
            config.key, config.scale, bpm
        ));

        let key_pc = match random_key {
            PitchSymbol::C => 0,
            PitchSymbol::D => 2,
            PitchSymbol::E => 4,
            PitchSymbol::F => 5,
            PitchSymbol::G => 7,
            PitchSymbol::A => 9,
            PitchSymbol::B => 11,
            _ => 0,
        };
        let tuning = TuningParams::default();

        let mut musical_params = MusicalParams::default();
        let meter = &tuning.profile.meter;
        musical_params.bpm = meter.default_bpm;
        musical_params.time_signature = meter.time_signature;
        musical_params.pulse = Some(meter.pulse_spec());
        musical_params.set_swing_ratio(meter.default_swing_ratio);
        musical_params.profile_mix = tuning.profile.mix.prepare(sample_rate);
        let bpm = musical_params.bpm;
        let config = SessionConfig {
            bpm,
            steps: meter.steps_per_bar,
            pulses: usize::from(meter.pulse_spec().pulses_per_bar),
            ..config
        };
        let generator = TradGenerator::new(musical_params.clone(), tuning);

        let writehead = Writehead::new(sample_rate, 4);

        Self {
            config,
            generator,
            writehead,
            measure_tx,
            return_rx,
            published: BTreeMap::new(),
            epoch: 0,
            pending_drop: None,
            pending_program_changes: Vec::new(),
            musical_params,
            rng,
            sample_rate,
            session_seed,
            controls: GameControls::new(bpm),
            last_chord_name: ArrayString::from(&Chord::new(key_pc, ChordType::Major).name())
                .unwrap_or_default(),
            last_chord_root_offset: 0,
            last_chord_is_minor: false,
            pending_measure_snapshots: Vec::new(),
            score_recording: None,
            transport_position,
            font_queue,
        }
    }

    /// Generate a specific number of bars synchronously.
    /// No audio stream needed — called directly on the main thread.
    pub fn generate_bars(&mut self, count: usize) {
        let old_lookahead = self.writehead.lookahead;
        self.writehead.lookahead = count;
        self.generate_ahead();
        self.writehead.lookahead = old_lookahead;
    }

    /// Generate measures ahead of the playhead, staging them for playback.
    ///
    /// Also drains the return ring (deallocating played boxes here, on the
    /// main thread) and republishes generated-but-unstaged bars — e.g. after
    /// a `DropFrom` eviction or ring backpressure. Staging stops at the first
    /// full ring and resumes on the next call; generation itself always runs
    /// to the lookahead bound, itself capped at the timeline capacity (see
    /// `Writehead::needs_generation`).
    pub fn generate_ahead(&mut self) {
        self.drain_returned();
        self.republish_missing();

        let playhead_bar = unpack(self.transport_position.load(Ordering::Relaxed)).0;

        let mut staging_open = true;
        while self.writehead.needs_generation(playhead_bar) {
            let bar_idx = self.writehead.current_bar;

            // If this bar already exists in the timeline (e.g. after seek),
            // ensure it is staged and advance.
            if self.writehead.timeline.get_measure(bar_idx).is_some() {
                if staging_open && !self.is_published(bar_idx) {
                    staging_open = self.stage_bar(bar_idx);
                }
                if let Some(existing) = self.writehead.timeline.get_measure(bar_idx) {
                    let mut snapshot =
                        harmonium_core::report::MeasureSnapshot::from_measure(existing);
                    snapshot.composition_bpm = self.controls.emotion_mapped_bpm();
                    self.pending_measure_snapshots.push(snapshot);
                }
                self.writehead.current_bar = bar_idx + 1;
                continue;
            }

            let measure = self.generator.generate_measure(bar_idx, &mut self.rng);
            if let Some((plan, _)) = self.generator.bar_info(bar_idx) {
                self.config.key = pitch_class_name(plan.key).to_string();
                self.config.scale = format!("{:?}", plan.mode);
            }

            // Update report cache
            self.last_chord_name =
                ArrayString::from(&measure.chord_context.chord_name).unwrap_or_default();
            self.last_chord_root_offset = measure.chord_context.root_offset;
            self.last_chord_is_minor = measure.chord_context.is_minor;

            let mut snapshot = harmonium_core::report::MeasureSnapshot::from_measure(&measure);
            snapshot.composition_bpm = self.controls.emotion_mapped_bpm();
            self.pending_measure_snapshots.push(snapshot);

            self.writehead.commit_measure(measure);
            if staging_open && !self.is_published(bar_idx) {
                staging_open = self.stage_bar(bar_idx);
            }
        }
        self.capture_recording_bar();
    }

    /// Whether `bar` was staged in the current epoch (and not returned or
    /// evicted since).
    fn is_published(&self, bar: usize) -> bool {
        self.published.get(&bar).is_some_and(|epoch| *epoch == self.epoch)
    }

    /// Stage timeline bar `bar` for playback (epoch-tagged box). Returns
    /// false on ring backpressure — the caller stops staging and retries on
    /// the next `generate_ahead`. Only the ring push can fail; a bar missing
    /// from the timeline is skipped (it is generated by the main loop).
    fn stage_bar(&mut self, bar: usize) -> bool {
        if self.measure_tx.is_full() {
            return false;
        }
        let Some(measure) = self.writehead.timeline.get_measure(bar) else { return true };
        let mut prepared = measure.clone();
        prepared.prepare_playback();
        let staged = TaggedMeasure { epoch: self.epoch, measure: Box::new(prepared) };
        match self.measure_tx.push(staged) {
            Ok(()) => {
                self.published.insert(bar, self.epoch);
                true
            }
            Err(_) => false,
        }
    }

    /// Republish generated bars the playback engine no longer holds: staging
    /// lost to a `DropFrom` eviction or to ring backpressure. Only bars in
    /// `[playhead, writehead)` present in the timeline are republished, in
    /// order, stopping at the first full ring.
    fn republish_missing(&mut self) {
        let playhead_bar = unpack(self.transport_position.load(Ordering::Relaxed)).0;
        let end = self.writehead.current_bar;
        for bar in playhead_bar..end {
            if self.is_published(bar) {
                continue;
            }
            if self.writehead.timeline.get_measure(bar).is_none() {
                continue;
            }
            if !self.stage_bar(bar) {
                break;
            }
        }
    }

    /// Libère les Box sur le thread principal ; seules les mesures futures
    /// rendues sont à republier. L'acquittement de la mesure en cours reste
    /// jusqu'à l'avancée du transport, même si son dernier pas a déjà rendu sa Box.
    fn drain_returned(&mut self) {
        let playing_bar = unpack(self.transport_position.load(Ordering::Relaxed)).0;
        self.published.retain(|&bar, _| bar >= playing_bar);
        while let Ok(returned) = self.return_rx.pop() {
            let bar = returned.measure.index;
            // La Box peut être rendue dès l'entrée du dernier pas, avant que sa fin
            // publie la mesure suivante. Garder l'acquittement de cette mesure :
            // la republier remplirait la fenêtre audio de copies déjà jouées.
            let playing_bar = unpack(self.transport_position.load(Ordering::Relaxed)).0;
            if bar > playing_bar
                && self.published.get(&bar).is_some_and(|epoch| *epoch == returned.epoch)
            {
                self.published.remove(&bar);
            }
            // `returned` drops here, on the main thread.
        }
    }

    /// Move to a new generation epoch: previously staged boxes are stale.
    /// Bars below `evicted_from` stay in the timeline (their content is
    /// unchanged, so they keep playing) but are unstaged in `published`:
    /// the next `generate_ahead` restages them with the new epoch — the
    /// playback engine hands the stale copies back on sight of the newer
    /// same-index box, or sweeps them at load time. The caller forwards
    /// `take_pending_drop` as `DropFrom`.
    fn bump_epoch(&mut self, evicted_from: usize) {
        self.drain_returned();
        self.epoch += 1;
        self.published.retain(|bar, _| *bar < evicted_from);
        self.pending_drop =
            Some(self.pending_drop.map_or(evicted_from, |bar| bar.min(evicted_from)));
    }

    /// Take the pending eviction request as `(bar, epoch)` for the owner of
    /// the command ring to send as `PlaybackCommand::DropFrom { bar }`.
    /// Returns `None` when playback holds nothing stale.
    pub fn take_pending_drop(&mut self) -> Option<(usize, u64)> {
        self.pending_drop.take().map(|bar| (bar, self.epoch))
    }

    /// Drain pending GM Program Change pairs `(logical_channel, program)` queued by
    /// a style transition (D10), to be forwarded as `PlaybackCommand::ProgramChange`.
    pub fn take_pending_program_changes(&mut self) -> Vec<(u8, u8)> {
        std::mem::take(&mut self.pending_program_changes)
    }

    /// Current generation epoch (tags every staged box).
    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    /// Drain pending measure snapshots (called by NativeHandle).
    pub fn take_snapshots(&mut self) -> Vec<harmonium_core::report::MeasureSnapshot> {
        std::mem::take(&mut self.pending_measure_snapshots)
    }

    /// Invalidate future measures and regenerate from the next bar.
    /// Called when musical params change.
    ///
    /// The bar under the playhead is kept only if it was actually generated:
    /// at startup (or after an underflow) the playhead already points at bar
    /// N while nothing exists yet, and regenerating from N+1 would leave the
    /// playback engine waiting for bar N forever.
    pub fn invalidate_future(&mut self) {
        let playhead_bar = unpack(self.transport_position.load(Ordering::Relaxed)).0;
        let regen_from = if self.writehead.timeline.get_measure(playhead_bar).is_some() {
            playhead_bar + 1
        } else {
            playhead_bar
        };

        self.writehead.timeline.invalidate_from(regen_from);
        self.generator.invalidate_future(playhead_bar);
        self.writehead.current_bar = regen_from;

        // New epoch: staged boxes at or past the cut are stale. The caller
        // forwards `take_pending_drop` as `DropFrom`; replacements are
        // staged by the next `generate_ahead`.
        self.bump_epoch(regen_from);

        // Sync generator with updated params and snap current_state
        // so new measures use the correct tempo/density immediately
        self.generator.update_params(self.musical_params.clone());
    }

    /// Invalidate measures beyond a preview window, preserving N bars ahead of playhead.
    ///
    /// Preview bars stay in the timeline and stay staged: the playback
    /// engine keeps reading them by index — no refill needed. Bars beyond
    /// are evicted (see `take_pending_drop`) and regenerated with new params.
    pub fn invalidate_after_preview(&mut self, preview_bars: usize) {
        let playhead_bar = unpack(self.transport_position.load(Ordering::Relaxed)).0;
        let keep_until = playhead_bar + preview_bars;

        // Invalidate timeline from keep_until onward (preview bars stay intact)
        self.writehead.timeline.invalidate_from(keep_until);
        self.writehead.current_bar = keep_until;

        // New epoch: staged boxes at or past the cut are stale.
        self.bump_epoch(keep_until);

        // Update generator with new params
        self.generator.update_params(self.musical_params.clone());
    }

    /// Reset to defaults and clear the timeline. Staged boxes are stale
    /// (see `take_pending_drop`).
    pub fn reset(&mut self) {
        self.musical_params = MusicalParams::default();
        self.controls.reset_tempo(self.musical_params.bpm);
        self.writehead.reset();
        self.set_tuning(TuningParams::default());
        self.generator.update_params(self.musical_params.clone());
        self.clear_staged();
    }

    /// Clear timeline and writehead WITHOUT resetting musical_params.
    /// Use this when you want to regenerate from scratch with current settings.
    pub fn reset_timeline(&mut self) {
        self.writehead.reset();
        self.generator.reset_to_initial();
        self.generator.update_params(self.musical_params.clone());
        self.clear_staged();
    }

    /// Forget every staged box: new epoch, empty published set, full eviction
    /// pending (see `take_pending_drop`).
    fn clear_staged(&mut self) {
        self.drain_returned();
        self.epoch += 1;
        self.published.clear();
        self.pending_drop = Some(1);
    }

    /// Seek the writehead to a specific bar (legacy, non-deterministic).
    pub fn seek_writehead(&mut self, bar: usize) {
        self.writehead.current_bar = bar;
    }

    /// Deterministic seek: reset RNG + generator, replay to target bar.
    ///
    /// After this call, the generator and RNG are in the exact state they would
    /// be at `target_bar` if generation had proceeded linearly from bar 1.
    /// Staged boxes from the target onward are evicted (see
    /// `take_pending_drop`) so they get regenerated.
    pub fn deterministic_seek(&mut self, target_bar: usize) {
        let target_bar = target_bar.max(1);

        // 1. Re-seed RNG from session seed (past initial key/scale draws)
        let mut rng = harmonium_core::time_plan::score::session_rng(self.session_seed);
        // 3. Reset generator to initial state
        self.generator.reset_to_initial();

        // 4. Silent advance to target bar
        if target_bar > 1 {
            self.generator.silent_advance(target_bar, &mut rng);
        }

        // 5. Update state
        self.rng = rng;
        self.writehead.current_bar = target_bar;

        // 6. New epoch: staged boxes at or past the target are stale (the
        // caller forwards `take_pending_drop` as `DropFrom`). The timeline
        // is kept — replayed content is identical — so staged boxes below
        // the target stay valid.
        self.bump_epoch(target_bar);

        log::info(&format!("Deterministic seek to bar {target_bar}"));
    }

    /// Fixe la graine utilisée au prochain rejeu déterministe.
    fn apply_seed(&mut self, seed: u64) {
        self.session_seed = seed;
    }

    /// Generate a new melody with a fresh random seed.
    pub fn new_melody(&mut self) {
        use rand::Rng;
        let new_seed: u64 = rand::thread_rng().r#gen();
        log::info(&format!("New melody with seed {new_seed}"));
        self.apply_seed(new_seed);
        self.writehead.reset();
        self.clear_staged();
        self.deterministic_seek(1);
    }

    /// Set an explicit seed and regenerate from bar 1.
    pub fn set_seed(&mut self, seed: u64) {
        log::info(&format!("Set seed to {seed}"));
        self.apply_seed(seed);
        self.writehead.reset();
        self.clear_staged();
        self.deterministic_seek(1);
    }

    /// Current session key as a display string (e.g. "C", "Bb").
    pub fn session_key_string(&self) -> String {
        self.config.key.clone()
    }

    /// Get the current session seed.
    pub fn session_seed(&self) -> u64 {
        self.session_seed
    }

    // === Parameter setters (called directly, no command queue) ===

    /// Tempo direct du jeu : désactive l'offset émotionnel et la rampe de set.
    pub fn set_bpm(&mut self, bpm: f32) {
        self.controls.set_bpm(bpm, &mut self.musical_params);
    }

    /// Rend le tempo aux émotions et à la rampe de set.
    pub fn reset_bpm(&mut self) {
        self.controls.reset_bpm(&mut self.musical_params);
    }

    /// Applique une commande du plan de temps partagé avec la même sémantique que le plan
    /// (`SharedTimePlan`) : à appeler avant de générer la mesure à laquelle elle est datée.
    pub fn apply_score_command(&mut self, command: harmonium_core::time_plan::score::ScoreCommand) {
        command.apply(&mut self.controls, &mut self.musical_params, &self.generator.tuning);
    }

    pub fn set_time_signature(&mut self, numerator: usize, denominator: usize) {
        self.musical_params.time_signature = TimeSignature { numerator, denominator };
    }

    /// Set the swing ratio (clamped 0.5..=0.75). Takes effect on the next
    /// generated measure: the caller invalidates and regenerates ahead, like
    /// the other structural setters.
    pub fn set_swing_ratio(&mut self, ratio: f32) {
        self.musical_params.set_swing_ratio(ratio);
    }

    /// Applique les défauts du profil actif.
    fn apply_profile_defaults(&mut self) {
        let profile = &self.generator.tuning.profile;
        let meter = &profile.meter;
        self.musical_params.apply_profile_meter(profile);
        self.controls.reset_tempo(meter.default_bpm);
        self.musical_params.profile_mix = profile.mix.prepare(self.sample_rate);
        self.config.bpm = meter.default_bpm;
        self.config.steps = meter.steps_per_bar;
        self.config.pulses = usize::from(meter.pulse_spec().pulses_per_bar);
        self.generator.update_params(self.musical_params.clone());
        self.pending_program_changes.clear();
        self.pending_program_changes.extend(
            harmonium_audio::voice_manager::gm_programs_for_profile(
                &self.musical_params.profile_instruments,
            ),
        );
    }

    /// Installe atomiquement un profil validé et sélectionne son générateur.
    pub fn set_profile(&mut self, profile: harmonium_core::StyleProfile) -> Result<(), String> {
        profile.validate().map_err(|e| e.to_string())?;
        let mut tuning = self.generator.tuning.clone();
        tuning.profile = profile;
        self.set_tuning(tuning);
        self.invalidate_future();
        Ok(())
    }

    pub fn set_profile_id(&mut self, id: &str) -> Result<(), String> {
        let profile = harmonium_core::StyleProfile::named(id).map_err(|e| e.to_string())?;
        self.set_profile(profile)
    }

    pub fn set_feet_density(&mut self, density: u8) {
        self.musical_params.set_feet_density(density);
    }

    /// Énergie du style (0..1), audible après `invalidate_future`.
    pub fn set_intensity(&mut self, intensity: f32) {
        self.musical_params.set_intensity(intensity);
    }

    /// Densité de texture appliquée par les courbes du profil.
    pub fn set_rhythm_density(&mut self, density: f32) {
        self.musical_params.rhythm_density = density.clamp(0.0, 1.0);
    }

    pub fn enable_rhythm(&mut self, e: bool) {
        self.musical_params.enable_rhythm = e;
    }
    pub fn enable_harmony(&mut self, e: bool) {
        self.musical_params.enable_harmony = e;
    }
    pub fn enable_melody(&mut self, e: bool) {
        self.musical_params.enable_melody = e;
    }

    /// Arrête la génération après le dernier tour de l'air courant (outro, P1-T11).
    pub fn set_end_after_tour(&mut self, e: bool) {
        self.musical_params.end_after_tour = e;
    }

    // === Instrument config ===

    pub fn set_instrument_lead(&mut self, config: harmonium_core::params::InstrumentConfig) {
        self.musical_params.instrument_lead = config;
    }

    pub fn set_instrument_bass(&mut self, config: harmonium_core::params::InstrumentConfig) {
        self.musical_params.instrument_bass = config;
    }

    // === Emotion mode ===

    pub fn use_emotion_mode(&mut self) {
        self.controls.set_emotion_mode(true);
        log::info("MusicComposer: switched to Emotion mode");
    }

    pub fn use_direct_mode(&mut self) {
        self.controls.set_emotion_mode(false);
        log::info("MusicComposer: switched to Direct mode");
    }

    pub fn is_emotion_mode(&self) -> bool {
        self.controls.is_emotion_mode()
    }

    /// Only stores params — does NOT sync the generator: `invalidate_future()` or
    /// `invalidate_after_preview()` do it, so already-generated bars are preserved.
    pub fn set_emotions(&mut self, arousal: f32, valence: f32, density: f32, tension: f32) {
        self.controls.set_emotions(
            [arousal, valence, density, tension],
            &self.generator.tuning.profile,
            &mut self.musical_params,
        );
    }

    /// Émotions courantes du contrôleur pour le morphing Odin audio.
    #[must_use]
    pub fn emotions(&self) -> [f32; 4] {
        self.controls.emotions()
    }

    // === Writehead controls ===

    pub fn set_writehead_lookahead(&mut self, n: usize) {
        self.writehead.lookahead = n.max(4);
    }

    fn capture_recording_bar(&mut self) {
        let bar = self.playhead_bar();
        if let Some((measures, last_bar)) = &mut self.score_recording
            && *last_bar != Some(bar)
            && let Some(measure) = self.writehead.timeline.get_measure(bar)
        {
            let mut measure = measure.clone();
            measure.index = measures.len() + 1;
            measures.push(measure);
            *last_bar = Some(bar);
        }
    }

    /// Démarre la capture audio ou une partition de mesures complètes, hors callback.
    pub fn start_recording(
        &mut self,
        format: harmonium_core::events::RecordFormat,
        commands: &mut rtrb::Producer<crate::playback::PlaybackCommand>,
    ) -> Result<(), String> {
        use harmonium_core::events::{AudioRecordFormat, RecordFormat};
        let audio = match format {
            RecordFormat::Wav => AudioRecordFormat::Wav,
            RecordFormat::Midi => AudioRecordFormat::Midi,
            RecordFormat::MusicXml => {
                self.score_recording = Some((Vec::new(), None));
                return Ok(());
            }
        };
        commands
            .push(crate::playback::PlaybackCommand::StartRecording(audio))
            .map_err(|_| "file de commandes audio pleine".to_string())
    }

    /// Termine la capture ; MusicXML utilise uniquement l'export canonique de timeline.
    pub fn stop_recording(
        &mut self,
        format: harmonium_core::events::RecordFormat,
        commands: &mut rtrb::Producer<crate::playback::PlaybackCommand>,
        finished: &crate::FinishedRecordings,
    ) -> Result<(), String> {
        use harmonium_core::events::{AudioRecordFormat, RecordFormat};
        let audio = match format {
            RecordFormat::Wav => AudioRecordFormat::Wav,
            RecordFormat::Midi => AudioRecordFormat::Midi,
            RecordFormat::MusicXml => {
                if let Some((measures, _)) = self.score_recording.take() {
                    let mut score = harmonium_core::timeline::ScoreTimeline::new(measures.len());
                    for measure in measures {
                        score.push_measure(measure);
                    }
                    let xml = harmonium_core::timeline::timeline_to_musicxml_with_instruments(
                        &score,
                        "Harmonium Export",
                        &self.musical_params.instrument_lead,
                        &self.musical_params.instrument_bass,
                    );
                    finished
                        .lock()
                        .map_err(|e| e.to_string())?
                        .push((RecordFormat::MusicXml, xml.into_bytes()));
                }
                return Ok(());
            }
        };
        commands
            .push(crate::playback::PlaybackCommand::StopRecording(audio))
            .map_err(|_| "file de commandes audio pleine".to_string())
    }

    /// Apply a TuningParams to the generator (for style profile loading).
    pub fn set_tuning(&mut self, tuning: harmonium_core::tuning::TuningParams) {
        self.musical_params.profile_id = Arc::from(tuning.profile.meta.id.as_str());
        self.musical_params.profile_instruments = tuning.profile.instruments;
        self.musical_params.profile_odin =
            harmonium_core::params::describe_odin_tracks(&tuning.profile.odin);
        self.generator.update_tuning(tuning);
        self.apply_profile_defaults();
    }

    /// Profil actif.
    pub fn profile(&self) -> &harmonium_core::StyleProfile {
        &self.generator.tuning.profile
    }

    /// Nombre de tours AABB joués par air (D11, `reel_arrangement.tours_per_tune`).
    #[must_use]
    pub fn tours_per_tune(&self) -> usize {
        self.generator.tuning.profile.arrangement.tours_per_tune
    }

    /// Force le nombre de tours par air (D11) ; refuse une valeur hors `1..=MAX_TOURS`.
    pub fn set_tours_per_tune(&mut self, tours: usize) -> Result<(), String> {
        let mut tuning = self.generator.tuning.clone();
        tuning.profile.arrangement.tours_per_tune = tours;
        tuning.profile.arrangement.validate().map_err(|e| e.to_string())?;
        self.generator.update_tuning(tuning);
        Ok(())
    }

    /// Reset the RNG to a deterministic seed (for reproducible rendering).
    pub fn set_rng_seed(&mut self, seed: u64) {
        self.rng = ChaCha8Rng::seed_from_u64(seed);
    }

    /// Sync generator with current musical params (call after batch param changes).
    pub fn sync_generator(&mut self) {
        self.generator.update_params(self.musical_params.clone());
    }

    /// Export timeline to MusicXML.
    pub fn export_timeline(&self, format: harmonium_core::events::RecordFormat) {
        match format {
            harmonium_core::events::RecordFormat::MusicXml => {
                let xml = harmonium_core::timeline::timeline_to_musicxml_with_instruments(
                    &self.writehead.timeline,
                    "Harmonium Export",
                    &self.musical_params.instrument_lead,
                    &self.musical_params.instrument_bass,
                );
                if let Ok(()) = std::fs::write("timeline_export.musicxml", &xml) {
                    log::info(&format!(
                        "Timeline exported to timeline_export.musicxml ({} bytes)",
                        xml.len()
                    ));
                }
            }
            _ => {
                log::warn(&format!("Timeline export only supports MusicXML, got {:?}", format));
            }
        }
    }

    // === Getters for report building ===

    /// Current writehead position (next bar to generate).
    pub fn writehead_position(&self) -> usize {
        self.writehead.current_bar
    }

    /// Current playhead bar — the bar half of the shared packed transport
    /// position (written by PlaybackEngine).
    pub fn playhead_bar(&self) -> usize {
        unpack(self.transport_position.load(Ordering::Relaxed)).0
    }

    /// Get a reference to a measure from the timeline by bar index.
    pub fn timeline_measure(&self, bar: usize) -> Option<&harmonium_core::timeline::Measure> {
        self.writehead.timeline.get_measure(bar)
    }

    /// Retourne le plan et le rôle de la mesure 1-based `bar`.
    #[must_use]
    pub fn profile_bar_info(&self, bar: usize) -> Option<(TunePlan, BarRole)> {
        self.generator.bar_info(bar)
    }
    pub fn last_chord_name(&self) -> &ArrayString<64> {
        &self.last_chord_name
    }
    pub fn last_chord_root_offset(&self) -> i32 {
        self.last_chord_root_offset
    }
    pub fn last_chord_is_minor(&self) -> bool {
        self.last_chord_is_minor
    }
    pub fn bpm_override(&self) -> Option<f32> {
        self.controls.bpm_override()
    }
    pub fn emotion_mapped_bpm(&self) -> f32 {
        self.controls.emotion_mapped_bpm()
    }
    pub fn musical_params(&self) -> &MusicalParams {
        &self.musical_params
    }
    pub fn sample_rate(&self) -> f64 {
        self.sample_rate
    }
}
