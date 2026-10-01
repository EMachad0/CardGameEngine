//! Attack, health and cost, derived on read (node C).

use rules::Game;
use rules::cards::{BLAST, CAPTAIN, GIANT, RECRUIT, SPARK};

use crate::support::*;

#[test]
fn minions_enter_with_their_printed_stats() {
    let deck0 = deck_with_top(&[CAPTAIN]);
    let mut game = Game::with_deck_order(0, [deck0, deck_with_top(&[RECRUIT])]);
    turn_with_mana(&mut game, P0, 3);
    let captain = summon(&mut game, P0, CAPTAIN);
    turn_with_mana(&mut game, P1, 2);
    let recruit = summon(&mut game, P1, RECRUIT);

    assert_eq!(game.attack(captain).unwrap(), Some(1));
    assert_eq!(game.health(captain).unwrap(), Some(1));
    assert_eq!(game.attack(recruit).unwrap(), Some(2));
    assert_eq!(
        game.health(recruit).unwrap(),
        Some(2),
        "an enemy Captain gives no buff"
    );
    assert_eq!(
        game.mana_cost(captain).unwrap(),
        None,
        "cost is for cards in a hand"
    );
}

#[test]
fn cards_in_hand_have_a_cost_and_no_stats() {
    let game = Game::with_deck_order(0, [deck_with_top(&[RECRUIT]), deck_with_top(&[])]);
    let recruit = in_hand(&game, P0, RECRUIT);

    assert_eq!(game.mana_cost(recruit).unwrap(), Some(2));
    assert_eq!(game.attack(recruit).unwrap(), None);
    assert_eq!(game.health(recruit).unwrap(), None);
}

#[test]
fn captain_buffs_every_other_friendly_minion_whenever_it_entered() {
    let deck0 = deck_with_top(&[RECRUIT, CAPTAIN, RECRUIT]);
    let mut game = Game::with_deck_order(0, [deck0, deck_with_top(&[])]);
    turn_with_mana(&mut game, P0, 2);
    let earlier = summon(&mut game, P0, RECRUIT);
    turn_with_mana(&mut game, P0, 3);
    let captain = summon(&mut game, P0, CAPTAIN);
    turn_with_mana(&mut game, P0, 2);
    let later = summon(&mut game, P0, RECRUIT);

    for recruit in [earlier, later] {
        assert_eq!(game.attack(recruit).unwrap(), Some(3));
        assert_eq!(game.health(recruit).unwrap(), Some(3));
    }
    assert_eq!(game.attack(captain).unwrap(), Some(1), "no buff to itself");
    assert_eq!(game.health(captain).unwrap(), Some(1), "no buff to itself");
}

#[test]
fn two_captains_buff_each_other_and_stack() {
    let deck0 = deck_with_top(&[CAPTAIN, CAPTAIN, RECRUIT]);
    let mut game = Game::with_deck_order(0, [deck0, deck_with_top(&[])]);
    turn_with_mana(&mut game, P0, 3);
    let first = summon(&mut game, P0, CAPTAIN);
    turn_with_mana(&mut game, P0, 3);
    let second = summon(&mut game, P0, CAPTAIN);
    turn_with_mana(&mut game, P0, 2);
    let recruit = summon(&mut game, P0, RECRUIT);

    for captain in [first, second] {
        assert_eq!(game.attack(captain).unwrap(), Some(2));
        assert_eq!(game.health(captain).unwrap(), Some(2));
    }
    assert_eq!(game.attack(recruit).unwrap(), Some(4));
    assert_eq!(game.health(recruit).unwrap(), Some(4));
}

#[test]
fn damage_stays_marked_and_health_is_max_minus_damage() {
    let deck0 = deck_with_top(&[GIANT, BLAST, BLAST]);
    let mut game = Game::with_deck_order(0, [deck0, deck_with_top(&[])]);
    turn_with_mana(&mut game, P0, 8);
    let giant = summon(&mut game, P0, GIANT);
    assert_eq!(game.attack(giant).unwrap(), Some(5));
    assert_eq!(game.health(giant).unwrap(), Some(5));

    turn_with_mana(&mut game, P0, 6);
    play_def(&mut game, P0, BLAST);
    assert_eq!(game.health(giant).unwrap(), Some(3));
    play_def(&mut game, P0, BLAST);
    assert_eq!(game.health(giant).unwrap(), Some(1));
    assert_eq!(game.attack(giant).unwrap(), Some(5));
}

