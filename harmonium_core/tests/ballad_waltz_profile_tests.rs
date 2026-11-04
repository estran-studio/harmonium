//! B2/B5 : métrique, grille publiée, arrivées de basse et couleurs du comping sous habillage.
use std::collections::BTreeSet;

use harmonium_core::{
    StyleProfile, TuningParams, compose::TuneGenerator, params::MusicalParams,
    report::MeasureSnapshot, timeline::ChordContext,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

type Resultat<T = ()> = Result<T, Box<dyn std::error::Error>>;
const PROFILS: [(&str, usize, usize, f32, f32); 2] =
    [("wjazzd_ballad", 4, 3, 44.0, 84.0), ("gmd_jazz_waltz", 3, 4, 100.0, 230.0)];

fn flux(id: &str, graine: u64, mesures: usize) -> Resultat<Vec<MeasureSnapshot>> {
    let p = StyleProfile::named(id)?;
    let params = MusicalParams {
        bpm: p.meter.default_bpm,
        time_signature: p.meter.time_signature,
        swing_ratio: p.meter.default_swing_ratio,
        harmony_valence: p.dynamics.neutral_valence,
        intensity: 0.5,
        percussion_density: 1,
        ..Default::default()
    };
    let mut generateur = TuneGenerator::new(params, TuningParams { profile: p });
    let mut rng = ChaCha8Rng::seed_from_u64(graine);
    Ok((1..=mesures)
        .map(|bar| MeasureSnapshot::from_measure(&generateur.generate_measure(bar, &mut rng)))
        .collect())
}

fn accord(s: &MeasureSnapshot, pas: usize) -> &ChordContext {
    &s.chord_changes[s.chord_changes.partition_point(|c| c.start_step <= pas) - 1].context
}

/// Notes publiées et tensions naturelles non altérées, jamais un demi-ton au-dessus d'une note d'accord.
fn permis(c: &ChordContext) -> BTreeSet<u8> {
    let mut pcs: BTreeSet<_> = c.pitch_classes.iter().copied().collect();
    let relatif: BTreeSet<_> = pcs.iter().map(|pc| (pc + 12 - c.root_pc) % 12).collect();
    for (tension, alterations) in [(2, &[1, 2, 3][..]), (5, &[5, 6][..]), (9, &[8, 9][..])] {
        if !alterations.iter().any(|t| relatif.contains(t))
            && !relatif.contains(&((tension + 11) % 12))
        {
            pcs.insert((c.root_pc + tension) % 12);
        }
    }
    pcs
}

#[test]
fn la_metrique_le_tempo_et_la_forme_aaba_restent_coherents() -> Resultat {
    for (id, temps, chorus, min, max) in PROFILS {
        for graine in 1..=16 {
            let flow = flux(id, graine, 32 * chorus)?;
            for (i, s) in flow.iter().enumerate() {
                assert_eq!(
                    (s.time_sig_numerator, s.time_sig_denominator, s.steps),
                    (temps, 4, temps * 4)
                );
                assert!((min..=max).contains(&s.tempo), "{id} : {}", s.tempo);
                assert!((0.5..=0.75).contains(&s.swing_ratio));
                assert!(s.notes.iter().all(|n| n.track != 0 && n.start_step < s.steps));
                assert_eq!(s.chord_changes.first().map(|c| c.start_step), Some(0));
                assert!(s.chord_changes.iter().all(|c| c.start_step % 4 == 0));
                // Même grille AABA au chorus suivant ; la dernière mesure résout la tonique.
                if i >= 32 && i + 1 < flow.len() {
                    assert_eq!(
                        serde_json::to_value(&s.chord_changes)?,
                        serde_json::to_value(&flow[i % 32].chord_changes)?
                    );
                }
            }
        }
    }
    Ok(())
}

#[test]
fn la_basse_arrive_dans_l_accord_a_chaque_changement() -> Resultat {
    for (id, _, chorus, _, _) in PROFILS {
        for graine in 1..=16 {
            let flow = flux(id, graine, 32 * chorus)?;
            for (i, s) in flow.iter().enumerate() {
                let basse: Vec<_> = s.notes.iter().filter(|n| n.track == 2).collect();
                // Les breaks retirent toute la piste ; les deux dernières mesures sont en stop-time sur le un.
                if basse.is_empty() || i + 2 >= flow.len() {
                    continue;
                }
                for c in &s.chord_changes {
                    let n =
                        basse.iter().find(|n| n.start_step == c.start_step).ok_or_else(|| {
                            format!("{id} graine {graine} mesure {} : arrivée absente", s.index)
                        })?;
                    assert!(c.context.pitch_classes.contains(&(n.pitch % 12)));
                    assert!((31..=51).contains(&n.pitch), "{id} basse {}", n.pitch);
                }
            }
        }
    }
    Ok(())
}

#[test]
fn le_comping_ne_contredit_pas_l_accord_publie_ni_ses_alterations() -> Resultat {
    for (id, _, chorus, _, _) in PROFILS {
        for graine in 1..=16 {
            let flow = flux(id, graine, 32 * chorus)?;
            for (i, s) in flow.iter().enumerate() {
                for n in s.notes.iter().filter(|n| n.track == 1) {
                    let courant = permis(accord(s, n.start_step));
                    // Les anticipations de fin de mesure jouent l'accord publié de la suivante.
                    let anticipe = n.start_step >= s.steps - 2
                        && flow
                            .get(i + 1)
                            .is_some_and(|next| permis(accord(next, 0)).contains(&(n.pitch % 12)));
                    assert!(
                        courant.contains(&(n.pitch % 12)) || anticipe,
                        "{id} graine {graine} mesure {} pas {} : {} hors de {:?}",
                        s.index,
                        n.start_step,
                        n.pitch,
                        courant
                    );
                }
            }
        }
    }
    Ok(())
}

#[test]
fn les_sessions_varient_et_la_basse_change_de_feel_sur_l_arc() -> Resultat {
    for (id, temps, chorus, _, _) in PROFILS {
        let mut habillages = BTreeSet::new();
        let mut marche = false;
        for graine in 1..=16 {
            let flow = flux(id, graine, 32 * chorus)?;
            let first = &flow[0];
            let voicing =
                first.session.iter().find(|v| v.group == "voicing").ok_or("voicing absent")?;
            habillages.insert(format!(
                "{}:{}:{:?}:{}",
                (first.tempo / 16.0).round(),
                first.chord_root_pc,
                first.instruments,
                voicing.name
            ));
            let choix = serde_json::to_value(&first.session)?;
            for s in &flow {
                assert_eq!(serde_json::to_value(&s.session)?, choix);
            }
            // Deux temps : la blanche du un, puis le trois ; marche : une noire par pulsation.
            let bass_steps = |s: &MeasureSnapshot| {
                s.notes
                    .iter()
                    .filter(|n| n.track == 2)
                    .map(|n| n.start_step)
                    .collect::<BTreeSet<_>>()
            };
            assert_eq!(bass_steps(first), BTreeSet::from([0, 8]));
            marche |= flow[32..64].iter().any(|s| bass_steps(s).len() >= temps);
        }
        assert!(habillages.len() >= 8, "{id} : {} habillages", habillages.len());
        assert!(marche, "{id} : aucun chorus en marche");
    }
    Ok(())
}
