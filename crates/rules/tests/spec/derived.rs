//! Attack, health and cost, derived on read (node C).

use rules::cards::{BLAST, CAPTAIN, GIANT, RECRUIT, SPARK};
use rules::{Game, ObjectId};

use crate::support::*;

/// `(attack, health)`.
fn stats(game: &Game, minion: ObjectId) -> (Option<i32>, Option<i32>) {
    (game.attack(minion).unwrap(), game.health(minion).unwrap())
}

#[test]
fn minions_enter_with_their_printed_stats() {
    let cases = [
        (RECRUIT, 2, (2, 2)),
        (CAPTAIN, 3, (1, 1)),
        (GIANT, 8, (5, 5)),
    ];
    for (def, cost, (attack, health)) in cases {
        let mut game = Game::with_deck_order(0, [deck_with_top(&[def]), deck_with_top(&[])]);
        turn_with_mana(&mut game, P0, cost);

        let minion = summon(&mut game, P0, def);

        assert_eq!(
            stats(&game, minion),
            (Some(attack), Some(health)),
            "{def:?}"
        );
    }
}

#[test]
fn a_captain_does_not_buff_enemy_minions() {
    let deck0 = deck_with_top(&[CAPTAIN]);
    let mut game = Game::with_deck_order(0, [deck0, deck_with_top(&[RECRUIT])]);
    turn_with_mana(&mut game, P0, 3);
    summon(&mut game, P0, CAPTAIN);
    turn_with_mana(&mut game, P1, 2);

    let recruit = summon(&mut game, P1, RECRUIT);

    assert_eq!(
        stats(&game, recruit),
        (Some(2), Some(2)),
        "the Captain is on the other board"
    );
}

#[test]
fn cards_in_hand_have_a_cost_and_no_stats() {
    let game = Game::with_deck_order(0, [deck_with_top(&[RECRUIT]), deck_with_top(&[])]);
    let recruit = in_hand(&game, P0, RECRUIT);

    assert_eq!(game.mana_cost(recruit).unwrap(), Some(2));
    assert_eq!(stats(&game, recruit), (None, None));
}

#[test]
fn a_minion_on_the_board_has_no_cost() {
    let mut game = Game::with_deck_order(0, [deck_with_top(&[RECRUIT]), deck_with_top(&[])]);
    turn_with_mana(&mut game, P0, 2);

    let recruit = summon(&mut game, P0, RECRUIT);

    assert_eq!(game.mana_cost(recruit).unwrap(), None);
}

/// Player 0's board: a Recruit, a Captain, then a second Recruit.
fn recruit_captain_recruit() -> (Game, [ObjectId; 3]) {
    let deck0 = deck_with_top(&[RECRUIT, CAPTAIN, RECRUIT]);
    let mut game = Game::with_deck_order(0, [deck0, deck_with_top(&[])]);
    turn_with_mana(&mut game, P0, 2);
    let earlier = summon(&mut game, P0, RECRUIT);
    turn_with_mana(&mut game, P0, 3);
    let captain = summon(&mut game, P0, CAPTAIN);
    turn_with_mana(&mut game, P0, 2);
    let later = summon(&mut game, P0, RECRUIT);
    (game, [earlier, captain, later])
}

#[test]
fn a_captain_buffs_friendly_minions_whenever_they_entered() {
    let (game, [earlier, _, later]) = recruit_captain_recruit();

    assert_eq!(
        stats(&game, earlier),
        (Some(3), Some(3)),
        "entered before the Captain"
    );
    assert_eq!(
        stats(&game, later),
        (Some(3), Some(3)),
        "entered after the Captain"
    );
}

#[test]
fn a_captain_does_not_buff_itself() {
    let (game, [_, captain, _]) = recruit_captain_recruit();

    assert_eq!(stats(&game, captain), (Some(1), Some(1)));
}

/// Player 0's board: two Captains, then a Recruit.
fn two_captains_and_a_recruit() -> (Game, [ObjectId; 3]) {
    let deck0 = deck_with_top(&[CAPTAIN, CAPTAIN, RECRUIT]);
    let mut game = Game::with_deck_order(0, [deck0, deck_with_top(&[])]);
    turn_with_mana(&mut game, P0, 3);
    let first = summon(&mut game, P0, CAPTAIN);
    turn_with_mana(&mut game, P0, 3);
    let second = summon(&mut game, P0, CAPTAIN);
    turn_with_mana(&mut game, P0, 2);
    let recruit = summon(&mut game, P0, RECRUIT);
    (game, [first, second, recruit])
}

#[test]
fn two_captains_buff_each_other() {
    let (game, [first, second, _]) = two_captains_and_a_recruit();

    for captain in [first, second] {
        assert_eq!(stats(&game, captain), (Some(2), Some(2)));
    }
}

#[test]
fn buffs_from_two_captains_stack() {
    let (game, [_, _, recruit]) = two_captains_and_a_recruit();

    assert_eq!(
        stats(&game, recruit),
        (Some(4), Some(4)),
        "2/2 plus 1/1 from each Captain"
    );
}

#[test]
fn health_is_max_health_minus_marked_damage() {
    let deck0 = deck_with_top(&[GIANT, BLAST, BLAST]);
    let mut game = Game::with_deck_order(0, [deck0, deck_with_top(&[])]);
    turn_with_mana(&mut game, P0, 8);
    let giant = summon(&mut game, P0, GIANT);
    turn_with_mana(&mut game, P0, 6);

    play_def(&mut game, P0, BLAST);
    assert_eq!(stats(&game, giant), (Some(5), Some(3)));
    play_def(&mut game, P0, BLAST);

    assert_eq!(
        stats(&game, giant),
        (Some(5), Some(1)),
        "damage stays marked and lowers health only"
    );
}

