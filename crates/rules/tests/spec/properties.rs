//! Invariants over seeded random playouts.

use rules::Game;

use crate::playout::{random_playout, sample_deck};
use crate::support::{Recorder, observe, players};

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
                .apply(p, a, &mut ())
                .unwrap_or_else(|e| panic!("seed {seed}: replayed action rejected: {e:?}"));
        }
        assert_eq!(replay, original, "seed {seed}: replay diverged");
    }
}

/// Replays random playouts with a `Recorder` on every `apply`, and calls `check` with the
/// game after each `apply` and what that `apply` recorded.
fn for_each_observed_apply(mut check: impl FnMut(&Game, &Recorder, &str)) {
    for seed in 0..20 {
        let (_, log) = random_playout(seed, seed ^ 0xD15C);
        let mut game = Game::new(seed, [sample_deck(), sample_deck()]);
        for (step, (p, a)) in log.into_iter().enumerate() {
            let recorder = observe(&mut game, p, a);
            check(&game, &recorder, &format!("seed {seed}, step {step}"));
        }
    }
}

#[test]
fn observing_leaves_the_same_game_as_not_observing() {
    for seed in 0..20 {
        let (original, log) = random_playout(seed, seed ^ 0xD15C);

        let mut observed = Game::new(seed, [sample_deck(), sample_deck()]);
        for (p, a) in log {
            observe(&mut observed, p, a);
        }
        assert_eq!(observed, original, "seed {seed}");
    }
}

#[test]
fn the_same_game_and_action_report_the_same_steps() {
    for seed in 0..20 {
        let (_, log) = random_playout(seed, seed ^ 0xD15C);
        let mut game = Game::new(seed, [sample_deck(), sample_deck()]);
        for (step, (p, a)) in log.into_iter().enumerate() {
            let first = observe(&mut game.clone(), p, a);
            let second = observe(&mut game, p, a);
            assert_eq!(first, second, "seed {seed}, step {step}");
        }
    }
}

#[test]
fn every_apply_ends_with_a_checkpoint_showing_the_game_it_leaves() {
    for_each_observed_apply(|game, recorder, context| {
        assert!(
            recorder.trailing.is_empty(),
            "{context}: events after the last checkpoint: {:?}",
            recorder.trailing
        );
        let last = recorder
            .steps
            .last()
            .unwrap_or_else(|| panic!("{context}: no checkpoint"));
        assert_eq!(last.views, players(game).map(|p| game.view(p)), "{context}");
    });
}

#[test]
fn hidden_cards_show_only_to_whoever_may_see_them() {
    for_each_observed_apply(|_, recorder, context| {
        assert!(!recorder.steps.is_empty(), "{context}: no checkpoint");
        for step in &recorder.steps {
            for view in &step.views {
                for player in &view.players {
                    let owner = player.player_id == view.viewer;
                    for card in &player.hand {
                        assert_eq!(
                            card.face.is_some(),
                            owner,
                            "{context}: {:?}'s view of {:?}'s hand card {:?}",
                            view.viewer,
                            player.player_id,
                            card
                        );
                    }
                    for card in &player.revealed {
                        assert_eq!(
                            card.def_id.is_some(),
                            owner,
                            "{context}: {:?}'s view of {:?}'s revealed card {:?}",
                            view.viewer,
                            player.player_id,
                            card
                        );
                    }
                }
            }
        }
    });
}
