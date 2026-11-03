//! A2 : l'habillage par graine et l'orchestration par chorus, observés dans le flux de
//! `MeasureSnapshot` et dans le MIDI exporté (principe VI).
//!
//! La fixture `grille_habillage` (blues de douze mesures, quatre chorus, arc et phrases) porte neuf
//! groupes de variantes : tempo et swing, tonalité, instrument du comping, de la basse, kit,
//! famille de voicings, registre, énergie, arc. Les tests réécrivent la section `[session]`.
use std::collections::{BTreeMap, BTreeSet};

use harmonium_core::{
    StyleProfile, TuningParams,
    compose::TuneGenerator,
    params::MusicalParams,
    report::{MeasureSnapshot, NoteSnapshot},
    timeline::{Measure, ScoreTimeline, timeline_to_midi},
};
use midly::{MidiMessage, Smf, TrackEventKind};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const CHORDS: u8 = 1;
const BASS: u8 = 2;
const BLUES: usize = 12;
const BARS: usize = 48;
const SEEDS: std::ops::RangeInclusive<u64> = 1..=16;
const SOME_SEEDS: [u64; 3] = [1, 2, 7];

fn fixture() -> TestResult<StyleProfile> {
    Ok(StyleProfile::load(format!(
        "{}/../tests/fixtures/profiles/grille_habillage",
        env!("CARGO_MANIFEST_DIR")
    ))?)
}

/// Même profil après `change` sur son arbre TOML.
fn edited(
    p: &StyleProfile,
    change: impl FnOnce(&mut toml::Value) -> TestResult,
) -> TestResult<StyleProfile> {
    let mut value: toml::Value = toml::from_str(&p.to_toml()?)?;
    change(&mut value)?;
    Ok(StyleProfile::from_toml(&toml::to_string(&value)?)?)
}

/// Même profil, `[session]` remplacée par `session` (TOML de la section).
fn with_session(p: &StyleProfile, session: &str) -> TestResult<StyleProfile> {
    edited(p, |v| {
        v.as_table_mut().ok_or("profil")?.insert("session".into(), toml::from_str(session)?);
        Ok(())
    })
}

fn params(p: &StyleProfile) -> MusicalParams {
    MusicalParams {
        bpm: p.meter.default_bpm,
        time_signature: p.meter.time_signature,
        swing_ratio: p.meter.default_swing_ratio,
        harmony_valence: p.dynamics.neutral_valence,
        intensity: 0.5,
        percussion_density: 1,
        ..Default::default()
    }
}

fn measures_with(
    p: &StyleProfile,
    params: MusicalParams,
    seed: u64,
    bars: usize,
) -> TestResult<Vec<Measure>> {
    p.validate()?;
    let mut g = TuneGenerator::new(params, TuningParams { profile: p.clone() });
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    Ok((1..=bars).map(|bar| g.generate_measure(bar, &mut rng)).collect())
}

fn stream_with(
    p: &StyleProfile,
    params: MusicalParams,
    seed: u64,
    bars: usize,
) -> TestResult<Vec<MeasureSnapshot>> {
    Ok(measures_with(p, params, seed, bars)?.iter().map(MeasureSnapshot::from_measure).collect())
}

fn stream(p: &StyleProfile, seed: u64, bars: usize) -> TestResult<Vec<MeasureSnapshot>> {
    stream_with(p, params(p), seed, bars)
}

fn track(s: &MeasureSnapshot, id: u8) -> Vec<&NoteSnapshot> {
    s.notes.iter().filter(|n| n.track == id).collect()
}

fn chorus_of(s: &MeasureSnapshot) -> usize {
    s.position.map_or(0, |p| p.chorus())
}

fn drawn(s: &MeasureSnapshot, group: &str) -> String {
    s.session.iter().find(|v| v.group == group).map(|v| v.name.clone()).unwrap_or_default()
}

/// Nombre de tempos distincts à `tolerance` BPM près (grappes d'au plus `tolerance` de large).
fn distinct_tempos(mut tempos: Vec<f32>, tolerance: f32) -> usize {
    tempos.sort_by(f32::total_cmp);
    let mut clusters = 0;
    let mut start = f32::NEG_INFINITY;
    for tempo in tempos {
        if tempo - start > tolerance {
            clusters += 1;
            start = tempo;
        }
    }
    clusters
}

