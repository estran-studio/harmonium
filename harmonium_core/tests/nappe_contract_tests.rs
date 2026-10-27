//! Contrats de la nappe G5 : accords plaqués tenus sur la piste Chords.
//!
//! Une nappe, c'est un coup `block` d'une mesure entière avec `tie = true` :
//! tenu sans réattaque quand l'accord ne change pas, rejoué quand il change,
//! sans note orpheline y compris après régénération. Le même mécanisme sert
//! la fixture valse (principe VIII : aucun nom de style dans le code).
use harmonium_core::{
    StyleProfile, TuningParams,
    events::AudioEvent,
    params::{MusicalParams, TimeSignature},
    report::MeasureSnapshot,
    timeline::{Articulation, Measure, Playhead, TrackId},
    trad::TradGenerator,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn load(id: &str) -> StyleProfile {
    StyleProfile::load(format!("{}/../tests/fixtures/profiles/{id}", env!("CARGO_MANIFEST_DIR")))
        .unwrap()
}

fn stream(profile: &StyleProfile, params: &MusicalParams, seed: u64, bars: usize) -> Vec<Measure> {
    profile.validate().unwrap();
    let mut g = TradGenerator::new(params.clone(), TuningParams { profile: profile.clone() });
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (1..=bars).map(|bar| g.generate_measure(bar, &mut rng)).collect()
}

/// Événements joués de la piste Chords : (mesure, pas, attaque ?, hauteur).
fn played(measures: &mut [Measure]) -> Vec<(usize, usize, bool, u8)> {
    let mut playhead = Playhead::new(44100.0, 4);
    let mut log = Vec::new();
    for m in measures.iter_mut() {
        m.prepare_playback();
        let (index, steps) = (m.index, m.steps);
        playhead.load_measure(Box::new(m.clone()));
        for step in 0..steps {
            playhead.tick(None);
            for i in 0.. {
                let Some(ev) = playhead.event(i) else { break };
                match &ev.event {
                    AudioEvent::NoteOn { channel: 1, note, .. } => {
                        log.push((index, step, true, *note));
                    }
                    AudioEvent::NoteOff { channel: 1, note } => {
                        log.push((index, step, false, *note));
                    }
                    _ => {}
                }
            }
        }
        let _ = playhead.take_finished();
    }
    log
}

fn chords_of(snapshot: &MeasureSnapshot) -> Vec<&harmonium_core::report::NoteSnapshot> {
    snapshot.notes.iter().filter(|n| n.track == TrackId::Chords as u8).collect()
}

#[test]
fn nappe_tenue_sans_reattaque_puis_rejouee_au_changement() {
    let profile = load("nappe_pad");
    let params = MusicalParams { bpm: profile.meter.default_bpm, ..Default::default() };
    let mut measures = stream(&profile, &params, 1, 64);
    let log = played(&mut measures);
    let snapshots: Vec<_> = measures.iter().map(MeasureSnapshot::from_measure).collect();

    // Chaque mesure sonnante porte un coup plaqué lié d'une mesure entière,
    // triade serrée en registre médium.
    let mut sounding_bars = 0;
    for s in &snapshots {
        let chords = chords_of(s);
        if chords.is_empty() {
            continue;
        }
        sounding_bars += 1;
        assert_eq!(chords.len(), 3, "mesure {} : triade serrée", s.index);
        for n in &chords {
            assert_eq!((n.start_step, n.duration_steps), (0, s.steps), "mesure {}", s.index);
            assert_eq!(n.articulation, Articulation::Tied, "mesure {}", s.index);
            assert!((55..=76).contains(&n.pitch), "mesure {} : registre médium", s.index);
        }
    }
    assert!(sounding_bars >= 8, "assez de mesures sonnantes pour lier, vu {sounding_bars}");

    // L'ensemble des hauteurs sonnantes après chaque barre vaut exactement
    // l'accord écrit : zéro attaque quand l'accord est inchangé, nouvelles
    // attaques quand il change.
    let mut sounding = std::collections::BTreeSet::new();
    let mut held = 0;
    let mut replayed = 0;
    for s in &snapshots {
        let expected: std::collections::BTreeSet<u8> =
            chords_of(s).iter().map(|n| n.pitch).collect();
        let mut attacks = 0;
        for (_, _, on, p) in log.iter().filter(|(b, _, _, _)| *b == s.index) {
            if *on {
                sounding.insert(*p);
                attacks += 1;
            } else {
                sounding.remove(p);
            }
        }
        assert_eq!(sounding, expected, "mesure {} : hauteurs sonnantes", s.index);
        if !expected.is_empty() {
            if attacks == 0 {
                held += 1;
            } else {
                replayed += 1;
            }
        }
    }
    assert!(held > 0, "au moins une tenue sans réattaque");
    assert!(replayed > 0, "au moins un changement rejoué");
}

#[test]
fn aucune_note_orpheline_y_compris_apres_regeneration() {
    let profile = load("nappe_pad");
    let base = MusicalParams { bpm: profile.meter.default_bpm, ..Default::default() };
    // Huit mesures au neutre, puis régénération « changement d'émotion »
    // (valence sombre : mode et accords changés) pompée sur le même playhead,
    // comme après un `invalidate_future` : la mesure courante garde ses fins
    // en vol, les mesures suivantes sont remplacées.
    let first = stream(&profile, &base, 1, 8);
    let changed = MusicalParams { harmony_valence: -1.0, ..base.clone() };
    let regenere = stream(&profile, &changed, 1, 18);
    assert!(
        regenere[8..18]
            .iter()
            .any(|m| m.chord_context.chord_name != first[7].chord_context.chord_name),
        "la régénération change vraiment l'harmonie"
    );
    let mut playhead = Playhead::new(44100.0, 4);
    let mut balance = std::collections::HashMap::new();
    let mut pump = |m: &Measure| {
        let mut m = m.clone();
        m.prepare_playback();
        let steps = m.steps;
        playhead.load_measure(Box::new(m));
        for _ in 0..steps {
            playhead.tick(None);
            for i in 0.. {
                let Some(ev) = playhead.event(i) else { break };
                match &ev.event {
                    AudioEvent::NoteOn { channel: 1, note, .. } => {
                        *balance.entry(*note).or_insert(0i32) += 1;
                    }
                    AudioEvent::NoteOff { channel: 1, note } => {
                        *balance.entry(*note).or_insert(0i32) -= 1;
                    }
                    _ => {}
                }
            }
        }
        let _ = playhead.take_finished();
    };
    for m in &first {
        pump(m);
    }
    for m in &regenere[8..18] {
        pump(m);
    }
    // Une mesure vide pour purger les fins en vol de la mesure 18.
    pump(&Measure::new(19, TimeSignature::new(4, 4), 90.0, 16));
    for (pitch, count) in &balance {
        assert_eq!(*count, 0, "hauteur {pitch} : NoteOn sans NoteOff");
    }
    assert!(!balance.is_empty(), "la nappe a joué pendant la régénération");
}

#[test]
fn valse_avec_nappe_emploie_le_meme_mecanisme() {
    let profile = load("valse_nappe");
    assert_eq!(profile.meta.id, "valse_nappe");
    let params = MusicalParams { bpm: profile.meter.default_bpm, ..Default::default() };
    let mut measures = stream(&profile, &params, 1, 96);
    let log = played(&mut measures);
    let snapshots: Vec<_> = measures.iter().map(MeasureSnapshot::from_measure).collect();
    // La valse garde sa grille 3/4 ; le motif par défaut est la nappe liée.
    assert!(
        snapshots.iter().any(|s| (s.time_sig_numerator, s.steps) == (3, 12)),
        "grille de valse conservée"
    );
    let mut tied_bars = 0;
    let mut held = 0;
    let mut previous: Option<(String, Vec<u8>)> = None;
    for s in &snapshots {
        let chords = chords_of(s);
        let tied = !chords.is_empty()
            && chords.iter().all(|n| n.articulation == Articulation::Tied)
            && chords.iter().all(|n| (n.start_step, n.duration_steps) == (0, s.steps));
        if !tied {
            previous = None;
            continue;
        }
        tied_bars += 1;
        let pitches: Vec<u8> = chords.iter().map(|n| n.pitch).collect();
        if previous.as_ref().is_some_and(|(c, p)| *c == s.chord_name && *p == pitches) {
            let attacks: usize =
                log.iter().filter(|(b, st, on, _)| *b == s.index && *st == 0 && *on).count();
            assert_eq!(attacks, 0, "mesure {} : réattaque", s.index);
            held += 1;
        }
        previous = Some((s.chord_name.clone(), pitches));
    }
    assert!(tied_bars >= 4, "la nappe sonne dans la valse, vu {tied_bars}");
    assert!(held > 0, "la valse lie sans réattaquer");
}

#[test]
fn liaison_opt_in_et_refusee_dans_les_regles_d_interpretation() {
    // Sans `tie`, les coups restent des attaques normales.
    let profile = load("nappe_pad");
    let text = profile.to_toml().unwrap().replace("tie = true", "");
    let plain = StyleProfile::from_toml(&text).unwrap();
    let params = MusicalParams { bpm: plain.meter.default_bpm, ..Default::default() };
    for m in stream(&plain, &params, 1, 32) {
        for n in m.notes_for_track(TrackId::Chords) {
            assert_eq!(n.articulation, Articulation::Normal);
        }
    }
    // La liaison se pose sur les coups, pas dans les règles d'interprétation.
    let ruled = profile.to_toml().unwrap().replace(
        "[performance.tracks]",
        "[performance.tracks]\n[[performance.tracks.Chords.rules]]\narticulation = \"Tied\"\nweight = 1.0",
    );
    let error = StyleProfile::from_toml(&ruled).unwrap_err().to_string();
    assert!(error.contains("liaison"), "{error}");
}

#[test]
fn aucun_profil_distribue_ne_lie() {
    // Garde-fou : les 10 profils distribués n'emploient pas `tie`, donc leurs
    // notes publiées sont inchangées.
    let distributed = StyleProfile::distributed();
    assert_eq!(distributed.len(), 10, "dix profils distribués");
    for p in distributed {
        if let Some(a) = &p.accompaniment {
            assert!(
                a.chords.patterns.iter().flat_map(|pat| &pat.hits).all(|h| !h.tie),
                "{} emploie tie",
                p.meta.id
            );
        }
        let params = MusicalParams { bpm: p.meter.default_bpm, ..Default::default() };
        for m in stream(p, &params, 1, 16) {
            for track in TrackId::ALL {
                for n in m.notes_for_track(track) {
                    assert_ne!(n.articulation, Articulation::Tied, "{}", p.meta.id);
                }
            }
        }
    }
}
