//! Lookups on an object outside every zone, and on an id the game never made.

use std::collections::BTreeSet;

use rules::cards::{BLAST, RECRUIT, SPARK};
use rules::{Game, ObjectId};

use crate::support::*;

/// Player 0's Blast has killed player 0's Recruit. Returns the game and the Recruit.
fn dead_recruit() -> (Game, ObjectId) {
    let deck0 = deck_with_top(&[RECRUIT, BLAST]);
    let mut game = Game::with_deck_order(0, [deck0, deck_with_top(&[])]);
    let [p0, _] = players(&game);
    turn_with_mana(&mut game, p0, 2);
    let recruit = summon(&mut game, p0, RECRUIT);
    turn_with_mana(&mut game, p0, 3);
    play_def(&mut game, p0, BLAST);
    assert!(
        !zone_ids(&game).contains(&recruit),
        "Blast left the Recruit in a zone"
    );
    (game, recruit)
}

#[test]
fn a_dead_minion_keeps_its_definition() {
    let (game, recruit) = dead_recruit();

    assert_eq!(game.def_id(recruit), RECRUIT);
}

#[test]
fn a_dead_minion_has_no_cost_attack_or_health() {
    let (game, recruit) = dead_recruit();

    assert_eq!(game.mana_cost(recruit), None);
    assert_eq!(game.attack(recruit), None);
    assert_eq!(game.health(recruit), None);
}

/// A new game of six cards, and an id from a larger game that the six-card game never made.
fn foreign_id() -> (Game, ObjectId) {
    let small = Game::with_deck_order(0, [vec![SPARK; 3], vec![SPARK; 3]]);
    let large = Game::with_deck_order(0, [vec![SPARK; 30], vec![SPARK; 30]]);
    let made: BTreeSet<ObjectId> = zone_ids(&small).into_iter().collect();
    let foreign = zone_ids(&large)
        .into_iter()
        .find(|id| !made.contains(id))
        .expect("the larger game has an id the smaller one lacks");
    (small, foreign)
}

#[test]
#[should_panic]
fn def_id_panics_on_an_id_the_game_never_made() {
    let (game, id) = foreign_id();

    game.def_id(id);
}

#[test]
#[should_panic]
fn mana_cost_panics_on_an_id_the_game_never_made() {
    let (game, id) = foreign_id();

    game.mana_cost(id);
}

#[test]
#[should_panic]
fn attack_panics_on_an_id_the_game_never_made() {
    let (game, id) = foreign_id();

    game.attack(id);
}

#[test]
#[should_panic]
fn health_panics_on_an_id_the_game_never_made() {
    let (game, id) = foreign_id();

    game.health(id);
}
