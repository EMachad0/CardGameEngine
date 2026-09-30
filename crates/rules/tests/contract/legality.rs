//! P on exact positions. An unlisted action returns the exact `Illegal` and
//! leaves the game unchanged.

use rules::{Action, Card, Game, Illegal};

use crate::support::*;

#[test]
fn acting_on_the_opponents_turn_is_rejected_and_changes_nothing() {
    let deck = vec![bolt(1); 6];
    let game = Game::with_deck_order(0, [deck.clone(), deck]);

    for a in [Action::EndTurn, play(0)] {
        let mut g = game.clone();
        assert_eq!(
            g.apply(P1, a.clone()),
            Err(Illegal {
                player: P1,
                action: a
            })
        );
        assert_eq!(g, game);
    }
}

#[test]
fn unaffordable_and_out_of_range_plays_are_rejected() {
    let deck = vec![bolt(1), bolt(5), bolt(5), bolt(5), bolt(5)];
    let game = Game::with_deck_order(0, [deck.clone(), deck]);

    assert_actions(&game, P0, &[play(0), Action::EndTurn]);
    assert_unlisted_rejected(&game);
}

#[test]
fn a_pending_pick_rejects_everything_else() {
    let deck0 = vec![
        Card::Forage,
        bolt(1),
        bolt(1),
        bolt(1),
        bolt(5),
        bolt(6),
        bolt(2),
    ];
    let mut game = Game::with_deck_order(0, [deck0, vec![bolt(1); 6]]);
    game.apply(P0, play(0)).unwrap();

    assert_actions(&game, P0, &[pick(0), pick(1)]);
    assert_unlisted_rejected(&game);
}

#[test]
fn a_finished_game_rejects_everything() {
    // Player 1 fatigues to 2 health, then player 0's Bolt 9 ends it.
    let mut game = Game::with_deck_order(0, [vec![bolt(9); 10], vec![bolt(9); 3]]);
    for _ in 0..8 {
        game.apply(P0, Action::EndTurn).unwrap();
        game.apply(P1, Action::EndTurn).unwrap();
    }
    game.apply(P0, play(0)).unwrap();

    assert_eq!(game.winner(), Some(P0));
    assert_unlisted_rejected(&game);
}
