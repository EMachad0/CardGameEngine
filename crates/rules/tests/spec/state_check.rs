//! The state check at the end of every `apply`.

use rules::cards::{BLAST, CAPTAIN, RECRUIT};
use rules::{Game, Outcome};

use crate::support::*;

#[test]
fn blast_removes_every_minion_it_brings_to_zero_health() {
    let deck = deck_with_top(&[RECRUIT, BLAST]);
    let mut game = Game::with_deck_order(0, [deck.clone(), deck]);
    let [p0, p1] = players(&game);
    turn_with_mana(&mut game, p0, 2);
    let mine = summon(&mut game, p0, RECRUIT);
    turn_with_mana(&mut game, p1, 2);
    let theirs = summon(&mut game, p1, RECRUIT);
    turn_with_mana(&mut game, p0, 3);

    play_def(&mut game, p0, BLAST);

    for p in players(&game) {
        assert!(game.board(p).is_empty(), "left {:?}", board_defs(&game, p));
    }
    for minion in [mine, theirs] {
        assert_eq!(game.health(minion), None, "a removed minion has no health");
    }
}

#[test]
fn the_check_repeats_until_a_pass_removes_nothing() {
    // Blast deals 2 to each. The Captain has 1 max health and dies in the first
    // pass. The Recruit has 3 with the buff and survives it, then dies in the
    // second pass once the buff is gone.
    let deck0 = deck_with_top(&[RECRUIT, CAPTAIN, BLAST]);
    let mut game = Game::with_deck_order(0, [deck0, deck_with_top(&[])]);
    let [p0, _] = players(&game);
    turn_with_mana(&mut game, p0, 2);
    let recruit = summon(&mut game, p0, RECRUIT);
    turn_with_mana(&mut game, p0, 3);
    summon(&mut game, p0, CAPTAIN);
    turn_with_mana(&mut game, p0, 3);
    assert_eq!(
        game.health(recruit),
        Some(3),
        "the Captain's buff must hold the Recruit above Blast's damage"
    );

    play_def(&mut game, p0, BLAST);

    assert!(
        game.board(p0).is_empty(),
        "left {:?}",
        board_defs(&game, p0)
    );
    assert_eq!(game.health(recruit), None);
}

#[test]
fn a_blast_that_drops_both_heroes_to_zero_ends_the_game_in_a_draw() {
    // Both decks are empty after player 0's first draw, so every later turn
    // start costs the active player 1 health.
    let mut game = Game::with_deck_order(0, [vec![BLAST; 4], vec![BLAST; 3]]);
    let [p0, p1] = players(&game);
    turn_with_mana(&mut game, p0, 9);
    assert_eq!(
        game.hero_health(p0),
        2,
        "fatigue leaves both heroes at 2 by player 0's ninth turn"
    );
    assert_eq!(game.hero_health(p1), 2);

    play_def(&mut game, p0, BLAST);

    assert_eq!(game.outcome(), Some(Outcome::Draw));
    assert_actions(&game, p0, &[]);
    assert_actions(&game, p1, &[]);
}

#[test]
fn a_blast_that_drops_only_its_casters_hero_to_zero_loses_the_game() {
    let mut game = Game::with_deck_order(0, [vec![BLAST; 4], deck_with_top(&[])]);
    let [p0, p1] = players(&game);
    turn_with_mana(&mut game, p0, 9);
    assert_eq!(
        game.hero_health(p0),
        2,
        "fatigue leaves player 0 at 2 by its ninth turn"
    );
    assert_eq!(game.hero_health(p1), 10, "player 1 still has cards to draw");

    play_def(&mut game, p0, BLAST);

    assert_eq!(game.outcome(), Some(Outcome::Won(p1)));
    assert_actions(&game, p0, &[]);
    assert_actions(&game, p1, &[]);
}
