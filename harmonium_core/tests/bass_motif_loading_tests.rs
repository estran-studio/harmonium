//! Refus des banques ambiguës ou injouables avant installation du profil.
use harmonium_core::{
    StyleProfile,
    trad::accomp_profile::{
        BassDegree, BassHit, BassMotif, BassMotifNote, BassReference, BassVoice,
    },
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn profile() -> TestResult<StyleProfile> {
    let mut p = StyleProfile::default();
    let b = &mut p.accompaniment.as_mut().ok_or("accompagnement absent")?.bass;
    b.hits.clear();
    b.passing = None;
    b.motifs = vec![BassMotif {
        bars: 2,
        weight: 1,
        part: None,
        reference: BassReference::ChordRoot,
        notes: vec![
            BassMotifNote {
                step: 0,
                duration: 16,
                semitones: Some(0),
                degree: None,
                octave: 0,
                velocity: 80,
                tie: false,
                legato: false,
                doubling: vec![],
            },
            BassMotifNote {
                step: 16,
                duration: 16,
                semitones: Some(1),
                degree: None,
                octave: 0,
                velocity: 72,
                tie: false,
                legato: false,
                doubling: vec![],
            },
        ],
    }];
    Ok(p)
}

#[test]
fn banques_ambigues_ou_hors_grille_refusees_au_chargement() -> TestResult {
    let p = profile()?;
    p.validate()?;
    for (field, mutate) in [
        (
            "bass.motifs[0].notes[0]",
            (|b: &mut BassVoice| b.motifs[0].notes[0].degree = Some(2)) as fn(&mut BassVoice),
        ),
        ("bass.motifs[0].notes[0]", |b| b.motifs[0].notes[0].semitones = None),
        ("bass.motifs[0].notes[0].degree", |b| {
            let n = &mut b.motifs[0].notes[0];
            n.semitones = None;
            n.degree = Some(0);
        }),
        ("bass.motifs[0].notes[1].step", |b| b.motifs[0].notes[1].step = 15),
        ("bass.motifs[0].notes[1].duration", |b| b.motifs[0].notes[1].duration = usize::MAX / 2),
        ("bass.motifs[0].notes[0].velocity", |b| b.motifs[0].notes[0].velocity = 0),
        ("bass.motifs[0].bars", |b| b.motifs[0].bars = 0),
        ("bass.motifs[0].part", |b| b.motifs[0].part = Some("inconnue".into())),
        ("bass.motifs", |b| b.motifs[0].weight = 0),
        ("bass.motifs", |b| {
            b.motifs[0].weight = u32::MAX;
            b.motifs.push(b.motifs[0].clone());
        }),
        ("bass.motifs", |b| b.motifs[0].part = Some("A".into())),
        ("bass.hits", |b| {
            b.hits = vec![BassHit {
                step: 0,
                degree: BassDegree::Root,
                duration: 4,
                velocity: 80,
                doubling: vec![],
            }]
        }),
    ] {
        let mut bad = p.clone();
        mutate(&mut bad.accompaniment.as_mut().ok_or("accompagnement absent")?.bass);
        let error = StyleProfile::from_toml(&bad.to_toml()?)
            .err()
            .ok_or("profil invalide accepté")?
            .to_string();
        assert!(error.contains(&format!("accompaniment.{field}")), "{field}: {error}");
    }
    let mut bad = p;
    bad.harmony.model = "none".into();
    let error = StyleProfile::from_toml(&bad.to_toml()?)
        .err()
        .ok_or("motifs sans harmonie acceptés")?
        .to_string();
    assert!(error.contains("accompaniment.bass.motifs"), "{error}");
    Ok(())
}

#[test]
fn bornes_midi_verifiees_apres_intervalles_octaves_et_doublures() -> TestResult {
    use harmonium_core::trad::accomp_profile::BassDoubling;
    for (low, high, semitones, degree, octave, doubling) in [
        (0, 11, Some(-1), None, 0, vec![]),
        (116, 127, Some(1), None, 0, vec![]),
        (0, 11, None, Some(-1), 0, vec![]),
        (116, 127, None, Some(2), 0, vec![]),
        (0, 11, Some(0), None, -1, vec![]),
        (116, 127, Some(0), None, 1, vec![]),
        (104, 115, Some(1), None, 0, vec![BassDoubling::Octave]),
        (104, 115, Some(0), None, 0, vec![BassDoubling::Tenth]),
    ] {
        let mut p = profile()?;
        let b = &mut p.accompaniment.as_mut().ok_or("accompagnement absent")?.bass;
        b.low = low;
        b.high = high;
        b.walk_low = None;
        b.walk_high = None;
        let n = &mut b.motifs[0].notes[0];
        n.semitones = semitones;
        n.degree = degree;
        n.octave = octave;
        n.doubling = doubling;
        let error = StyleProfile::from_toml(&p.to_toml()?)
            .err()
            .ok_or("dépassement MIDI accepté")?
            .to_string();
        assert!(error.contains("accompaniment.bass.motifs[0].notes[0]"), "{error}");
    }
    Ok(())
}
