//! E3 : l'identité complète d'un accord (qualité typée, extensions, fondamentale et basse
//! absolues, notes) dans le flux de `MeasureSnapshot`, pour tous les modèles d'harmonie.
//!
//! Les attendus de la grille riche sont écrits en clair, relatifs à la tonique (intervalles
//! au-dessus de la fondamentale) ; aucun n'est calculé par l'API de théorie testée.
use harmonium_core::{
    StyleProfile, TuningParams, compose::TuneGenerator, params::MusicalParams,
    report::MeasureSnapshot, theory::chord::Chord, timeline::ChordContext,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const SEEDS: [u64; 3] = [1, 2, 17];

fn fixture(name: &str) -> TestResult<StyleProfile> {
    Ok(StyleProfile::load(format!(
        "{}/../tests/fixtures/profiles/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))?)
}

fn shipped(name: &str) -> TestResult<StyleProfile> {
    Ok(StyleProfile::load(format!("{}/../profiles/{name}", env!("CARGO_MANIFEST_DIR")))?)
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

/// Identité attendue, relative à la tonique.
#[derive(Clone, Copy)]
struct Id {
    /// Demi-tons de la fondamentale au-dessus de la tonique.
    root: u8,
    quality: &'static str,
    extensions: &'static [u8],
    /// Demi-tons de la basse au-dessus de la tonique (fondamentale si absente).
    bass: Option<u8>,
    /// Notes, en demi-tons au-dessus de la fondamentale : ordre publié.
    intervals: &'static [u8],
}

const F7: Id = Id { root: 0, quality: "7", extensions: &[], bass: None, intervals: &[0, 4, 7, 10] };
const BB7: Id = Id { root: 5, ..F7 };
const C7: Id = Id { root: 7, ..F7 };
const GM7: Id =
    Id { root: 2, quality: "m7", extensions: &[], bass: None, intervals: &[0, 3, 7, 10] };
const C_OVER_E: Id =
    Id { root: 0, quality: "", extensions: &[], bass: Some(4), intervals: &[0, 4, 7] };
const G7B9: Id =
    Id { root: 7, quality: "7b9", extensions: &[1], bass: None, intervals: &[0, 1, 4, 7, 10] };
const DM7B5: Id =
    Id { root: 2, quality: "m7b5", extensions: &[], bass: None, intervals: &[0, 3, 6, 10] };
const G7ALT: Id = Id {
    root: 7,
    quality: "7alt",
    extensions: &[1, 3, 6, 8],
    bass: None,
    intervals: &[0, 1, 3, 4, 6, 8, 10],
};
const CMAJ9: Id =
    Id { root: 0, quality: "maj9", extensions: &[2], bass: None, intervals: &[0, 2, 4, 7, 11] };
// Basse hors de l'accord : do n'est pas une note de ré majeur.
const D_OVER_C: Id =
    Id { root: 2, quality: "", extensions: &[], bass: Some(0), intervals: &[0, 4, 7] };

type Bar = &'static [(usize, Id)];

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

const CADENCE: [Bar; 4] =
    [&[(0, C_OVER_E), (8, G7B9)], &[(0, DM7B5), (8, G7ALT)], &[(0, CMAJ9)], &[(0, D_OVER_C)]];

/// `context` publie exactement `want` dans la tonalité `key`.
fn check_context(context: &ChordContext, want: Id, key: u8, at: &str) {
    let root = (key + want.root) % 12;
    assert_eq!(context.root_pc, root, "fondamentale, {at}");
    assert_eq!(context.bass_pc, want.bass.map_or(root, |b| (key + b) % 12), "basse, {at}");
    assert_eq!(context.quality.map(|q| q.to_string()).as_deref(), Some(want.quality), "{at}");
    assert_eq!(context.extensions, want.extensions, "extensions, {at}");
    let notes: Vec<u8> = want.intervals.iter().map(|i| (root + i) % 12).collect();
    assert_eq!(context.pitch_classes, notes, "notes, {at}");
    assert_eq!(context.root_offset, i32::from(want.root), "décalage, {at}");
}

fn check_bar(s: &MeasureSnapshot, key: u8, expected: Bar) {
    let at = |step: usize| format!("tonique {key}, mesure {}, pas {step}", s.index);
    assert_eq!(s.chord_changes.len(), expected.len(), "{}", at(0));
    for (change, (step, want)) in s.chord_changes.iter().zip(expected) {
        assert_eq!(change.start_step, *step, "{}", at(*step));
        check_context(&change.context, *want, key, &at(*step));
    }
    // Les champs de mesure décrivent le premier accord, avec les mêmes valeurs.
    let (_, first) = expected[0];
    let root = (key + first.root) % 12;
    let at = at(0);
    assert_eq!(s.chord_root_pc, root, "{at}");
    assert_eq!(s.chord_bass_pc, first.bass.map_or(root, |b| (key + b) % 12), "{at}");
    assert_eq!(s.chord_quality.map(|q| q.to_string()).as_deref(), Some(first.quality), "{at}");
    assert_eq!(s.chord_extensions, first.extensions, "{at}");
    assert_eq!(s.chord_pitch_classes, s.chord_changes[0].context.pitch_classes, "{at}");
}

#[test]
fn le_blues_publie_qualite_basse_fondamentale_et_notes_de_chaque_accord() -> TestResult {
    let p = fixture("grille_riche")?;
    // Fa = 5 ; do et la prouvent que l'identité suit la tonique verrouillée.
    for key in [5u8, 0, 9] {
        let transposed = in_key(&p, key)?;
        for seed in SEEDS {
            for (bar, expected) in stream(&transposed, seed, 12)?.iter().zip(BLUES) {
                check_bar(bar, key, expected);
            }
        }
    }
    Ok(())
}

#[test]
fn la_cadence_publie_slash_alterations_et_tensions_aux_bons_pas() -> TestResult {
    let p = fixture("grille_riche")?;
    for key in [0u8, 7] {
        let transposed = in_key(&p, key)?;
        for seed in SEEDS {
            for (bar, expected) in stream(&transposed, seed, 16)?[12..].iter().zip(CADENCE) {
                check_bar(bar, key, expected);
            }
        }
    }
    Ok(())
}

#[test]
fn la_json_publie_la_qualite_en_texte_et_se_relit() -> TestResult {
    let p = in_key(&fixture("grille_riche")?, 0)?;
    let bars = stream(&p, 1, 16)?;
    let json = serde_json::to_value(&bars[12])?;
    assert_eq!(json["chord_quality"], "");
    assert_eq!(json["chord_bass_pc"], 4);
    assert_eq!(json["chord_pitch_classes"], serde_json::json!([0, 4, 7]));
    let second = &json["chord_changes"][1];
    assert_eq!(second["start_step"], 8);
    assert_eq!(second["context"]["quality"], "7b9");
    assert_eq!(second["context"]["extensions"], serde_json::json!([1]));
    assert_eq!(second["context"]["root_pc"], 7);
    let back: MeasureSnapshot = serde_json::from_value(json)?;
    assert_eq!(back.chord_changes[1].context.quality, bars[12].chord_changes[1].context.quality);
    // Un snapshot sans les champs d'identité se relit (défauts).
    let mut old = serde_json::to_value(&bars[12])?;
    let fields = old.as_object_mut().ok_or("objet attendu")?;
    for key in ["chord_root_pc", "chord_bass_pc", "chord_quality", "chord_extensions"] {
        fields.remove(key);
    }
    fields.remove("chord_pitch_classes");
    let back: MeasureSnapshot = serde_json::from_value(old)?;
    assert_eq!(back.chord_quality, None);
    assert!(back.chord_pitch_classes.is_empty());
    Ok(())
}

#[test]
fn les_gammes_de_loop_suivent_le_drapeau_sans_toucher_aux_notes() -> TestResult {
    let p = in_key(&fixture("grille_riche")?, 0)?;
    let guided = edited(&p, |v| {
        v["harmony"]["scale_guidance"] = true.into();
        Ok(())
    })?;
    for seed in SEEDS {
        let plain = stream(&p, seed, 16)?;
        let with = stream(&guided, seed, 16)?;
        for (a, b) in plain.iter().zip(&with) {
            assert!(a.scale_guidance.is_none(), "mesure {} : gammes sans drapeau", a.index);
            assert!(a.chord_changes.iter().all(|c| c.context.scale_guidance.is_none()));
            assert_eq!(
                serde_json::to_value(&a.notes)?,
                serde_json::to_value(&b.notes)?,
                "mesure {} : les gammes ne changent pas les notes",
                a.index
            );
            assert_eq!(a.chord_name, b.chord_name);
            for (x, y) in a.chord_changes.iter().zip(&b.chord_changes) {
                assert_eq!(x.start_step, y.start_step);
                let guidance = y.context.scale_guidance.as_ref().ok_or("gammes absentes")?;
                let at = format!("mesure {} pas {}", b.index, y.start_step);
                assert_eq!(guidance.chord_symbol, y.context.chord_name, "{at}");
                assert_eq!(guidance.chord_root, y.context.root_pc, "{at}");
                assert!((1..=12).contains(&guidance.lcc_level), "{at}");
                assert!(!guidance.scale_notes.is_empty(), "{at}");
                // Notes de l'accord de la gamme : le type seul, sans les tensions.
                assert!(guidance.chord_tones.iter().all(|t| y.context.pitch_classes.contains(t)));
                assert!(guidance.chord_tones.contains(&y.context.root_pc), "{at}");
            }
            assert_eq!(
                b.scale_guidance.as_ref(),
                b.chord_changes[0].context.scale_guidance.as_ref(),
                "mesure {} : gammes de la mesure = premier accord",
                b.index
            );
        }
    }
    // Gammes exactes pour G7b9 à tension neutre : fondamentale, tierce, quinte, septième.
    let with = stream(&guided, 1, 16)?;
    let g7b9 = with[12].chord_changes[1].context.scale_guidance.as_ref().ok_or("gammes")?;
    let mut tones = g7b9.chord_tones.clone();
    tones.sort_unstable();
    assert_eq!(tones, [2, 5, 7, 11]);
    Ok(())
}

/// Le texte du nom, relu par le parseur de symboles, redonne l'identité publiée.
fn check_coherent_with_name(context: &ChordContext, at: &str) -> TestResult {
    let named = Chord::from_name(&context.chord_name)?;
    assert_eq!(named.root, context.root_pc, "fondamentale, {at} ({})", context.chord_name);
    assert_eq!(named.bass.unwrap_or(named.root), context.bass_pc, "basse, {at}");
    assert_eq!(
        named.quality().map(|q| q.to_string()),
        context.quality.map(|q| q.to_string()),
        "qualité, {at} ({})",
        context.chord_name
    );
    let mut notes = named.all_pitch_classes();
    let mut published = context.pitch_classes.clone();
    notes.sort_unstable();
    published.sort_unstable();
    assert_eq!(published, notes, "notes, {at}");
    let mut extensions = named.extensions.clone();
    extensions.sort_unstable();
    assert_eq!(context.extensions, extensions, "extensions, {at}");
    Ok(())
}

fn check_stream_names(bars: &[MeasureSnapshot], label: &str) -> TestResult<usize> {
    let mut checked = 0;
    for s in bars {
        let flat = ChordContext {
            chord_name: s.chord_name.clone(),
            root_pc: s.chord_root_pc,
            bass_pc: s.chord_bass_pc,
            quality: s.chord_quality,
            extensions: s.chord_extensions.clone(),
            pitch_classes: s.chord_pitch_classes.clone(),
            ..ChordContext::default()
        };
        if s.chord_quality.is_some() {
            check_coherent_with_name(&flat, &format!("{label} mesure {}", s.index))?;
            checked += 1;
        }
        for change in &s.chord_changes {
            check_coherent_with_name(
                &change.context,
                &format!("{label} mesure {} pas {}", s.index, change.start_step),
            )?;
            checked += 1;
        }
    }
    Ok(checked)
}

#[test]
fn melody_functional_publie_une_identite_coherente_avec_chord_name() -> TestResult {
    for name in ["reel_quebecois", "valse_quebecoise", "gigue_acadienne"] {
        let p = shipped(name)?;
        for seed in SEEDS {
            let bars = stream(&p, seed, 64)?;
            let checked = check_stream_names(&bars, name)?;
            assert!(checked >= 32, "{name} : {checked} accords vérifiés");
            // Les accords sont des triades majeures ou mineures : qualité "" ou "m".
            for s in bars.iter().filter(|s| s.chord_quality.is_some()) {
                let quality = s.chord_quality.map(|q| q.to_string());
                assert!(matches!(quality.as_deref(), Some("" | "m")), "{name} : {quality:?}");
                assert_eq!(s.chord_pitch_classes.len(), 3, "{name} mesure {}", s.index);
                assert_eq!(s.chord_pitch_classes[0], s.chord_root_pc);
                assert_eq!(s.chord_bass_pc, s.chord_root_pc);
                assert_eq!(quality.as_deref() == Some("m"), s.chord_is_minor);
            }
        }
    }
    Ok(())
}

#[test]
fn rewrite_functional_publie_une_identite_coherente_avec_chord_name() -> TestResult {
    let p = fixture("jazz_test")?;
    for seed in SEEDS {
        let bars = stream(&p, seed, 32)?;
        let checked = check_stream_names(&bars, "jazz_test")?;
        assert!(checked >= 32, "{checked} accords vérifiés");
        for s in &bars {
            for change in &s.chord_changes {
                let guidance = change.context.scale_guidance.as_ref().ok_or("gammes absentes")?;
                assert_eq!(guidance.chord_root, change.context.root_pc);
                assert_eq!(guidance.chord_symbol, change.context.chord_name);
            }
        }
    }
    Ok(())
}

#[test]
fn pedale_et_harmonie_nulle_n_inventent_pas_de_qualite() -> TestResult {
    // Pédale : fondamentale et hauteur tenue, aucune qualité ; les parties en accords de la
    // même fixture gardent leur qualité typée.
    let p = fixture("nt_pedale_boucle")?;
    let (mut pedal, mut typed) = (0, 0);
    for s in stream(&p, 1, 64)? {
        for change in &s.chord_changes {
            let c = &change.context;
            if c.quality.is_some() {
                typed += 1;
                continue;
            }
            pedal += 1;
            assert!(c.extensions.is_empty());
            assert_eq!(c.pitch_classes, [c.root_pc], "mesure {}", s.index);
            assert_eq!(c.bass_pc, c.root_pc);
        }
    }
    assert!(pedal > 0 && typed > 0, "pédale {pedal}, accords typés {typed}");
    // Harmonie `none` : la tonique de l'air, aucun accord joué.
    let p = shipped("marche_britannique_coloniale")?;
    let bars = stream(&p, 1, 16)?;
    for s in &bars {
        assert_eq!(s.chord_quality, None, "mesure {}", s.index);
        assert!(s.chord_pitch_classes.is_empty() && s.chord_extensions.is_empty());
        assert_eq!(s.chord_root_pc, s.chord_bass_pc);
        assert!(s.chord_changes.is_empty());
        assert_eq!(Chord::from_name(&s.chord_name)?.root, s.chord_root_pc);
        assert_eq!(s.chord_root_offset, 0);
    }
    Ok(())
}
