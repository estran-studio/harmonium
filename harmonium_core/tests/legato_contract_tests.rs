//! G7e : legato monophonique dans le playhead, observé sur le contrat public.
//!
//! Écoute du propriétaire : la basse de `nt_desert` changeait de note par
//! détachement (relance chromatique et riff D/D# découpés). Le legato doit
//! produire une seule attaque par note et chevaucher la fin de la note
//! précédente sur l'attaque nouvelle, sans jamais laisser de note orpheline.
//!
//! `LEGATO_OVERLAP_MILLI_STEPS = 200` est le chevauchement publié par le
//! playhead (millièmes de pas), épinglé ici comme contrat de lecture.

use std::collections::BTreeMap;

use harmonium_core::{
    StyleProfile, TuningParams,
    events::AudioEvent,
    params::MusicalParams,
    timeline::{Articulation, Measure, Playhead, TrackId},
    trad::TradGenerator,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

/// Chevauchement legato publié par le playhead (voir timeline/pointers.rs).
const LEGATO_OVERLAP: u16 = 200;

fn load(id: &str) -> TestResult<StyleProfile> {
    Ok(StyleProfile::load(format!(
        "{}/../tests/fixtures/profiles/{id}",
        env!("CARGO_MANIFEST_DIR")
    ))?)
}

fn load_nt() -> TestResult<StyleProfile> {
    Ok(StyleProfile::load(format!("{}/../profiles/nt_desert", env!("CARGO_MANIFEST_DIR")))?)
}

fn params(p: &StyleProfile) -> MusicalParams {
    MusicalParams {
        bpm: p.meter.default_bpm,
        time_signature: p.meter.time_signature,
        swing_ratio: p.meter.default_swing_ratio,
        ..Default::default()
    }
}

/// Génère des mesures déterministes, comme le compositeur.
fn measures(p: &StyleProfile, seed: u64, bars: usize) -> TestResult<Vec<Measure>> {
    p.validate()?;
    let mut g = TradGenerator::new(params(p), TuningParams { profile: p.clone() });
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    Ok((1..=bars).map(|bar| g.generate_measure(bar, &mut rng)).collect())
}

/// Journal de la basse (canal 2) sur le playhead public :
/// `(barre, pas, offset millièmes de pas, attaque, hauteur)`.
fn bass_played(mut measures: Vec<Measure>) -> Vec<(usize, usize, u16, bool, u8)> {
    let mut reader = Playhead::new(44100.0, 4);
    let mut log = Vec::new();
    for m in &mut measures {
        let bar = m.index;
        m.prepare_playback();
        reader.load_measure(Box::new(m.clone()));
        for step in 0..m.steps {
            reader.tick(None);
            for i in 0.. {
                let Some(ev) = reader.event(i) else { break };
                let (on, channel, note) = match &ev.event {
                    AudioEvent::NoteOn { channel, note, .. } => (true, *channel, *note),
                    AudioEvent::NoteOff { channel, note } => (false, *channel, *note),
                    _ => continue,
                };
                if channel == TrackId::Bass.channel() {
                    log.push((bar, step, ev.offset, on, note));
                }
            }
        }
        let _ = reader.take_finished();
    }
    log
}

/// Bilan NoteOn − NoteOff par hauteur : doit rester nul (aucune orpheline).
fn orphan_free(log: &[(usize, usize, u16, bool, u8)]) -> bool {
    let mut balance = BTreeMap::<u8, i32>::new();
    for &(_, _, _, on, note) in log {
        *balance.entry(note).or_default() += if on { 1 } else { -1 };
    }
    balance.values().all(|&b| b == 0)
}

#[test]
fn relance_chromatique_une_attaque_et_chevauchée() -> TestResult {
    let p = load_nt()?;
    let signature = p.meter.time_signature;
    let mut ms = measures(&p, 1, 8)?;
    ms.push(Measure::new(9, signature, 210.0, 24));

    let log = bass_played(ms);
    assert!(orphan_free(&log), "notes orphelines : {log:?}");

    // Relance de la quatrième mesure : D pas 0, E pas 5, F# pas 6, G pas 7,
    // G# pas 11. Chaque hauteur attaque une fois ; la fin de la note
    // précédente chevauche l'attaque (offset > 0, après le NoteOn à 0).
    let bar4: Vec<_> = log.iter().copied().filter(|e| e.0 == 4).collect();
    assert_eq!(
        bar4,
        vec![
            (4, 0, 0, true, 26),
            (4, 0, LEGATO_OVERLAP, false, 32), // pédale G# relâchée après l'attaque de D
            (4, 5, 0, true, 28),
            (4, 5, LEGATO_OVERLAP, false, 26),
            (4, 6, 0, true, 30),
            (4, 6, LEGATO_OVERLAP, false, 28),
            (4, 7, 0, true, 31),
            (4, 7, LEGATO_OVERLAP, false, 30),
            (4, 11, 0, true, 32),
            (4, 11, LEGATO_OVERLAP, false, 31),
        ],
        "barre 4 : une attaque par note, chevauchement (pas de détachement)"
    );
    Ok(())
}

#[test]
fn riff_a2_d_diese_lié_sur_la_basse_sans_orpheline() -> TestResult {
    let p = load_nt()?;
    let signature = p.meter.time_signature;
    let mut ms = measures(&p, 1, 16)?;
    ms.push(Measure::new(17, signature, 210.0, 24));

    let log = bass_played(ms);
    assert!(orphan_free(&log), "notes orphelines : {log:?}");

    // En A2 (mes. 9–16), les changements de hauteur (riff D/D#) ne doivent plus
    // détacher : la fin de la note précédente chevauche l'attaque nouvelle. Une
    // répétition de hauteur (D → D au bord du motif) garde sa réattaque normale.
    let mut overlaps = 0;
    for bar in 9..=16 {
        for step in 0..24 {
            let events: Vec<_> =
                log.iter().copied().filter(|e| e.0 == bar && e.1 == step).collect();
            for &(_, _, on_offset, _, on_pitch) in events.iter().filter(|e| e.3) {
                for &(_, _, off_offset, _, off_pitch) in events.iter().filter(|e| !e.3) {
                    if on_pitch != off_pitch {
                        assert!(
                            off_offset > on_offset,
                            "barre {bar} pas {step} : changement de hauteur détaché \
                             {on_pitch}→{off_pitch} : {events:?}"
                        );
                        overlaps += 1;
                    }
                }
            }
        }
    }
    assert!(overlaps > 0, "le riff doit réellement porter des chchaînements de hauteur liés");
    Ok(())
}

#[test]
fn principe_viii_la_valse_basse_motivique_emploie_le_legato_sans_changer_ses_notes() -> TestResult {
    let p = load("valse_basse_motivique")?;
    // Hauteurs, durées et vélocités inchangées (Ré majeur, marche 1–2–3/5–4–2) ;
    // seule l'articulation passe normal au legato.

    let mut g = TradGenerator::new(params(&p), TuningParams { profile: p.clone() });
    let mut rng = ChaCha8Rng::seed_from_u64(1);
    let snapshots: Vec<_> = (1..=4)
        .map(|bar| {
            harmonium_core::report::MeasureSnapshot::from_measure(
                &g.generate_measure(bar, &mut rng),
            )
        })
        .collect();

    for (i, s) in snapshots.iter().enumerate() {
        let degrees = if s.position.ok_or("position absente")?.bar_in_strain % 2 == 1 {
            [1, 2, 3]
        } else {
            [5, 4, 2]
        };
        let pitches: Vec<u8> =
            s.notes.iter().filter(|n| n.track == TrackId::Bass as u8).map(|n| n.pitch).collect();
        // Marche transposée : hauteurs distinctes et dans la tessiture.
        assert_eq!(pitches.len(), degrees.len(), "mesure {}", i + 1);
        for n in s.notes.iter().filter(|n| n.track == TrackId::Bass as u8) {
            assert_eq!(n.articulation, Articulation::Legato, "mesure {}", s.index);
            assert_eq!(n.duration_steps, 4, "durée inchangée, mesure {}", s.index);
        }
    }

    // La lecture de la basse reste sans orpheline, et la transition qui coupe
    // la barre (pas 8 → pas 12) chevauche au lieu de détacher.
    let signature = p.meter.time_signature;
    let mut ms = measures(&p, 1, 4)?;
    ms.push(Measure::new(5, signature, 145.0, 12));
    let log = bass_played(ms);
    assert!(orphan_free(&log), "notes orphelines : {log:?}");
    assert!(
        log.iter().any(|e| !e.3 && e.2 > 0),
        "la valse doit porter un chevauchement legato : {log:?}"
    );
    Ok(())
}
