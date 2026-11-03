//! E10 : le filtre de difficulté de la mélodie (`[melody.difficulty]`), observé dans le flux de
//! `MeasureSnapshot` (principe VI) : plus grand saut, attaques par mesure, plus petite durée,
//! ambitus d'une mesure, gamme, ornements et ajustement aux accords.
use harmonium_core::{
    StyleProfile, TuningParams,
    compose::{IntroConfig, IntroForm, IntroRole, MelodyDifficulty, TuneGenerator},
    params::MusicalParams,
    report::{MeasureSnapshot, NoteSnapshot},
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const SEEDS: [u64; 3] = [42, 137, 2026];
const BARS: usize = 64;
const MELODY: u8 = 0;

fn distributed(id: &str) -> StyleProfile {
    let p = StyleProfile::distributed().iter().find(|p| p.meta.id == id).cloned();
    assert!(p.is_some(), "{id} est un profil distribué");
    p.unwrap_or_default()
}

/// Profils distribués dont la mélodie joue.
fn with_melody() -> Vec<StyleProfile> {
    StyleProfile::distributed()
        .iter()
        .filter(|p| p.arrangement.has_track(harmonium_core::timeline::TrackId::Melody))
        .cloned()
        .collect()
}

fn easy() -> MelodyDifficulty {
    MelodyDifficulty {
        max_leap_semitones: Some(4),
        max_attacks_per_bar: Some(4),
        min_note_units: Some(2),
        max_cell_ambitus_semitones: Some(7),
        out_of_scale: false,
        ornaments: false,
        chord_fit: false,
    }
}

fn limited(mut p: StyleProfile, limits: MelodyDifficulty) -> StyleProfile {
    p.melody.difficulty = Some(limits);
    assert!(p.validate().is_ok(), "profil valide");
    p
}

fn generator(p: &StyleProfile) -> TuneGenerator {
    TuneGenerator::new(
        MusicalParams { bpm: p.meter.default_bpm, ..Default::default() },
        TuningParams { profile: p.clone() },
    )
}

fn stream(p: &StyleProfile, seed: u64, bars: usize) -> Vec<MeasureSnapshot> {
    let mut g = generator(p);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (1..=bars).map(|b| MeasureSnapshot::from_measure(&g.generate_measure(b, &mut rng))).collect()
}

fn melody(s: &MeasureSnapshot) -> Vec<&NoteSnapshot> {
    s.notes.iter().filter(|n| n.track == MELODY).collect()
}

fn pitches(flow: &[MeasureSnapshot]) -> Vec<u8> {
    flow.iter().flat_map(|s| melody(s)).map(|n| n.pitch).collect()
}

fn largest_leap(flow: &[MeasureSnapshot]) -> u8 {
    pitches(flow).windows(2).map(|w| w[0].abs_diff(w[1])).max().unwrap_or(0)
}

/// Les bornes de `limits` tiennent sur tout le flux de `p`.
fn assert_within(
    p: &StyleProfile,
    limits: &MelodyDifficulty,
    flow: &[MeasureSnapshot],
    what: &str,
) {
    let unit = p.meter.steps_per_unit;
    if let Some(leap) = limits.max_leap_semitones {
        assert!(largest_leap(flow) <= leap, "{what} : saut {} > {leap}", largest_leap(flow));
    }
    for s in flow {
        let notes = melody(s);
        if let Some(attacks) = limits.max_attacks_per_bar {
            assert!(notes.len() <= attacks, "{what} mesure {} : {} attaques", s.index, notes.len());
        }
        if let Some(min) = limits.min_note_units {
            for n in &notes {
                assert!(
                    n.duration_steps >= min * unit,
                    "{what} mesure {} : note de {} pas",
                    s.index,
                    n.duration_steps
                );
            }
        }
        if let (Some(ambitus), Some(low), Some(high)) = (
            limits.max_cell_ambitus_semitones,
            notes.iter().map(|n| n.pitch).min(),
            notes.iter().map(|n| n.pitch).max(),
        ) {
            assert!(high - low <= ambitus, "{what} mesure {} : ambitus {}", s.index, high - low);
        }
    }
}

#[test]
fn les_bornes_tiennent_sur_les_profils_distribues_qui_ont_une_melodie() {
    let limits = easy();
    let profiles = with_melody();
    assert_eq!(profiles.len(), 15);
    for p in profiles {
        let p = limited(p, limits.clone());
        for seed in SEEDS {
            let flow = stream(&p, seed, BARS);
            assert!(pitches(&flow).len() > 8, "{} : une mélodie est jouée", p.meta.id);
            assert_within(&p, &limits, &flow, &format!("{} graine {seed}", p.meta.id));
        }
    }
}

#[test]
fn le_niveau_facile_est_nettement_plus_calme_que_le_profil() {
    for id in ["reel_quebecois", "valse_quebecoise", "isaac_basement"] {
        let open = distributed(id);
        let closed = limited(open.clone(), easy());
        for seed in SEEDS {
            let (wide, narrow) = (stream(&open, seed, BARS), stream(&closed, seed, BARS));
            assert!(largest_leap(&wide) > largest_leap(&narrow) + 2, "{id} graine {seed} : saut");
            if id != "isaac_basement" {
                // Une mélodie déjà clairsemée n'a pas de croches à fusionner.
                assert!(
                    pitches(&wide).len() > pitches(&narrow).len(),
                    "{id} graine {seed} : densité"
                );
            }
        }
    }
}

#[test]
fn les_bornes_tiennent_d_un_tour_a_l_autre_et_a_l_entree_par_la_fin() {
    // Intro « par la fin » : les deux dernières mesures du premier tour jouent avant la mesure 1
    // de l'air ; la ligne se raccorde aussi d'un tour au suivant (32 mesures, mesure 33).
    let mut p = distributed("reel_quebecois");
    p.arrangement.intro = IntroConfig {
        forms: vec![IntroForm { role: IntroRole::Tail, weight: 1, bars: 2 }],
        pickup_eighths: 0,
    };
    let limits = easy();
    let p = limited(p, limits.clone());
    for seed in SEEDS {
        let flow = stream(&p, seed, 100);
        let first = flow.iter().position(|s| !s.position.is_some_and(|pos| pos.is_intro()));
        assert_eq!(first, Some(2), "deux mesures d'intro");
        assert!(!melody(&flow[0]).is_empty(), "l'intro joue la fin de l'air");
        assert_within(&p, &limits, &flow[..96 + 2], &format!("graine {seed}"));
    }
}

#[test]
fn la_levee_d_intro_est_retiree_sous_un_filtre() {
    // Intro réduite à la levée : sans filtre, la mesure 1 est la mesure de levée ; avec, l'air
    // commence à la mesure 1.
    let mut open = distributed("reel_quebecois");
    open.arrangement.intro = IntroConfig { forms: Vec::new(), pickup_eighths: 2 };
    let closed = limited(
        open.clone(),
        MelodyDifficulty { max_leap_semitones: Some(7), ..Default::default() },
    );
    for seed in SEEDS {
        let first = stream(&open, seed, 2);
        assert!(first[0].position.is_some_and(|pos| pos.is_intro()), "levée sans filtre");
        assert!(!melody(&first[0]).is_empty());
        let first = stream(&closed, seed, 2);
        assert!(first[0].position.is_some_and(|pos| !pos.is_intro()), "pas de levée sous filtre");
    }
}

#[test]
fn une_mesure_et_une_graine_se_rejouent_a_l_identique() -> TestResult {
    let p = limited(distributed("valse_quebecoise"), easy());
    let flow = stream(&p, 9, BARS);
    assert_eq!(serde_json::to_string(&flow)?, serde_json::to_string(&stream(&p, 9, BARS))?);
    assert_ne!(
        serde_json::to_string(&flow)?,
        serde_json::to_string(&stream(&p, 10, BARS))?,
        "la graine fait la mélodie"
    );
    for start in [3, 20, 35, 50] {
        let mut g = generator(&p);
        let mut rng = ChaCha8Rng::seed_from_u64(9);
        g.silent_advance(start, &mut rng);
        for bar in start..=BARS {
            let again = MeasureSnapshot::from_measure(&g.generate_measure(bar, &mut rng));
            assert_eq!(
                serde_json::to_string(&again)?,
                serde_json::to_string(&flow[bar - 1])?,
                "mesure {bar} après un seek à {start}"
            );
        }
    }
    Ok(())
}

#[test]
fn les_ornements_et_l_ajustement_aux_accords_ne_s_ajoutent_que_si_le_profil_le_dit() {
    let limits = MelodyDifficulty {
        max_leap_semitones: Some(5),
        max_attacks_per_bar: Some(4),
        min_note_units: Some(2),
        ..easy()
    };
    // Sans ornement : la ligne écrite. Avec : les agréments du profil dépassent les bornes.
    let reel = distributed("reel_quebecois");
    let plain = limited(reel.clone(), limits.clone());
    let ornate = limited(reel, MelodyDifficulty { ornaments: true, ..limits.clone() });
    assert_within(&plain, &limits, &stream(&plain, 42, BARS), "sans ornement");
    let flow = stream(&ornate, 42, BARS);
    assert!(
        flow.iter().any(|s| melody(s).len() > 4),
        "les ornements du profil s'ajoutent à la ligne écrite"
    );
    // Boucle d'accords : sans ajustement, la mélodie reste celle des cellules.
    let game = distributed("isaac_basement");
    let written = limited(game.clone(), limits.clone());
    let fitted = limited(game, MelodyDifficulty { chord_fit: true, ..limits.clone() });
    assert_within(&written, &limits, &stream(&written, 42, BARS), "sans ajustement");
    assert_ne!(
        pitches(&stream(&fitted, 42, BARS)),
        pitches(&stream(&written, 42, BARS)),
        "l'ajustement aux accords déplace des notes"
    );
}

#[test]
fn sans_alteration_la_melodie_reste_dans_la_gamme_de_l_air() {
    let limits = MelodyDifficulty { out_of_scale: false, ..MelodyDifficulty::default() };
    for id in ["isaac_basement", "isaac_downpour", "nt_desert"] {
        let p = limited(distributed(id), limits.clone());
        for seed in SEEDS {
            for s in stream(&p, seed, BARS) {
                let Some(position) = s.position else { continue };
                for n in melody(&s) {
                    assert!(
                        position.mode.contains_midi(position.key, n.pitch),
                        "{id} graine {seed} mesure {} : {} hors gamme",
                        s.index,
                        n.pitch
                    );
                }
            }
        }
    }
}

#[test]
fn un_filtre_vide_ne_borne_rien_et_les_bornes_invalides_sont_refusees() -> TestResult {
    let p = distributed("reel_quebecois");
    assert!(p.melody.difficulty.is_none() && !p.to_toml()?.contains("difficulty"));
    let round = StyleProfile::from_toml(&limited(p.clone(), easy()).to_toml()?)?;
    assert_eq!(round.melody.difficulty, Some(easy()));
    for (name, limits) in [
        ("max_leap_semitones", MelodyDifficulty { max_leap_semitones: Some(0), ..easy() }),
        ("max_leap_semitones", MelodyDifficulty { max_leap_semitones: Some(25), ..easy() }),
        ("max_attacks_per_bar", MelodyDifficulty { max_attacks_per_bar: Some(0), ..easy() }),
        ("max_attacks_per_bar", MelodyDifficulty { max_attacks_per_bar: Some(9), ..easy() }),
        ("min_note_units", MelodyDifficulty { min_note_units: Some(9), ..easy() }),
        (
            "max_cell_ambitus_semitones",
            MelodyDifficulty { max_cell_ambitus_semitones: Some(0), ..easy() },
        ),
    ] {
        let mut bad = p.clone();
        bad.melody.difficulty = Some(limits);
        let error = bad.validate().err().ok_or(format!("{name} aurait dû être refusé"))?;
        assert_eq!(error.field, format!("melody.difficulty.{name}"));
    }
    Ok(())
}
