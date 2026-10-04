//! Turn start: mana, the draw and fatigue.

use rules::static_card_definition::{BOLT, RECRUIT, SPARK};
use rules::{Action, Game, Outcome};

use crate::support::*;

#[test]
fn ending_a_turn_hands_the_decision_to_the_opponent() {
    let deck = vec![SPARK, SPARK, BOLT, BOLT, RECRUIT, RECRUIT];
    let mut game = Game::with_deck_order(0, [deck.clone(), deck]);
    let [p0, p1] = players(&game);

    game.apply(p0, Action::EndTurn, &mut ()).unwrap();

    // With 1 mana, player 1 can afford the two Sparks but not the Bolts.
    let hand = game.hand(p1);
    assert_actions(&game, p0, &[]);
    assert_actions(&game, p1, &[play(hand[0]), play(hand[1]), Action::EndTurn]);
}

#[test]
fn turn_start_refills_mana_to_a_max_one_higher() {
    let deck = vec![SPARK; 10];
    let mut game = Game::with_deck_order(0, [deck.clone(), deck]);
    let [p0, p1] = players(&game);
    play_def(&mut game, p0, SPARK);
    assert_eq!(game.mana(p0), 0, "the Spark spent player 0's only mana");

    end_turn(&mut game);
    assert_eq!(game.mana(p1), 1, "player 1's first turn");
    end_turn(&mut game);

    assert_eq!(game.mana(p0), 2, "max mana went from 1 to 2, then refilled");
}

#[test]
fn turn_start_draws_one_card_to_the_end_of_the_hand() {
    let deck = vec![SPARK, SPARK, BOLT, BOLT, RECRUIT, RECRUIT];
    let mut game = Game::with_deck_order(0, [deck.clone(), deck]);
    let [p0, p1] = players(&game);

    end_turn(&mut game);
    assert_eq!(hand_defs(&game, p1), [SPARK, SPARK, BOLT, BOLT]);

    end_turn(&mut game);
    assert_eq!(hand_defs(&game, p0), [SPARK, SPARK, BOLT, BOLT, RECRUIT]);
}

#[test]
fn mana_caps_at_ten() {
    let deck = vec![BOLT; 30];
    let mut game = Game::with_deck_order(0, [deck.clone(), deck]);
    let [p0, _] = players(&game);
    for _ in 0..24 {
        end_turn(&mut game);
    }
    assert_eq!(
        game.mana(p0),
        10,
        "player 0's 13th turn start stays at the cap"
    );
}

#[test]
fn drawing_from_an_empty_deck_deals_one_damage() {
    let mut game = Game::with_deck_order(0, [vec![BOLT; 6], vec![SPARK; 3]]);
    let [_, p1] = players(&game);

    end_turn(&mut game);

    assert_eq!(game.hero_health(p1), 9);
    assert_eq!(game.hand(p1).len(), 3, "an empty deck has nothing to draw");
}

#[test]
fn fatigue_at_turn_start_can_end_the_game() {
    let mut game = Game::with_deck_order(0, [vec![BOLT; 30], vec![BOLT; 3]]);
    let [p0, p1] = players(&game);

    end_turns_until_over(&mut game);

    assert_eq!(
        game.outcome(),
        Some(Outcome::Won(p0)),
        "player 1's deck runs out first"
    );
    assert!(game.hero_health(p1) <= 0);
    assert_actions(&game, p0, &[]);
    assert_actions(&game, p1, &[]);
}
