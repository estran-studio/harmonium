//! Profil distribué `nt_desert` : arche, voix et émotions observées dans les
//! snapshots et le lecteur public, sans figer les tirages ni recopier le TOML.
use std::collections::{BTreeMap, BTreeSet};

use harmonium_core::{
    StyleProfile, TuningParams,
    events::AudioEvent,
    mapper::GameControls,
    params::MusicalParams,
    report::{MeasureSnapshot, NoteSnapshot},
    timeline::{Measure, Playhead},
    trad::TradGenerator,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn load() -> TestResult<StyleProfile> {
    Ok(StyleProfile::load(format!("{}/../profiles/nt_desert", env!("CARGO_MANIFEST_DIR")))?)
}

fn params(p: &StyleProfile) -> MusicalParams {
    MusicalParams {
        bpm: p.meter.default_bpm,
        time_signature: p.meter.time_signature,
        swing_ratio: p.meter.default_swing_ratio,
        harmony_valence: p.dynamics.neutral_valence,
        ..Default::default()
    }
}

fn stream_with_params(
    p: &StyleProfile,
    par: MusicalParams,
    seed: u64,
    bars: usize,
) -> TestResult<Vec<MeasureSnapshot>> {
    p.validate()?;
    let mut g = TradGenerator::new(par, TuningParams { profile: p.clone() });
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    Ok((1..=bars)
        .map(|bar| MeasureSnapshot::from_measure(&g.generate_measure(bar, &mut rng)))
        .collect())
}

fn stream(p: &StyleProfile, seed: u64, bars: usize) -> TestResult<Vec<MeasureSnapshot>> {
    stream_with_params(p, params(p), seed, bars)
}

fn emotional_stream(p: &StyleProfile, emotions: [f32; 4]) -> TestResult<Vec<MeasureSnapshot>> {
    let mut par = params(p);
    let mut controls = GameControls::new(p.meter.default_bpm);
    controls.set_emotion_mode(true, &par);
    controls.set_emotions(emotions, p, &mut par);
    stream_with_params(p, par, 1, 96)
}

fn notes(s: &MeasureSnapshot, track: u8) -> Vec<&NoteSnapshot> {
    s.notes.iter().filter(|n| n.track == track).collect()
}

// Le motif écrit ne dépend pas des vélocités et décalages d'interprétation.
fn motif(s: &MeasureSnapshot, track: u8) -> Vec<(u8, usize, usize)> {
    notes(s, track).iter().map(|n| (n.pitch, n.start_step, n.duration_steps)).collect()
}

#[test]
fn arche_48_entrees_et_pont_de_nappes() -> TestResult {
    let p = load()?;
    for seed in 1..=5 {
        let snapshots = stream(&p, seed, 96)?;
        for (i, s) in snapshots.iter().enumerate() {
            let local = i % 48;
            assert!(!s.notes.is_empty(), "mesure {} vide (graine {seed})", s.index);
            assert_eq!((s.time_sig_numerator, s.time_sig_denominator), (12, 8));
            if (16..32).contains(&local) {
                assert!(
                    s.notes.iter().all(|n| n.track == 1),
                    "pont avec une autre voix que la nappe, mesure {} (graine {seed})",
                    s.index
                );
            } else {
                assert!(!notes(s, 2).is_empty() || (32..40).contains(&local));
            }
            if local < 8 || local >= 40 {
                let pad = notes(s, 1);
                assert_eq!(pad.len(), 1, "pédale seule, mesure {}", s.index);
                assert_eq!(pad[0].pitch % 12, 8, "pédale G#, mesure {}", s.index);
            } else {
                let pad = notes(s, 1);
                let classes: BTreeSet<_> = pad.iter().map(|n| n.pitch % 12).collect();
                assert_eq!(pad.len(), 4, "triade doublée, mesure {}", s.index);
                assert_eq!(classes.len(), 3, "trois classes d'accord, mesure {}", s.index);
                assert!(
                    pad.iter().all(|n| n.start_step == 0 && n.duration_steps == s.steps),
                    "nappe non tenue, mesure {}",
                    s.index
                );
            }
        }
        // Le premier A1 ouvre sans pieds ni lead ; la voix entre en A2 et
        // revient après le pont, notamment sur la reprise A1'.
        for s in &snapshots[..8] {
            for track in [0, 3, 4, 5] {
                assert!(
                    notes(s, track).is_empty(),
                    "entrée précoce piste {track}, mes. {}",
                    s.index
                );
            }
        }
        for start in [0, 48] {
            for s in
                snapshots[start + 8..start + 16].iter().chain(&snapshots[start + 32..start + 48])
            {
                for track in [0, 3, 4] {
                    assert!(
                        !notes(s, track).is_empty(),
                        "voix attendue piste {track}, mesure {} (graine {seed})",
                        s.index
                    );
                }
            }
            assert!(
                snapshots[start + 32..start + 40].iter().any(|s| !notes(s, 2).is_empty()),
                "basse absente de tout B"
            );
            assert!(
                snapshots[start + 32..start + 40].iter().any(|s| notes(s, 2).is_empty()),
                "B sans respiration de basse"
            );
        }
        for s in &snapshots[8..16] {
            let classes: BTreeSet<_> = notes(s, 1).iter().map(|n| n.pitch % 12).collect();
            assert_eq!(classes, BTreeSet::from([2, 5, 9]), "A2 en Dm, mesure {}", s.index);
        }
    }
    Ok(())
}

#[test]
fn contrevoix_uniquement_sur_les_huit_dernieres_mesures_du_second_tour() -> TestResult {
    let p = load()?;
    for seed in 1..=5 {
        let snapshots = stream(&p, seed, 96)?;
        for s in &snapshots[..88] {
            assert!(
                notes(s, 5).is_empty(),
                "contrevoix précoce, mesure {} (graine {seed})",
                s.index
            );
        }
        let ending = &snapshots[88..96];
        assert!(
            ending.iter().any(|s| !notes(s, 5).is_empty()),
            "contrevoix absente de la reprise finale (graine {seed})"
        );
        for s in ending {
            let lead_low = notes(s, 0).iter().map(|n| n.pitch).min().ok_or("dessus absent")?;
            assert!(
                notes(s, 5).iter().all(|n| n.pitch < lead_low),
                "doublure hors du registre inférieur (graine {seed}, mesure {})",
                s.index
            );
        }
    }
    Ok(())
}

#[test]
fn motifs_reconnaissables_et_second_tour_varie_sans_changer_accompagnement() -> TestResult {
    let p = load()?;
    let mut copies = 0;
    for seed in 1..=5 {
        let snapshots = stream(&p, seed, 96)?;
        for (a, b) in [(8, 9), (8, 12), (8, 14), (10, 11), (10, 13), (10, 15)] {
            copies += usize::from(motif(&snapshots[a], 0) == motif(&snapshots[b], 0));
        }
    }
    assert!(copies > 0, "aucune répétition reconnaissable du motif en A2");

    let snapshots = stream(&p, 1, 96)?;
    assert!(
        (8..16).chain(32..48).any(|i| motif(&snapshots[i], 0) != motif(&snapshots[i + 48], 0)),
        "aucune variation mélodique effective au second tour, graine 1"
    );
    for i in 0..48 {
        for track in [1, 2] {
            assert_eq!(
                motif(&snapshots[i], track),
                motif(&snapshots[i + 48], track),
                "variation d'accompagnement piste {track}, mesure {}",
                i + 1
            );
        }
    }
    // Les relances de pédale restent distinctes des longues tenues.
    for i in [3, 7, 43, 47] {
        assert!(notes(&snapshots[i], 2).len() > 1, "relance de basse absente, mesure {}", i + 1);
    }
    Ok(())
}

#[test]
fn liaisons_sans_reattaque_ni_note_orpheline() -> TestResult {
    let p = load()?;
    let signature = p.meter.time_signature;
    let mut g = TradGenerator::new(params(&p), TuningParams { profile: p });
    let mut rng = ChaCha8Rng::seed_from_u64(1);
    let mut measures: Vec<_> = (1..=96).map(|b| g.generate_measure(b, &mut rng)).collect();
    measures.push(Measure::new(97, signature, 210.0, 24));
    let mut reader = Playhead::new(44100.0, 4);
    let mut bass_attacks = Vec::new();
    let mut chords_attacks = Vec::new();
    let mut balance = BTreeMap::<(u8, u8), i32>::new();
    for mut m in measures {
        let (bar, steps) = (m.index, m.steps);
        m.prepare_playback();
        reader.load_measure(Box::new(m));
        for step in 0..steps {
            reader.tick(None);
            for i in 0.. {
                let Some(ev) = reader.event(i) else { break };
                match &ev.event {
                    AudioEvent::NoteOn { channel, note, .. } if [0, 1, 2, 5].contains(channel) => {
                        *balance.entry((*channel, *note)).or_default() += 1;
                        if *channel == 2 && bar <= 8 {
                            bass_attacks.push((bar, step));
                        }
                        if *channel == 1 && bar <= 16 {
                            chords_attacks.push((bar, step, *note));
                        }
                    }
                    AudioEvent::NoteOff { channel, note } if [0, 1, 2, 5].contains(channel) => {
                        *balance.entry((*channel, *note)).or_default() -= 1;
                    }
                    _ => {}
                }
            }
        }
        let _ = reader.take_finished();
    }
    assert!(bass_attacks.contains(&(1, 0)), "attaque de la pédale absente");
    assert!(
        bass_attacks.iter().all(|(bar, _)| ![2, 3].contains(bar)),
        "pédale réattaquée au lieu d'être liée : {bass_attacks:?}"
    );
    let a1: Vec<_> = chords_attacks.iter().filter(|(bar, _, _)| *bar <= 8).collect();
    assert_eq!(a1, vec![&(1, 0, 56)], "pédale de nappe réattaquée");
    let a2: Vec<_> = chords_attacks.iter().filter(|(bar, _, _)| *bar >= 9).collect();
    assert_eq!(a2.len(), 4, "triade doublée réattaquée : {a2:?}");
    assert!(a2.iter().all(|(bar, step, _)| (*bar, *step) == (9, 0)));
    assert!(balance.values().all(|&v| v == 0), "notes orphelines : {balance:?}");
    Ok(())
}

#[test]
fn palette_du_kit_et_fills_uniquement_aux_frontieres_hors_pont() -> TestResult {
    let p = load()?;
    let mut toms = BTreeSet::new();
    let mut filled = BTreeSet::new();
    for seed in 1..=5 {
        for s in stream(&p, seed, 96)? {
            for n in &s.notes {
                assert!(n.track <= 5, "piste inconnue : {n:?}");
                if n.track == 3 {
                    assert_eq!(n.pitch, 36, "grosse caisse : {n:?}");
                } else if n.track == 4 {
                    assert!([38, 41, 42, 45, 48].contains(&n.pitch), "hors palette du kit : {n:?}");
                    if [41, 45, 48].contains(&n.pitch) {
                        assert!(
                            [16, 40, 48, 56, 64, 88, 96].contains(&s.index),
                            "fill hors frontière A1/A2/B, mesure {} : {n:?}",
                            s.index
                        );
                        assert!(
                            n.start_step >= 18,
                            "fill avant le dernier temps, mesure {}",
                            s.index
                        );
                        toms.insert(n.pitch);
                        filled.insert(s.index);
                    }
                }
            }
        }
    }
    assert_eq!(toms, BTreeSet::from([41, 45, 48]), "palette de toms non entendue");
    assert_eq!(
        filled,
        BTreeSet::from([16, 40, 48, 56, 64, 88, 96]),
        "frontières sans fill sur les cinq graines"
    );
    Ok(())
}

#[test]
fn quatre_emotions_observees_dans_les_snapshots() -> TestResult {
    let p = load()?;
    let calm = emotional_stream(&p, [0.0, 0.3, 0.5, 0.3])?;
    let intense = emotional_stream(&p, [1.0, 0.3, 0.5, 0.3])?;
    assert!(intense[8].tempo > calm[8].tempo, "arousal sans effet sur le tempo");
    let average_velocity = |snapshots: &[MeasureSnapshot]| {
        let velocities: Vec<_> = snapshots
            .iter()
            .flat_map(|s| &s.notes)
            .filter(|n| [3, 4].contains(&n.track))
            .map(|n| f32::from(n.velocity))
            .collect();
        velocities.iter().sum::<f32>() / velocities.len().max(1) as f32
    };
    assert!(
        average_velocity(&intense) > average_velocity(&calm),
        "arousal sans effet sur la dynamique des pieds"
    );

    // La densité enlève les couches, pas la nappe ; les deux voix arrivent
    // séparément au-dessus de leurs seuils, même à la reprise finale.
    let sparse = emotional_stream(&p, [0.5, 0.3, 0.2, 0.3])?;
    for s in &sparse {
        assert!(!s.notes.is_empty(), "nappe absente à faible densité, mesure {}", s.index);
        assert!(
            s.notes.iter().all(|n| n.track == 1),
            "autre couche à faible densité, mesure {}",
            s.index
        );
    }
    let below_lead = emotional_stream(&p, [0.5, 0.3, 0.39, 0.3])?;
    assert!(below_lead.iter().all(|s| notes(s, 0).is_empty()));
    let below_counter = emotional_stream(&p, [0.5, 0.3, 0.49, 0.3])?;
    assert!(
        below_counter.iter().any(|s| !notes(s, 0).is_empty()),
        "lead absent au-dessus du seuil"
    );
    assert!(below_counter.iter().all(|s| notes(s, 5).is_empty()), "contrevoix sous son seuil");

    let dark = emotional_stream(&p, [0.5, 0.0, 0.5, 0.3])?;
    let bright = emotional_stream(&p, [0.5, 1.0, 0.5, 0.3])?;
    assert!(
        dark.iter().zip(&bright).any(|(a, b)| {
            a.position.as_ref().map(|p| p.mode) != b.position.as_ref().map(|p| p.mode)
        }),
        "valence sans effet sur le mode publié"
    );

    let relaxed = emotional_stream(&p, [0.5, 0.3, 0.5, 0.0])?;
    let tense = emotional_stream(&p, [0.5, 0.3, 0.5, 1.0])?;
    let average_duration = |snapshots: &[MeasureSnapshot]| {
        let durations: Vec<_> = snapshots
            .iter()
            .flat_map(|s| notes(s, 0))
            .map(|n| n.performance.duration(n.duration_steps) as f64)
            .collect();
        durations.iter().sum::<f64>() / durations.len().max(1) as f64
    };
    assert!(
        average_duration(&tense) < average_duration(&relaxed),
        "tension sans raccourcissement mélodique"
    );
    Ok(())
}
