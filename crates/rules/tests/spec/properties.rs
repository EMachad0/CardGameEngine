//! R1, L, P, B and C over seeded random playouts.

use rules::Game;

use crate::playout::{random_playout, sample_deck};

#[test]
fn new_is_a_function_of_seed_and_decks() {
    for seed in 0..20 {
        let a = Game::new(seed, [sample_deck(), sample_deck()]);
        let b = Game::new(seed, [sample_deck(), sample_deck()]);
        assert_eq!(a, b, "seed {seed}");
    }
}

#[test]
fn random_playouts_keep_every_invariant_and_terminate() {
    for seed in 0..200 {
        let (game, log) = random_playout(seed, seed.wrapping_mul(31).wrapping_add(7));
        assert!(
            game.outcome().is_some(),
            "seed {seed}: ended without an outcome"
        );
        assert!(
            !log.is_empty(),
            "seed {seed}: the game ended before any decision"
        );
    }
}

#[test]
fn replaying_seed_plus_decisions_reproduces_the_game() {
    for seed in 0..50 {
        let (original, log) = random_playout(seed, seed ^ 0xC0FFEE);

        let mut replay = Game::new(seed, [sample_deck(), sample_deck()]);
        for (p, a) in log {
            replay
                .apply(p, a.clone())
                .unwrap_or_else(|e| panic!("seed {seed}: replayed action rejected: {e:?}"));
        }
        assert_eq!(replay, original, "seed {seed}: replay diverged");
    }
}
