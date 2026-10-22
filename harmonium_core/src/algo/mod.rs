//! Algorithmes purs réutilisables par les profils et les générateurs.
pub mod euclidean;

use rand::Rng;

/// Tirages injectés dans les algorithmes de profil, compatibles avec `dyn`.
pub trait RngCore {
    fn next_f32(&mut self) -> f32;
    fn next_range_usize(&mut self, range: std::ops::Range<usize>) -> usize;
}

impl<R: Rng> RngCore for R {
    fn next_f32(&mut self) -> f32 {
        self.r#gen::<f32>()
    }

    fn next_range_usize(&mut self, range: std::ops::Range<usize>) -> usize {
        self.gen_range(range)
    }
}
