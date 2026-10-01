//! Turn start: mana, the draw and fatigue.

use rules::cards::{BOLT, RECRUIT, SPARK};
use rules::{Action, Game, Outcome};

use crate::support::*;

#[test]
fn end_turn_starts_the_opponents_turn() {
    let deck = vec![SPARK, SPARK, BOLT, BOLT, RECRUIT, RECRUIT];
    let mut game = Game::with_deck_order(0, [deck.clone(), deck]);

    game.apply(P0, Action::EndTurn).unwrap();
    assert_eq!(game.mana(P1), 1);
    assert_eq!(hand_defs(&game, P1), [SPARK, SPARK, BOLT, BOLT]);
    let hand = game.hand(P1);
    assert_actions(&game, P0, &[]);
    assert_actions(&game, P1, &[play(hand[0]), play(hand[1]), Action::EndTurn]);

    game.apply(P1, Action::EndTurn).unwrap();
    assert_eq!(game.mana(P0), 2, "max mana grows by one per turn");
    assert_eq!(hand_defs(&game, P0), [SPARK, SPARK, BOLT, BOLT, RECRUIT]);
    let mut expected: Vec<Action> = game.hand(P0).iter().map(|&o| play(o)).collect();
    expected.push(Action::EndTurn);
    assert_actions(&game, P0, &expected);
}

#[test]
fn mana_caps_at_ten() {
    let deck = vec![BOLT; 30];
    let mut game = Game::with_deck_order(0, [deck.clone(), deck]);
    for _ in 0..24 {
        end_turn(&mut game);
    }
    assert_eq!(game.mana(P0), 10);
}

#[test]
fn drawing_from_an_empty_deck_deals_one_damage() {
    let mut game = Game::with_deck_order(0, [vec![BOLT; 6], vec![SPARK; 3]]);

    game.apply(P0, Action::EndTurn).unwrap();
    assert_eq!(game.hero_health(P1), 9);
    assert_eq!(game.hand(P1).len(), 3);
}

#[test]
fn fatigue_at_turn_start_can_end_the_game() {
    let mut game = Game::with_deck_order(0, [vec![BOLT; 30], vec![BOLT; 3]]);
    end_turns_until_over(&mut game);
    assert_eq!(game.outcome(), Some(Outcome::Won(P0)));
    assert!(game.hero_health(P1) <= 0);
    assert_actions(&game, P0, &[]);
    assert_actions(&game, P1, &[]);
}
