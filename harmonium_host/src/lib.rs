use std::sync::{Arc, Mutex};

use harmonium_core::report::MeasureSnapshot;
use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

pub mod timeline_engine;

// Decoupled architecture: MusicComposer (main thread) + PlaybackEngine (audio thread)
#[cfg(feature = "standalone")]
pub mod composer;
#[cfg(feature = "standalone")]
pub mod playback;

// Re-exports from workspace crates
#[cfg(feature = "ai")]
pub use harmonium_ai::ai;
pub use harmonium_ai::mapper;
pub use harmonium_audio::{backend, realtime, synthesis, voice_manager, voicing};
pub use harmonium_core::{events, fractal, harmony, log, params, sequencer};

// Real-time safety: Global allocator that panics on allocations in audio thread (debug builds only)
// Uses fully qualified path to avoid local mod ambiguity
#[cfg(debug_assertions)]
#[global_allocator]
static GLOBAL: harmonium_audio::realtime::rt_check::RTCheckAllocator =
    harmonium_audio::realtime::rt_check::RTCheckAllocator;

// Audio module (only for standalone/WASM builds with cpal)
#[cfg(feature = "standalone")]
pub mod audio;

// Native handle (standalone without wasm)
#[cfg(feature = "standalone")]
pub mod native_handle;
#[cfg(feature = "standalone")]
pub use native_handle::NativeHandle;

// VST Plugin module (only for VST builds)
#[cfg(feature = "vst")]
pub mod vst_plugin;

// VST GUI module (only for VST builds with GUI)
#[cfg(feature = "vst-gui")]
pub mod vst_gui;

// Re-exports pour compatibilité avec l'ancien code
// Re-export audio backend type (for runtime switching)
#[cfg(feature = "standalone")]
pub use audio::AudioBackendType;
pub use harmonium_ai::mapper::{EmotionMapper, MapperConfig};
// Re-exports pour la nouvelle architecture découplée
// Note: HarmonyStrategy removed if not in core/params or changed
pub use harmonium_core::params::{ControlMode, MusicalParams};
pub use harmonium_core::{
    harmony::{HarmonyMode, basic as progression, melody as harmony_melody},
    sequencer::RhythmMode,
};
// Re-export VST plugin when building with vst feature
#[cfg(feature = "vst")]
pub use vst_plugin::HarmoniumPlugin;

#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct RecordedData {
    format_str: String,
    data: Vec<u8>,
}

