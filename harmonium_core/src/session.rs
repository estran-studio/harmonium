//! Ouverture d'une session : le RNG de la graine et ses deux premiers tirages.

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Nombre de tonalités tirables à l'ouverture d'une session (do à si).
pub const SESSION_KEYS: u64 = 7;
/// Nombre de gammes tirables à l'ouverture d'une session (pentatonique mineure ou majeure).
pub const SESSION_SCALES: u64 = 2;

/// RNG d'une session et les deux premiers tirages (indice de tonalité dans `0..SESSION_KEYS`,
/// indice de gamme dans `0..SESSION_SCALES`) : seule initialisation, partagée par le composer et
/// `harmonium_determinism`. Les deux indices ne pilotent plus rien (ils servaient à un affichage
/// retiré, qui ne disait pas ce qui joue : la tonalité vient du plan du profil) ; les tirages
/// restent, car ils avancent le RNG et fixent donc la suite de chaque session. Tirages en `u64`,
/// comme tout le moteur (voir `algo::RngCore`).
#[must_use]
pub fn session_start(seed: u64) -> (ChaCha8Rng, u64, u64) {
    use rand::Rng;
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let key = rng.gen_range(0..SESSION_KEYS);
    let scale = rng.gen_range(0..SESSION_SCALES);
    (rng, key, scale)
}

/// RNG d'une session après les deux premiers tirages de `session_start`.
#[must_use]
pub fn session_rng(seed: u64) -> ChaCha8Rng {
    session_start(seed).0
}
