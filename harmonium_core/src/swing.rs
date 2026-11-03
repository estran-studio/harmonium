//! Swing selon le tempo, par piste (phase 019, A6).
//!
//! Une courbe de données donne, pour chaque piste, le rapport de swing en fonction du tempo
//! (`[swing.tracks.<Piste>]`, interpolation linéaire entre points, bornée aux extrémités : aucune
//! extrapolation hors des tempos mesurés). Le rapport de la piste de référence
//! (`reference`, la percussion aiguë par défaut) est celui de la **grille** : il part dans
//! `Measure::swing_ratio` et le lecteur (`time_plan::BarTiming`) l'applique à tous les pas. Les
//! autres pistes dont le rapport diffère sont replacées sur leur propre swing par un décalage de
//! chaque attaque, en millièmes de pas (`NotePerformance::offset`) : la position musicale d'une
//! croche de contretemps change, jamais la grille ni la longueur des mesures. Le décalage est
//! une fonction du rapport et non du tempo : le rejeu après un seek, l'export MIDI et le plan
//! partagé reproduisent la même placement.
//!
//! Priorité : une commande directe de l'hôte (`MusicalParams::swing_explicit`, posé par
//! `MusicComposer::set_swing_ratio`) l'emporte sur toute courbe, pour toutes les pistes. Sans
//! elle, l'écart entre le rapport demandé (`MusicalParams::swing_ratio`) et le défaut du profil
//! (`meter.default_swing_ratio`) décale la courbe de toutes les pistes (un dosage tiré par graine
//! reste donc possible : un seul point d'entrée, [`SwingByTempo::resolve`]).
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    style_profile::ProfileError,
    time_plan::BarTiming,
    timeline::{Measure, TrackId},
};

/// Rapport de swing borné comme `MusicalParams::set_swing_ratio`.
pub const MIN_RATIO: f32 = 0.5;
pub const MAX_RATIO: f32 = 0.75;
/// Plus grand écart toléré entre une piste et la référence : au-delà, le décalage de ses
/// attaques sortirait du demi-pas que borne l'interprétation.
pub const MAX_TRACK_GAP: f32 = 0.12;
/// Points d'une courbe, au plus.
pub const MAX_POINTS: usize = 16;
/// Décalage maximal d'une attaque, en millièmes de pas (le même que `performance`).
const MAX_OFFSET: f64 = 490.0;

fn default_reference() -> TrackId {
    TrackId::PercussionHigh
}

/// Courbe tempo → rapport de swing : points `[BPM, rapport]` à BPM strictement croissants.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SwingCurve {
    pub points: Vec<[f32; 2]>,
}

impl SwingCurve {
    /// Rapport au tempo `bpm` : interpolation linéaire, valeur de l'extrémité hors des points.
    #[must_use]
    pub fn at(&self, bpm: f32) -> f32 {
        let (Some(first), Some(last)) = (self.points.first(), self.points.last()) else {
            return MIN_RATIO;
        };
        if bpm.is_nan() || bpm <= first[0] {
            return first[1];
        }
        if bpm >= last[0] {
            return last[1];
        }
        for pair in self.points.windows(2) {
            if bpm <= pair[1][0] {
                let phase = (bpm - pair[0][0]) / (pair[1][0] - pair[0][0]);
                return pair[0][1] + (pair[1][1] - pair[0][1]) * phase;
            }
        }
        last[1]
    }
}

/// Swing par piste selon le tempo (`[swing]` de `profile.toml`, ou `swing.toml` via `[files]`).
/// Absent : un seul rapport pour toutes les pistes (`meter.default_swing_ratio`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SwingByTempo {
    /// Piste dont le rapport est celui de la grille (le lecteur le lit dans la mesure). Les
    /// autres pistes sont décalées de leur écart à elle. Défaut : `PercussionHigh` (le ride).
    #[serde(default = "default_reference")]
    pub reference: TrackId,
    /// Une courbe par piste ; une piste sans courbe suit la grille.
    pub tracks: BTreeMap<TrackId, SwingCurve>,
}

/// Rapports d'une mesure : celui de la grille et, par piste de `TrackId::ALL`, celui de la piste
/// quand il diffère de la grille.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedSwing {
    pub grid: f32,
    pub tracks: [Option<f32>; TrackId::ALL.len()],
}

impl ResolvedSwing {
    /// Un seul rapport pour toutes les pistes : le comportement sans courbe.
    #[must_use]
    pub const fn uniform(ratio: f32) -> Self {
        Self { grid: ratio, tracks: [None; TrackId::ALL.len()] }
    }

