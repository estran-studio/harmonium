//! E5 + E6 : comping (notes guides, sans fondamentale, drop 2, anticipations, silences) conduit
//! par la conduite de voix, et marche de basse au pas près. Fixture technique
//! `grille_accompagnee` : aucune validation d'écoute revendiquée ; le contrat est observé dans
//! le flux de `MeasureSnapshot` (principe VI), les attendus relatifs à la tonique sont écrits
//! ici sans appeler la théorie testée.
//!
//! La fixture est en fa : le blues (mesures 1 à 12, un ii-V dans la mesure 9) puis un ii-V-I de
//! quatre mesures (mesures 13 à 16, la mesure 15 a trois accords : un par pulsation, puis une
//! blanche). Le flux dure deux tours.
use harmonium_core::{
    StyleProfile, TuningParams,
    compose::TuneGenerator,
    params::MusicalParams,
    report::{MeasureSnapshot, NoteSnapshot},
    theory::voice_leading::voicing_movement,
    timeline::ChordContext,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const SEEDS: [u64; 3] = [1, 2, 17];
const CHORDS: u8 = 1;
const BASS: u8 = 2;
/// Deux tours de seize mesures, plus la mesure qui suit (cible des attaques anticipées).
const BARS: usize = 32;
/// Gamme majeure : classes de hauteur relatives à la tonique.
const MAJOR_SCALE: [u8; 7] = [0, 2, 4, 5, 7, 9, 11];

fn fixture(name: &str) -> TestResult<StyleProfile> {
    Ok(StyleProfile::load(format!(
        "{}/../tests/fixtures/profiles/{name}",
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

/// Même profil, tonalité verrouillée sur `key`.
fn in_key(p: &StyleProfile, key: u8) -> TestResult<StyleProfile> {
    edited(p, |v| {
        v["fixed_tonality"]["key"] = i64::from(key).into();
        v["melody"]["key_modes"][0]["key"] = i64::from(key).into();
        v["dynamics"]["color"][0]["key"] = i64::from(key).into();
        Ok(())
    })
}

fn weights(text: &str) -> TestResult<toml::Value> {
    Ok(toml::from_str(text)?)
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

/// Attaque d'accord : pas, durée, hauteurs croissantes.
struct Attack {
    step: usize,
    duration: usize,
    pitches: Vec<u8>,
}

fn attacks(s: &MeasureSnapshot) -> Vec<Attack> {
    let mut out: Vec<Attack> = Vec::new();
    for n in track(s, CHORDS) {
        match out.last_mut() {
            Some(a) if a.step == n.start_step => a.pitches.push(n.pitch),
            _ => out.push(Attack {
                step: n.start_step,
                duration: n.duration_steps,
                pitches: vec![n.pitch],
            }),
        }
    }
    for a in &mut out {
        a.pitches.sort_unstable();
    }
    out
}

fn context_at(s: &MeasureSnapshot, step: usize) -> &ChordContext {
    &s.chord_changes[s.chord_changes.partition_point(|c| c.start_step <= step) - 1].context
}

/// Accord que joue une attaque. Dans la fixture, toute attaque dont la durée finit sur la barre
/// est anticipée : elle joue le premier accord de la mesure suivante.
fn played<'a>(stream: &'a [MeasureSnapshot], bar: usize, a: &Attack) -> &'a ChordContext {
    if a.step + a.duration == stream[bar].steps {
        &stream[bar + 1].chord_changes[0].context
    } else {
        context_at(&stream[bar], a.step)
    }
}

fn pcs(pitches: &[u8]) -> Vec<u8> {
    let mut out: Vec<u8> = pitches.iter().map(|p| p % 12).collect();
    out.sort_unstable();
    out.dedup();
    out
}

/// Tierce (4 avant 3 : un accord altéré porte les deux) et septième de l'accord publié.
fn third_and_seventh(c: &ChordContext) -> (u8, u8) {
    let find = |intervals: [u8; 2]| {
        intervals
            .into_iter()
            .map(|i| (c.root_pc + i) % 12)
            .find(|pc| c.pitch_classes.contains(pc))
            .unwrap_or(255)
    };
    (find([4, 3]), find([10, 11]))
}

/// Pour chaque attaque de chaque mesure (sauf la dernière) : (mesure, attaque, accord joué).
fn chord_attacks(stream: &[MeasureSnapshot]) -> Vec<(usize, Attack, &ChordContext)> {
    let mut out = Vec::new();
    for bar in 0..stream.len() - 1 {
        for a in attacks(&stream[bar]) {
            let chord = played(stream, bar, &a);
            out.push((bar, a, chord));
        }
    }
    out
}

fn voicing_variant(p: &StyleProfile, voicing: &str) -> TestResult<StyleProfile> {
    edited(p, |v| {
        v["accompaniment"]["chords"]["voicing"] = voicing.into();
        Ok(())
    })
}

#[test]
fn le_comping_sans_fondamentale_ne_joue_que_des_notes_de_l_accord_publie_avec_ses_tensions()
-> TestResult {
    for key in [5, 0, 9] {
        let p = in_key(&fixture("grille_accompagnee")?, key)?;
        for seed in SEEDS {
            let flow = stream(&p, seed, BARS + 1)?;
            let all = chord_attacks(&flow);
            assert!(all.len() > 20, "clé {key} graine {seed} : {} attaques", all.len());
            for (bar, a, chord) in &all {
                let ctx = format!("clé {key} graine {seed} mesure {} pas {}", bar + 1, a.step);
                let mut distinct = a.pitches.clone();
                distinct.dedup();
                assert_eq!(a.pitches.len(), 4, "{ctx} : quatre sons");
                assert_eq!(distinct.len(), 4, "{ctx} : quatre hauteurs distinctes");
                let played = pcs(&a.pitches);
                assert!(
                    played.iter().all(|pc| chord.pitch_classes.contains(pc)),
                    "{ctx} : {played:?} hors de {} {:?}",
                    chord.chord_name,
                    chord.pitch_classes
                );
                assert!(!played.contains(&chord.root_pc), "{ctx} : fondamentale jouée");
                let (third, seventh) = third_and_seventh(chord);
                assert!(
                    played.contains(&third) && played.contains(&seventh),
                    "{ctx} : notes guides {third}/{seventh} absentes de {played:?}"
                );
                if !chord.extensions.is_empty() {
                    assert!(
                        chord
                            .extensions
                            .iter()
                            .any(|e| played.contains(&((chord.root_pc + e) % 12))),
                        "{ctx} : tension de {} absente de {played:?}",
                        chord.chord_name
                    );
                }
                let bottom = a.pitches[0] % 12;
                assert!(
                    bottom == third || bottom == seventh,
                    "{ctx} : type A (tierce) ou B (septième) en bas, pas {bottom}"
                );
                assert!(
                    a.pitches[0] >= 48 && a.pitches[3] <= 80,
                    "{ctx} : tessiture {:?} hors de la fenêtre 52..76 élargie d'une tierce",
                    a.pitches
                );
            }
        }
    }
    Ok(())
}

#[test]
fn les_notes_guides_donnent_tierce_septieme_et_une_tension_si_la_qualite_en_a_une() -> TestResult {
    let p = voicing_variant(&fixture("grille_accompagnee")?, "guide_tones")?;
    for seed in SEEDS {
        let flow = stream(&p, seed, BARS + 1)?;
        for (bar, a, chord) in chord_attacks(&flow) {
            let ctx = format!("graine {seed} mesure {} pas {}", bar + 1, a.step);
            let played = pcs(&a.pitches);
            assert!(played.iter().all(|pc| chord.pitch_classes.contains(pc)), "{ctx}");
            let (third, seventh) = third_and_seventh(chord);
            assert!(played.contains(&third) && played.contains(&seventh), "{ctx}");
            let expected = if chord.extensions.is_empty() { 2 } else { 3 };
            assert_eq!(a.pitches.len(), expected, "{ctx} : {} {:?}", chord.chord_name, a.pitches);
            if !chord.extensions.is_empty() {
                assert!(
                    chord.extensions.iter().any(|e| played.contains(&((chord.root_pc + e) % 12))),
                    "{ctx}"
                );
            }
        }
    }
    Ok(())
}

#[test]
fn le_drop_2_descend_d_une_octave_la_deuxieme_voix_depuis_l_aigu() -> TestResult {
    let p = voicing_variant(&fixture("grille_accompagnee")?, "drop2")?;
    for seed in SEEDS {
        let flow = stream(&p, seed, BARS + 1)?;
        let all = chord_attacks(&flow);
        assert!(!all.is_empty());
        for (bar, a, chord) in all {
            let ctx = format!("graine {seed} mesure {} pas {}", bar + 1, a.step);
            let n = &a.pitches;
            assert_eq!(n.len(), 4, "{ctx}");
            assert!(pcs(n).iter().all(|pc| chord.pitch_classes.contains(pc)), "{ctx}");
            // Le drop 2 d'une position serrée [c0, c1, c2, c3] est [c2 - 12, c0, c1, c3] : la
            // voix grave, remontée d'une octave, tombe strictement entre la deuxième et la
            // troisième voix, et le reste tient dans une octave et une tierce.
            assert!(n[2] < n[0] + 12 && n[0] + 12 < n[3], "{ctx} : {n:?} n'est pas un drop 2");
            assert!(
                n[3] - n[1] <= 13,
                "{ctx} : {n:?} voix hautes plus larges qu'une position serrée"
            );
        }
    }
    Ok(())
}

/// Position fondamentale naïve de l'accord publié : ses notes empilées depuis la fondamentale
/// placée dans l'octave du do central.
fn naive_root_position(c: &ChordContext) -> Vec<u8> {
    let mut out = vec![48 + c.root_pc];
    for pc in &c.pitch_classes[1..] {
        let mut pitch = out[out.len() - 1] + 1;
        while pitch % 12 != *pc {
            pitch += 1;
        }
        out.push(pitch);
    }
    out
}

fn mean_movement(voicings: &[Vec<u8>]) -> f64 {
    let moves: Vec<f64> = voicings
        .windows(2)
        .map(|w| {
            let voices = w[0].len().min(w[1].len()) as f64;
            f64::from(voicing_movement(&w[0], &w[1])) / voices
        })
        .collect();
    moves.iter().sum::<f64>() / moves.len() as f64
}

/// Seuil documenté : déplacement moyen par voix entre deux voicings successifs, en demi-tons.
const MAX_MEAN_VOICE_MOVEMENT: f64 = 1.0;

#[test]
fn la_conduite_de_voix_bouge_moins_que_la_position_fondamentale_naive() -> TestResult {
    for voicing in ["rootless", "guide_tones", "drop2"] {
        let p = voicing_variant(&fixture("grille_accompagnee")?, voicing)?;
        for seed in SEEDS {
            let flow = stream(&p, seed, BARS + 1)?;
            let all = chord_attacks(&flow);
            let led: Vec<Vec<u8>> = all.iter().map(|(_, a, _)| a.pitches.clone()).collect();
            let naive: Vec<Vec<u8>> = all.iter().map(|(_, _, c)| naive_root_position(c)).collect();
            let (ours, theirs) = (mean_movement(&led), mean_movement(&naive));
            assert!(
                ours <= MAX_MEAN_VOICE_MOVEMENT,
                "{voicing} graine {seed} : {ours:.2} demi-tons par voix (seuil {MAX_MEAN_VOICE_MOVEMENT})"
            );
            assert!(
                ours < theirs,
                "{voicing} graine {seed} : {ours:.2} contre {theirs:.2} en position fondamentale"
            );
        }
    }
    Ok(())
}

#[test]
fn le_comping_joue_des_attaques_anticipees_sur_l_accord_suivant_et_des_silences() -> TestResult {
    let (mut anticipated, mut silent) = (0, 0);
    for seed in SEEDS {
        let flow = stream(&fixture("grille_accompagnee")?, seed, BARS + 1)?;
        for bar in 0..BARS {
            let attacks = attacks(&flow[bar]);
            silent += usize::from(attacks.is_empty());
            for a in attacks {
                assert!(
                    [0, 4, 6, 8, 10, 14].contains(&a.step),
                    "mesure {} : attaque au pas {} hors des rythmes du profil",
                    bar + 1,
                    a.step
                );
                if a.step == 14 {
                    let now = pcs(&a.pitches);
                    let this = &context_at(&flow[bar], 14).pitch_classes;
                    let next = &flow[bar + 1].chord_changes[0].context.pitch_classes;
                    if now.iter().all(|pc| next.contains(pc))
                        && !now.iter().all(|pc| this.contains(pc))
                    {
                        anticipated += 1;
                    }
                }
            }
        }
    }
    assert!(anticipated >= 3, "{anticipated} attaque(s) anticipée(s) sur l'accord suivant");
    assert!(silent >= 3, "{silent} mesure(s) silencieuse(s)");
    Ok(())
}

#[test]
fn la_marche_joue_quatre_noires_par_mesure_dans_la_fenetre() -> TestResult {
    for key in [5, 0, 7] {
        let p = in_key(&fixture("grille_accompagnee")?, key)?;
        for seed in SEEDS {
            for s in stream(&p, seed, BARS)? {
                let bass = track(&s, BASS);
                let layout: Vec<_> =
                    bass.iter().map(|n| (n.start_step, n.duration_steps)).collect();
                assert_eq!(layout, [(0, 4), (4, 4), (8, 4), (12, 4)], "mesure {}", s.index);
                assert!(
                    bass.iter().all(|n| (33..=55).contains(&n.pitch)),
                    "clé {key} graine {seed} mesure {} : {:?}",
                    s.index,
                    bass.iter().map(|n| n.pitch).collect::<Vec<_>>()
                );
            }
        }
    }
    Ok(())
}

/// Pas d'arrivée d'une mesure : le premier temps et chaque changement d'accord.
fn arrival_steps(s: &MeasureSnapshot) -> Vec<usize> {
    let mut steps: Vec<usize> = s.chord_changes.iter().map(|c| c.start_step).collect();
    if !steps.contains(&0) {
        steps.insert(0, 0);
    }
    steps
}

fn bass_at(s: &MeasureSnapshot, step: usize) -> u8 {
    let found = track(s, BASS).into_iter().find(|n| n.start_step == step);
    assert!(found.is_some(), "mesure {} : aucune note de basse au pas {step}", s.index);
    found.map_or(0, |n| n.pitch)
}

#[test]
fn la_marche_arrive_sur_la_fondamentale_a_chaque_changement_d_accord_au_pas_pres() -> TestResult {
    for key in [5, 0, 7] {
        let p = in_key(&fixture("grille_accompagnee")?, key)?;
        for seed in SEEDS {
            let flow = stream(&p, seed, BARS)?;
            for s in &flow {
                for step in arrival_steps(s) {
                    let chord = context_at(s, step);
                    assert_eq!(
                        bass_at(s, step) % 12,
                        chord.bass_pc,
                        "clé {key} graine {seed} mesure {} pas {step} : {}",
                        s.index,
                        chord.chord_name
                    );
                }
            }
            // ii-V dans une mesure (mesure 9, Gm7 puis C7 en fa) : la blanche change au pas 8,
            // la mesure 15 change à chaque noire (Gm7, C13, Fmaj9).
            let [ii, v, i] = [2, 7, 0].map(|d| (key + d) % 12);
            assert_eq!(arrival_steps(&flow[8]), [0, 8]);
            assert_eq!((bass_at(&flow[8], 0) % 12, bass_at(&flow[8], 8) % 12), (ii, v));
            assert_eq!(arrival_steps(&flow[14]), [0, 4, 8]);
            assert_eq!(
                [0, 4, 8].map(|step| bass_at(&flow[14], step) % 12),
                [ii, v, i],
                "clé {key} graine {seed}"
            );
        }
    }
    Ok(())
}

/// La pulsation qui précède une arrivée sans en être une : (mesure, pas, hauteur de l'approche,
/// hauteur de l'arrivée). Une arrivée qui suit une pulsation d'arrivée n'a pas d'approche.
fn approaches(flow: &[MeasureSnapshot]) -> Vec<(usize, usize, u8, u8)> {
    let mut out = Vec::new();
    for bar in 0..flow.len() - 1 {
        let arrivals = arrival_steps(&flow[bar]);
        for step in (0..16).step_by(4) {
            if arrivals.contains(&step) {
                continue;
            }
            let target = if step == 12 {
                bass_at(&flow[bar + 1], 0)
            } else if arrivals.contains(&(step + 4)) {
                bass_at(&flow[bar], step + 4)
            } else {
                continue;
            };
            out.push((bar, step, bass_at(&flow[bar], step), target));
        }
    }
    out
}

#[test]
fn l_approche_chromatique_arrive_a_un_demi_ton_de_la_cible() -> TestResult {
    let p = edited(&fixture("grille_accompagnee")?, |v| {
        v["accompaniment"]["bass"]["walking"]["approach"] =
            weights("none_weight = 0\nchromatic_weight = 1\ndiatonic_weight = 0")?;
        Ok(())
    })?;
    for seed in SEEDS {
        let flow = stream(&p, seed, BARS)?;
        let found = approaches(&flow);
        assert!(found.len() > 30, "{} approches", found.len());
        for (bar, step, approach, target) in found {
            assert_eq!(
                approach.abs_diff(target),
                1,
                "graine {seed} mesure {} pas {step} : {approach} vers {target}",
                bar + 1
            );
        }
    }
    Ok(())
}

#[test]
fn l_approche_diatonique_arrive_sur_la_note_de_la_gamme_voisine_de_la_cible() -> TestResult {
    let key = 5u8;
    let p = edited(&fixture("grille_accompagnee")?, |v| {
        v["accompaniment"]["bass"]["walking"]["approach"] =
            weights("none_weight = 0\nchromatic_weight = 0\ndiatonic_weight = 1")?;
        Ok(())
    })?;
    let in_scale = |pitch: u8| MAJOR_SCALE.contains(&((pitch + 12 - key) % 12));
    let mut wide = 0;
    for seed in SEEDS {
        let flow = stream(&p, seed, BARS)?;
        for (bar, step, approach, target) in approaches(&flow) {
            let ctx =
                format!("graine {seed} mesure {} pas {step} : {approach} vers {target}", bar + 1);
            let distance = approach.abs_diff(target);
            assert!((1..=3).contains(&distance), "{ctx}");
            assert!(in_scale(approach), "{ctx} : hors de la gamme");
            let (low, high) = (approach.min(target), approach.max(target));
            assert!(
                (low + 1..high).all(|between| !in_scale(between)),
                "{ctx} : une note de la gamme se trouve entre l'approche et la cible"
            );
            wide += usize::from(distance > 1);
        }
    }
    assert!(wide > 0, "un ton entier attendu au moins une fois");
    Ok(())
}

#[test]
fn les_notes_de_passage_sans_approche_suivent_l_accord_du_pas() -> TestResult {
    let p = edited(&fixture("grille_accompagnee")?, |v| {
        let walking = &mut v["accompaniment"]["bass"]["walking"];
        walking["approach"] =
            weights("none_weight = 1\nchromatic_weight = 0\ndiatonic_weight = 0")?;
        walking["passing"] = weights("chord_tone_weight = 1\nscale_step_weight = 0")?;
        Ok(())
    })?;
    for seed in SEEDS {
        for s in stream(&p, seed, BARS)? {
            for n in track(&s, BASS) {
                let chord = context_at(&s, n.start_step);
                assert!(
                    chord.pitch_classes.contains(&(n.pitch % 12)),
                    "graine {seed} mesure {} pas {} : {} sur {}",
                    s.index,
                    n.start_step,
                    n.pitch,
                    chord.chord_name
                );
            }
        }
    }
    Ok(())
}

#[test]
fn la_marche_et_le_comping_se_rejouent_a_l_identique_et_varient_selon_la_graine() -> TestResult {
    let p = fixture("grille_accompagnee")?;
    let a = stream(&p, 1, BARS)?;
    assert_eq!(serde_json::to_value(&a)?, serde_json::to_value(stream(&p, 1, BARS)?)?);
    let b = stream(&p, 2, BARS)?;
    assert_ne!(serde_json::to_value(&a)?, serde_json::to_value(&b)?);
    Ok(())
}

/// `jazz_test` (rewrite_functional, un accord par mesure) : même mécanisme de basse et de
/// comping que les autres modèles, aucun second chemin (principe IV).
#[test]
fn rewrite_functional_joue_la_marche_et_le_comping_du_profil() -> TestResult {
    let p = edited(&fixture("jazz_test")?, |v| {
        v["harmony"]["steps_per_chord"] = 8.into();
        let bass = &mut v["accompaniment"]["bass"];
        bass.as_table_mut().ok_or("bass")?.remove("hits");
        bass.as_table_mut().ok_or("bass")?.insert(
            "walking".into(),
            toml::from_str(
                "velocity = 70\nduration = 4\nmax_leap = 5\n\
                 approach = { none_weight = 1, chromatic_weight = 3, diatonic_weight = 1 }\n\
                 passing = { chord_tone_weight = 2, scale_step_weight = 2 }",
            )?,
        );
        let chords = &mut v["accompaniment"]["chords"];
        chords["voicing"] = "rootless".into();
        chords["low"] = 52.into();
        chords["high"] = 76.into();
        Ok(())
    })?;
    for seed in [1, 2] {
        let flow = stream(&p, seed, 32)?;
        let mut played = 0;
        for s in &flow {
            let layout: Vec<_> =
                track(s, BASS).iter().map(|n| (n.start_step, n.duration_steps)).collect();
            assert_eq!(layout, [(0, 4), (4, 4), (8, 4), (12, 4)], "mesure {}", s.index);
            assert_eq!(arrival_steps(s), [0, 8], "mesure {}", s.index);
            for step in [0, 8] {
                assert_eq!(
                    bass_at(s, step) % 12,
                    context_at(s, step).bass_pc,
                    "mesure {}",
                    s.index
                );
            }
            for a in attacks(s) {
                let chord = context_at(s, a.step);
                assert_eq!(a.pitches.len(), 4, "mesure {}", s.index);
                assert!(pcs(&a.pitches).iter().all(|pc| chord.pitch_classes.contains(pc)));
                assert!(!pcs(&a.pitches).contains(&chord.root_pc));
                played += 1;
            }
        }
        assert!(played > 10);
    }
    Ok(())
}

#[test]
fn les_options_incoherentes_sont_refusees_au_chargement() -> TestResult {
    let p = fixture("grille_accompagnee")?;
    type Mutation = fn(&mut toml::Value) -> TestResult;
    let cases: [(&str, Mutation); 9] = [
        ("accompaniment.bass.walking", |v| {
            v["accompaniment"]["bass"].as_table_mut().ok_or("bass")?.insert(
                "hits".into(),
                toml::from_str::<toml::Value>(
                    "hits = [{ step = 0, degree = \"root\", duration = 3, velocity = 72 }]",
                )?["hits"]
                    .clone(),
            );
            Ok(())
        }),
        ("accompaniment.bass.walking", |v| {
            v["harmony"]["steps_per_chord"] = 2.into();
            Ok(())
        }),
        ("accompaniment.bass.walking.duration", |v| {
            v["accompaniment"]["bass"]["walking"]["duration"] = 5.into();
            Ok(())
        }),
        ("accompaniment.bass.walking.max_leap", |v| {
            v["accompaniment"]["bass"]["walking"]["max_leap"] = 0.into();
            Ok(())
        }),
        ("accompaniment.bass.walking.approach", |v| {
            v["accompaniment"]["bass"]["walking"]["approach"] =
                weights("none_weight = 0\nchromatic_weight = 0\ndiatonic_weight = 0")?;
            Ok(())
        }),
        ("accompaniment.bass.walking.passing", |v| {
            v["accompaniment"]["bass"]["walking"]["passing"] =
                weights("chord_tone_weight = 0\nscale_step_weight = 0")?;
            Ok(())
        }),
        ("accompaniment.chords.high", |v| {
            v["accompaniment"]["chords"]["high"] = 60.into();
            Ok(())
        }),
        ("accompaniment.chords.comping", |v| {
            for motif in v["accompaniment"]["chords"]["comping"].as_array_mut().ok_or("comping")? {
                motif["weight"] = 0.into();
            }
            Ok(())
        }),
        ("accompaniment.chords.comping[0].hits[0].probability", |v| {
            v["accompaniment"]["chords"]["comping"][0]["hits"][0]["probability"] = 1.5.into();
            Ok(())
        }),
    ];
    for (field, mutate) in cases {
        let bad = edited(&p, mutate);
        let error = bad.err().ok_or(format!("{field} : profil invalide accepté"))?.to_string();
        assert!(error.contains(field), "{field} : {error}");
    }
    Ok(())
}

/// Accords slash de `grille_riche` (`C/E`, `D/C`, dont la basse sort de l'accord) : la marche
/// arrive sur la basse publiée, pas sur la fondamentale.
#[test]
fn la_marche_arrive_sur_la_basse_d_un_accord_slash() -> TestResult {
    let p = edited(&fixture("grille_riche")?, |v| {
        let bass = v["accompaniment"]["bass"].as_table_mut().ok_or("bass")?;
        bass.remove("hits");
        bass.insert(
            "walking".into(),
            toml::from_str(
                "velocity = 70\nduration = 4\nmax_leap = 5\n\
                 approach = { none_weight = 1, chromatic_weight = 3, diatonic_weight = 1 }\n\
                 passing = { chord_tone_weight = 2, scale_step_weight = 2 }",
            )?,
        );
        Ok(())
    })?;
    let mut slash = 0;
    for seed in SEEDS {
        for s in stream(&p, seed, 16)? {
            for step in arrival_steps(&s) {
                let chord = context_at(&s, step);
                if chord.bass_pc != chord.root_pc {
                    slash += 1;
                }
                assert_eq!(
                    bass_at(&s, step) % 12,
                    chord.bass_pc,
                    "graine {seed} mesure {} pas {step} : {}",
                    s.index,
                    chord.chord_name
                );
            }
        }
    }
    assert!(slash >= 6, "{slash} arrivée(s) sur un accord slash");
    Ok(())
}
