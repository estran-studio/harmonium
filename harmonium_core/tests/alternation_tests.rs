//! E11 : l'alternance de mesures jouées et muettes d'une piste (`[alternation]` de
//! `[arrangement]`), observée dans le flux de `MeasureSnapshot` (principe VI).
//!
//! Le profil de base est `reel_britannique_coloniale` (mélodie dès la mesure 1, sans seconde voix),
//! sans intro : la mesure `n` du flux est la mesure `n - 1` (0-based) de la forme.
use harmonium_core::{
    StyleProfile, TuningParams,
    compose::{IntroConfig, TuneGenerator},
    params::MusicalParams,
    report::MeasureSnapshot,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const MELODY: u8 = 0;
const BARS: usize = 64;

fn base() -> StyleProfile {
    let p = StyleProfile::distributed().iter().find(|p| p.meta.id == "reel_britannique_coloniale");
    assert!(p.is_some(), "profil distribué");
    let mut p = p.cloned().unwrap_or_default();
    p.arrangement.intro = IntroConfig::none();
    p
}

/// Même profil, `[alternation]` lu dans `toml` (table des pistes).
fn alternating(toml: &str) -> TestResult<StyleProfile> {
    let table: toml::Table = toml::from_str(toml)?;
    let mut value: toml::Value = toml::from_str(&base().to_toml()?)?;
    value["arrangement"]
        .as_table_mut()
        .ok_or("arrangement")?
        .insert("alternation".into(), toml::Value::Table(table));
    Ok(StyleProfile::from_toml(&toml::to_string(&value)?)?)
}

fn params(p: &StyleProfile) -> MusicalParams {
    MusicalParams {
        bpm: p.meter.default_bpm,
        time_signature: p.meter.time_signature,
        swing_ratio: p.meter.default_swing_ratio,
        ..Default::default()
    }
}

fn generator(p: &StyleProfile) -> TuneGenerator {
    TuneGenerator::new(params(p), TuningParams { profile: p.clone() })
}

fn stream(p: &StyleProfile, seed: u64) -> Vec<MeasureSnapshot> {
    let mut g = generator(p);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (1..=BARS).map(|b| MeasureSnapshot::from_measure(&g.generate_measure(b, &mut rng))).collect()
}

fn melody(s: &MeasureSnapshot) -> Vec<(u8, usize, usize)> {
    s.notes
        .iter()
        .filter(|n| n.track == MELODY)
        .map(|n| (n.pitch, n.start_step, n.duration_steps))
        .collect()
}

/// Notes des autres pistes, dans l'ordre publié.
fn others(s: &MeasureSnapshot) -> Vec<(u8, u8, usize, usize, u8)> {
    s.notes
        .iter()
        .filter(|n| n.track != MELODY)
        .map(|n| (n.track, n.pitch, n.start_step, n.duration_steps, n.velocity))
        .collect()
}

#[test]
fn la_melodie_joue_deux_mesures_sur_quatre_et_les_autres_pistes_ne_changent_pas() -> TestResult {
    let p = alternating("Melody = { period = 4 }")?;
    for seed in [1, 2, 42] {
        let plain = stream(&base(), seed);
        let flow = stream(&p, seed);
        for (s, reference) in flow.iter().zip(&plain) {
            let played = (s.index - 1) % 4 < 2;
            assert_eq!(melody(s).is_empty(), !played, "graine {seed}, mesure {}", s.index);
            if played {
                assert_eq!(melody(s), melody(reference), "graine {seed}, mesure {}", s.index);
            }
            assert_eq!(others(s), others(reference), "graine {seed}, mesure {}", s.index);
        }
        // Mesures 1-2, 5-6… jouées, 3-4, 7-8… muettes, sur 64 mesures.
        let heard: Vec<usize> =
            flow.iter().filter(|s| !melody(s).is_empty()).map(|s| s.index).collect();
        assert_eq!(heard.len(), BARS / 2);
        assert_eq!(&heard[..6], &[1, 2, 5, 6, 9, 10]);
    }
    Ok(())
}

#[test]
fn phase_et_mesures_jouees_decalent_le_cycle_et_se_comptent_en_continu_d_un_tour_a_l_autre()
-> TestResult {
    // Cycle de 6 mesures dont 1 jouée, commencé en silence : la forme dure 32 mesures, qui n'est
    // pas un multiple de 6 : le compte continue au tour 2 (mesure 33) au lieu de repartir.
    let p = alternating("Melody = { period = 6, play = 1, phase = 1 }")?;
    for s in &stream(&p, 7) {
        let played = (s.index - 1 + 1) % 6 < 1;
        assert_eq!(!melody(s).is_empty(), played, "mesure {}", s.index);
    }
    Ok(())
}

#[test]
fn une_mesure_se_rejoue_a_l_identique_apres_un_seek_et_la_graine_la_fixe() -> TestResult {
    let p = alternating("Melody = { period = 4, phase = 2 }")?;
    let flow = stream(&p, 9);
    assert_eq!(serde_json::to_string(&flow)?, serde_json::to_string(&stream(&p, 9))?);
    for start in [3, 4, 21, 36, 50] {
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
fn une_alternance_vide_ou_absente_laisse_le_flux_intact() -> TestResult {
    let p = alternating("")?;
    assert_eq!(serde_json::to_string(&stream(&p, 3))?, serde_json::to_string(&stream(&base(), 3))?);
    assert!(!p.to_toml()?.contains("alternation"), "une table vide ne se sérialise pas");
    Ok(())
}

#[test]
fn le_plafond_enable_melody_et_l_alternance_s_ajoutent() -> TestResult {
    let p = alternating("Melody = { period = 2 }")?;
    let mut g = TuneGenerator::new(
        MusicalParams { enable_melody: false, ..params(&p) },
        TuningParams { profile: p.clone() },
    );
    let mut rng = ChaCha8Rng::seed_from_u64(1);
    for bar in 1..=8 {
        let s = MeasureSnapshot::from_measure(&g.generate_measure(bar, &mut rng));
        assert!(melody(&s).is_empty(), "mesure {bar}");
    }
    Ok(())
}

#[test]
fn les_alternances_invalides_sont_refusees_avec_leur_chemin() -> TestResult {
    for (toml, field) in [
        ("Melody = { period = 1 }", "arrangement.alternation.Melody.period"),
        ("Melody = { period = 33 }", "arrangement.alternation.Melody.period"),
        ("Melody = { period = 4, play = 4 }", "arrangement.alternation.Melody.play"),
        ("Melody = { period = 4, play = 0 }", "arrangement.alternation.Melody.play"),
        ("Melody = { period = 4, phase = 4 }", "arrangement.alternation.Melody.phase"),
        ("Countermelody = { period = 4 }", "arrangement.alternation.Countermelody"),
    ] {
        let error = alternating(toml).err().ok_or(format!("{toml} aurait dû être refusé"))?;
        assert!(error.to_string().contains(field), "{toml} : {error}");
    }
    let unknown = toml::from_str::<harmonium_core::compose::Arrangement>(
        "[entries]\nMelody = { tour = 1, part = 0, pass = 1 }\n\
         PercussionLow = { tour = 1, part = 0, pass = 1 }\n\
         PercussionHigh = { tour = 1, part = 0, pass = 1 }\n\
         [alternation]\nViolon = { period = 4 }\n",
    );
    assert!(unknown.is_err_and(|e| e.to_string().contains("piste inconnue")));
    Ok(())
}