    /// Rapport de `track` (celui de la grille si elle n'a pas le sien).
    #[must_use]
    pub fn of(&self, track: TrackId) -> f32 {
        self.tracks[track_index(track)].unwrap_or(self.grid)
    }
}

fn track_index(track: TrackId) -> usize {
    TrackId::ALL.iter().position(|&t| t == track).unwrap_or(0)
}

impl SwingByTempo {
    /// Rapports de la mesure au tempo `bpm`. `requested` : `MusicalParams::swing_ratio` ;
    /// `default` : `meter.default_swing_ratio`. Le rapport d'une piste est sa courbe décalée de
    /// `requested − default`, borné à `0,5..=0,75`.
    #[must_use]
    pub fn resolve(&self, bpm: f32, requested: f32, default: f32) -> ResolvedSwing {
        let shift = requested - default;
        let value = |curve: &SwingCurve| (curve.at(bpm) + shift).clamp(MIN_RATIO, MAX_RATIO);
        let grid = self.tracks.get(&self.reference).map_or(requested, value);
        let mut tracks = [None; TrackId::ALL.len()];
        for (&track, curve) in &self.tracks {
            let ratio = value(curve);
            if (ratio - grid).abs() > f32::EPSILON {
                tracks[track_index(track)] = Some(ratio);
            }
        }
        ResolvedSwing { grid, tracks }
    }

    /// Valide les courbes ; les erreurs portent le chemin du champ.
    pub fn validate(&self) -> Result<(), ProfileError> {
        let fail = |field: String, message: &str| Err(ProfileError::new(field, message));
        if self.tracks.is_empty() {
            return fail("swing.tracks".into(), "au moins une piste requise");
        }
        for (track, curve) in &self.tracks {
            let field = format!("swing.tracks.{}", track.name());
            let points = &curve.points;
            if !(1..=MAX_POINTS).contains(&points.len()) {
                return fail(format!("{field}.points"), "1 à 16 points requis");
            }
            for (i, point) in points.iter().enumerate() {
                if !point[0].is_finite() || !(20.0..=400.0).contains(&point[0]) {
                    return fail(format!("{field}.points[{i}]"), "tempo fini dans 20..400 BPM");
                }
                if !point[1].is_finite() || !(MIN_RATIO..=MAX_RATIO).contains(&point[1]) {
                    return fail(format!("{field}.points[{i}]"), "rapport fini dans 0.5..0.75");
                }
                if i > 0 && point[0] <= points[i - 1][0] {
                    return fail(format!("{field}.points"), "tempos strictement croissants requis");
                }
            }
        }
        if let Some(reference) = self.tracks.get(&self.reference) {
            // Les deux courbes sont linéaires par morceaux : l'écart maximal tombe à un point
            // de l'une d'elles.
            for (track, curve) in &self.tracks {
                let worst = curve
                    .points
                    .iter()
                    .chain(&reference.points)
                    .map(|point| (curve.at(point[0]) - reference.at(point[0])).abs())
                    .fold(0.0_f32, f32::max);
                if worst > MAX_TRACK_GAP + 1e-4 {
                    return fail(
                        format!("swing.tracks.{}", track.name()),
                        "écart à la piste de référence limité à 0.12",
                    );
                }
            }
        }
        Ok(())
    }
}

/// Position, en fraction de la noire, du pas fractionnaire `p` (en pas depuis le début de la
/// noire, de `unit` pas) sous le rapport `ratio` ; `None` : grille droite. Le premier segment va
/// de 0 au pas `knot`, le second au pas `unit` (voir `BarTiming::step_offset`). Linéaire de
/// part et d'autre : définie aussi pour `p` hors de `0..unit` (attaque en avance ou en retard).
fn fraction(unit: f64, knot: f64, ratio: Option<f64>, p: f64) -> f64 {
    match ratio {
        None => p / unit,
        Some(r) if p <= knot => p * r / knot,
        Some(r) => r + (p - knot) * (1.0 - r) / (unit - knot),
    }
}

/// Réciproque de [`fraction`].
fn step_of(unit: f64, knot: f64, ratio: Option<f64>, f: f64) -> f64 {
    match ratio {
        None => f * unit,
        Some(r) if f <= r => f * knot / r,
        Some(r) => knot + (f - r) * (unit - knot) / (1.0 - r),
    }
}

fn swung(ratio: f32) -> Option<f64> {
    (ratio > MIN_RATIO).then(|| f64::from(ratio))
}