#[test]
fn a_captains_buff_ends_when_the_captain_leaves() {
    let deck0 = deck_with_top(&[GIANT, CAPTAIN, BLAST]);
    let mut game = Game::with_deck_order(0, [deck0, deck_with_top(&[])]);
    turn_with_mana(&mut game, P0, 8);
    let giant = summon(&mut game, P0, GIANT);
    turn_with_mana(&mut game, P0, 6);
    summon(&mut game, P0, CAPTAIN);
    assert_eq!(game.attack(giant).unwrap(), Some(6));
    assert_eq!(game.health(giant).unwrap(), Some(6));

    play_def(&mut game, P0, BLAST);

    assert_eq!(board_defs(&game, P0), [GIANT]);
    assert_eq!(game.attack(giant).unwrap(), Some(5));
    assert_eq!(game.health(giant).unwrap(), Some(3));
}

#[test]
fn giant_costs_one_less_per_spell_its_holder_has_cast() {
    let deck0 = deck_with_top(&[GIANT, SPARK, SPARK, SPARK, RECRUIT]);
    let mut game = Game::with_deck_order(0, [deck0, deck_with_top(&[SPARK])]);
    let giant = in_hand(&game, P0, GIANT);
    assert_eq!(game.mana_cost(giant).unwrap(), Some(8));

    play_def(&mut game, P0, SPARK);
    assert_eq!(game.mana_cost(giant).unwrap(), Some(7));

    turn_with_mana(&mut game, P1, 1);
    play_def(&mut game, P1, SPARK);
    assert_eq!(
        game.mana_cost(giant).unwrap(),
        Some(7),
        "the opponent's spells don't count"
    );

    turn_with_mana(&mut game, P0, 2);
    play_def(&mut game, P0, RECRUIT);
    assert_eq!(
        game.mana_cost(giant).unwrap(),
        Some(7),
        "minions don't count"
    );

    turn_with_mana(&mut game, P0, 2);
    play_def(&mut game, P0, SPARK);
    play_def(&mut game, P0, SPARK);
    assert_eq!(game.mana_cost(giant).unwrap(), Some(5));
}

#[test]
fn a_giant_drawn_later_counts_spells_cast_before_it_arrived() {
    let deck0 = deck_with_top(&[SPARK, SPARK, SPARK, BLAST, BLAST, GIANT]);
    let mut game = Game::with_deck_order(0, [deck0, deck_with_top(&[])]);
    play_def(&mut game, P0, SPARK);
    turn_with_mana(&mut game, P0, 2);
    play_def(&mut game, P0, SPARK);
    play_def(&mut game, P0, SPARK);
    assert!(!has_in_hand(&game, P0, GIANT));

    turn_with_mana(&mut game, P0, 3);

    assert_eq!(game.mana_cost(in_hand(&game, P0, GIANT)).unwrap(), Some(5));
}

#[test]
fn giant_cost_stops_at_zero_and_decides_legality() {
    let mut deck0 = vec![GIANT];
    deck0.extend(vec![SPARK; 20]);
    let mut game = Game::with_deck_order(0, [deck0, deck_with_top(&[])]);
    let giant = in_hand(&game, P0, GIANT);

    let mut cast: u8 = 0;
    while cast < 9 {
        turn_with_mana(&mut game, P0, 1);
        while cast < 9 && game.mana(P0) >= 1 && has_in_hand(&game, P0, SPARK) {
            play_def(&mut game, P0, SPARK);
            cast += 1;

            let cost = 8u8.saturating_sub(cast);
            assert_eq!(
                game.mana_cost(giant).unwrap(),
                Some(cost),
                "after {cast} spells"
            );
            assert_eq!(
                game.legal_actions(P0).contains(&play(giant)),
                cost <= game.mana(P0),
                "after {cast} spells, with {} mana",
                game.mana(P0)
            );
        }
        end_turn(&mut game);
    }
    assert_eq!(game.mana_cost(giant).unwrap(), Some(0));
}
