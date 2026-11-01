//! P1 : boucles d'accords générées par contraintes, pédale à alternatives, basse par gabarits et
//! batterie à portée de partie et de tour. Fixture technique `regles_procedurales` (do ionien
//! verrouillé) : aucune validation d'écoute revendiquée ; le contrat est observé dans le flux de
//! `MeasureSnapshot` (principe VI), les attendus sont écrits ici relativement à l'accord publié.
//!
//! Forme de la fixture, deux tours de vingt mesures : `Chroma` (mesures 1 à 8), `Quintes` (9 à
//! 16), `Pedale` (17 à 20), puis la même chose aux mesures 21 à 40.
use std::collections::BTreeSet;

use harmonium_core::{
    StyleProfile, TuningParams,
    compose::TuneGenerator,
    params::MusicalParams,
    report::{MeasureSnapshot, NoteSnapshot},
    timeline::ChordContext,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const SEEDS: std::ops::RangeInclusive<u64> = 1..=16;
const BARS: usize = 40;
const CHORDS: u8 = 1;
const BASS: u8 = 2;
const PERCUSSION_LOW: u8 = 3;
const PERCUSSION_HIGH: u8 = 4;
const MAJOR_SCALE: [u8; 7] = [0, 2, 4, 5, 7, 9, 11];

fn fixture() -> TestResult<StyleProfile> {
    Ok(StyleProfile::load(format!(
        "{}/../tests/fixtures/profiles/regles_procedurales",
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

/// Même profil sans boucle écrite dans `Chroma` : seule la boucle générée reste.
fn generated_only() -> TestResult<StyleProfile> {
    edited(&fixture()?, |v| {
        v["harmony"]["parts"]["Chroma"].as_table_mut().ok_or("partie Chroma")?.remove("loops");
        Ok(())
    })
}

fn stream(p: &StyleProfile, seed: u64) -> TestResult<Vec<MeasureSnapshot>> {
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
    Ok((1..=BARS)
        .map(|bar| MeasureSnapshot::from_measure(&g.generate_measure(bar, &mut rng)))
        .collect())
}

fn track(s: &MeasureSnapshot, id: u8) -> Vec<&NoteSnapshot> {
    s.notes.iter().filter(|n| n.track == id).collect()
}

/// Mesures 1-based `from..=to` du flux.
fn bars(flow: &[MeasureSnapshot], from: usize, to: usize) -> &[MeasureSnapshot] {
    &flow[from - 1..to]
}

fn first(s: &MeasureSnapshot) -> &ChordContext {
    &s.chord_changes[0].context
}

fn context_at(s: &MeasureSnapshot, step: usize) -> &ChordContext {
    &s.chord_changes[s.chord_changes.partition_point(|c| c.start_step <= step) - 1].context
}

fn quality(c: &ChordContext) -> String {
    serde_json::to_value(c.quality)
        .map_or_else(|_| "?".into(), |v| v.as_str().unwrap_or("-").into())
}

/// Suite (fondamentale, qualité, basse) des accords du début de chaque mesure.
fn progression(flow: &[MeasureSnapshot], from: usize, to: usize) -> Vec<(u8, String, u8)> {
    bars(flow, from, to)
        .iter()
        .map(|s| (first(s).root_pc, quality(first(s)), first(s).bass_pc))
        .collect()
}

fn json(value: &impl serde::Serialize) -> serde_json::Value {
    serde_json::to_value(value).unwrap_or_default()
}

fn interval(from: u8, to: u8) -> u8 {
    (to + 12 - from) % 12
}

// ───────────────────────── boucles générées ─────────────────────────

#[test]
fn la_boucle_generee_respecte_qualites_mouvement_pedale_et_longueur() -> TestResult {
    let p = generated_only()?;
    let mut sequences = BTreeSet::new();
    for seed in SEEDS {
        let flow = stream(&p, seed)?;
        let chords = progression(&flow, 1, 8);
        assert_eq!(chords[0].0, 0, "graine {seed} : la boucle part de la tonique");
        for (index, (root, quality, bass)) in chords.iter().enumerate() {
            assert_eq!(quality, "m", "graine {seed} mesure {} : qualité permise", index + 1);
            assert_eq!(*bass, 0, "graine {seed} mesure {} : pédale de tonique", index + 1);
            if index % 4 != 0 {
                let step = interval(chords[index - 1].0, *root);
                assert!(
                    step == 1 || step == 11,
                    "graine {seed} mesure {} : mouvement chromatique, pas {step}",
                    index + 1
                );
            }
        }
        assert_eq!(chords[..4], chords[4..], "graine {seed} : la boucle de 4 mesures se répète");
        assert_eq!(
            progression(&flow, 21, 28),
            chords,
            "graine {seed} : même boucle au tour suivant (tirée une fois par air)"
        );
        assert_eq!(json(&stream(&p, seed)?), json(&flow), "graine {seed} : même graine, même flux");
        sequences.insert(chords.iter().map(|(root, ..)| *root).collect::<Vec<_>>());
    }
    assert!(sequences.len() >= 4, "la boucle varie entre graines : {sequences:?}");
    Ok(())
}

#[test]
fn une_boucle_generee_partage_le_tirage_pondere_avec_les_boucles_ecrites() -> TestResult {
    let (mut written, mut generated) = (0, 0);
    for seed in SEEDS {
        let flow = stream(&fixture()?, seed)?;
        let roots: Vec<u8> = progression(&flow, 1, 8).iter().map(|c| c.0).collect();
        if roots == [0, 0, 0, 0, 5, 5, 5, 5] {
            written += 1;
        } else {
            assert_eq!(roots[..4], roots[4..], "graine {seed} : boucle générée répétée");
            generated += 1;
        }
    }
    assert!(written > 0 && generated > 0, "écrite {written}, générée {generated} sur 16 graines");
    assert!(generated > written, "poids 3 contre 1 : écrite {written}, générée {generated}");
    Ok(())
}

#[test]
fn les_quintes_montantes_dans_la_gamme_gardent_leurs_racines_et_varient_en_qualite() -> TestResult {
    let p = fixture()?;
    let mut qualities = BTreeSet::new();
    for seed in SEEDS {
        let flow = stream(&p, seed)?;
        // Deux mesures, un accord par demi-mesure : 0, 7, 2, 9 (quintes montantes depuis la tonique).
        let mut loop_chords = Vec::new();
        for s in bars(&flow, 9, 10) {
            assert_eq!(s.chord_changes.len(), 2, "graine {seed} : deux accords par mesure");
            loop_chords.extend(s.chord_changes.iter().map(|c| &c.context));
        }
        let roots: Vec<u8> = loop_chords.iter().map(|c| c.root_pc).collect();
        assert_eq!(roots, [0, 7, 2, 9], "graine {seed}");
        for chord in &loop_chords {
            assert!(
                chord.pitch_classes.iter().all(|pc| MAJOR_SCALE.contains(pc)),
                "graine {seed} : {} hors de la gamme {:?}",
                chord.chord_name,
                chord.pitch_classes
            );
            assert!(
                ["", "m", "7", "m7", "maj7"].contains(&quality(chord).as_str()),
                "graine {seed} : qualité {} non permise",
                quality(chord)
            );
        }
        // Quatre répétitions de la boucle dans la partie de huit mesures.
        for repeat in 1..4 {
            for (offset, expected) in bars(&flow, 9, 10).iter().enumerate() {
                let got = &flow[8 + 2 * repeat + offset];
                assert_eq!(
                    json(&got.chord_changes),
                    json(&expected.chord_changes),
                    "graine {seed} : répétition {repeat} de la boucle"
                );
            }
        }
        qualities.insert(loop_chords.iter().map(|c| quality(c)).collect::<Vec<_>>());
    }
    assert!(qualities.len() >= 3, "les qualités varient entre graines : {qualities:?}");
    Ok(())
}

#[test]
fn la_pedale_tire_une_alternative_par_air_et_la_garde() -> TestResult {
    let mut roots = BTreeSet::new();
    for seed in SEEDS {
        let flow = stream(&fixture()?, seed)?;
        let chosen: Vec<u8> = bars(&flow, 17, 20).iter().map(|s| first(s).root_pc).collect();
        assert!(chosen.iter().all(|&root| root == chosen[0]), "graine {seed} : {chosen:?}");
        assert_eq!(
            bars(&flow, 37, 40).iter().map(|s| first(s).root_pc).collect::<Vec<_>>(),
            chosen,
            "graine {seed} : même pédale au tour suivant"
        );
        // Les notes d'accord suivent : la pédale à voisin joue deux classes (quarte et sa seconde mineure).
        let played: BTreeSet<u8> = track(&flow[16], CHORDS).iter().map(|n| n.pitch % 12).collect();
        let neighbor = chosen[0] == 5;
        assert_eq!(played.len(), if neighbor { 2 } else { 1 }, "graine {seed} : {played:?}");
        assert!(played.contains(&chosen[0]), "graine {seed} : fondamentale jouée");
        roots.insert(chosen[0]);
    }
    assert_eq!(roots, BTreeSet::from([0, 5, 7]), "les trois pédales sont tirées sur 16 graines");
    Ok(())
}

// ───────────────────────── basse par gabarits ─────────────────────────

#[test]
fn la_quinte_fondamentale_alterne_la_basse_publiee_et_la_quinte_de_la_fondamentale() -> TestResult {
    let p = generated_only()?;
    for seed in SEEDS {
        let flow = stream(&p, seed)?;
        for (index, s) in bars(&flow, 1, 8).iter().enumerate() {
            let bass = track(s, BASS);
            let steps: Vec<usize> = bass.iter().map(|n| n.start_step).collect();
            assert_eq!(steps, [0, 4, 8, 12], "graine {seed} mesure {}", index + 1);
            let chord = first(s);
            for (n, note) in bass.iter().enumerate() {
                let expected = if n % 2 == 0 { chord.bass_pc } else { (chord.root_pc + 7) % 12 };
                assert_eq!(
                    note.pitch % 12,
                    expected,
                    "graine {seed} mesure {} pas {} : {}",
                    index + 1,
                    note.start_step,
                    chord.chord_name
                );
                assert!((36..=47).contains(&note.pitch), "registre : {}", note.pitch);
                assert_eq!(note.duration_steps, 4);
            }
        }
    }
    Ok(())
}

#[test]
fn les_octaves_alternent_et_la_relance_anticipe_la_basse_de_l_accord_suivant() -> TestResult {
    for seed in SEEDS {
        let flow = stream(&fixture()?, seed)?;
        for bar in 9..=16 {
            let s = &flow[bar - 1];
            let next = &flow[bar];
            let bass = track(s, BASS);
            let at: Vec<(usize, usize)> =
                bass.iter().map(|n| (n.start_step, n.duration_steps)).collect();
            // Attaques 0, 4, 8, 12 (durée 3), la relance (durée 2, 2 pas avant le changement) retire
            // ou raccourcit ce qu'elle recouvre : l'attaque 4 (6 > 4 + 3) et l'attaque 12.
            assert_eq!(
                at,
                [(0, 3), (4, 2), (6, 2), (8, 3), (12, 2), (14, 2)],
                "graine {seed} mesure {bar}"
            );
            let (second, end) = (context_at(s, 8), first(next));
            // Octaves : la deuxième attaque est une octave au-dessus de la première.
            assert_eq!(bass[1].pitch, bass[0].pitch + 12, "graine {seed} mesure {bar} : octave");
            assert_eq!(bass[0].pitch % 12, first(s).bass_pc);
            assert_eq!(bass[3].pitch % 12, second.bass_pc);
            // Relances : la basse de l'accord qui suit, jouée avant son arrivée.
            assert_eq!(
                bass[2].pitch % 12,
                second.bass_pc,
                "graine {seed} mesure {bar} : relance 1"
            );
            assert_eq!(bass[5].pitch % 12, end.bass_pc, "graine {seed} mesure {bar} : relance 2");
            assert_eq!(bass[5].velocity, 74);
        }
    }
    Ok(())
}

#[test]
fn la_relance_suit_les_changements_d_accord_ou_chaque_barre_selon_when() -> TestResult {
    let with_pickup = |template: usize, when: Option<&'static str>| {
        edited(&fixture()?, |v| {
            let t = &mut v["accompaniment"]["bass"]["templates"][template];
            let pickup = toml::from_str::<toml::Value>("lead = 2\nduration = 2\nvelocity = 74")?;
            t.as_table_mut().ok_or("gabarit")?.insert("pickup".into(), pickup);
            if let Some(when) = when {
                t["pickup"].as_table_mut().ok_or("relance")?.insert("when".into(), when.into());
            }
            Ok(())
        })
    };
    for seed in SEEDS {
        // Pédale tenue : aucune relance tant que l'accord ne change pas, une seule avant le
        // premier accord de la partie suivante (mesure 20 vers 21).
        let flow = stream(&with_pickup(2, None)?, seed)?;
        for bar in 17..=19 {
            let bass = track(&flow[bar - 1], BASS);
            assert_eq!(
                bass.len(),
                2,
                "graine {seed} mesure {bar} : aucune relance sur accord tenu"
            );
        }
        let last: Vec<(usize, u8)> =
            track(&flow[19], BASS).iter().map(|n| (n.start_step, n.pitch % 12)).collect();
        assert_eq!(last.len(), 3, "graine {seed} : relance avant le changement de partie");
        assert_eq!(last[2], (14, first(&flow[20]).bass_pc), "graine {seed}");
        // `bar_end` : une relance avant chaque barre, même sur accord tenu.
        let flow = stream(&with_pickup(2, Some("bar_end"))?, seed)?;
        for bar in 17..=20 {
            let bass = track(&flow[bar - 1], BASS);
            assert_eq!(bass.len(), 3, "graine {seed} mesure {bar} : relance à chaque barre");
            assert_eq!((bass[2].start_step, bass[2].duration_steps), (14, 2));
        }
        // Dernière mesure du flux : aucun accord suivant, aucune relance.
        assert_eq!(
            track(&flow[BARS - 1], BASS).len(),
            2,
            "graine {seed} : pas de relance après la dernière mesure"
        );
        // `bar_end` sur deux accords par mesure : seule la relance de la barre.
        let flow = stream(&with_pickup(1, Some("bar_end"))?, seed)?;
        let at: Vec<(usize, usize)> =
            track(&flow[8], BASS).iter().map(|n| (n.start_step, n.duration_steps)).collect();
        assert_eq!(at, [(0, 3), (4, 3), (8, 3), (12, 2), (14, 2)], "graine {seed}");
    }
    Ok(())
}

#[test]
fn la_montee_ajoute_un_demi_ton_par_mesure_puis_repart_au_cycle_suivant() -> TestResult {
    for seed in SEEDS {
        let flow = stream(&fixture()?, seed)?;
        for from in [17, 37] {
            let base = track(&flow[from - 1], BASS)[0].pitch;
            assert_eq!(base % 12, first(&flow[from - 1]).bass_pc, "graine {seed} : départ");
            for k in 0..4 {
                let s = &flow[from - 1 + k];
                let pitches: Vec<u8> = track(s, BASS).iter().map(|n| n.pitch).collect();
                assert_eq!(
                    pitches,
                    [base + k as u8, base + k as u8],
                    "graine {seed} mesure {} : +{k} demi-ton(s)",
                    from + k
                );
            }
        }
    }
    Ok(())
}

#[test]
fn la_descente_par_pas_de_gamme_suit_la_gamme_du_mode() -> TestResult {
    let p = edited(&fixture()?, |v| {
        let templates = v["accompaniment"]["bass"]["templates"].as_array_mut().ok_or("gabarits")?;
        let ramp = templates[2]["ramp"].as_table_mut().ok_or("montée")?;
        ramp.insert("stride".into(), (-2).into());
        ramp.insert("motion".into(), "scale_step".into());
        Ok(())
    })?;
    let scale_down = |mut pitch: u8, steps: usize| {
        for _ in 0..steps {
            loop {
                pitch -= 1;
                if MAJOR_SCALE.contains(&(pitch % 12)) {
                    break;
                }
            }
        }
        pitch
    };
    for seed in SEEDS {
        let flow = stream(&p, seed)?;
        let base = track(&flow[16], BASS)[0].pitch;
        assert!(MAJOR_SCALE.contains(&(base % 12)));
        for k in 0..4 {
            let got = track(&flow[16 + k], BASS)[0].pitch;
            assert_eq!(got, scale_down(base, 2 * k), "graine {seed} mesure {} (k = {k})", 17 + k);
        }
    }
    Ok(())
}

// ───────────────────────── batterie par portée ─────────────────────────

/// Pas des attaques de percussion de la mesure : (grave, aigu).
fn drum_steps(s: &MeasureSnapshot) -> (Vec<usize>, Vec<usize>) {
    let steps = |id| track(s, id).iter().map(|n| n.start_step).collect();
    (steps(PERCUSSION_LOW), steps(PERCUSSION_HIGH))
}

#[test]
fn le_pattern_le_plus_specifique_gagne_selon_la_partie_et_le_tour() -> TestResult {
    let fallback = (vec![0], vec![8]);
    let part_scoped = (vec![0, 6], vec![10]);
    let tour_scoped = (vec![4], vec![12]);
    let both_scoped = (vec![2], vec![]);
    let mut chroma_tour_2 = BTreeSet::new();
    for seed in SEEDS {
        let flow = stream(&fixture()?, seed)?;
        let at = |bar: usize| drum_steps(&flow[bar - 1]);
        // Tour 1 : `Chroma` a son pattern, le reste se replie.
        assert_eq!(at(1), part_scoped, "graine {seed} : Chroma, tour 1");
        assert_eq!(at(8), part_scoped);
        assert_eq!(at(9), fallback, "graine {seed} : Quintes, tour 1");
        assert_eq!(at(17), fallback, "graine {seed} : Pedale, tour 1");
        // Tour 2 : le pattern de tour concourt ; Quintes en hérite, Pedale a le sien (deux portées).
        assert_eq!(at(29), tour_scoped, "graine {seed} : Quintes, tour 2");
        assert_eq!(at(37), both_scoped, "graine {seed} : Pedale, tour 2");
        // Chroma, tour 2 : deux candidats à portée unique, tirés par poids puis fixés pour l'air.
        let chroma = at(21);
        assert!(chroma == part_scoped || chroma == tour_scoped, "graine {seed} : {chroma:?}");
        assert!((21..=28).all(|bar| at(bar) == chroma), "graine {seed} : stable dans la partie");
        chroma_tour_2.insert(chroma);
    }
    assert_eq!(chroma_tour_2.len(), 2, "les deux candidats sont tirés sur 16 graines");
    Ok(())
}

#[test]
fn la_batterie_sans_portee_ne_change_pas_de_tour_en_tour() -> TestResult {
    let p = edited(&fixture()?, |v| {
        let patterns = v["percussion"]["patterns"].as_array_mut().ok_or("patterns")?;
        patterns.truncate(3);
        Ok(())
    })?;
    let flow = stream(&p, 3)?;
    let (low, high) = drum_steps(&flow[0]);
    assert_eq!((low, high), (vec![0], vec![8]));
    for bar in 1..=BARS {
        assert_eq!(drum_steps(&flow[bar - 1]), (vec![0], vec![8]), "mesure {bar}");
    }
    Ok(())
}

// ───────────────────────── validation ─────────────────────────

fn refused(change: impl FnOnce(&mut toml::Value) -> TestResult) -> TestResult<(String, String)> {
    let mut value: toml::Value = toml::from_str(&fixture()?.to_toml()?)?;
    change(&mut value)?;
    let error =
        StyleProfile::from_toml(&toml::to_string(&value)?).err().ok_or("profil refusé attendu")?;
    Ok((error.field.clone(), error.message.clone()))
}

fn set(value: &mut toml::Value, path: &[&str], new: impl Into<toml::Value>) -> TestResult {
    let (last, parents) = path.split_last().ok_or("chemin vide")?;
    let mut node = value;
    for key in parents {
        node = match key.parse::<usize>() {
            Ok(index) => node.get_mut(index),
            Err(_) => node.get_mut(*key),
        }
        .ok_or_else(|| format!("{key} absent"))?;
    }
    node.as_table_mut().ok_or("table attendue")?.insert((*last).into(), new.into());
    Ok(())
}

#[test]
fn la_validation_refuse_les_contraintes_insatisfaisables_en_nommant_le_champ() -> TestResult {
    let at = |last: &'static str| ["harmony", "parts", "Quintes", "generated", "0", last];
    // Nombre d'accords au-delà des créneaux de la boucle (2 mesures de 2 créneaux).
    let (field, message) = refused(|v| set(v, &at("count"), 5))?;
    assert_eq!(field, "harmony.parts.Quintes.generated[0].count");
    assert!(message.contains("1..4"), "{message}");
    // Longueur de boucle qui ne divise pas la partie de huit mesures.
    let (field, _) = refused(|v| set(v, &at("bars"), 3))?;
    assert_eq!(field, "harmony.parts.Quintes.generated[0].bars");
    // Aucune qualité.
    let (field, _) = refused(|v| set(v, &at("qualities"), Vec::<toml::Value>::new()))?;
    assert_eq!(field, "harmony.parts.Quintes.generated[0].qualities");
    // Quintes descendantes dans la gamme : sib n'a aucune qualité diatonique.
    let (field, message) = refused(|v| {
        set(
            v,
            &at("motion"),
            toml::from_str::<toml::Value>("kind = \"fifths\"\ndirection = \"down\"")?,
        )
    })?;
    assert_eq!(field, "harmony.parts.Quintes.generated[0].motion");
    assert!(message.contains("Ionian"), "{message}");
    // Départ sur do dièse : aucune qualité permise n'est diatonique.
    let (field, _) = refused(|v| set(v, &at("start"), 1))?;
    assert_eq!(field, "harmony.parts.Quintes.generated[0].key_fit");
    // Intervalles permis vides.
    let (field, _) = refused(|v| {
        set(
            v,
            &at("motion"),
            toml::from_str::<toml::Value>("kind = \"intervals\"\nintervals = []")?,
        )
    })?;
    assert_eq!(field, "harmony.parts.Quintes.generated[0].motion.intervals");
    // Pédale hors de 0..11 ; qualité inconnue.
    let (field, _) = refused(|v| set(v, &at("pedal"), 12))?;
    assert_eq!(field, "harmony.parts.Quintes.generated[0].pedal");
    // Pédale sans alternative valide : poids nul.
    let (field, _) =
        refused(|v| set(v, &["harmony", "parts", "Pedale", "alternatives", "1", "weight"], 0))?;
    assert_eq!(field, "harmony.parts.Pedale.alternatives[1].weight");
    Ok(())
}

#[test]
fn la_validation_refuse_les_gabarits_et_les_portees_incoherents() -> TestResult {
    let template = |index: &'static str, last: &'static str| {
        ["accompaniment", "bass", "templates", index, last]
    };
    let (field, _) =
        refused(|v| set(v, &template("2", "ramp"), toml::from_str::<toml::Value>("stride = 0")?))?;
    assert_eq!(field, "accompaniment.bass.templates[2].ramp.stride");
    let (field, _) = refused(|v| {
        set(
            v,
            &template("1", "pickup"),
            toml::from_str::<toml::Value>("lead = 8\nduration = 2\nvelocity = 70")?,
        )
    })?;
    assert_eq!(field, "accompaniment.bass.templates[1].pickup.lead");
    let (field, _) = refused(|v| set(v, &template("0", "part"), "Inconnue"))?;
    assert_eq!(field, "accompaniment.bass.templates[0].part");
    let (field, _) = refused(|v| set(v, &template("0", "bars"), 17))?;
    assert_eq!(field, "accompaniment.bass.templates[0].bars");
    let (field, _) = refused(|v| set(v, &template("0", "rhythm"), Vec::<toml::Value>::new()))?;
    assert_eq!(field, "accompaniment.bass.templates[0].rhythm");
    // Un gabarit exclut la marche de basse.
    let (field, _) = refused(|v| {
        set(
            v,
            &["accompaniment", "bass", "walking"],
            toml::from_str::<toml::Value>(
                "velocity = 70\nduration = 4\nmax_leap = 5\napproach = { none_weight = 1 }\npassing = { chord_tone_weight = 1 }",
            )?,
        )
    })?;
    assert_eq!(field, "accompaniment.bass.walking");
    // Batterie : partie inconnue.
    let (field, _) = refused(|v| set(v, &["percussion", "patterns", "3", "part"], "Inconnue"))?;
    assert_eq!(field, "percussion.patterns[3].part");
    // Batterie : une densité sans motif de repli.
    let (field, message) = refused(|v| {
        let patterns = v["percussion"]["patterns"].as_array_mut().ok_or("patterns")?;
        patterns.remove(1);
        Ok(())
    })?;
    assert_eq!(field, "percussion");
    assert!(message.contains("repli"), "{message}");
    Ok(())
}