/// Replace les attaques de chaque piste sur son propre rapport de swing, la grille de `measure`
/// étant swinguée à `swing.grid` : une attaque à la position `p` de la grille (pas entier plus
/// décalage) est jouée là où le rapport de sa piste placerait `p`. Les points d'appui (début de
/// noire) ne bougent pas. Sans effet pour les pistes au rapport de la grille et pour une grille
/// qui ne swingue pas (mesure composée : `BarTiming::swing_unit_steps` vaut `None`).
pub fn apply_track_swing(measure: &mut Measure, swing: &ResolvedSwing) {
    if swing.tracks.iter().all(Option::is_none) {
        return;
    }
    let Some(unit) = BarTiming::of(measure).swing_unit_steps() else { return };
    let unit = f64::from(unit);
    let knot = (unit / 2.0).ceil();
    let grid = swung(swing.grid);
    for track in TrackId::ALL {
        let Some(ratio) = swing.tracks[track_index(track)] else { continue };
        let wanted = swung(ratio);
        let Some(notes) = measure.tracks.get_mut(&track) else { continue };
        for note in notes {
            let position = note.start_step as f64 + f64::from(note.performance.offset) / 1000.0;
            let whole = (position / unit).floor();
            let within = position - whole * unit;
            let placed =
                whole * unit + step_of(unit, knot, grid, fraction(unit, knot, wanted, within));
            let offset =
                ((placed - note.start_step as f64) * 1000.0).round().clamp(-MAX_OFFSET, MAX_OFFSET);
            note.performance.offset = offset as i16;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn curve(points: &[[f32; 2]]) -> SwingCurve {
        SwingCurve { points: points.to_vec() }
    }

    #[test]
    fn a_curve_interpolates_and_holds_its_ends() {
        let c = curve(&[[140.0, 0.72], [240.0, 0.62]]);
        assert!((c.at(100.0) - 0.72).abs() < 1e-6);
        assert!((c.at(190.0) - 0.67).abs() < 1e-6);
        assert!((c.at(300.0) - 0.62).abs() < 1e-6);
    }

    #[test]
    fn the_grid_takes_the_reference_curve_and_others_keep_their_gap() {
        let swing = SwingByTempo {
            reference: TrackId::PercussionHigh,
            tracks: BTreeMap::from([
                (TrackId::PercussionHigh, curve(&[[100.0, 0.7], [300.0, 0.6]])),
                (TrackId::Bass, curve(&[[100.0, 0.7], [300.0, 0.65]])),
            ]),
        };
        let resolved = swing.resolve(200.0, 0.71, 0.71);
        assert!((resolved.grid - 0.65).abs() < 1e-6);
        assert!((resolved.of(TrackId::Bass) - 0.675).abs() < 1e-6);
        assert!((resolved.of(TrackId::Chords) - resolved.grid).abs() < 1e-9);
        // Dosage : l'écart au défaut décale toutes les courbes, borné à 0,5..0,75.
        let shifted = swing.resolve(200.0, 0.76, 0.71);
        assert!((shifted.grid - 0.70).abs() < 1e-6);
    }

    #[test]
    fn remapping_is_the_identity_at_the_grid_ratio_and_fixes_the_beats() {
        let (unit, knot) = (4.0, 2.0);
        for p in [-0.4, 0.0, 0.7, 2.0, 3.3, 4.0] {
            let same = step_of(unit, knot, Some(0.66), fraction(unit, knot, Some(0.66), p));
            assert!((same - p).abs() < 1e-9, "{p}");
        }
        // Le contretemps (pas 2) d'une piste à 0,6 sur une grille à 0,7 : en avance d'un peu
        // moins de 0,5 pas ; les temps ne bougent pas.
        let early = step_of(unit, knot, Some(0.7), fraction(unit, knot, Some(0.6), 2.0));
        assert!((early - (1.0 + (0.6 - 0.35) / 0.35)).abs() < 1e-9);
        assert!(step_of(unit, knot, Some(0.7), fraction(unit, knot, Some(0.6), 4.0)) - 4.0 < 1e-9);
    }

    #[test]
    fn schema_errors_name_the_field() {
        let bad = |points: &[[f32; 2]]| SwingByTempo {
            reference: TrackId::PercussionHigh,
            tracks: BTreeMap::from([(TrackId::Bass, curve(points))]),
        };
        assert!(bad(&[[100.0, 0.7], [100.0, 0.6]]).validate().is_err());
        assert!(bad(&[[100.0, 0.9]]).validate().is_err());
        assert!(bad(&[]).validate().is_err());
        assert!(bad(&[[100.0, 0.7], [200.0, 0.6]]).validate().is_ok());
        let far = SwingByTempo {
            reference: TrackId::PercussionHigh,
            tracks: BTreeMap::from([
                (TrackId::PercussionHigh, curve(&[[100.0, 0.55]])),
                (TrackId::Bass, curve(&[[100.0, 0.75]])),
            ]),
        };
        assert!(far.validate().is_err());
    }
}
