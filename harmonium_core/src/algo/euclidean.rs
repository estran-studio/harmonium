//! Distribution régulière des pulsations selon Bjorklund.
/// Distribue `pulses` sur `steps` ; zéro donne le silence et `pulses >= steps`
/// sature la grille. Une grille vide produit un vecteur vide.
#[must_use]
pub fn generate_euclidean_bools(steps: usize, pulses: usize) -> Vec<bool> {
    if pulses == 0 {
        return vec![false; steps];
    }
    if pulses >= steps {
        return vec![true; steps];
    }

    let mut pattern: Vec<Vec<u8>> = Vec::new();
    for _ in 0..pulses {
        pattern.push(vec![1]);
    }
    for _ in 0..(steps - pulses) {
        pattern.push(vec![0]);
    }

    let mut count = std::cmp::min(pulses, steps - pulses);
    let mut remainder = pattern.len() - count;

    while remainder > 1 && count > 0 {
        for i in 0..count {
            if let Some(last) = pattern.pop() {
                pattern[i].extend(last);
            }
        }
        remainder = pattern.len() - count;
        count = std::cmp::min(count, remainder);
    }

    let mut result = Vec::new();
    for group in pattern {
        for val in group {
            result.push(val == 1);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::generate_euclidean_bools;

    #[test]
    fn boundaries_and_even_distribution() {
        assert_eq!(generate_euclidean_bools(0, 0), Vec::<bool>::new());
        assert_eq!(generate_euclidean_bools(8, 0), vec![false; 8]);
        assert_eq!(generate_euclidean_bools(3, 8), vec![true; 3]);
        assert_eq!(
            generate_euclidean_bools(8, 3),
            [true, false, false, true, false, false, true, false]
        );
        for steps in 1..=64 {
            for pulses in 1..=steps {
                let pattern = generate_euclidean_bools(steps, pulses);
                let onsets: Vec<_> = pattern
                    .iter()
                    .enumerate()
                    .filter_map(|(step, hit)| hit.then_some(step))
                    .collect();
                assert_eq!(pattern.len(), steps);
                assert_eq!(onsets.len(), pulses);
                let gaps: Vec<_> = (0..pulses)
                    .map(|i| (onsets[(i + 1) % pulses] + steps - onsets[i]) % steps)
                    .collect();
                assert!(gaps.iter().max().unwrap() - gaps.iter().min().unwrap() <= 1);
            }
        }
    }
}
