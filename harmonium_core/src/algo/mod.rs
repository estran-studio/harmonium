//! Algorithmes purs réutilisables par les profils et les générateurs.
pub mod euclidean;

use rand::Rng;

/// Tirages injectés dans les algorithmes de profil, compatibles avec `dyn`.
///
/// Les tirages d'entiers sont en `u64`, jamais en `usize` : rand 0.8 échantillonne un `usize` avec
/// l'entier de la largeur du pointeur, donc le même flux ChaCha8 serait consommé autrement en
/// wasm32 (32 bits) qu'en natif 64 bits (constitution, principe II).
pub trait RngCore {
    fn next_f32(&mut self) -> f32;
    fn next_range_u64(&mut self, range: std::ops::Range<u64>) -> u64;
}

impl<R: Rng> RngCore for R {
    fn next_f32(&mut self) -> f32 {
        self.r#gen::<f32>()
    }

    fn next_range_u64(&mut self, range: std::ops::Range<u64>) -> u64 {
        self.gen_range(range)
    }
}
