//! Fixture technique E1 : la grille riche du modèle `loop`. Aucune validation stylistique à
//! l'écoute revendiquée ; le contrat est observé dans les snapshots publics.
//!
//! La fixture `grille_riche` écrit ses accords relativement à la tonique : les tests la
//! transposent par `[fixed_tonality]` (fa pour le blues, do et sol pour la cadence).
use std::collections::BTreeSet;

use harmonium_core::{
    StyleProfile, TuningParams,
    compose::TuneGenerator,
    params::MusicalParams,
    report::{MeasureSnapshot, NoteSnapshot},
    theory::chord::{Chord, ChordType},
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const SEEDS: [u64; 3] = [1, 2, 17];
const MELODY: u8 = 0;
const CHORDS: u8 = 1;
const BASS: u8 = 2;

fn load() -> TestResult<StyleProfile> {
    Ok(StyleProfile::load(format!(
        "{}/../tests/fixtures/profiles/grille_riche",
        env!("CARGO_MANIFEST_DIR")
    ))?)
}

fn edited(
    p: &StyleProfile,
    change: impl FnOnce(&mut toml::Value) -> TestResult,
) -> TestResult<StyleProfile> {
    let mut value: toml::Value = toml::from_str(&p.to_toml()?)?;
    change(&mut value)?;
    Ok(StyleProfile::from_toml(&toml::to_string(&value)?)?)
}

/// Même profil, tonalité verrouillée sur `key` : les accords relatifs se transposent.
fn in_key(p: &StyleProfile, key: u8) -> TestResult<StyleProfile> {
    edited(p, |v| {
        v["fixed_tonality"]["key"] = i64::from(key).into();
        v["melody"]["key_modes"][0]["key"] = i64::from(key).into();
        v["dynamics"]["color"][0]["key"] = i64::from(key).into();
        Ok(())
    })
}

fn stream(p: &StyleProfile, seed: u64, bars: usize) -> TestResult<Vec<MeasureSnapshot>> {
    p.validate()?;
    let params = MusicalParams {
        bpm: p.meter.default_bpm,
        time_signature: p.meter.time_signature,
        swing_ratio: p.meter.default_swing_ratio,
        harmony_valence: p.dynamics.neutral_valence,
        ..Default::default()
    };
    let mut g = TuneGenerator::new(params, TuningParams { profile: p.clone() });
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    Ok((1..=bars)
        .map(|bar| MeasureSnapshot::from_measure(&g.generate_measure(bar, &mut rng)))
        .collect())
}

fn track(s: &MeasureSnapshot, id: u8) -> Vec<&NoteSnapshot> {
    s.notes.iter().filter(|n| n.track == id).collect()
}

/// Accord attendu, relatif à la tonique.
#[derive(Clone, Copy)]
struct Want {
    root: u8,
    chord_type: ChordType,
    extensions: &'static [u8],
    bass: Option<u8>,
}

const fn want(root: u8, chord_type: ChordType, extensions: &'static [u8]) -> Want {
    Want { root, chord_type, extensions, bass: None }
}

impl Want {
    const fn over(mut self, bass: u8) -> Self {
        self.bass = Some(bass);
        self
    }

    fn root_pc(self, key: u8) -> u8 {
        (key + self.root) % 12
    }

    fn bass_pc(self, key: u8) -> u8 {
        self.bass.map_or(self.root_pc(key), |b| (key + b) % 12)
    }

    fn tones(self, key: u8) -> BTreeSet<u8> {
        let mut chord = Chord::new(self.root_pc(key), self.chord_type);
        for &e in self.extensions {
            chord = chord.with_extension(e);
        }
        chord.all_pitch_classes().into_iter().collect()
    }
}

const F7: Want = want(0, ChordType::Dominant7, &[]);
const BB7: Want = want(5, ChordType::Dominant7, &[]);
const GM7: Want = want(2, ChordType::Minor7, &[]);
const C7: Want = want(7, ChordType::Dominant7, &[]);

type Bar = &'static [(usize, Want)];

/// Les changements publiés d'une mesure sont exactement ceux attendus, aux bons pas.
fn check_changes(s: &MeasureSnapshot, key: u8, expected: Bar) -> TestResult {
    let got: Vec<(usize, Chord)> = s
        .chord_changes
        .iter()
        .map(|c| Ok((c.start_step, Chord::from_name(&c.context.chord_name)?)))
        .collect::<TestResult<_>>()?;
    assert_eq!(got.len(), expected.len(), "mesure {} : {got:?}", s.index);
    for ((step, chord), (want_step, want)) in got.iter().zip(expected) {
        let ctx = format!("mesure {} pas {step}", s.index);
        assert_eq!(step, want_step, "{ctx}");
        assert_eq!(chord.root, want.root_pc(key), "fondamentale, {ctx}");
        assert_eq!(chord.chord_type, want.chord_type, "type, {ctx}");
        assert_eq!(chord.extensions, want.extensions, "tensions, {ctx}");
        assert_eq!(chord.bass, want.bass.map(|b| (key + b) % 12), "basse, {ctx}");
    }
    let first = expected[0].1;
    assert_eq!(s.chord_root_offset, i32::from(first.root), "décalage de la mesure {}", s.index);
    Ok(())
}

/// Piste Chords : les hauteurs de chaque demi-mesure sont des notes de l'accord en vigueur,
/// la plus grave est sa basse pour un accord slash ; piste Bass : basse au premier pas, quinte de la fondamentale
/// au pas 4 ; mélodie : chaque note tombe dans l'accord du pas où elle commence.
fn check_notes(s: &MeasureSnapshot, key: u8, expected: Bar) -> TestResult {
    for start in [0usize, 8] {
        let active = expected
            .iter()
            .rev()
            .find(|(step, _)| *step <= start)
            .map(|(_, w)| *w)
            .ok_or("changement initial absent")?;
        let ctx = format!("mesure {} demi-mesure {start}", s.index);
        let mut voiced: Vec<u8> = track(s, CHORDS)
            .iter()
            .filter(|n| (start..start + 8).contains(&n.start_step))
            .map(|n| n.pitch)
            .collect();
        voiced.sort_unstable();
        let tones = active.tones(key);
        let bass_pc = active.bass_pc(key);
        let foreign = !tones.contains(&bass_pc);
        let voiced_pcs: BTreeSet<u8> = voiced.iter().map(|p| p % 12).collect();
        let mut allowed = tones.clone();
        allowed.insert(bass_pc);
        assert!((3..=5).contains(&voiced.len()), "{ctx} : {voiced:?}");
        assert!(voiced_pcs.is_subset(&allowed), "{ctx} : {voiced_pcs:?} hors de {allowed:?}");
        if active.bass.is_some() {
            assert_eq!(voiced[0] % 12, bass_pc, "{ctx} : accord slash, la plus grave est la basse");
        }
        // Tant que les tensions tiennent dans le voicing, aucune note de l'accord ne manque.
        if tones.len() <= if foreign { 4 } else { 5 } {
            assert_eq!(voiced_pcs, allowed, "{ctx} : accord complet");
        }
        let bass_at = |step: usize| {
            track(s, BASS).iter().find(|n| n.start_step == step).map(|n| n.pitch % 12)
        };
        assert_eq!(bass_at(start), Some(bass_pc), "{ctx} : basse");
        assert_eq!(
            bass_at(start + 4),
            Some((active.root_pc(key) + 7) % 12),
            "{ctx} : quinte de la fondamentale"
        );
    }
    for n in track(s, MELODY) {
        let active = expected
            .iter()
            .rev()
            .find(|(step, _)| *step <= n.start_step)
            .map(|(_, w)| *w)
            .ok_or("changement initial absent")?;
        assert!(
            active.tones(key).contains(&(n.pitch % 12)),
            "mesure {} pas {} : mélodie {} hors de l'accord {:?}",
            s.index,
            n.start_step,
            n.pitch,
            active.tones(key)
        );
    }
    Ok(())
}

const BLUES: [Bar; 12] = [
    &[(0, F7)],
    &[(0, F7)],
    &[(0, F7)],
    &[(0, F7)],
    &[(0, BB7)],
    &[(0, BB7)],
    &[(0, F7)],
    &[(0, F7)],
    &[(0, GM7), (8, C7)],
    &[(0, BB7)],
    &[(0, F7)],
    &[(0, F7), (8, C7)],
];

#[test]
fn blues_en_fa_en_accords_de_septieme_dont_un_ii_v_dans_une_mesure() -> TestResult {
    let p = load()?;
    // Fa = 5 ; do et la prouvent que la grille suit la tonique verrouillée.
    for key in [5u8, 0, 9] {
        let transposed = in_key(&p, key)?;
        for seed in SEEDS {
            let bars = stream(&transposed, seed, 12)?;
            for (bar, expected) in bars.iter().zip(BLUES) {
                check_changes(bar, key, expected)?;
                check_notes(bar, key, expected)?;
            }
            let name = match key {
                0 => "C7",
                5 => "F7",
                _ => "A7",
            };
            assert_eq!(bars[0].chord_name, name);
        }
    }
    Ok(())
}

fn cadence() -> [Bar; 4] {
    const CE: Want = want(0, ChordType::Major, &[]).over(4);
    const G7B9: Want = want(7, ChordType::Dominant7, &[1]);
    const DM7B5: Want = want(2, ChordType::HalfDiminished, &[]);
    const G7ALT: Want = want(7, ChordType::Altered, &[1, 3, 6, 8]);
    const CMAJ9: Want = want(0, ChordType::Major7, &[2]);
    const DC: Want = want(2, ChordType::Major, &[]).over(0);
    [&[(0, CE), (8, G7B9)], &[(0, DM7B5), (8, G7ALT)], &[(0, CMAJ9)], &[(0, DC)]]
}

#[test]
fn accords_slash_alteres_et_tensions_aux_bons_pas() -> TestResult {
    let p = load()?;
    for key in [0u8, 7] {
        let transposed = in_key(&p, key)?;
        for seed in SEEDS {
            let bars = stream(&transposed, seed, 16)?;
            for (bar, expected) in bars[12..].iter().zip(cadence()) {
                check_changes(bar, key, expected)?;
                check_notes(bar, key, expected)?;
            }
        }
    }
    Ok(())
}

#[test]
fn duree_omise_vaut_un_creneau_et_les_accords_egaux_ne_publient_pas_de_changement() -> TestResult {
    let p = load()?;
    // `maj9` sur 16 pas devient quatre `maj9` sans durée (un créneau de 4 pas chacun).
    let split = edited(&p, |v| {
        let chords = v["harmony"]["parts"]["Cadence"]["loops"][0]["chords"]
            .as_array_mut()
            .ok_or("liste d'accords requise")?;
        let mut slot = chords[4].clone();
        slot.as_table_mut().ok_or("table accord")?.remove("steps");
        chords[4] = slot.clone();
        for _ in 0..3 {
            chords.insert(5, slot.clone());
        }
        Ok(())
    })?;
    for seed in SEEDS {
        assert_eq!(
            serde_json::to_value(stream(&p, seed, 16)?)?,
            serde_json::to_value(stream(&split, seed, 16)?)?,
            "graine {seed}"
        );
    }
    Ok(())
}

#[test]
fn validation_refuse_durees_basses_et_qualites_invalides() -> TestResult {
    let p = load()?;
    let base: toml::Value = toml::from_str(&p.to_toml()?)?;
    let reject = |v: toml::Value| -> TestResult {
        let text = toml::to_string(&v)?;
        assert!(StyleProfile::from_toml(&text).is_err(), "profil invalide accepté : {text}");
        Ok(())
    };
    // Durées : zéro, hors grille harmonique (4 pas), somme différente de la partie.
    for steps in [0, 5, 62, 72] {
        reject(with_chord(&base, "steps", steps)?)?;
    }
    reject(with_chord(&base, "bass", 12)?)?;
    // Ancienne écriture en mesures : refusée sans alias.
    reject(with_chord(&base, "bars", 2)?)?;
    let mut v = base.clone();
    v["harmony"]
        .as_table_mut()
        .ok_or("table harmony requise")?
        .insert("default_bars_per_chord".into(), 1.into());
    reject(v)?;
    for quality in [
        "Major", "Minor", "Pedal", "maj7b9", "7b9b9", "9b9", "13b13", "m7b9", "69", "7alt9",
        "7b5b5", "7b5#5", "m7#11", "7#9#9", "sus9", "x", "7 b9", "7/9",
    ] {
        reject(with_chord(&base, "quality", quality)?)?;
    }
    // Voicing à fenêtre fixe : triades majeures et mineures nues seulement.
    let mut fixed = p.clone();
    let chords = &mut fixed.accompaniment.as_mut().ok_or("accompagnement absent")?.chords;
    chords.voicing = harmonium_core::compose::accomp_profile::ChordVoicing::FixedTriad;
    chords.low = 36;
    chords.high = 60;
    assert_eq!(
        fixed.validate().err().ok_or("fixed_triad accepté")?.field,
        "accompaniment.chords.voicing"
    );
    Ok(())
}

/// Le premier accord du blues, avec `field` remplacé.
fn with_chord(
    base: &toml::Value,
    field: &str,
    value: impl Into<toml::Value>,
) -> TestResult<toml::Value> {
    let mut v = base.clone();
    v["harmony"]["parts"]["Blues"]["loops"][0]["chords"][0]
        .as_table_mut()
        .ok_or("table accord requise")?
        .insert(field.into(), value.into());
    Ok(v)
}
