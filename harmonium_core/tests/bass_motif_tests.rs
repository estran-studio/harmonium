//! G3 : cellules dérivées de G0 et réemploi en valse, uniquement sur le contrat public.
use std::collections::BTreeSet;

use harmonium_core::{
    StyleProfile, TuningParams,
    events::AudioEvent,
    params::{MusicalParams, TimeSignature},
    report::MeasureSnapshot,
    timeline::{Articulation, Measure, Playhead, TrackId},
    trad::{Mode, TradGenerator},
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn load(id: &str) -> TestResult<StyleProfile> {
    Ok(StyleProfile::load(format!(
        "{}/../tests/fixtures/profiles/{id}",
        env!("CARGO_MANIFEST_DIR")
    ))?)
}

fn params(p: &StyleProfile) -> MusicalParams {
    MusicalParams { bpm: p.meter.default_bpm, ..Default::default() }
}

fn measures(p: &StyleProfile, seed: u64, bars: usize) -> TestResult<Vec<Measure>> {
    p.validate()?;
    let mut g = TradGenerator::new(params(p), TuningParams { profile: p.clone() });
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    Ok((1..=bars).map(|bar| g.generate_measure(bar, &mut rng)).collect())
}

fn snapshots(p: &StyleProfile, seed: u64, bars: usize) -> TestResult<Vec<MeasureSnapshot>> {
    Ok(measures(p, seed, bars)?.iter().map(MeasureSnapshot::from_measure).collect())
}

type Note = (usize, usize, u8, u8);
fn bass(s: &MeasureSnapshot) -> Vec<Note> {
    s.notes
        .iter()
        .filter(|n| n.track == TrackId::Bass as u8)
        .map(|n| (n.start_step, n.duration_steps, n.pitch, n.velocity))
        .collect()
}

/// Oracle indépendant : la fenêtre contient exactement une occurrence de chaque classe.
fn root(s: &MeasureSnapshot, low: u8) -> TestResult<u8> {
    let pc = (i32::from(s.position.ok_or("position absente")?.key) + s.chord_root_offset)
        .rem_euclid(12) as u8;
    Ok(low + (pc + 12 - low % 12) % 12)
}

fn exact_nt() -> TestResult<StyleProfile> {
    let mut p = load("nt_basse_motivique")?;
    // Garder le motif analytique ; les autres restent dans la banque mais sans poids.
    p.accompaniment.as_mut().ok_or("accompagnement absent")?.bass.motifs[2].weight = 0;
    Ok(p)
}

#[test]
fn pedale_sol_diese_et_relance_exactes_sur_la_grille_g0() -> TestResult {
    let p = exact_nt()?;
    assert_eq!(p.instruments.bass_program, Some(81), "Lead 2 saw, index zéro");
    assert!(!p.has_track(TrackId::Melody), "fixture réellement sans mélodie audible");
    let ss = snapshots(&p, 1, 8)?;
    for s in &ss {
        assert_eq!((s.time_sig_numerator, s.time_sig_denominator, s.steps), (12, 8, 24));
        assert_eq!(s.tempo, 210.0); // 140 pulsations pointées/min, pas 140 noires/min.
        assert_eq!(s.position.ok_or("position absente")?.mode, Mode::Phrygian);
        assert!(s.notes.iter().all(|n| n.track != TrackId::Melody as u8));
    }
    for s in &ss[..3] {
        assert_eq!(bass(s), vec![(0, 24, 32, 95)]);
        assert_eq!(
            s.notes
                .iter()
                .find(|n| n.track == TrackId::Bass as u8)
                .ok_or("basse absente")?
                .articulation,
            Articulation::Tied
        );
    }
    // G0 §6.1 : D–E–F#–G–G#, positions 0,5,6,7,11 de la quatrième mesure.
    assert_eq!(
        bass(&ss[3]),
        vec![(0, 5, 26, 85), (5, 1, 28, 85), (6, 1, 30, 90), (7, 4, 31, 95), (11, 13, 32, 100),]
    );
    for i in 0..4 {
        assert_eq!(bass(&ss[i]), bass(&ss[i + 4]), "reprise pédale, mesure {}", i + 1);
    }
    Ok(())
}

#[test]
fn riff_deux_mesures_exact_et_transpose_avec_les_accords() -> TestResult {
    let p = exact_nt()?;
    let ss = snapshots(&p, 3, 24)?;
    // Durées regroupées depuis G0 §6.1 ; vélocités de la cellule dérivée.
    let first = [
        (0, 4, 0, 95),
        (4, 1, 1, 85),
        (5, 3, 0, 90),
        (8, 3, 1, 85),
        (11, 6, 0, 95),
        (17, 1, 1, 85),
        (18, 6, 0, 95),
    ];
    let second = [
        (0, 5, 0, 95),
        (5, 2, 1, 85),
        (7, 1, 3, 85),
        (8, 3, 5, 90),
        (11, 2, 6, 95),
        (13, 2, 0, 90),
        (15, 2, 1, 85),
        (17, 2, 0, 90),
        (19, 1, 2, 85),
        (20, 4, 0, 95),
    ];
    let mut roots = BTreeSet::new();
    for s in &ss[8..] {
        let r = root(s, 26)?;
        roots.insert(r);
        let expected = if s.position.ok_or("position absente")?.bar_in_strain % 2 == 1 {
            &first[..]
        } else {
            &second[..]
        };
        let expected: Vec<_> =
            expected.iter().map(|&(st, dur, offset, vel)| (st, dur, r + offset, vel)).collect();
        assert_eq!(bass(s), expected, "mesure {}, accord {}", s.index, s.chord_name);
    }
    assert!(roots.len() > 1, "la transposition doit réellement rencontrer plusieurs fondamentales");
    Ok(())
}

#[test]
fn banque_ponderee_variantes_reelles_et_poids_nul_jamais_choisi() -> TestResult {
    let p = load("nt_basse_motivique")?;
    let mut choices = BTreeSet::new();
    for seed in 1..=64 {
        let ss = snapshots(&p, seed, 24)?;
        for s in &ss[8..] {
            let notes = bass(s);
            assert!(notes.iter().all(|n| n.3 != 1), "sentinelle de poids nul sélectionnée");
            if s.position.ok_or("position absente")?.bar_in_strain % 2 == 1 {
                choices.insert(notes[0].3);
            }
        }
    }
    assert_eq!(choices, BTreeSet::from([76, 95]), "les deux variantes positives doivent sonner");
    // Forcer chacun des poids à zéro doit laisser exclusivement l'autre motif.
    for (disabled, velocity) in [(1, 76), (2, 95)] {
        let mut forced = p.clone();
        forced.accompaniment.as_mut().ok_or("accompagnement absent")?.bass.motifs[disabled]
            .weight = 0;
        for s in &snapshots(&forced, 2, 24)?[8..] {
            assert_eq!(bass(s)[0].3, velocity);
        }
    }
    Ok(())
}

/// Journal public de la piste Bass, y compris les fins en vol d'une mesure à l'autre.
fn played(mut ms: Vec<Measure>) -> Vec<(usize, usize, bool, u8)> {
    let mut playhead = Playhead::new(44100.0, 4);
    let mut log = Vec::new();
    for m in &mut ms {
        m.prepare_playback();
        playhead.load_measure(Box::new(m.clone()));
        for step in 0..m.steps {
            playhead.tick(None);
            for i in 0.. {
                let Some(event) = playhead.event(i) else { break };
                match &event.event {
                    AudioEvent::NoteOn { channel: 2, note, .. } => {
                        log.push((m.index, step, true, *note))
                    }
                    AudioEvent::NoteOff { channel: 2, note } => {
                        log.push((m.index, step, false, *note))
                    }
                    _ => {}
                }
            }
        }
        let _ = playhead.take_finished();
    }
    log
}

#[test]
fn pedale_une_seule_attaque_pendant_trois_mesures_sans_note_orpheline() -> TestResult {
    let p = exact_nt()?;
    let mut ms = measures(&p, 1, 4)?;
    ms.push(Measure::new(5, TimeSignature::new(12, 8), 210.0, 24));
    let log = played(ms);
    let attacks: Vec<_> = log.iter().copied().filter(|e| e.2).collect();
    assert_eq!(
        attacks,
        vec![
            (1, 0, true, 32),
            (4, 0, true, 26),
            (4, 5, true, 28),
            (4, 6, true, 30),
            (4, 7, true, 31),
            (4, 11, true, 32)
        ]
    );
    assert!(log.iter().all(|e| e.0 != 2 && e.0 != 3), "aucune réattaque ni coupure à la barre");
    let mut balance = std::collections::BTreeMap::new();
    for (_, _, on, pitch) in log {
        *balance.entry(pitch).or_insert(0i32) += if on { 1 } else { -1 };
    }
    assert!(balance.values().all(|&v| v == 0), "aucune note orpheline : {balance:?}");
    Ok(())
}

#[test]
fn valse_marche_modale_et_copies_interparties_sur_le_meme_mecanisme() -> TestResult {
    let p = load("valse_basse_motivique")?;
    let ss = snapshots(&p, 1, 32)?;
    for s in &ss {
        assert_eq!((s.time_sig_numerator, s.time_sig_denominator, s.steps), (3, 4, 12));
        // Oracle de marche dans D majeur, réancré sur l'accord publié à destination :
        // les copies locales/interparties copient les degrés, pas les hauteurs absolues.
        let r = root(s, 38)?;
        let scale: Vec<_> =
            (r..=r + 24).filter(|n| [2, 4, 6, 7, 9, 11, 1].contains(&(n % 12))).collect();
        let degrees = if s.position.ok_or("position absente")?.bar_in_strain % 2 == 1 {
            [1, 2, 3]
        } else {
            [5, 4, 2]
        };
        let expected: Vec<_> = degrees
            .iter()
            .zip([78, 70, 72])
            .enumerate()
            .map(|(i, (&degree, velocity))| (i * 4, 4, scale[degree - 1], velocity))
            .collect();
        assert_eq!(bass(s), expected, "marche transposée, mesure {}", s.index);
    }
    Ok(())
}

#[test]
fn copie_interrompant_un_motif_ne_decale_pas_sa_seconde_tranche() -> TestResult {
    let mut p = load("valse_basse_motivique")?;
    // B : début, copie de A2, début, suite. La copie n'impose qu'une mesure.
    let free = p.form.parts[0].roles[1].clone();
    p.form.parts[1].roles[1] = p.form.parts[1].roles[3].clone();
    p.form.parts[1].roles[2] = free.clone();
    p.form.parts[1].roles[3] = free;
    let ss = snapshots(&p, 2, 12)?;
    assert_eq!(bass(&ss[1]), bass(&ss[9]), "B2 copie exactement A2");
    // La première note d'une première tranche est la fondamentale, celle d'une suite la quinte.
    assert_eq!(bass(&ss[10])[0].2, root(&ss[10], 38)?);
    assert_eq!(bass(&ss[11])[0].2 % 12, (root(&ss[11], 38)? + 7) % 12);
    Ok(())
}

#[test]
fn queue_de_partie_impaire_tronque_la_cellule_sans_silence() -> TestResult {
    let mut p = load("valse_basse_motivique")?;
    let free = p.form.parts[0].roles[1].clone();
    p.form.parts[0].roles.truncate(3);
    p.form.parts[0].roles[2] = free;
    // Éviter les références vers A4 après raccourcissement ; B utilise son propre ostinato.
    p.form.sequence = vec![0];
    p.form.parts.truncate(1);
    let ss = snapshots(&p, 4, 6)?;
    for s in &ss {
        assert_eq!(bass(s).len(), 3, "la queue reste sonnante");
    }
    for i in [0, 2, 3, 5] {
        assert_eq!(bass(&ss[i])[0].2, root(&ss[i], 38)?, "première tranche, mesure {}", i + 1);
    }
    Ok(())
}

#[test]
fn liaisons_explicites_hors_temps_fort_reste_observable() -> TestResult {
    let mut p = load("valse_basse_motivique")?;
    p.accompaniment.as_mut().ok_or("accompagnement absent")?.bass.motifs[0].notes[1].tie = true;
    let ss = snapshots(&p, 1, 1)?;
    let note = ss[0]
        .notes
        .iter()
        .find(|n| n.track == TrackId::Bass as u8 && n.start_step == 4)
        .ok_or("note attendue absente")?;
    assert_eq!(note.articulation, Articulation::Tied);
    assert_eq!((note.pitch, note.duration_steps, note.velocity), (40, 4, 70));
    Ok(())
}

#[test]
fn densite_silence_et_dynamique_s_appliquent_a_la_basse_motivique() -> TestResult {
    use harmonium_core::trad::dynamics::TrackThreshold;
    let p = exact_nt()?;
    let mut soft = p.clone();
    soft.dynamics.bass_velocity.0 = vec![[0.0, 0.5], [1.0, 0.5]];
    for (a, b) in snapshots(&p, 1, 24)?.iter().zip(snapshots(&soft, 1, 24)?) {
        for (a, b) in bass(a).iter().zip(bass(&b)) {
            assert_eq!((a.0, a.1, a.2), (b.0, b.1, b.2));
            assert_eq!(b.3, (f32::from(a.3) * 0.5).round() as u8);
        }
    }
    let mut gated = p.clone();
    gated.dynamics.density_tracks.push(TrackThreshold { track: TrackId::Bass, minimum: 0.8 });
    let mut g = TradGenerator::new(
        MusicalParams { rhythm_density: 0.0, ..params(&gated) },
        TuningParams { profile: gated },
    );
    let mut rng = ChaCha8Rng::seed_from_u64(1);
    for bar in 1..=8 {
        assert!(
            bass(&MeasureSnapshot::from_measure(&g.generate_measure(bar, &mut rng))).is_empty()
        );
    }
    let mut off = TradGenerator::new(
        MusicalParams { enable_harmony: false, ..params(&p) },
        TuningParams { profile: p },
    );
    for bar in 1..=4 {
        assert!(
            bass(&MeasureSnapshot::from_measure(&off.generate_measure(bar, &mut rng))).is_empty()
        );
    }
    Ok(())
}

#[test]
fn passing_preserve_les_cibles_et_chromatismes_ecrits() -> TestResult {
    let mut p = exact_nt()?;
    p.arrangement.entries.melody = p.arrangement.entries.bass;
    let mut passing = p.clone();
    let mut config = StyleProfile::default()
        .accompaniment
        .ok_or("accompagnement absent")?
        .bass
        .passing
        .ok_or("passing absent")?;
    config.none_weight = 0;
    config.density.0 = vec![[0.0, 1.0], [1.0, 1.0]];
    config.phrase.0 = vec![[0.0, 1.0], [1.0, 1.0]];
    passing.accompaniment.as_mut().ok_or("accompagnement absent")?.bass.passing = Some(config);
    let mut additions = 0;
    for (a, b) in snapshots(&p, 3, 24)?.iter().zip(snapshots(&passing, 3, 24)?) {
        let written = bass(a);
        let realized = bass(&b);
        for &(step, duration, pitch, velocity) in &written {
            let note = realized.iter().find(|n| n.0 == step).ok_or("note attendue absente")?;
            assert_eq!((note.2, note.3), (pitch, velocity), "cible écrite, mesure {}", a.index);
            assert!(note.1 > 0 && note.1 <= duration);
        }
        additions += realized.iter().filter(|n| !written.iter().any(|w| w.0 == n.0)).count();
    }
    assert!(additions > 0, "les approches doivent réellement être exercées");
    Ok(())
}

#[test]
fn invalidation_et_rejeu_conservent_les_snapshots_et_les_choix_ponderes() -> TestResult {
    for id in ["nt_basse_motivique", "valse_basse_motivique"] {
        let p = load(id)?;
        let mut g = TradGenerator::new(params(&p), TuningParams { profile: p.clone() });
        let mut rng = ChaCha8Rng::seed_from_u64(7);
        let before: Vec<_> = (1..=48)
            .map(|bar| MeasureSnapshot::from_measure(&g.generate_measure(bar, &mut rng)))
            .collect();
        // Au milieu de la pédale, puis au milieu d'un motif de deux mesures.
        for boundary in [2, 9] {
            g.invalidate_future(boundary);
            for original in &before[boundary..] {
                let replay =
                    MeasureSnapshot::from_measure(&g.generate_measure(original.index, &mut rng));
                assert_eq!(
                    serde_json::to_value(original)?,
                    serde_json::to_value(replay)?,
                    "{id}, invalidation après {boundary}, mesure {}",
                    original.index
                );
            }
        }
        assert_eq!(serde_json::to_value(&before)?, serde_json::to_value(snapshots(&p, 7, 48)?)?);
    }
    Ok(())
}