/// Un habillage tel que les snapshots du premier chorus le montrent.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Dressing {
    tempo_class: usize,
    key: u8,
    mode: String,
    chords: u8,
    bass: u8,
    kit: u8,
    voicing: String,
    register: String,
    energy: String,
    arc: String,
}

fn dressings(
    p: &StyleProfile,
    seeds: impl Iterator<Item = u64>,
) -> TestResult<Vec<(f32, Dressing)>> {
    seeds
        .map(|seed| {
            let flow = stream(p, seed, BLUES)?;
            let first = &flow[0];
            let position = first.position.ok_or("position")?;
            let instruments = first.instruments.ok_or("instruments publiés")?;
            Ok((
                first.tempo,
                Dressing {
                    tempo_class: 0,
                    key: position.key,
                    mode: format!("{:?}", position.mode),
                    chords: instruments.chords_program.ok_or("comping")?,
                    bass: instruments.bass_program.ok_or("basse")?,
                    kit: instruments.percussion_program,
                    voicing: drawn(first, "voicing"),
                    register: drawn(first, "registre"),
                    energy: drawn(first, "energie"),
                    arc: drawn(first, "arc"),
                },
            ))
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tirage par graine
// ---------------------------------------------------------------------------

#[test]
fn le_tirage_est_stable_par_graine_et_rejoue_apres_un_seek() -> TestResult {
    let p = fixture()?;
    for seed in SOME_SEEDS {
        let first = stream(&p, seed, BARS)?;
        let second = stream(&p, seed, BARS)?;
        assert_eq!(
            serde_json::to_string(&first)?,
            serde_json::to_string(&second)?,
            "graine {seed} : même sortie"
        );
        // Une variante par groupe, la même sur toutes les mesures de la session.
        let groups: BTreeSet<_> = first[0].session.iter().map(|v| v.group.clone()).collect();
        assert_eq!(groups.len(), 9, "un tirage par groupe : {:?}", first[0].session);
        for s in &first {
            assert_eq!(s.session, first[0].session, "graine {seed}, mesure {}", s.index);
        }
        // Seek : un générateur remis à zéro puis avancé en silence jusqu'à la mesure 31 rejoue
        // les mesures 31 à 48 comme le flux linéaire (variante tirée de nouveau, la même).
        let mut g = TuneGenerator::new(params(&p), TuningParams { profile: p.clone() });
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        for bar in 1..=BARS {
            let _ = g.generate_measure(bar, &mut rng);
        }
        g.reset_to_initial();
        assert!(g.session().is_none(), "le rejeu retire la variante tirée");
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        g.silent_advance(31, &mut rng);
        for bar in 31..=BARS {
            let replayed = MeasureSnapshot::from_measure(&g.generate_measure(bar, &mut rng));
            assert_eq!(
                serde_json::to_string(&replayed)?,
                serde_json::to_string(&first[bar - 1])?,
                "graine {seed}, mesure {bar} après seek"
            );
        }
    }
    Ok(())
}

#[test]
fn des_graines_differentes_donnent_des_habillages_differents() -> TestResult {
    let p = fixture()?;
    let all = dressings(&p, SEEDS)?;
    let tempos: Vec<f32> = all.iter().map(|(tempo, _)| *tempo).collect();
    let distinct: BTreeSet<_> = all.iter().map(|(_, d)| d.clone()).collect();
    let keys: BTreeSet<_> = all.iter().map(|(_, d)| (d.key, d.mode.clone())).collect();
    let chords: BTreeSet<_> = all.iter().map(|(_, d)| d.chords).collect();
    let bass: BTreeSet<_> = all.iter().map(|(_, d)| d.bass).collect();
    let kits: BTreeSet<_> = all.iter().map(|(_, d)| d.kit).collect();
    let voicings: BTreeSet<_> = all.iter().map(|(_, d)| d.voicing.clone()).collect();
    // ± 8 BPM : des grappes de 16 BPM de large au plus.
    let clusters = distinct_tempos(tempos.clone(), 16.0);
    println!(
        "16 graines : {} habillages distincts ; tempos à ±8 BPM {clusters} ({tempos:?}) ; \
         tonalités {} ; comping {chords:?} ; basse {bass:?} ; kit {kits:?} ; voicings {}",
        distinct.len(),
        keys.len(),
        voicings.len()
    );
    assert_eq!(clusters, 3, "trois tempos à ±8 BPM près : {tempos:?}");
    assert!(keys.len() >= 4, "tonalités : {keys:?}");
    assert_eq!(chords, BTreeSet::from([0, 16, 26]), "piano, orgue, guitare");
    assert_eq!(bass, BTreeSet::from([32, 33]), "contrebasse, basse électrique");
    assert_eq!(kits, BTreeSet::from([32, 40]), "baguettes, balais");
    assert_eq!(voicings.len(), 3, "trois familles de voicings : {voicings:?}");
    // Sur 3 456 habillages possibles, 16 graines en montrent au moins 15 distincts.
    assert!(distinct.len() >= 15, "{} habillages distincts sur 16 graines", distinct.len());
    Ok(())
}

#[test]
fn le_tempo_tire_vient_de_la_plage_de_la_variante() -> TestResult {
    let p = fixture()?;
    let ranges = [("lent", 100.0..=116.0), ("medium", 136.0..=152.0), ("vif", 176.0..=196.0)];
    for seed in SEEDS {
        let flow = stream(&p, seed, BLUES)?;
        let name = drawn(&flow[0], "tempo");
        let range = &ranges.iter().find(|(n, _)| *n == name).ok_or("variante de tempo")?.1;
        for s in &flow {
            assert!(range.contains(&s.tempo), "graine {seed} ({name}) : {} BPM", s.tempo);
            assert_eq!(s.tempo, flow[0].tempo, "le tempo d'une session ne bouge pas");
            assert_eq!(s.tempo.fract(), 0.0, "BPM entier tiré");
            // Le tempo de composition publié est celui de la variante, pas celui du profil.
        }
    }
    // La courbe de `dynamics` s'ajoute au tempo tiré : intensité 1,0 = +20 BPM.
    let base = stream(&p, 3, 1)?[0].tempo;
    let loud = stream_with(&p, MusicalParams { intensity: 1.0, ..params(&p) }, 3, 1)?[0].tempo;
    assert!((loud - base - 20.0).abs() < 1e-3, "{base} puis {loud}");
    Ok(())
}

#[test]
fn la_tonalite_tiree_transpose_la_grille_et_le_swing_suit_le_tempo() -> TestResult {
    let p = fixture()?;
    let keys: BTreeMap<&str, (u8, &str)> = BTreeMap::from([
        ("fa", (5, "Ionian")),
        ("si_bemol", (10, "Ionian")),
        ("mi_bemol", (3, "Ionian")),
        ("re_eolien", (2, "Aeolian")),
    ]);
    let swings = BTreeMap::from([("lent", 0.70), ("medium", 0.68), ("vif", 0.62)]);
    for seed in SEEDS {
        let flow = stream(&p, seed, BLUES)?;
        let (key, mode) = keys[drawn(&flow[0], "tonalite").as_str()];
        let position = flow[0].position.ok_or("position")?;
        assert_eq!((position.key, format!("{:?}", position.mode).as_str()), (key, mode));
        // Le blues commence sur I7 : fondamentale = tonique de la variante, à chaque chorus.
        for s in flow.iter().filter(|s| (s.index - 1) % BLUES == 0) {
            assert_eq!(s.chord_root_pc, key, "graine {seed}, mesure {}", s.index);
        }
        let swing = swings[drawn(&flow[0], "tempo").as_str()];
        assert!(flow.iter().all(|s| (s.swing_ratio - swing).abs() < 1e-6), "graine {seed}");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// L'hôte gagne
// ---------------------------------------------------------------------------

#[test]
fn la_commande_de_l_hote_gagne_sur_la_variante() -> TestResult {
    let p = fixture()?;
    for seed in SOME_SEEDS {
        // Tempo direct (`SetBpm`) : exactement la valeur de l'hôte, sur toutes les mesures.
        let flow = stream_with(
            &p,
            MusicalParams { bpm: 133.0, bpm_explicit: true, ..params(&p) },
            seed,
            24,
        )?;
        assert!(flow.iter().all(|s| s.tempo == 133.0), "graine {seed}");
        // Swing commandé (`SetSwing` : le compositeur de l'hôte règle le rapport et le marque
        // explicite) : celui de l'hôte, pas celui de la variante.
        let mut host = params(&p);
        host.set_swing_ratio(0.55);
        host.swing_explicit = true;
        let flow = stream_with(&p, host, seed, 12)?;
        assert!(flow.iter().all(|s| (s.swing_ratio - 0.55).abs() < 1e-6), "graine {seed}");
        // Le reste de la variante tient : les instruments sont ceux de la graine.
        let free = stream(&p, seed, 12)?;
        assert_eq!(flow[0].instruments, free[0].instruments);
        assert_eq!(flow[0].session, free[0].session);
        // La grille de l'hôte (`[fixed_tonality]`) gagne sur la tonalité de la variante.
        let locked = edited(&p, |v| {
            v.as_table_mut()
                .ok_or("profil")?
                .insert("fixed_tonality".into(), toml::from_str("key = 7\nmode = \"ionian\"")?);
            Ok(())
        })?;
        let flow = stream(&locked, seed, 12)?;
        let position = flow[0].position.ok_or("position")?;
        assert_eq!(position.key, 7, "graine {seed} : `fixed_tonality` gagne");
        assert!(flow.iter().step_by(BLUES).all(|s| s.chord_root_pc == 7));
        // La variante n'en reste pas moins tirée et visible.
        assert_eq!(flow[0].session.len(), 9);
        // Tempo de la variante quand l'hôte n'impose rien.
        assert_ne!(free[0].tempo, 133.0);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Orchestration par chorus
// ---------------------------------------------------------------------------

/// Graine dont l'arc tiré est « orchestre » (programmes qui changent par chorus), avec un comping
/// qui n'est pas déjà à l'orgue et un kit aux baguettes : chaque changement de l'arc s'entend.
fn orchestrated_seed(p: &StyleProfile) -> TestResult<u64> {
    for seed in 1..=256 {
        let first = &stream(p, seed, 1)?[0];
        if drawn(first, "arc") == "orchestre"
            && drawn(first, "comping") != "orgue"
            && drawn(first, "kit") == "baguettes"
        {
            return Ok(seed);
        }
    }
    Err("aucune graine n'a tiré l'arc « orchestre » avec baguettes et sans orgue".into())
}

#[test]
fn le_programme_change_a_la_frontiere_du_chorus_dans_les_snapshots() -> TestResult {
    let p = fixture()?;
    let seed = orchestrated_seed(&p)?;
    let flow = stream(&p, seed, BARS)?;
    let base = flow[0].instruments.ok_or("instruments publiés dès la première mesure")?;
    let session_kit = {
        // Le kit de la session : celui des chorus qui ne déclarent rien (chorus 2 et 3).
        let chorus2 = flow.iter().find(|s| chorus_of(s) == 2).ok_or("chorus 2")?;
        chorus2.instruments.ok_or("chorus 2")?.percussion_program
    };
    let session_chords = flow
        .iter()
        .find(|s| chorus_of(s) == 2)
        .and_then(|s| s.instruments)
        .and_then(|i| i.chords_program)
        .ok_or("comping du chorus 2")?;
    assert_eq!(base.percussion_program, 40, "chorus 1 aux balais");
    for s in &flow {
        let instruments = s.instruments.ok_or("instruments publiés sur chaque mesure")?;
        let (kit, chords) = match chorus_of(s) {
            1 | 4 => (40, session_chords),
            2 => (session_kit, session_chords),
            3 => (session_kit, 16),
            other => return Err(format!("chorus {other}").into()),
        };
        assert_eq!(instruments.percussion_program, kit, "mesure {}", s.index);
        assert_eq!(instruments.chords_program, Some(chords), "mesure {}", s.index);
        // La basse ne change jamais : celle de la session.
        assert_eq!(instruments.bass_program, base.bass_program, "mesure {}", s.index);
    }
    // Les changements tombent exactement sur la première mesure d'un chorus.
    let changes: Vec<usize> = flow
        .windows(2)
        .filter(|pair| pair[0].instruments != pair[1].instruments)
        .map(|pair| pair[1].index)
        .collect();
    assert!(
        changes.iter().all(|bar| (bar - 1) % BLUES == 0),
        "changements de programme hors frontière de chorus : {changes:?}"
    );
    assert!(changes.contains(&(3 * BLUES + 1)), "retombée aux balais : {changes:?}");
    Ok(())
}

/// Changements de programme du MIDI : `(tick, canal, programme)`.
fn program_changes(midi: &[u8]) -> TestResult<Vec<(u64, u8, u8)>> {
    let smf = Smf::parse(midi)?;
    let mut changes = Vec::new();
    for track in &smf.tracks {
        let mut tick = 0u64;
        for event in track {
            tick += u64::from(u32::from(event.delta));
            if let TrackEventKind::Midi {
                channel,
                message: MidiMessage::ProgramChange { program },
            } = event.kind
            {
                changes.push((tick, u8::from(channel), u8::from(program)));
            }
        }
    }
    changes.sort_unstable();
    Ok(changes)
}

#[test]
fn le_programme_change_a_la_frontiere_du_chorus_dans_le_midi_exporte() -> TestResult {
    let p = fixture()?;
    let seed = orchestrated_seed(&p)?;
    let measures = measures_with(&p, params(&p), seed, BARS)?;
    let flow: Vec<_> = measures.iter().map(MeasureSnapshot::from_measure).collect();
    let mut timeline = ScoreTimeline::new(BARS);
    for measure in measures {
        timeline.push_measure(measure);
    }
    let midi = timeline_to_midi(&timeline, &p)?;
    let changes = program_changes(&midi)?;
    let bar_tick = |bar: usize| ((bar - 1) * 16 * 120) as u64;
    let kit_changes: Vec<(u64, u8)> =
        changes.iter().filter(|(_, ch, _)| *ch == 9).map(|(t, _, prog)| (*t, *prog)).collect();
    let chords_changes: Vec<(u64, u8)> =
        changes.iter().filter(|(_, ch, _)| *ch == CHORDS).map(|(t, _, prog)| (*t, *prog)).collect();
    // Les changements de programme du MIDI sont ceux des snapshots, à la barre du chorus.
    let expected = |pick: &dyn Fn(&MeasureSnapshot) -> Option<u8>, initial: u8| {
        let mut current = initial;
        let mut out = Vec::new();
        for s in &flow {
            if let Some(program) = pick(s)
                && program != current
            {
                out.push((bar_tick(s.index), program));
                current = program;
            }
        }
        out
    };
    let kit_of = |s: &MeasureSnapshot| s.instruments.map(|i| i.percussion_program);
    let chords_of = |s: &MeasureSnapshot| s.instruments.and_then(|i| i.chords_program);
    assert_eq!(kit_changes, expected(&kit_of, p.instruments.percussion_program));
    let initial_chords =
        chords_changes.first().map(|(_, prog)| *prog).ok_or("programme initial")?;
    assert_eq!(initial_chords, 0, "tête de piste : programme du profil");
    assert_eq!(chords_changes[1..], expected(&chords_of, 0)[..]);
    assert!(
        chords_changes.iter().any(|(tick, prog)| *tick == bar_tick(2 * BLUES + 1) && *prog == 16),
        "l'orgue entre à la mesure 25 : {chords_changes:?}"
    );
    assert!(
        kit_changes.iter().all(|(tick, _)| *tick % bar_tick(BLUES + 1) == 0),
        "{kit_changes:?}"
    );
    Ok(())
}

#[test]
fn un_profil_sans_orchestration_ne_publie_ni_programmes_ni_variantes() -> TestResult {
    let p = StyleProfile::load(format!(
        "{}/../tests/fixtures/profiles/grille_phrases",
        env!("CARGO_MANIFEST_DIR")
    ))?;
    let flow = stream(&p, 1, 24)?;
    assert!(flow.iter().all(|s| s.instruments.is_none() && s.session.is_empty()));
    let json = serde_json::to_string(&flow)?;
    assert!(!json.contains("\"instruments\"") && !json.contains("\"session\""), "contrat inchangé");
    // Un arc nommé sans variante qui le choisit ne tire rien : le flux est celui du profil nu.
    let named_only =
        with_session(&p, "[arcs.x]\n[[arcs.x.stages]]\nvariants = [{ intensity = 1.0 }]\n")?;
    assert_eq!(
        serde_json::to_string(&stream(&named_only, 1, 24)?)?,
        json,
        "sans variante, le RNG n'est pas touché"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Surcharges, une à une
// ---------------------------------------------------------------------------

/// Profil dont la section `[session]` ne tire qu'une variante du groupe `group` (`body`).
fn one_variant(p: &StyleProfile, body: &str) -> TestResult<StyleProfile> {
    with_session(p, &format!("[[variants]]\ngroup = \"g\"\nweight = 1\n{body}\n"))
}

fn comping_notes(flow: &[MeasureSnapshot]) -> Vec<Vec<u8>> {
    flow.iter().map(|s| track(s, CHORDS).iter().map(|n| n.pitch).collect()).collect()
}

#[test]
fn la_famille_de_voicings_tiree_change_les_notes_du_comping() -> TestResult {
    let p = fixture()?;
    let mut by_family = BTreeMap::new();
    for family in ["rootless", "guide_tones", "drop2"] {
        let flow = stream(&one_variant(&p, &format!("voicing = \"{family}\""))?, 4, BLUES)?;
        let most = flow
            .iter()
            .flat_map(|s| {
                let mut per_attack: BTreeMap<usize, usize> = BTreeMap::new();
                for n in track(s, CHORDS) {
                    *per_attack.entry(n.start_step).or_default() += 1;
                }
                per_attack.into_values()
            })
            .max()
            .unwrap_or(0);
        by_family.insert(family, (comping_notes(&flow), most));
    }
    let notes: BTreeSet<_> = by_family.values().map(|(n, _)| n.clone()).collect();
    assert_eq!(notes.len(), 3, "trois familles, trois comping");
    assert!(
        by_family["guide_tones"].1 < by_family["rootless"].1,
        "notes guides : moins de voix que la famille sans fondamentale"
    );
    Ok(())
}

#[test]
fn le_registre_tire_decale_les_pistes_d_une_octave() -> TestResult {
    let p = fixture()?;
    let plain = stream(&one_variant(&p, "")?, 4, 24)?;
    let low = stream(&one_variant(&p, "register_octaves = { Chords = -1, Bass = 1 }")?, 4, 24)?;
    for (a, b) in plain.iter().zip(&low) {
        for (id, delta) in [(CHORDS, -12i16), (BASS, 12)] {
            let (notes_a, notes_b) = (track(a, id), track(b, id));
            assert_eq!(notes_a.len(), notes_b.len(), "mesure {}", a.index);
            for (x, y) in notes_a.iter().zip(&notes_b) {
                assert_eq!(i16::from(y.pitch), i16::from(x.pitch) + delta);
                assert_eq!(x.start_step, y.start_step);
            }
        }
        // Les autres pistes ne bougent pas.
        assert_eq!(
            a.notes.iter().filter(|n| n.track > BASS).count(),
            b.notes.iter().filter(|n| n.track > BASS).count()
        );
    }
    assert!(plain.iter().any(|s| !track(s, BASS).is_empty()));
    Ok(())
}

#[test]
fn les_tags_de_session_restreignent_le_comping_et_la_batterie() -> TestResult {
    let p = edited(&fixture()?, |v| {
        v.as_table_mut().ok_or("profil")?.remove("arc");
        Ok(())
    })?;
    let mean_attacks = |tag: &str| -> TestResult<f64> {
        let q = one_variant(&p, &format!("comping = [{{ tag = \"{tag}\" }}]"))?;
        let flow = stream(&q, 6, 36)?;
        let attacks: usize = flow
            .iter()
            .map(|s| track(s, CHORDS).iter().map(|n| n.start_step).collect::<BTreeSet<_>>().len())
            .sum();
        Ok(attacks as f64 / flow.len() as f64)
    };
    let (sparse, dense) = (mean_attacks("sparse")?, mean_attacks("dense")?);
    assert!(sparse <= 1.0, "clairsemé : au plus une attaque par mesure ({sparse})");
    assert!(dense >= 2.0, "dense : au moins deux attaques par mesure ({dense})");
    // Batterie : charleston seul (`hat`) contre ride.
    let ride = |tag: &str| -> TestResult<bool> {
        let q = one_variant(&p, &format!("percussion = [{{ tag = \"{tag}\" }}]"))?;
        Ok(stream(&q, 6, 24)?.iter().any(|s| s.notes.iter().any(|n| n.pitch == 51)))
    };
    assert!(ride("ride")?, "tag `ride` : le ride joue");
    assert!(!ride("hat")?, "tag `hat` : charleston seul");
    Ok(())
}

#[test]
fn l_arc_nomme_remplace_l_arc_du_profil() -> TestResult {
    let p = fixture()?;
    let named = "[arcs.deux_temps]\n[[arcs.deux_temps.stages]]\nvariants = [{ bass_feel = \"two_feel\" }]\n";
    let two_feel =
        with_session(&p, &format!("{named}[[variants]]\ngroup = \"g\"\narc = \"deux_temps\"\n"))?;
    let bass_per_bar = |q: &StyleProfile| -> TestResult<Vec<usize>> {
        Ok(stream(q, 3, BARS)?
            .iter()
            .filter(|s| chorus_of(s) == 3 && (s.index - 1) % BLUES < BLUES - 2)
            .map(|s| track(s, BASS).len())
            .collect())
    };
    // Chorus 3 du profil : marche (4 attaques) ; sous l'arc nommé : deux temps (2 attaques).
    let walking = bass_per_bar(&one_variant(&p, "")?)?;
    let two = bass_per_bar(&two_feel)?;
    assert!(walking.iter().all(|&n| n >= 4), "marche : {walking:?}");
    assert!(two.iter().all(|&n| n <= 3), "deux temps : {two:?}");
    Ok(())
}

#[test]
fn l_energie_de_base_change_les_velocites_sans_toucher_au_tempo() -> TestResult {
    let p = edited(&fixture()?, |v| {
        v.as_table_mut().ok_or("profil")?.remove("arc");
        Ok(())
    })?;
    let mean_velocity = |body: &str| -> TestResult<(f64, f32)> {
        let flow = stream(&one_variant(&p, body)?, 5, 24)?;
        let notes: Vec<_> = flow.iter().flat_map(|s| track(s, BASS)).collect();
        let sum: f64 = notes.iter().map(|n| f64::from(n.velocity)).sum();
        Ok((sum / notes.len() as f64, flow[0].tempo))
    };
    let (soft, soft_tempo) = mean_velocity("intensity = 0.7")?;
    let (loud, loud_tempo) = mean_velocity("intensity = 1.4")?;
    assert!(loud > soft + 3.0, "{soft} puis {loud}");
    assert_eq!(soft_tempo, loud_tempo, "l'intensité de base ne touche pas le tempo");
    Ok(())
}

#[test]
fn les_programmes_de_session_vont_du_profil_au_snapshot() -> TestResult {
    let p = fixture()?;
    let q = one_variant(
        &p,
        "instruments = { melody_program = 5, chords_program = 24, bass_program = 35, \
         percussion_bank = 128, percussion_program = 25 }",
    )?;
    let first = &stream(&q, 1, 1)?[0];
    let i = first.instruments.ok_or("instruments")?;
    assert_eq!(
        (
            i.melody_program,
            i.chords_program,
            i.bass_program,
            i.percussion_bank,
            i.percussion_program
        ),
        (5, Some(24), Some(35), 128, 25)
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Schéma
// ---------------------------------------------------------------------------

fn invalid(session: &str) -> TestResult<String> {
    match with_session(&fixture()?, session) {
        Ok(_) => Err(format!("accepté : {session}").into()),
        Err(error) => Ok(error.to_string()),
    }
}

#[test]
fn les_erreurs_de_schema_portent_le_chemin_du_champ() -> TestResult {
    let v = |body: &str| format!("[[variants]]\n{body}\n");
    let cases: Vec<(String, &str)> = vec![
        (v("weight = 0"), "session.variants[0].weight"),
        (v("group = \"\""), "session.variants[0].group"),
        (v("bpm = 50.0"), "session.variants[0].bpm"),
        (v("bpm = 400.0"), "session.variants[0].bpm"),
        (v("bpm = [140.0, 120.0]"), "session.variants[0].bpm"),
        (v("tonality = { key = 12, mode = \"ionian\" }"), "session.variants[0].tonality.key"),
        (v("tonality = { key = 0, mode = \"phrygian\" }"), "session.variants[0].tonality.mode"),
        (v("instruments = { chords_program = 128 }"), "session.variants[0].instruments.chords_program"),
        (
            v("instruments = { countermelody_program = 3 }"),
            "session.variants[0].instruments.countermelody_program",
        ),
        (
            v("instruments = { percussion_bank = 300 }"),
            "session.variants[0].instruments.percussion_bank",
        ),
        (v("swing = 0.9"), "session.variants[0].swing"),
        (v("swing = 0.4"), "session.variants[0].swing"),
        (v("register_octaves = { Chords = 2 }"), "session.variants[0].register_octaves.Chords"),
        (
            v("register_octaves = { PercussionLow = 1 }"),
            "session.variants[0].register_octaves.PercussionLow",
        ),
        (v("comping = [{ tag = \"inconnu\" }]"), "session.variants[0].comping[0].tag"),
        (v("percussion = [{ tag = \"inconnu\" }]"), "session.variants[0].percussion[0].tag"),
        (v("comping = [{ tag = \"dense\", weight = 0 }]"), "session.variants[0].comping[0].weight"),
        (v("arc = \"absent\""), "session.variants[0].arc"),
        (v("intensity = 2.5"), "session.variants[0].intensity"),
        (v("density = -0.1"), "session.variants[0].density"),
        (
            "[[variants]]\ngroup = \"g\"\nweight = 1\nodin = { Chords = { zones = [] } }\n".into(),
            "session.variants[0].odin.Chords.zones",
        ),
        (
            "[arcs.x]\n[[arcs.x.stages]]\nvariants = [{ bass_feel = \"two_feel\", intensity = 3.0 }]\n"
                .into(),
            "session.arcs.x.stages[0].variants[0].intensity",
        ),
        (
            "[arcs.x]\n[[arcs.x.stages]]\nvariants = [{ instruments = { bass_program = 200 } }]\n"
                .into(),
            "session.arcs.x.stages[0].variants[0].instruments.bass_program",
        ),
    ];
    for (session, field) in &cases {
        let message = invalid(session)?;
        assert!(message.contains(field), "{session:?} : {message}");
    }
    // Deux groupes qui règlent le même champ.
    let message = invalid(
        "[[variants]]\ngroup = \"a\"\nbpm = 120.0\n[[variants]]\ngroup = \"b\"\nbpm = 130.0\n",
    )?;
    assert!(message.contains("session.variants[1].bpm") && message.contains("« a »"), "{message}");
    let message = invalid(
        "[[variants]]\ngroup = \"a\"\ninstruments = { bass_program = 33 }\n\
         [[variants]]\ngroup = \"b\"\ninstruments = { bass_program = 34 }\n",
    )?;
    assert!(message.contains("session.variants[1].instruments.bass_program"), "{message}");
    // Deux variantes du même groupe peuvent régler le même champ ; deux groupes, des champs
    // différents d'un même sous-ensemble d'instruments.
    with_session(
        &fixture()?,
        "[[variants]]\ngroup = \"a\"\ninstruments = { bass_program = 33 }\n\
         [[variants]]\ngroup = \"a\"\ninstruments = { bass_program = 34 }\n\
         [[variants]]\ngroup = \"b\"\ninstruments = { chords_program = 4 }\n",
    )?;
    // Champ inconnu : refusé.
    let message = invalid("[[variants]]\ngroup = \"g\"\ntempo = 120.0\n")?;
    assert!(message.contains("tempo"), "{message}");
    Ok(())
}

#[test]
fn la_tonalite_d_une_variante_est_ignoree_sous_un_verrou_meme_inatteignable() -> TestResult {
    // Sous `[fixed_tonality]`, la variante n'est jamais jouée : son mode n'a pas à être atteignable.
    let locked = edited(&fixture()?, |v| {
        let table = v.as_table_mut().ok_or("profil")?;
        table.insert("fixed_tonality".into(), toml::from_str("key = 5\nmode = \"ionian\"")?);
        table.insert(
            "session".into(),
            toml::from_str(
                "[[variants]]\ngroup = \"g\"\ntonality = { key = 9, mode = \"phrygian\" }\n",
            )?,
        );
        Ok(())
    })?;
    let flow = stream(&locked, 1, 1)?;
    assert_eq!(flow[0].position.ok_or("position")?.key, 5);
    Ok(())
}

#[test]
fn la_section_session_survit_a_l_aller_retour_toml() -> TestResult {
    let p = fixture()?;
    assert_eq!(StyleProfile::from_toml(&p.to_toml()?)?, p);
    assert_eq!(p.session.variants.len(), 23);
    assert_eq!(p.session.arcs.len(), 1);
    Ok(())
}

#[test]
fn une_erreur_de_la_session_est_situee_dans_session_toml() -> TestResult {
    let files = |session: &str| -> Vec<(String, String)> {
        let dir =
            format!("{}/../tests/fixtures/profiles/grille_habillage", env!("CARGO_MANIFEST_DIR"));
        let mut out = Vec::new();
        for name in [
            "profile.toml",
            "cells.toml",
            "percussion.toml",
            "arrangement.toml",
            "dynamics.toml",
            "set.toml",
            "arc.toml",
        ] {
            out.push((
                name.to_owned(),
                std::fs::read_to_string(format!("{dir}/{name}")).unwrap_or_default(),
            ));
        }
        out.push(("session.toml".to_owned(), session.to_owned()));
        out
    };
    let error = StyleProfile::from_files(files("[[variants]]\ngroup = \"g\"\nweight = 0\n"))
        .err()
        .ok_or("accepté")?;
    assert_eq!(error.field, "session.variants[0].weight");
    assert_eq!(error.file.as_deref(), Some("session.toml"));
    assert_eq!(error.line, Some(3));
    Ok(())
}