// === Lookahead DTOs (compatible with harmonium_practice) ===

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NoteEvent {
    pub bar: usize,
    pub beat: f32,
    pub duration_beats: f32,
    pub pitch: u8,
    pub velocity: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChordInfo {
    pub root: String,
    pub quality: String,
    pub display_name: String,
    pub bass: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChordEvent {
    pub bar: usize,
    pub beat: f32,
    pub duration_beats: f32,
    pub root: u8,
    pub quality: String,
    pub display_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScaleSuggestionData {
    pub name: String,
    pub notes: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TempoMarking {
    pub bar: u32,
    pub bpm: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LookaheadResponse {
    pub notes: Vec<NoteEvent>,
    pub chords: Vec<ChordEvent>,
    pub current_bar: usize,
    pub current_beat: f32,
    pub time_signature: (u8, u8),
    pub scale_suggestion: Option<ScaleSuggestionData>,
    pub tempo_markings: Vec<TempoMarking>,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl RecordedData {
    #[wasm_bindgen(getter)]
    pub fn format(&self) -> String {
        self.format_str.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn data(&self) -> Vec<u8> {
        self.data.clone()
    }
}

#[cfg(not(feature = "wasm"))]
impl RecordedData {
    pub fn format(&self) -> String {
        self.format_str.clone()
    }

    pub fn data(&self) -> Vec<u8> {
        self.data.clone()
    }
}

// Type aliases for complex types
pub type FontQueue = Arc<Mutex<Vec<(u32, Vec<u8>)>>>;
pub type FinishedRecordings = Arc<Mutex<Vec<(events::RecordFormat, Vec<u8>)>>>;
/// Shared measure pages: Composer writes by index, PlaybackEngine reads by index.
pub type SharedPages = Arc<Mutex<Vec<harmonium_core::timeline::Measure>>>;

// Handle and WASM bindings only available with wasm feature
// TODO: Phase 3 - Rebuild this API to use controller properly
#[cfg(all(feature = "standalone", feature = "wasm"))]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct Handle {
    #[allow(dead_code)]
    stream: cpal::Stream,
    /// Unified controller for all engine communication
    controller: harmonium_core::HarmoniumController,
    /// Queue de chargement de SoundFonts
    font_queue: FontQueue,
    /// Enregistrements terminés
    finished_recordings: FinishedRecordings,
    /// Internal buffer for measure snapshots
    measures_buffer: Vec<MeasureSnapshot>,
    /// Cached UI-side parameters for getters
    cached_params: harmonium_core::EngineParams,
    bpm: f32,
    key: String,
    scale: String,
    pulses: usize,
    steps: usize,
}

#[cfg(all(feature = "standalone", feature = "wasm"))]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl Handle {
    // === Session Info (static, from initial config) ===

    pub fn get_bpm(&self) -> f32 {
        self.bpm
    }

    pub fn get_key(&self) -> String {
        self.key.clone()
    }

    pub fn get_scale(&self) -> String {
        self.scale.clone()
    }

    pub fn get_pulses(&self) -> usize {
        self.pulses
    }

    pub fn get_steps(&self) -> usize {
        self.steps
    }

    // === Emotion Controls ===

    /// Set arousal (0.0-1.0) - controls BPM (70-180)
    pub fn set_arousal(&mut self, arousal: f32) {
        self.cached_params.arousal = arousal.clamp(0.0, 1.0);
        let _ = self.controller.set_emotions(
            self.cached_params.arousal,
            self.cached_params.valence,
            self.cached_params.density,
            self.cached_params.tension,
        );
    }

    /// Set valence (-1.0 to 1.0) - major/minor bias
    pub fn set_valence(&mut self, valence: f32) {
        self.cached_params.valence = valence.clamp(-1.0, 1.0);
        let _ = self.controller.set_emotions(
            self.cached_params.arousal,
            self.cached_params.valence,
            self.cached_params.density,
            self.cached_params.tension,
        );
    }

    /// Set rhythmic density (0.0-1.0)
    pub fn set_density(&mut self, density: f32) {
        self.cached_params.density = density.clamp(0.0, 1.0);
        let _ = self.controller.set_emotions(
            self.cached_params.arousal,
            self.cached_params.valence,
            self.cached_params.density,
            self.cached_params.tension,
        );
    }

    /// Set harmonic tension (0.0-1.0)
    pub fn set_tension(&mut self, tension: f32) {
        self.cached_params.tension = tension.clamp(0.0, 1.0);
        let _ = self.controller.set_emotions(
            self.cached_params.arousal,
            self.cached_params.valence,
            self.cached_params.density,
            self.cached_params.tension,
        );
    }

    /// Set all emotion parameters at once
    pub fn set_params(&mut self, arousal: f32, valence: f32, density: f32, tension: f32) {
        self.cached_params.arousal = arousal.clamp(0.0, 1.0);
        self.cached_params.valence = valence.clamp(-1.0, 1.0);
        self.cached_params.density = density.clamp(0.0, 1.0);
        self.cached_params.tension = tension.clamp(0.0, 1.0);
        let _ = self.controller.set_emotions(
            self.cached_params.arousal,
            self.cached_params.valence,
            self.cached_params.density,
            self.cached_params.tension,
        );
    }

    // === Emotion Getters (cached UI-side values) ===

    pub fn get_target_arousal(&self) -> f32 {
        self.cached_params.arousal
    }

    pub fn get_target_valence(&self) -> f32 {
        self.cached_params.valence
    }

    pub fn get_target_density(&self) -> f32 {
        self.cached_params.density
    }

    pub fn get_target_tension(&self) -> f32 {
        self.cached_params.tension
    }

    pub fn get_computed_bpm(&self) -> f32 {
        self.cached_params.compute_bpm()
    }

    // === Rhythm Algorithm ===

    /// Set rhythm algorithm (0=Euclidean, 1=PerfectBalance, 2=ClassicGroove)
    pub fn set_algorithm(&mut self, algorithm: u8) {
        let mode = match algorithm {
            0 => RhythmMode::Euclidean,
            1 => RhythmMode::PerfectBalance,
            2 => RhythmMode::ClassicGroove,
            _ => RhythmMode::Euclidean,
        };
        let _ = self.controller.set_rhythm_mode(mode);
    }

    pub fn get_algorithm(&mut self) -> u8 {
        let _ = self.controller.poll_reports();
        match self.controller.get_state().map(|s| s.rhythm_mode) {
            Some(RhythmMode::Euclidean) => 0,
            Some(RhythmMode::PerfectBalance) => 1,
            Some(RhythmMode::ClassicGroove) => 2,
            None => 0,
        }
    }

    // === Harmony Mode ===

    /// Set harmony mode (0=Basic, 1=Driver, 2=Chart)
    pub fn set_harmony_mode(&mut self, mode: u8) {
        let harmony_mode = match mode {
            0 => HarmonyMode::Basic,
            1 => HarmonyMode::Driver,
            2 => HarmonyMode::Chart,
            _ => HarmonyMode::Driver,
        };
        let _ = self.controller.set_harmony_mode(harmony_mode);
    }

    pub fn get_harmony_mode(&mut self) -> u8 {
        let _ = self.controller.poll_reports();
        self.controller
            .get_state()
            .map(|s| match s.harmony_mode {
                HarmonyMode::Basic => 0,
                HarmonyMode::Driver => 1,
                HarmonyMode::Chart => 2,
            })
            .unwrap_or(1)
    }

    // === Harmony State Getters (from engine reports) ===

    pub fn get_current_chord_name(&mut self) -> String {
        let _ = self.controller.poll_reports();
        self.controller
            .get_state()
            .map(|s| s.current_chord.to_string())
            .unwrap_or_else(|| "?".to_string())
    }

    pub fn get_current_chord_index(&mut self) -> usize {
        let _ = self.controller.poll_reports();
        self.controller.get_state().map(|s| s.chord_root_offset as usize).unwrap_or(0)
    }

    pub fn is_current_chord_minor(&mut self) -> bool {
        let _ = self.controller.poll_reports();
        self.controller.get_state().map(|s| s.chord_is_minor).unwrap_or(false)
    }

    pub fn get_current_measure(&mut self) -> usize {
        let _ = self.controller.poll_reports();
        self.controller.get_state().map(|s| s.current_bar).unwrap_or(1)
    }

    pub fn get_current_cycle(&mut self) -> usize {
        let _ = self.controller.poll_reports();
        // Cycle = bar / progression_length
        self.controller
            .get_state()
            .map(|s| {
                if s.progression_length > 0 { s.current_bar / s.progression_length + 1 } else { 1 }
            })
            .unwrap_or(1)
    }

    pub fn get_current_step(&mut self) -> usize {
        let _ = self.controller.poll_reports();
        self.controller.get_state().map(|s| s.current_step).unwrap_or(0)
    }

    pub fn get_progression_name(&mut self) -> String {
        let _ = self.controller.poll_reports();
        self.controller
            .get_state()
            .map(|s| s.progression_name.to_string())
            .unwrap_or_else(|| "?".to_string())
    }

    pub fn get_progression_length(&mut self) -> usize {
        let _ = self.controller.poll_reports();
        self.controller.get_state().map(|s| s.progression_length).unwrap_or(4)
    }

    // === Rhythm Visualization Getters ===

    pub fn get_primary_pulses(&mut self) -> usize {
        let _ = self.controller.poll_reports();
        self.controller.get_state().map(|s| s.primary_pulses).unwrap_or(4)
    }

    pub fn get_secondary_pulses(&mut self) -> usize {
        let _ = self.controller.poll_reports();
        self.controller.get_state().map(|s| s.secondary_pulses).unwrap_or(3)
    }

    pub fn get_primary_rotation(&mut self) -> usize {
        let _ = self.controller.poll_reports();
        self.controller.get_state().map(|s| s.primary_rotation).unwrap_or(0)
    }

    pub fn get_secondary_rotation(&mut self) -> usize {
        let _ = self.controller.poll_reports();
        self.controller.get_state().map(|s| s.secondary_rotation).unwrap_or(0)
    }

    pub fn get_primary_steps(&mut self) -> usize {
        let _ = self.controller.poll_reports();
        self.controller.get_state().map(|s| s.primary_steps).unwrap_or(16)
    }

    pub fn get_secondary_steps(&mut self) -> usize {
        let _ = self.controller.poll_reports();
        self.controller.get_state().map(|s| s.secondary_steps).unwrap_or(12)
    }

    /// Get primary pattern as Vec<u8> (1=active, 0=silent) for WASM
    pub fn get_primary_pattern(&mut self) -> Vec<u8> {
        let _ = self.controller.poll_reports();
        self.controller
            .get_state()
            .map(|s| {
                let len = s.primary_steps.min(192);
                s.primary_pattern[..len].iter().map(|&b| if b { 1 } else { 0 }).collect()
            })
            .unwrap_or_else(|| vec![0; 16])
    }

    /// Get secondary pattern as Vec<u8>
    pub fn get_secondary_pattern(&mut self) -> Vec<u8> {
        let _ = self.controller.poll_reports();
        self.controller
            .get_state()
            .map(|s| {
                let len = s.secondary_steps.min(192);
                s.secondary_pattern[..len].iter().map(|&b| if b { 1 } else { 0 }).collect()
            })
            .unwrap_or_else(|| vec![0; 12])
    }

    /// Get newly generated measures as JSON array.
    ///
    /// Returns a JSON string like `[{index, tempo, time_sig_numerator, ...}, ...]`.
    /// Call this on each animation frame; the frontend should append the returned
    /// measures to its score cache for VexFlow rendering.
    /// Returns `"[]"` when no new measures are available.
    pub fn get_new_measures_json(&mut self) -> String {
        let measures = self.controller.poll_new_measures();
        serde_json::to_string(&measures).unwrap_or_else(|_| "[]".to_string())
    }

    /// Get visualization events as flat array [note, channel, step, velocity, ...]
    pub fn get_events(&mut self) -> Vec<u32> {
        let mut result = Vec::new();
        let reports = self.controller.poll_reports();
        for report in &reports {
            for note in &report.notes {
                result.push(note.note_midi as u32);
                result.push(note.channel as u32);
                result.push(0u32); // step placeholder
                result.push(note.velocity as u32);
            }
        }
        result
    }

    // === Channel Routing & Muting ===

    /// Set channel routing (-1=FundSP, >=0=Bank ID)
    pub fn set_channel_routing(&mut self, channel: usize, mode: i32) {
        if channel < 16 {
            let _ = self.controller.send(harmonium_core::EngineCommand::SetChannelRoute {
                channel: channel as u8,
                bank_id: mode,
            });
        }
    }

    /// Set channel mute
    pub fn set_channel_muted(&mut self, channel: usize, is_muted: bool) {
        if channel < 16 {
            let _ = self.controller.set_channel_mute(channel as u8, is_muted);
        }
    }

    // === Mixer Controls ===

    pub fn set_gain_lead(&mut self, gain: f32) {
        self.cached_params.gain_lead = gain.clamp(0.0, 1.0);
        let _ = self.controller.set_channel_gain(1, self.cached_params.gain_lead);
    }

    pub fn set_gain_bass(&mut self, gain: f32) {
        self.cached_params.gain_bass = gain.clamp(0.0, 1.0);
        let _ = self.controller.set_channel_gain(0, self.cached_params.gain_bass);
    }

    pub fn set_gain_snare(&mut self, gain: f32) {
        self.cached_params.gain_snare = gain.clamp(0.0, 1.0);
        let _ = self.controller.set_channel_gain(2, self.cached_params.gain_snare);
    }

    pub fn set_gain_hat(&mut self, gain: f32) {
        self.cached_params.gain_hat = gain.clamp(0.0, 1.0);
        let _ = self.controller.set_channel_gain(3, self.cached_params.gain_hat);
    }

    pub fn set_vel_base_bass(&mut self, vel: u8) {
        self.cached_params.vel_base_bass = vel.min(127);
        let _ = self.controller.send(harmonium_core::EngineCommand::SetVelocityBase {
            channel: 0,
            velocity: self.cached_params.vel_base_bass,
        });
    }

    pub fn set_vel_base_snare(&mut self, vel: u8) {
        self.cached_params.vel_base_snare = vel.min(127);
        let _ = self.controller.send(harmonium_core::EngineCommand::SetVelocityBase {
            channel: 2,
            velocity: self.cached_params.vel_base_snare,
        });
    }

    pub fn get_gain_lead(&self) -> f32 {
        self.cached_params.gain_lead
    }

    pub fn get_gain_bass(&self) -> f32 {
        self.cached_params.gain_bass
    }

    pub fn get_gain_snare(&self) -> f32 {
        self.cached_params.gain_snare
    }

    pub fn get_gain_hat(&self) -> f32 {
        self.cached_params.gain_hat
    }

    pub fn get_vel_base_bass(&self) -> u8 {
        self.cached_params.vel_base_bass
    }

    pub fn get_vel_base_snare(&self) -> u8 {
        self.cached_params.vel_base_snare
    }

    /// Set polyrhythm steps (must be multiple of 4)
    pub fn set_poly_steps(&mut self, steps: usize) {
        let valid_steps = (steps / 4) * 4;
        self.cached_params.poly_steps = valid_steps.clamp(16, 384);
        let _ = self.controller.set_rhythm_steps(self.cached_params.poly_steps);
    }

    pub fn get_poly_steps(&self) -> usize {
        self.cached_params.poly_steps
    }

    /// Add a SoundFont to a specific bank
    pub fn add_soundfont(&self, bank_id: u32, sf2_bytes: Box<[u8]>) {
        if let Ok(mut queue) = self.font_queue.lock() {
            queue.push((bank_id, sf2_bytes.into_vec()));
        }
    }

    // === Practice Logic (ported from PracticeEngine) ===

    /// Drain any new measures from the controller into the internal buffer.
    pub fn poll_measures(&mut self) {
        let new_measures = self.controller.poll_new_measures();
        if !new_measures.is_empty() {
            // Append and sort by index to ensure monotonic order
            self.measures_buffer.extend(new_measures);
            self.measures_buffer.sort_by_key(|m| m.index);
            // Deduplicate in case of overlaps
            self.measures_buffer.dedup_by_key(|m| m.index);

            // Cap buffer size (keep last 256 measures)
            if self.measures_buffer.len() > 256 {
                let to_remove = self.measures_buffer.len() - 256;
                self.measures_buffer.drain(0..to_remove);
            }
        }
    }

    /// Retrieve a range of measures from the buffer.
    fn get_buffered_measures(&self, from_bar: usize, count: usize) -> Vec<MeasureSnapshot> {
        self.measures_buffer
            .iter()
            .filter(|m| m.index >= from_bar && m.index < from_bar + count)
            .cloned()
            .collect()
    }

    /// Retrieve a range of measures from the buffer as JSON string.
    pub fn get_buffered_measures_json(&self, from_bar: usize, count: usize) -> String {
        let measures = self.get_buffered_measures(from_bar, count);
        serde_json::to_string(&measures).unwrap_or_else(|_| "[]".to_string())
    }

    /// Clear the internal measure buffer (e.g., on regeneration).
    pub fn clear_measures(&mut self) {
        self.measures_buffer.clear();
    }

    /// Get lookahead data as JSON (VexFlow-compatible).
    pub fn get_lookahead_json(&mut self, lookahead_bars: usize) -> String {
        self.poll_measures();

        let (current_bar, current_step, time_sig_num, time_sig_denom) =
            if let Some(report) = self.controller.get_state() {
                (
                    report.current_bar,
                    report.current_step,
                    report.time_signature.numerator as u8,
                    report.time_signature.denominator as u8,
                )
            } else {
                (1, 0, 4, 4)
            };

        let current_beat = (current_step as f32) / 4.0 + 1.0;
        let measures = self.get_buffered_measures(current_bar, lookahead_bars);

        let notes = self.convert_snapshots_to_notes(&measures);
        let chords = self.convert_snapshots_to_chords(&measures);

        let preceding_bpm = if current_bar > 1 {
            self.get_buffered_measures(current_bar - 1, 1)
                .first()
                .map(|m| m.composition_bpm.round() as u32)
        } else {
            None
        };
        let tempo_markings = self.build_tempo_markings(&measures, preceding_bpm);

        let first_chord_name = chords.first().map(|c| c.display_name.as_str()).unwrap_or("C");
        let scale_suggestion = self.get_scale_for_chord(first_chord_name);

        let response = LookaheadResponse {
            notes,
            chords,
            current_bar,
            current_beat,
            time_signature: (time_sig_num, time_sig_denom),
            scale_suggestion,
            tempo_markings,
        };

        serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string())
    }

    fn convert_snapshots_to_notes(&self, measures: &[MeasureSnapshot]) -> Vec<NoteEvent> {
        let mut notes = Vec::new();
        for measure in measures {
            let steps = measure.steps;
            let beats = measure.time_sig_numerator;
            if beats == 0 || steps == 0 {
                continue;
            }
            let ticks_per_beat = steps / beats;
            if ticks_per_beat == 0 {
                continue;
            }

            for note in &measure.notes {
                if note.track != 1 {
                    continue;
                }
                let beat = (note.start_step / ticks_per_beat) as f32
                    + 1.0
                    + (note.start_step % ticks_per_beat) as f32 / ticks_per_beat as f32;
                let duration_beats = note.duration_steps as f32 / ticks_per_beat as f32;
                let beats_remaining = (beats as f32 + 1.0) - beat;
                let clipped_duration = duration_beats.min(beats_remaining).max(0.25);

                notes.push(NoteEvent {
                    bar: measure.index,
                    beat,
                    duration_beats: clipped_duration,
                    pitch: note.pitch,
                    velocity: note.velocity,
                });
            }
        }
        notes.sort_by(|a, b| a.bar.cmp(&b.bar).then(a.beat.partial_cmp(&b.beat).unwrap()));
        notes
    }

    fn convert_snapshots_to_chords(&self, measures: &[MeasureSnapshot]) -> Vec<ChordEvent> {
        let mut chords = Vec::new();
        for measure in measures {
            let info = self.parse_chord_name_to_info(&measure.chord_name);
            let root_pitch = self.note_name_to_pitch(&info.root);
            chords.push(ChordEvent {
                bar: measure.index,
                beat: 1.0,
                duration_beats: measure.time_sig_numerator as f32,
                root: root_pitch + 60,
                quality: info.quality.clone(),
                display_name: info.display_name.clone(),
            });
        }
        chords
    }

    fn build_tempo_markings(
        &self,
        measures: &[MeasureSnapshot],
        preceding_bpm: Option<u32>,
    ) -> Vec<TempoMarking> {
        let mut markings = Vec::new();
        let mut prev_bpm = preceding_bpm;
        for measure in measures {
            let bpm = measure.composition_bpm.round() as u32;
            if prev_bpm != Some(bpm) {
                markings.push(TempoMarking { bar: measure.index as u32, bpm });
                prev_bpm = Some(bpm);
            }
        }
        markings
    }

    fn get_scale_for_chord(&self, chord_name: &str) -> Option<ScaleSuggestionData> {
        let info = self.parse_chord_name_to_info(chord_name);
        let root_pitch = self.note_name_to_pitch(&info.root);
        let (scale_name, intervals) = if info.quality.contains("m7b5") || info.quality.contains("ø")
        {
            ("Locrian", vec![0, 1, 3, 5, 6, 8, 10])
        } else if info.quality.contains("dim") {
            ("Diminished", vec![0, 2, 3, 5, 6, 8, 9, 11])
        } else if info.quality.contains("maj") {
            ("Lydian", vec![0, 2, 4, 6, 7, 9, 11])
        } else if info.quality.starts_with('m') || info.quality.contains("min") {
            ("Dorian", vec![0, 2, 3, 5, 7, 9, 10])
        } else if info.quality.contains('7') {
            ("Mixolydian", vec![0, 2, 4, 5, 7, 9, 10])
        } else {
            ("Lydian", vec![0, 2, 4, 6, 7, 9, 11])
        };

        let notes = intervals.iter().map(|&i| ((root_pitch + i) % 12) + 60).collect();
        Some(ScaleSuggestionData { name: format!("{} {}", info.root, scale_name), notes })
    }

    fn note_name_to_pitch(&self, name: &str) -> u8 {
        let base = match name.chars().next().unwrap_or('C') {
            'C' => 0,
            'D' => 2,
            'E' => 4,
            'F' => 5,
            'G' => 7,
            'A' => 9,
            'B' => 11,
            _ => 0,
        };
        let modifier = if name.contains('#') {
            1
        } else if name.contains('b') {
            11
        } else {
            0
        };
        (base + modifier) % 12
    }

    fn parse_chord_name_to_info(&self, chord_name: &str) -> ChordInfo {
        if chord_name.is_empty() || chord_name == "?" {
            return ChordInfo {
                root: "C".to_string(),
                quality: "".to_string(),
                display_name: "C".to_string(),
                bass: None,
            };
        }
        let mut root_end = 1;
        if chord_name.len() > 1
            && (chord_name.as_bytes()[1] == b'#' || chord_name.as_bytes()[1] == b'b')
        {
            root_end = 2;
        }
        let root = chord_name[..root_end].to_string();
        let quality = if chord_name.len() > root_end {
            chord_name[root_end..].to_string()
        } else {
            String::new()
        };
        ChordInfo {
            root: root.clone(),
            quality: quality.clone(),
            display_name: format!("{}{}", root, quality),
            bass: None,
        }
    }

    // === Playback Controls ===

    #[cfg(feature = "wasm")]
    pub fn resume(&self) -> Result<(), JsValue> {
        use cpal::traits::StreamTrait;
        self.stream.play().map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[cfg(not(feature = "wasm"))]
    pub fn resume(&self) -> Result<(), String> {
        use cpal::traits::StreamTrait;
        self.stream.play().map_err(|e| e.to_string())
    }

    #[cfg(feature = "wasm")]
    pub fn pause(&self) -> Result<(), JsValue> {
        use cpal::traits::StreamTrait;
        self.stream.pause().map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[cfg(not(feature = "wasm"))]
    pub fn pause(&self) -> Result<(), String> {
        use cpal::traits::StreamTrait;
        self.stream.pause().map_err(|e| e.to_string())
    }

    // === Timeline Controls ===
    // Mirror of NativeHandle::seek/set_loop/clear_loop, but routed through
    // HarmoniumController so the same behavior runs in the browser.
    // 1-based bars (matching the Tauri/Practice surface).

    /// Deterministic seek: reset RNG + generator, replay to target bar.
    pub fn seek(&mut self, bar: u32) {
        let _ = self.controller.seek(bar.max(1) as usize);
    }

    /// Seek playhead without resetting the writehead — uses already-generated
    /// measures.
    pub fn seek_playhead(&mut self, bar: u32) {
        let _ = self.controller.seek_playhead(bar.max(1) as usize);
    }

    /// Set a loop region (1-based, inclusive).
    pub fn set_loop(&mut self, start_bar: u32, end_bar: u32) {
        if start_bar < 1 || end_bar < start_bar {
            return;
        }
        let _ = self
            .controller
            .set_loop(start_bar as usize, end_bar as usize);
    }

    pub fn clear_loop(&mut self) {
        let _ = self.controller.clear_loop();
    }

    /// Generate a new melody with a fresh random seed. Maps the
    /// composer-side new_melody onto the wasm controller path so the
    /// browser shuffle / regenerate buttons actually produce new content.
    pub fn new_melody(&mut self) {
        let _ = self.controller.send(harmonium_core::EngineCommand::NewMelody);
    }

    pub fn set_seed(&mut self, seed: u64) {
        let _ = self
            .controller
            .send(harmonium_core::EngineCommand::SetSeed(seed));
    }

    // === Recording ===

    pub fn start_recording_wav(&mut self) {
        let _ = self.controller.start_recording(events::RecordFormat::Wav);
    }

    pub fn stop_recording_wav(&mut self) {
        let _ = self.controller.stop_recording(events::RecordFormat::Wav);
    }

    pub fn start_recording_midi(&mut self) {
        let _ = self.controller.start_recording(events::RecordFormat::Midi);
    }

    pub fn stop_recording_midi(&mut self) {
        let _ = self.controller.stop_recording(events::RecordFormat::Midi);
    }

    pub fn start_recording_musicxml(&mut self) {
        let _ = self.controller.start_recording(events::RecordFormat::MusicXml);
    }

    pub fn stop_recording_musicxml(&mut self) {
        let _ = self.controller.stop_recording(events::RecordFormat::MusicXml);
    }

    pub fn pop_finished_recording(&self) -> Option<RecordedData> {
        if let Ok(mut queue) = self.finished_recordings.lock()
            && let Some((fmt, data)) = queue.pop() as Option<(events::RecordFormat, Vec<u8>)>
        {
            let format_str = match fmt {
                events::RecordFormat::Wav => "wav".to_string(),
                events::RecordFormat::Midi => "midi".to_string(),
                events::RecordFormat::MusicXml => "musicxml".to_string(),
            };
            return Some(RecordedData { format_str, data });
        }
        None
    }

    // === Control Mode ===

    /// Switch to emotion mode (arousal/valence/density/tension sliders)
    pub fn use_emotion_mode(&mut self) {
        let _ = self.controller.use_emotion_mode();
    }

    /// Switch to direct technical control mode
    pub fn use_direct_mode(&mut self) {
        let _ = self.controller.use_direct_mode();
    }

    /// Returns true if in emotion mode
    pub fn is_emotion_mode(&self) -> bool {
        self.controller.get_mode() == harmonium_core::ControlMode::Emotion
    }

    // === Direct Mode Controls ===

    pub fn set_direct_bpm(&mut self, bpm: f32) {
        let _ = self.controller.set_bpm(bpm.clamp(30.0, 300.0));
    }

    pub fn get_direct_bpm(&mut self) -> f32 {
        let _ = self.controller.poll_reports();
        self.controller.get_state().map(|s| s.musical_params.bpm).unwrap_or(120.0)
    }

    pub fn set_direct_enable_rhythm(&mut self, enabled: bool) {
        let _ = self.controller.enable_rhythm(enabled);
    }

    pub fn set_direct_enable_harmony(&mut self, enabled: bool) {
        let _ = self.controller.enable_harmony(enabled);
    }

    pub fn set_direct_enable_melody(&mut self, enabled: bool) {
        let _ = self.controller.enable_melody(enabled);
    }

    pub fn set_direct_enable_voicing(&mut self, enabled: bool) {
        let _ = self.controller.enable_voicing(enabled);
    }

    pub fn set_direct_fixed_kick(&mut self, enabled: bool) {
        self.cached_params.fixed_kick = enabled;
        let _ = self.controller.send(harmonium_core::EngineCommand::SetFixedKick(enabled));
    }

    pub fn get_direct_fixed_kick(&self) -> bool {
        self.cached_params.fixed_kick
    }

    /// Set all rhythm parameters at once
    #[allow(clippy::too_many_arguments)]
    pub fn set_all_rhythm_params(
        &mut self,
        mode: u8,
        steps: usize,
        pulses: usize,
        rotation: usize,
        density: f32,
        tension: f32,
        secondary_steps: usize,
        secondary_pulses: usize,
        secondary_rotation: usize,
    ) {
        let rhythm_mode = match mode {
            0 => RhythmMode::Euclidean,
            1 => RhythmMode::PerfectBalance,
            2 => RhythmMode::ClassicGroove,
            _ => RhythmMode::Euclidean,
        };
        let valid_steps = (steps / 4) * 4;
        let _ = self.controller.send(harmonium_core::EngineCommand::SetAllRhythmParams {
            mode: rhythm_mode,
            steps: valid_steps.clamp(16, 384),
            pulses: pulses.clamp(1, 32),
            rotation,
            density: density.clamp(0.0, 1.0),
            tension: tension.clamp(0.0, 1.0),
            secondary_steps: secondary_steps.clamp(4, 32),
            secondary_pulses: secondary_pulses.clamp(1, 32),
            secondary_rotation,
        });
    }

    pub fn set_direct_rhythm_mode(&mut self, mode: u8) {
        let rhythm_mode = match mode {
            0 => RhythmMode::Euclidean,
            1 => RhythmMode::PerfectBalance,
            2 => RhythmMode::ClassicGroove,
            _ => RhythmMode::Euclidean,
        };
        let _ = self.controller.set_rhythm_mode(rhythm_mode);
    }

    pub fn set_direct_rhythm_steps(&mut self, steps: usize) {
        let valid_steps = (steps / 4) * 4;
        let _ = self.controller.set_rhythm_steps(valid_steps.clamp(16, 384));
    }

    pub fn set_direct_rhythm_pulses(&mut self, pulses: usize) {
        let _ = self.controller.set_rhythm_pulses(pulses.clamp(1, 32));
    }

    pub fn set_direct_rhythm_rotation(&mut self, rotation: usize) {
        let _ = self.controller.set_rhythm_rotation(rotation);
    }

    pub fn set_direct_rhythm_density(&mut self, density: f32) {
        let _ = self.controller.set_rhythm_density(density.clamp(0.0, 1.0));
    }

    pub fn set_direct_rhythm_tension(&mut self, tension: f32) {
        let _ = self.controller.set_rhythm_tension(tension.clamp(0.0, 1.0));
    }

    pub fn set_direct_secondary_steps(&mut self, steps: usize) {
        let _ = self.controller.poll_reports();
        let cur_pulses = self.controller.get_state().map(|s| s.secondary_pulses).unwrap_or(3);
        let cur_rotation = self.controller.get_state().map(|s| s.secondary_rotation).unwrap_or(0);
        let _ = self.controller.send(harmonium_core::EngineCommand::SetRhythmSecondary {
            steps: steps.clamp(4, 32),
            pulses: cur_pulses,
            rotation: cur_rotation,
        });
    }

    pub fn set_direct_secondary_pulses(&mut self, pulses: usize) {
        let _ = self.controller.poll_reports();
        let cur_steps = self.controller.get_state().map(|s| s.secondary_steps).unwrap_or(12);
        let cur_rotation = self.controller.get_state().map(|s| s.secondary_rotation).unwrap_or(0);
        let _ = self.controller.send(harmonium_core::EngineCommand::SetRhythmSecondary {
            steps: cur_steps,
            pulses: pulses.clamp(1, 32),
            rotation: cur_rotation,
        });
    }

    pub fn set_direct_secondary_rotation(&mut self, rotation: usize) {
        let _ = self.controller.poll_reports();
        let cur_steps = self.controller.get_state().map(|s| s.secondary_steps).unwrap_or(12);
        let cur_pulses = self.controller.get_state().map(|s| s.secondary_pulses).unwrap_or(3);
        let _ = self.controller.send(harmonium_core::EngineCommand::SetRhythmSecondary {
            steps: cur_steps,
            pulses: cur_pulses,
            rotation,
        });
    }

    pub fn set_direct_harmony_mode(&mut self, mode: u8) {
        let harmony_mode = match mode {
            0 => HarmonyMode::Basic,
            1 => HarmonyMode::Driver,
            2 => HarmonyMode::Chart,
            _ => HarmonyMode::Driver,
        };
        let _ = self.controller.set_harmony_mode(harmony_mode);
    }

    pub fn set_direct_harmony_tension(&mut self, tension: f32) {
        let _ = self.controller.set_harmony_tension(tension.clamp(0.0, 1.0));
    }

    pub fn set_direct_harmony_valence(&mut self, valence: f32) {
        let _ = self.controller.set_harmony_valence(valence.clamp(-1.0, 1.0));
    }

    pub fn set_direct_melody_smoothness(&mut self, smoothness: f32) {
        let _ = self.controller.set_melody_smoothness(smoothness.clamp(0.0, 1.0));
    }

    pub fn set_direct_voicing_density(&mut self, density: f32) {
        let _ = self.controller.set_voicing_density(density.clamp(0.0, 1.0));
    }

    pub fn set_direct_voicing_tension(&mut self, tension: f32) {
        let _ = self
            .controller
            .send(harmonium_core::EngineCommand::SetVoicingTension(tension.clamp(0.0, 1.0)));
    }

    pub fn get_direct_params_json(&mut self) -> String {
        let _ = self.controller.poll_reports();
        self.controller
            .get_state()
            .map(|s| serde_json::to_string(&s.musical_params).unwrap_or_else(|_| "{}".to_string()))
            .unwrap_or_else(|| "{}".to_string())
    }

    pub fn set_direct_params_json(&mut self, json: &str) {
        if let Ok(params) = serde_json::from_str::<MusicalParams>(json) {
            let _ = self.controller.set_bpm(params.bpm);
            let _ = self.controller.set_rhythm_mode(params.rhythm_mode);
            let _ = self.controller.set_rhythm_density(params.rhythm_density);
            let _ = self.controller.set_harmony_tension(params.harmony_tension);
            let _ = self.controller.set_harmony_valence(params.harmony_valence);
            let _ = self.controller.set_melody_smoothness(params.melody_smoothness);
        }
    }

    // === Direct Mode Getters (from engine reports) ===

    pub fn get_direct_enable_rhythm(&mut self) -> bool {
        let _ = self.controller.poll_reports();
        self.controller.get_state().map(|s| s.musical_params.enable_rhythm).unwrap_or(true)
    }

    pub fn get_direct_enable_harmony(&mut self) -> bool {
        let _ = self.controller.poll_reports();
        self.controller.get_state().map(|s| s.musical_params.enable_harmony).unwrap_or(true)
    }

    pub fn get_direct_enable_melody(&mut self) -> bool {
        let _ = self.controller.poll_reports();
        self.controller.get_state().map(|s| s.musical_params.enable_melody).unwrap_or(true)
    }

    pub fn get_direct_enable_voicing(&mut self) -> bool {
        let _ = self.controller.poll_reports();
        self.controller.get_state().map(|s| s.musical_params.enable_voicing).unwrap_or(false)
    }

    pub fn get_direct_rhythm_mode(&mut self) -> u8 {
        let _ = self.controller.poll_reports();
        self.controller
            .get_state()
            .map(|s| match s.rhythm_mode {
                RhythmMode::Euclidean => 0,
                RhythmMode::PerfectBalance => 1,
                RhythmMode::ClassicGroove => 2,
            })
            .unwrap_or(0)
    }

    pub fn get_direct_rhythm_steps(&mut self) -> usize {
        let _ = self.controller.poll_reports();
        self.controller.get_state().map(|s| s.primary_steps).unwrap_or(16)
    }

    pub fn get_direct_rhythm_pulses(&mut self) -> usize {
        let _ = self.controller.poll_reports();
        self.controller.get_state().map(|s| s.primary_pulses).unwrap_or(4)
    }

    pub fn get_direct_rhythm_rotation(&mut self) -> usize {
        let _ = self.controller.poll_reports();
        self.controller.get_state().map(|s| s.primary_rotation).unwrap_or(0)
    }

    pub fn get_direct_rhythm_density(&mut self) -> f32 {
        let _ = self.controller.poll_reports();
        self.controller.get_state().map(|s| s.musical_params.rhythm_density).unwrap_or(0.5)
    }

    pub fn get_direct_rhythm_tension(&mut self) -> f32 {
        let _ = self.controller.poll_reports();
        self.controller.get_state().map(|s| s.musical_params.rhythm_tension).unwrap_or(0.3)
    }

    pub fn get_direct_secondary_steps(&mut self) -> usize {
        let _ = self.controller.poll_reports();
        self.controller.get_state().map(|s| s.secondary_steps).unwrap_or(12)
    }

    pub fn get_direct_secondary_pulses(&mut self) -> usize {
        let _ = self.controller.poll_reports();
        self.controller.get_state().map(|s| s.secondary_pulses).unwrap_or(3)
    }

    pub fn get_direct_secondary_rotation(&mut self) -> usize {
        let _ = self.controller.poll_reports();
        self.controller.get_state().map(|s| s.secondary_rotation).unwrap_or(0)
    }

    pub fn get_direct_harmony_tension(&mut self) -> f32 {
        let _ = self.controller.poll_reports();
        self.controller.get_state().map(|s| s.musical_params.harmony_tension).unwrap_or(0.3)
    }

    pub fn get_direct_harmony_valence(&mut self) -> f32 {
        let _ = self.controller.poll_reports();
        self.controller.get_state().map(|s| s.musical_params.harmony_valence).unwrap_or(0.3)
    }

    pub fn get_direct_melody_smoothness(&mut self) -> f32 {
        let _ = self.controller.poll_reports();
        self.controller.get_state().map(|s| s.musical_params.melody_smoothness).unwrap_or(0.7)
    }

    pub fn get_direct_voicing_density(&mut self) -> f32 {
        let _ = self.controller.poll_reports();
        self.controller.get_state().map(|s| s.musical_params.voicing_density).unwrap_or(0.5)
    }

    pub fn get_direct_voicing_tension(&mut self) -> f32 {
        let _ = self.controller.poll_reports();
        self.controller.get_state().map(|s| s.musical_params.voicing_tension).unwrap_or(0.3)
    }
}

#[cfg(all(feature = "standalone", feature = "wasm"))]
#[wasm_bindgen]
pub fn start(sf2_bytes: Option<Box<[u8]>>) -> Result<Handle, JsValue> {
    start_with_backend(sf2_bytes, "fundsp")
}

/// Start Harmonium with a specific audio backend
/// backend: "fundsp" (default) or "odin2" (if compiled with odin2 feature)
#[cfg(all(feature = "standalone", feature = "wasm"))]
#[wasm_bindgen]
pub fn start_with_backend(sf2_bytes: Option<Box<[u8]>>, backend: &str) -> Result<Handle, JsValue> {
    console_error_panic_hook::set_once();

    // Parse backend type
    let backend_type = match backend.to_lowercase().as_str() {
        "fundsp" | "synth" | "default" => audio::AudioBackendType::FundSP,
        #[cfg(feature = "odin2")]
        "odin2" | "odin" => audio::AudioBackendType::Odin2,
        _ => {
            log::warn(&format!("Unknown backend '{}', using FundSP", backend));
            audio::AudioBackendType::FundSP
        }
    };

    let (stream, config, controller, font_queue, finished_recordings) =
        audio::create_timeline_stream_legacy(sf2_bytes.as_deref(), backend_type)
            .map_err(|e| JsValue::from_str(&e))?;

    Ok(Handle {
        stream,
        controller,
        font_queue,
        finished_recordings,
        measures_buffer: Vec::new(),
        cached_params: harmonium_core::EngineParams::default(),
        bpm: config.bpm,
        key: config.key,
        scale: config.scale,
        pulses: config.pulses,
        steps: config.steps,
    })
}

/// Get list of available audio backends
#[cfg(all(feature = "standalone", feature = "wasm"))]
#[wasm_bindgen]
pub fn get_available_backends() -> Vec<JsValue> {
    let mut backends = vec![JsValue::from_str("fundsp")];
    #[cfg(feature = "odin2")]
    backends.push(JsValue::from_str("odin2"));
    backends
}
