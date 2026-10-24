//! Contrat du flux `[ornaments]` : taux, durée porteuse minimale, tessiture, copies,
//! arousal, neutralité et validation. Tout passe par `generate_measure` (principe VI).
use harmonium_core::{
    StyleProfile, TuningParams,
    ornaments::{IntervalClass, OrnamentKind, OrnamentRule, PhraseEnd},
    params::MusicalParams,
    report::MeasureSnapshot,
    timeline::{Measure, TimelineNote, TrackId},
    trad::{ImposedRole, TradGenerator, dynamics::Curve},
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

const KINDS: [OrnamentKind; 8] = [
    OrnamentKind::GraceSingle,
    OrnamentKind::GraceMultiple,
    OrnamentKind::Roll,
    OrnamentKind::Turn,
    OrnamentKind::TripletRepeated,
    OrnamentKind::DoubleStop,
    OrnamentKind::Chord,
    OrnamentKind::Trill,
];

fn base() -> StyleProfile {
    let mut p = StyleProfile::default();
    p.performance = Default::default();
    p.ornaments = Default::default();
    p.accompaniment.as_mut().unwrap().bass.passing = None;
    for part in &mut p.form.parts {
        part.ornament_protected_bars = Some(vec![]);
    }
    p
}
fn rule(rate: f32) -> OrnamentRule {
    OrnamentRule { written_per_100_notes: rate, ..Default::default() }
}
fn with(p: &StyleProfile, kind: OrnamentKind, mut rule: OrnamentRule) -> StyleProfile {
    if matches!(kind, OrnamentKind::DoubleStop | OrnamentKind::Chord) && rule.strings.is_empty() {
        rule.strings = vec![55, 62, 69, 76];
        rule.max_interval = 24;
    }
    let mut p = p.clone();
    p.ornaments.types.insert(kind, rule);
    p
}
fn stream_with(p: &StyleProfile, seed: u64, bars: usize, arousal: f32) -> Vec<Measure> {
    p.validate().unwrap();
    let mut g = TradGenerator::new(
        MusicalParams { bpm: p.meter.default_bpm, arousal, ..Default::default() },
        TuningParams { profile: p.clone() },
    );
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (1..=bars).map(|b| g.generate_measure(b, &mut rng)).collect()
}
fn stream(p: &StyleProfile, seed: u64, bars: usize) -> Vec<Measure> {
    stream_with(p, seed, bars, 0.5)
}
fn melody(m: &Measure) -> &[TimelineNote] {
    m.notes_for_track(TrackId::Melody)
}
/// Notes publiées (sans identifiants ni axes émotionnels) : le contrat du flux.
fn json(measures: &[Measure]) -> serde_json::Value {
    let notes: Vec<_> = measures.iter().map(|m| MeasureSnapshot::from_measure(m).notes).collect();
    serde_json::to_value(notes).unwrap()
}
/// Mesures complètes, identifiants compris.
fn full(measures: &[Measure]) -> serde_json::Value {
    serde_json::to_value(measures).unwrap()
}
fn in_tour(m: &Measure) -> bool {
    m.position.is_some_and(|p| !p.is_intro())
}
fn onset(n: &TimelineNote) -> i64 {
    n.start_step as i64 * 1000 + i64::from(n.performance.offset)
}
fn end(n: &TimelineNote) -> i64 {
    onset(n) + n.performance.duration(n.duration_steps)
}

/// Notes en plus par mesure de tour et notes écrites, sur `seeds` airs.
fn extras(p: &StyleProfile, seeds: u64, arousal: f32) -> (usize, usize) {
    let plain = {
        let mut q = p.clone();
        q.ornaments.types.clear();
        q
    };
    let (mut extra, mut notes) = (0, 0);
    for seed in 0..seeds {
        let a = stream_with(&plain, seed, 32, arousal);
        let b = stream_with(p, seed, 32, arousal);
        for (x, y) in a.iter().zip(&b).filter(|(x, _)| in_tour(x)) {
            notes += melody(x).len();
            extra += melody(y).len() - melody(x).len();
        }
    }
    (extra, notes)
}

#[test]
fn inert_sections_leave_the_stream_identical_and_draw_nothing() {
    for profile in StyleProfile::distributed() {
        let mut absent = profile.clone();
        absent.ornaments = Default::default();
        let mut disabled = profile.clone();
        for rule in disabled.ornaments.types.values_mut() {
            rule.enabled = false;
        }
        let mut zero_scale = profile.clone();
        zero_scale.ornaments.rate_scale = 0.0;
        zero_scale.ornaments.types.remove(&OrnamentKind::Passing);
        for seed in [3, 11] {
            let reference = full(&stream(&absent, seed, 100));
            assert_eq!(reference, full(&stream(&disabled, seed, 100)), "{}", profile.meta.id);
            assert_eq!(reference, full(&stream(&zero_scale, seed, 100)), "{}", profile.meta.id);
        }
    }
}

#[test]
fn each_type_follows_its_configured_rate_per_100_notes() {
    let p = base();
    for (kind, per_extra) in [
        (OrnamentKind::GraceSingle, 1),
        (OrnamentKind::GraceMultiple, 2),
        (OrnamentKind::Roll, 4),
        (OrnamentKind::Turn, 3),
        (OrnamentKind::TripletRepeated, 2),
        (OrnamentKind::DoubleStop, 1),
        (OrnamentKind::Chord, 2),
    ] {
        let (extra, notes) = extras(&with(&p, kind, rule(6.0)), 200, 0.5);
        let rate = extra as f32 / per_extra as f32 / notes as f32 * 100.0;
        // Une triple corde n'existe que si deux cordes à vide voisines sont dans le mode.
        let low = if kind == OrnamentKind::Chord { 4.2 } else { 5.4 };
        assert!((low..=6.6).contains(&rate), "{kind:?}: {rate:.2} pour 100 notes (6 configurés)");
    }
    let (extra, _) = extras(&with(&p, OrnamentKind::Roll, rule(0.0)), 20, 0.5);
    assert_eq!(extra, 0);
}

#[test]
fn rate_scale_multiplies_every_rate() {
    let mut p = with(&base(), OrnamentKind::Roll, rule(2.0));
    let per_100 = |p: &StyleProfile| {
        let (extra, notes) = extras(p, 200, 0.5);
        extra as f32 / 4.0 / notes as f32 * 100.0
    };
    let single = per_100(&p);
    p.ornaments.rate_scale = 3.0;
    let triple = per_100(&p);
    assert!((1.8..=2.2).contains(&single), "{single} pour 100 notes (2 configurés)");
    assert!((5.4..=6.6).contains(&triple), "{triple} pour 100 notes (2 × 3 configurés)");
}

#[test]
fn per_bar_rates_target_the_bar_of_the_part() {
    let mut r = rule(0.0);
    r.per_100_notes_by_bar = vec![0.0, 0.0, 40.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let p = with(&base(), OrnamentKind::GraceSingle, r);
    let plain = with(&base(), OrnamentKind::GraceSingle, rule(0.0));
    let mut hits = [0usize; 8];
    for seed in 0..30 {
        let (a, b) = (stream(&plain, seed, 32), stream(&p, seed, 32));
        for (x, y) in a.iter().zip(&b).filter(|(x, _)| in_tour(x)) {
            let bar = x.position.unwrap().bar_in_strain;
            hits[bar - 1] += melody(y).len() - melody(x).len();
        }
    }
    assert!(hits[2] > 50, "{hits:?}");
    // Une mesure copie de la mesure 3 (CopyOf) en emprunte les ornements.
    assert!(hits[6] <= hits[2] && hits[2] + hits[6] == hits.iter().sum::<usize>(), "{hits:?}");
}

#[test]
fn no_ornament_below_the_minimum_carrier_duration_and_pitches_stay_in_range() {
    let params = StyleProfile::default().melody.params;
    for (kind, min_eighths) in [
        (OrnamentKind::Roll, 2.0),
        (OrnamentKind::GraceSingle, 3.0),
        (OrnamentKind::GraceMultiple, 2.0),
        (OrnamentKind::Turn, 2.0),
        (OrnamentKind::TripletRepeated, 2.0),
        (OrnamentKind::DoubleStop, 1.5),
        (OrnamentKind::Chord, 1.5),
        (OrnamentKind::Trill, 2.0),
    ] {
        let mut r = rule(30.0);
        r.min_carrier_eighths = min_eighths;
        let p = with(&base(), kind, r);
        let plain = with(&base(), kind, rule(0.0));
        let min_steps = (min_eighths * 2.0).ceil() as usize;
        let mut changed = 0;
        for seed in 0..20 {
            let (a, b) = (stream(&plain, seed, 32), stream(&p, seed, 32));
            for (x, y) in a.iter().zip(&b) {
                let written = melody(x);
                for n in melody(y) {
                    assert!(
                        (params.tessitura_min..=params.tessitura_max).contains(&n.pitch),
                        "{kind:?}"
                    );
                    if written.iter().any(|w| {
                        (w.pitch, w.start_step, w.duration_steps)
                            == (n.pitch, n.start_step, n.duration_steps)
                    }) {
                        continue;
                    }
                    changed += 1;
                    assert!(
                        written.iter().any(|w| w.duration_steps >= min_steps
                            && onset(n) >= onset(w) - 490
                            && onset(n) < end(w)),
                        "{kind:?} : note {n:?} hors d'une porteuse de {min_steps} pas"
                    );
                }
            }
        }
        assert!(changed > 30, "{kind:?} : {changed}");
    }
}

#[test]
fn ornament_notes_stay_on_the_published_grid_within_performance_bounds() {
    let mut p = base();
    for kind in KINDS {
        p = with(&p, kind, rule(10.0));
    }
    for seed in 0..20 {
        for m in stream(&p, seed, 32) {
            for n in melody(&m) {
                assert!(n.start_step < m.steps);
                assert!(n.performance.offset >= -490, "{n:?}");
                assert!(n.performance.duration(n.duration_steps) >= 1);
                assert!(end(n) <= 1000 * (m.steps as i64 + 16), "{n:?}");
            }
        }
    }
}

#[test]
fn graces_end_on_the_carrier_and_double_stops_share_its_timing() {
    let plain = base();
    let (a, b) = (
        stream(&plain, 5, 32),
        stream(&with(&base(), OrnamentKind::GraceSingle, rule(40.0)), 5, 32),
    );
    let mut graces = 0;
    for (x, y) in a.iter().zip(&b) {
        for g in melody(y).iter().filter(|n| n.performance.offset < 0) {
            graces += 1;
            let carrier =
                melody(x).iter().find(|w| w.start_step == g.start_step).expect("porteuse");
            assert_eq!(end(g), onset(carrier), "l'appogiature finit sur la porteuse");
            assert!(g.velocity < carrier.velocity);
            assert!(
                !melody(y).iter().any(|n| onset(n) < onset(g) && end(n) > onset(g)),
                "la note précédente est écourtée : pas de chevauchement avec l'appogiature"
            );
        }
    }
    assert!(graces > 10);

    let r = OrnamentRule { min_carrier_eighths: 1.0, ..rule(40.0) };
    let stops = stream(&with(&base(), OrnamentKind::DoubleStop, r), 5, 32);
    let mut doubled = 0;
    for (x, y) in a.iter().zip(&stops) {
        for n in melody(y) {
            if let Some(w) =
                melody(x).iter().find(|w| w.start_step == n.start_step && w.pitch != n.pitch)
            {
                doubled += 1;
                let carrier = melody(y)
                    .iter()
                    .find(|c| c.start_step == n.start_step && c.pitch == w.pitch)
                    .unwrap();
                assert_eq!(
                    (n.duration_steps, n.performance),
                    (carrier.duration_steps, carrier.performance)
                );
                assert!(
                    [55, 62, 69, 76].contains(&n.pitch) || n.pitch.abs_diff(carrier.pitch) <= 4
                );
                assert!((3..=12).contains(&n.pitch.abs_diff(carrier.pitch)));
            }
        }
    }
    assert!(doubled > 10);
}

#[test]
fn arousal_scales_the_rate_between_the_configured_multipliers() {
    let mut r = rule(3.0);
    r.arousal_low_multiplier = 0.2;
    r.arousal_high_multiplier = 3.0;
    let p = with(&base(), OrnamentKind::GraceSingle, r);
    let low = extras(&p, 200, 0.0).0;
    let mid = extras(&p, 200, 0.5).0;
    let high = extras(&p, 200, 1.0).0;
    assert!(low < mid && mid < high, "{low} {mid} {high}");
    let ratio = high as f32 / low as f32;
    assert!((11.0..=19.0).contains(&ratio), "{ratio} (15 attendu)");
    let neutral = with(&base(), OrnamentKind::GraceSingle, rule(3.0));
    assert_eq!(json(&stream_with(&neutral, 4, 32, 0.0)), json(&stream_with(&neutral, 4, 32, 1.0)));
}

#[test]
fn the_written_line_survives_every_type_except_the_turn_which_opens_on_a_neighbour() {
    let mut p = base();
    for kind in KINDS.into_iter().filter(|&k| k != OrnamentKind::Turn) {
        p = with(&p, kind, rule(40.0));
    }
    p.ornaments.types.insert(
        OrnamentKind::Passing,
        OrnamentRule {
            density_curve: Some(Curve(vec![[0.0, 1.0], [1.0, 1.0]])),
            ..Default::default()
        },
    );
    let plain = {
        let mut q = p.clone();
        q.ornaments.types.clear();
        q
    };
    let mut ornamented = 0;
    for seed in 0..20 {
        let (a, b) = (stream(&plain, seed, 32), stream(&p, seed, 32));
        for (x, y) in a.iter().zip(&b) {
            let after: Vec<_> = melody(y).iter().map(|n| (n.start_step, n.pitch)).collect();
            for n in melody(x) {
                assert!(after.contains(&(n.start_step, n.pitch)), "m.{} {n:?}", x.index);
            }
            ornamented += melody(y).len() - melody(x).len();
        }
    }
    assert!(ornamented > 300, "{ornamented}");
}

#[test]
fn same_seed_same_ornaments_and_other_seeds_differ() {
    let mut p = base();
    for kind in KINDS {
        p = with(&p, kind, rule(8.0));
    }
    assert_eq!(json(&stream(&p, 9, 64)), json(&stream(&p, 9, 64)));
    assert_ne!(json(&stream(&p, 9, 64)), json(&stream(&p, 10, 64)));
}

#[test]
fn copied_bars_carry_the_same_ornaments_unless_the_profile_varies_repeats() {
    let mut p = base();
    p.form.parts.truncate(1);
    p.form.sequence = vec![0, 0];
    p.form.parts[0].second_ending = None;
    p.form.parts[0].rhythm.clear();
    p.form.parts[0].roles = vec![ImposedRole::Open.into(); 8];
    p.form.parts[0].roles[3] = ImposedRole::CadHalf.into();
    p.form.parts[0].roles[4] = ImposedRole::CopyOf(1).into();
    p.form.parts[0].roles[7] = ImposedRole::CadTonic.into();
    let mut p = with(&p, OrnamentKind::GraceSingle, rule(60.0));
    p = with(&p, OrnamentKind::Roll, rule(60.0));
    let pitches = |m: &Measure| {
        melody(m)
            .iter()
            .map(|n| (n.pitch, n.start_step, n.duration_steps, n.performance))
            .collect::<Vec<_>>()
    };
    let mut compared = 0;
    for seed in 0..10 {
        let tour = stream(&p, seed, 40);
        let first = tour.iter().position(in_tour).unwrap();
        for bar in 0..8 {
            assert_eq!(
                pitches(&tour[first + bar]),
                pitches(&tour[first + bar + 8]),
                "seed {seed} m.{bar}"
            );
            compared += melody(&tour[first + bar]).len();
        }
        assert_eq!(pitches(&tour[first]), pitches(&tour[first + 4]), "CopyOf");
    }
    assert!(compared > 100);
    p.ornaments.repeat_variation = 1.0;
    let differs = (0..10).any(|seed| {
        let tour = stream(&p, seed, 40);
        let first = tour.iter().position(in_tour).unwrap();
        (0..8).any(|bar| pitches(&tour[first + bar]) != pitches(&tour[first + bar + 8]))
    });
    assert!(differs, "repeat_variation = 1 doit libérer les reprises");
}

#[test]
fn protected_bars_stay_plain_and_context_filters_apply() {
    // Mesure protégée (dernière de la partie par défaut, ou liste du profil) : aucun type.
    for (kind, protected) in [
        (OrnamentKind::Roll, None),
        (OrnamentKind::GraceSingle, None),
        (OrnamentKind::DoubleStop, Some(vec![2, 8])),
    ] {
        let mut p = with(&base(), kind, rule(100.0));
        p.form.parts.iter_mut().for_each(|part| part.ornament_protected_bars = protected.clone());
        let plain = with(&base(), kind, rule(0.0));
        let mut touched = 0;
        for seed in 0..10 {
            let (a, b) = (stream(&plain, seed, 32), stream(&p, seed, 32));
            for (x, y) in a.iter().zip(&b).filter(|(x, _)| in_tour(x)) {
                let pos = x.position.unwrap();
                let same = json(std::slice::from_ref(x)) == json(std::slice::from_ref(y));
                let guarded = protected
                    .as_ref()
                    .map_or(pos.bar_in_strain == p.form.parts[pos.part.0].roles.len(), |bars| {
                        bars.contains(&pos.bar_in_strain)
                    });
                assert!(
                    same || !guarded,
                    "{kind:?} m.{} ornée alors que protégée",
                    pos.bar_in_strain
                );
                touched += usize::from(!same);
            }
        }
        assert!(touched > 20, "{kind:?}");
    }
    let mut leaps = rule(100.0);
    leaps.intervals = vec![IntervalClass::Leap];
    leaps.phrase_end = PhraseEnd::Never;
    let plain = with(&base(), OrnamentKind::GraceSingle, rule(0.0));
    let q = with(&base(), OrnamentKind::GraceSingle, leaps);
    let (a, b) = (stream(&plain, 2, 32), stream(&q, 2, 32));
    let mut graces = 0;
    for (x, y) in a.iter().zip(&b) {
        let written = melody(x);
        for g in melody(y).iter().filter(|n| n.performance.offset < 0) {
            graces += 1;
            let index = written.iter().position(|w| w.start_step == g.start_step).unwrap();
            assert!(index > 0 && written[index].pitch.abs_diff(written[index - 1].pitch) > 2);
            let bar = x.position.unwrap().bar_in_strain;
            assert!(bar % 4 != 0, "phrase_end = never : m.{bar} est une fin de phrase");
        }
    }
    assert!(graces > 3);
}

#[test]
fn out_of_range_values_are_rejected_with_the_field_name() {
    type Mutation = fn(&mut StyleProfile);
    let bad: &[(&str, Mutation)] = &[
        ("ornaments.rate_scale", |p| p.ornaments.rate_scale = 11.0),
        ("ornaments.rate_scale", |p| p.ornaments.rate_scale = f32::NAN),
        ("ornaments.repeat_variation", |p| p.ornaments.repeat_variation = 1.5),
        ("ornaments.types.roll.written_per_100_notes", |p| {
            p.ornaments.types.insert(OrnamentKind::Roll, rule(101.0));
        }),
        ("ornaments.types.roll.per_100_notes_by_bar[1]", |p| {
            let mut r = rule(1.0);
            r.per_100_notes_by_bar = vec![0.5, -0.1];
            p.ornaments.types.insert(OrnamentKind::Roll, r);
        }),
        ("ornaments.types.roll.min_carrier_eighths", |p| {
            let mut r = rule(1.0);
            r.min_carrier_eighths = -1.0;
            p.ornaments.types.insert(OrnamentKind::Roll, r);
        }),
        ("ornaments.types.roll.arousal_high_multiplier", |p| {
            let mut r = rule(1.0);
            r.arousal_high_multiplier = 5.0;
            p.ornaments.types.insert(OrnamentKind::Roll, r);
        }),
        ("ornaments.types.roll.velocity_scale", |p| {
            let mut r = rule(1.0);
            r.velocity_scale = 0.0;
            p.ornaments.types.insert(OrnamentKind::Roll, r);
        }),
        ("ornaments.types.roll.span", |p| {
            let mut r = rule(1.0);
            r.span = Some(0.3);
            p.ornaments.types.insert(OrnamentKind::Roll, r);
        }),
        ("ornaments.types.grace_single.span", |p| {
            let mut r = rule(1.0);
            r.span = Some(0.9);
            p.ornaments.types.insert(OrnamentKind::GraceSingle, r);
        }),
        ("ornaments.types.grace_multiple.count", |p| {
            let mut r = rule(1.0);
            r.count = Some(9);
            p.ornaments.types.insert(OrnamentKind::GraceMultiple, r);
        }),
        ("ornaments.types.double_stop.strings", |p| {
            let mut r = rule(1.0);
            r.strings = vec![62, 55];
            p.ornaments.types.insert(OrnamentKind::DoubleStop, r);
        }),
        ("ornaments.types.roll.strings", |p| {
            p.ornaments.types.insert(OrnamentKind::Roll, rule(1.0));
            p.ornaments.types.get_mut(&OrnamentKind::Roll).unwrap().strings = vec![55, 62];
        }),
        ("ornaments.types.passing.written_per_100_notes", |p| {
            p.ornaments.types.insert(OrnamentKind::Passing, rule(1.0));
        }),
        ("ornaments.types.passing.density_curve", |p| {
            p.ornaments.types.insert(OrnamentKind::Passing, OrnamentRule::default());
        }),
    ];
    for &(field, mutate) in bad {
        let mut p = base();
        mutate(&mut p);
        let error = p.validate().unwrap_err();
        assert_eq!(error.field, field, "{error}");
        assert!(!error.message.is_empty());
    }
    assert!(StyleProfile::from_toml("[ornaments]\nrate_scale = 2.0\nsurprise = 1\n").is_err());
}

#[test]
fn distributed_profiles_round_trip_and_carry_their_corpus_rates() {
    let mut with_types = 0;
    for profile in StyleProfile::distributed() {
        let decoded = StyleProfile::from_toml(&profile.to_toml().unwrap()).unwrap();
        assert_eq!(decoded.ornaments, profile.ornaments, "{}", profile.meta.id);
        profile.validate().unwrap();
        with_types += usize::from(!profile.ornaments.types.is_empty());
    }
    assert_eq!(with_types, 7);
}