#[test]
fn a_captains_buff_ends_when_the_captain_leaves() {
    let deck0 = deck_with_top(&[GIANT, CAPTAIN, BLAST]);
    let mut game = Game::with_deck_order(0, [deck0, deck_with_top(&[])]);
    turn_with_mana(&mut game, P0, 8);
    let giant = summon(&mut game, P0, GIANT);
    turn_with_mana(&mut game, P0, 6);
    summon(&mut game, P0, CAPTAIN);
    assert_eq!(stats(&game, giant), (Some(6), Some(6)));

    play_def(&mut game, P0, BLAST);

    assert_eq!(board_defs(&game, P0), [GIANT], "Blast killed the Captain");
    assert_eq!(
        stats(&game, giant),
        (Some(5), Some(3)),
        "the Giant keeps Blast's 2 damage and loses the buff"
    );
}

#[test]
fn giant_costs_one_less_per_spell_its_holder_has_cast() {
    let deck0 = deck_with_top(&[GIANT, SPARK, SPARK, SPARK]);
    let mut game = Game::with_deck_order(0, [deck0, deck_with_top(&[])]);
    let giant = in_hand(&game, P0, GIANT);
    assert_eq!(game.mana_cost(giant).unwrap(), Some(8));

    play_def(&mut game, P0, SPARK);
    assert_eq!(game.mana_cost(giant).unwrap(), Some(7));

    turn_with_mana(&mut game, P0, 2);
    play_def(&mut game, P0, SPARK);
    play_def(&mut game, P0, SPARK);
    assert_eq!(game.mana_cost(giant).unwrap(), Some(5));
}

#[test]
fn opponents_spells_do_not_lower_a_giants_cost() {
    let deck0 = deck_with_top(&[GIANT]);
    let mut game = Game::with_deck_order(0, [deck0, deck_with_top(&[SPARK])]);
    let giant = in_hand(&game, P0, GIANT);
    turn_with_mana(&mut game, P1, 1);

    play_def(&mut game, P1, SPARK);

    assert_eq!(game.mana_cost(giant).unwrap(), Some(8));
}

#[test]
fn minions_do_not_lower_a_giants_cost() {
    let deck0 = deck_with_top(&[GIANT, RECRUIT]);
    let mut game = Game::with_deck_order(0, [deck0, deck_with_top(&[])]);
    let giant = in_hand(&game, P0, GIANT);
    turn_with_mana(&mut game, P0, 2);

    play_def(&mut game, P0, RECRUIT);

    assert_eq!(game.mana_cost(giant).unwrap(), Some(8));
}

#[test]
fn a_giant_drawn_later_counts_spells_cast_before_it_arrived() {
    let deck0 = deck_with_top(&[SPARK, SPARK, SPARK, BLAST, BLAST, GIANT]);
    let mut game = Game::with_deck_order(0, [deck0, deck_with_top(&[])]);
    play_def(&mut game, P0, SPARK);
    turn_with_mana(&mut game, P0, 2);
    play_def(&mut game, P0, SPARK);
    play_def(&mut game, P0, SPARK);
    assert!(
        !has_in_hand(&game, P0, GIANT),
        "the Giant is still in the deck"
    );

    turn_with_mana(&mut game, P0, 3);

    assert_eq!(game.mana_cost(in_hand(&game, P0, GIANT)).unwrap(), Some(5));
}

/// Player 0 holds a Giant, and the rest of the deck is Sparks.
fn giant_among_sparks() -> (Game, ObjectId) {
    let mut deck0 = vec![GIANT];
    deck0.extend(vec![SPARK; 20]);
    let game = Game::with_deck_order(0, [deck0, deck_with_top(&[])]);
    let giant = in_hand(&game, P0, GIANT);
    (game, giant)
}

/// Casts `count` Sparks from player 0's hand over as many turns as it takes. Calls
/// `after_each` with the number cast so far.
fn cast_sparks(game: &mut Game, count: u8, mut after_each: impl FnMut(&Game, u8)) {
    let mut cast = 0;
    while cast < count {
        turn_with_mana(game, P0, 1);
        while cast < count && game.mana(P0) >= 1 && has_in_hand(game, P0, SPARK) {
            play_def(game, P0, SPARK);
            cast += 1;
            after_each(game, cast);
        }
        end_turn(game);
    }
}

#[test]
fn a_giants_cost_never_drops_below_zero() {
    let (mut game, giant) = giant_among_sparks();

    cast_sparks(&mut game, 9, |_, _| {});

    assert_eq!(
        game.mana_cost(giant).unwrap(),
        Some(0),
        "nine spells against a cost of 8"
    );
}

#[test]
fn a_giant_is_playable_exactly_when_its_cost_fits_the_mana() {
    let (mut game, giant) = giant_among_sparks();

    cast_sparks(&mut game, 9, |game, cast| {
        let cost = 8u8.saturating_sub(cast);
        assert_eq!(
            game.legal_actions(P0).contains(&play(giant)),
            cost <= game.mana(P0),
            "after {cast} spells, with {} mana",
            game.mana(P0)
        );
    });
}
