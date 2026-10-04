//! Playing a card, and what each card does.

use std::collections::BTreeSet;

use rules::static_card_definition::{
    BARRACKS, BLAST, BOLT, CAPTAIN, FORAGE, GIANT, RECRUIT, SPARK, SQUIRE, STRAY_SHOT, WILD_BOLT,
    ZAP,
};
use rules::{Action, Event, Game, ObjectId};

use crate::support::*;

#[test]
fn playing_a_card_keeps_the_rest_of_the_hand_in_order() {
    let deck = vec![BOLT, SPARK, RECRUIT, CAPTAIN, BLAST];
    let mut game = Game::with_deck_order(0, [deck.clone(), deck]);
    let [p0, _] = players(&game);

    play_def(&mut game, p0, SPARK);

    assert_eq!(hand_defs(&game, p0), [BOLT, RECRUIT, CAPTAIN]);
}

#[test]
fn damage_spells_hit_the_casters_enemy() {
    for (spell, damage) in [(SPARK, 1), (BOLT, 2), (ZAP, 1)] {
        for (caster, enemy) in [(0, 1), (1, 0)] {
            let deck = deck_with_top(&[spell]);
            let mut game = Game::with_deck_order(0, [deck.clone(), deck]);
            let ids = players(&game);
            let (caster, enemy) = (ids[caster], ids[enemy]);
            turn_with_mana(&mut game, caster, 2);

            play_def(&mut game, caster, spell);

            assert_eq!(
                game.hero_health(enemy),
                10 - damage,
                "{spell:?} by {caster:?}"
            );
            assert_eq!(game.hero_health(caster), 10, "{spell:?} by {caster:?}");
        }
    }
}

#[test]
fn zap_draws_the_top_card_of_its_casters_deck() {
    let mut game = Game::with_deck_order(0, [deck_with_top(&[ZAP]), deck_with_top(&[])]);
    let [p0, _] = players(&game);
    turn_with_mana(&mut game, p0, 2);
    let zap = in_hand(&game, p0, ZAP);
    let top = game.deck(p0)[0];
    let mut expected: Vec<ObjectId> = game
        .hand(p0)
        .iter()
        .copied()
        .filter(|&id| id != zap)
        .collect();
    expected.push(top);

    game.apply(p0, play(zap), &mut ()).unwrap();

    assert_eq!(game.hand(p0), expected);
}

#[test]
fn every_card_pays_its_printed_cost() {
    // Written out by hand, so a wrong cost in the core can't check itself.
    let cases = [
        (SPARK, 1),
        (BOLT, 2),
        (WILD_BOLT, 1),
        (FORAGE, 1),
        (BLAST, 3),
        (RECRUIT, 2),
        (CAPTAIN, 3),
        (GIANT, 8),
        (BARRACKS, 2),
        (SQUIRE, 1),
        (ZAP, 2),
        (STRAY_SHOT, 1),
    ];
    for (def, cost) in cases {
        let mut game = Game::with_deck_order(0, [deck_with_top(&[def]), deck_with_top(&[])]);
        let [p0, _] = players(&game);
        turn_with_mana(&mut game, p0, cost);
        let card = in_hand(&game, p0, def);
        assert_eq!(game.mana_cost(card), Some(cost), "{def:?}");

        let before = game.mana(p0);
        game.apply(p0, play(card), &mut ()).unwrap();
        assert_eq!(game.mana(p0), before - cost, "{def:?}");
    }
}

#[test]
fn minions_enter_at_the_right_end_of_their_owners_board() {
    let deck0 = deck_with_top(&[RECRUIT, CAPTAIN, RECRUIT]);
    let mut game = Game::with_deck_order(0, [deck0, deck_with_top(&[])]);
    let [p0, p1] = players(&game);
    turn_with_mana(&mut game, p0, 2);
    play_def(&mut game, p0, RECRUIT);
    turn_with_mana(&mut game, p0, 3);
    play_def(&mut game, p0, CAPTAIN);
    turn_with_mana(&mut game, p0, 2);
    play_def(&mut game, p0, RECRUIT);

    assert_eq!(board_defs(&game, p0), [RECRUIT, CAPTAIN, RECRUIT]);
    assert!(
        game.board(p1).is_empty(),
        "a minion enters its owner's board only"
    );
}

#[test]
fn blast_deals_two_damage_to_every_character() {
    let deck0 = deck_with_top(&[GIANT, BLAST]);
    let mut game = Game::with_deck_order(0, [deck0, deck_with_top(&[GIANT])]);
    let [p0, p1] = players(&game);
    turn_with_mana(&mut game, p0, 8);
    let mine = summon(&mut game, p0, GIANT);
    turn_with_mana(&mut game, p1, 8);
    let theirs = summon(&mut game, p1, GIANT);
    turn_with_mana(&mut game, p0, 3);

    play_def(&mut game, p0, BLAST);

    assert_eq!(game.hero_health(p0), 8);
    assert_eq!(game.hero_health(p1), 8);
    assert_eq!(game.health(mine), Some(3));
    assert_eq!(game.health(theirs), Some(3));
}

#[test]
fn barracks_summons_a_squire_at_the_right_end_of_its_casters_board() {
    let deck0 = deck_with_top(&[RECRUIT, BARRACKS]);
    let mut game = Game::with_deck_order(0, [deck0, deck_with_top(&[])]);
    let [p0, p1] = players(&game);
    turn_with_mana(&mut game, p0, 2);
    play_def(&mut game, p0, RECRUIT);
    turn_with_mana(&mut game, p0, 2);

    play_def(&mut game, p0, BARRACKS);

    assert_eq!(board_defs(&game, p0), [RECRUIT, SQUIRE]);
    assert!(
        game.board(p1).is_empty(),
        "the Squire enters its caster's board only"
    );
}

#[test]
fn a_summoned_squire_has_its_printed_stats() {
    let mut game = Game::with_deck_order(0, [deck_with_top(&[BARRACKS]), deck_with_top(&[])]);
    let [p0, _] = players(&game);
    turn_with_mana(&mut game, p0, 2);

    play_def(&mut game, p0, BARRACKS);

    let squire = game.board(p0)[0];
    assert_eq!(
        (game.attack(squire), game.health(squire)),
        (Some(1), Some(1))
    );
}

#[test]
fn each_barracks_summons_a_new_object() {
    let deck0 = deck_with_top(&[BARRACKS, BARRACKS]);
    let mut game = Game::with_deck_order(0, [deck0, deck_with_top(&[])]);
    let [p0, _] = players(&game);
    turn_with_mana(&mut game, p0, 4);
    let before: BTreeSet<ObjectId> = zone_ids(&game).into_iter().collect();

    play_def(&mut game, p0, BARRACKS);
    play_def(&mut game, p0, BARRACKS);

    let &[first, second] = game.board(p0) else {
        panic!("two Barracks left {:?}", board_defs(&game, p0));
    };
    assert_ne!(first, second);
    for squire in [first, second] {
        assert!(
            !before.contains(&squire),
            "{squire:?} was already in a zone before the summon"
        );
    }
}

/// Player 0 casts Forage with a Recruit and a Captain on top of the deck and a Bolt under them.
fn forage_revealing_recruit_and_captain() -> Game {
    let deck0 = vec![FORAGE, SPARK, SPARK, SPARK, RECRUIT, CAPTAIN, BOLT];
    let mut game = Game::with_deck_order(0, [deck0, vec![SPARK; 6]]);
    let [p0, _] = players(&game);
    play_def(&mut game, p0, FORAGE);
    game
}

#[test]
fn forage_reveals_the_top_two_cards_of_the_deck() {
    let game = forage_revealing_recruit_and_captain();
    let [p0, _] = players(&game);

    assert_eq!(revealed_defs(&game, p0), [RECRUIT, CAPTAIN]);
    assert_eq!(deck_defs(&game, p0), [BOLT]);
}

#[test]
fn a_pending_forage_offers_nothing_but_picks() {
    let game = forage_revealing_recruit_and_captain();
    let [p0, p1] = players(&game);

    let revealed = game.revealed(p0);
    assert_actions(&game, p0, &[pick(revealed[0]), pick(revealed[1])]);
    assert_actions(&game, p1, &[]);
}

#[test]
fn the_picked_card_goes_to_the_end_of_the_hand() {
    let mut game = forage_revealing_recruit_and_captain();
    let [p0, _] = players(&game);

    let captain = game.revealed(p0)[1];
    game.apply(p0, pick(captain), &mut ()).unwrap();

    assert_eq!(hand_defs(&game, p0), [SPARK, SPARK, SPARK, CAPTAIN]);
}

#[test]
fn the_unpicked_card_goes_to_the_bottom_of_the_deck() {
    let mut game = forage_revealing_recruit_and_captain();
    let [p0, _] = players(&game);

    let captain = game.revealed(p0)[1];
    game.apply(p0, pick(captain), &mut ()).unwrap();

    assert_eq!(deck_defs(&game, p0), [BOLT, RECRUIT]);
}

#[test]
fn picking_ends_the_pending_forage() {
    let mut game = forage_revealing_recruit_and_captain();
    let [p0, _] = players(&game);

    let captain = game.revealed(p0)[1];
    game.apply(p0, pick(captain), &mut ()).unwrap();

    assert!(game.revealed(p0).is_empty());
    // Forage spent player 0's only mana, so ending the turn is all that is left.
    assert_actions(&game, p0, &[Action::EndTurn]);
}

#[test]
fn forage_with_one_card_left_still_asks_for_the_pick() {
    let deck0 = vec![FORAGE, SPARK, SPARK, SPARK, RECRUIT];
    let mut game = Game::with_deck_order(0, [deck0, vec![SPARK; 6]]);
    let [p0, _] = players(&game);

    play_def(&mut game, p0, FORAGE);

    assert_eq!(revealed_defs(&game, p0), [RECRUIT]);
    let revealed = game.revealed(p0);
    assert_actions(&game, p0, &[pick(revealed[0])]);
}

#[test]
fn forage_on_an_empty_deck_does_nothing() {
    let deck0 = vec![FORAGE, SPARK, SPARK, SPARK];
    let mut game = Game::with_deck_order(0, [deck0, vec![SPARK; 6]]);
    let [p0, _] = players(&game);

    play_def(&mut game, p0, FORAGE);

    assert!(game.revealed(p0).is_empty());
    assert_eq!(hand_defs(&game, p0), [SPARK, SPARK, SPARK]);
    assert_actions(&game, p0, &[Action::EndTurn]);
}

/// Health lost by (caster, enemy) after player 0 casts Wild Bolt in a game seeded with `seed`.
fn cast_wild_bolt(seed: u64) -> (i32, i32) {
    let mut game = Game::with_deck_order(seed, [vec![WILD_BOLT; 6], vec![SPARK; 6]]);
    let [p0, p1] = players(&game);
    play_def(&mut game, p0, WILD_BOLT);
    (10 - game.hero_health(p0), 10 - game.hero_health(p1))
}

#[test]
fn wild_bolt_hits_exactly_one_hero_for_three() {
    for seed in 0..100 {
        let lost = cast_wild_bolt(seed);
        assert!(
            lost == (3, 0) || lost == (0, 3),
            "seed {seed}: health lost (caster, enemy) = {lost:?}"
        );
    }
}

#[test]
fn wild_bolt_can_hit_either_hero() {
    let outcomes: Vec<(i32, i32)> = (0..100).map(cast_wild_bolt).collect();
    assert!(
        outcomes.contains(&(3, 0)),
        "never hit the caster in 100 seeds"
    );
    assert!(
        outcomes.contains(&(0, 3)),
        "never hit the enemy in 100 seeds"
    );
}

/// Player 0 has one Recruit on the board and player 1 has two, and player 0 can cast the Stray
/// Shot in hand. Returns the game and player 1's Recruits, left to right.
fn stray_shot_facing_two_recruits(seed: u64) -> (Game, [ObjectId; 2]) {
    let deck0 = deck_with_top(&[RECRUIT, STRAY_SHOT]);
    let deck1 = deck_with_top(&[RECRUIT, RECRUIT]);
    let mut game = Game::with_deck_order(seed, [deck0, deck1]);
    let [p0, p1] = players(&game);
    turn_with_mana(&mut game, p0, 2);
    summon(&mut game, p0, RECRUIT);
    turn_with_mana(&mut game, p1, 2);
    let first = summon(&mut game, p1, RECRUIT);
    turn_with_mana(&mut game, p1, 2);
    let second = summon(&mut game, p1, RECRUIT);
    turn_with_mana(&mut game, p0, 1);
    (game, [first, second])
}

/// Which of player 1's two Recruits Stray Shot killed, in a game seeded with `seed`.
fn cast_stray_shot(seed: u64) -> ObjectId {
    let (mut game, enemy_recruits) = stray_shot_facing_two_recruits(seed);
    let [p0, p1] = players(&game);
    let mine = game.board(p0)[0];

    play_def(&mut game, p0, STRAY_SHOT);

    let &[survivor] = game.board(p1) else {
        panic!(
            "seed {seed}: player 1's board after Stray Shot: {:?}",
            board_defs(&game, p1)
        );
    };
    assert_eq!(game.health(mine), Some(2), "seed {seed}: friendly Recruit");
    assert_eq!(game.hero_health(p0), 10, "seed {seed}: caster's hero");
    assert_eq!(game.hero_health(p1), 10, "seed {seed}: enemy hero");
    let [first, second] = enemy_recruits;
    if survivor == first { second } else { first }
}

#[test]
fn stray_shot_hits_one_enemy_minion_only() {
    for seed in 0..50 {
        cast_stray_shot(seed);
    }
}

#[test]
fn stray_shot_can_hit_either_enemy_minion() {
    let killed: BTreeSet<ObjectId> = (0..50).map(cast_stray_shot).collect();
    assert_eq!(
        killed.len(),
        2,
        "the same Recruit died in all 50 seeds: {killed:?}"
    );
}

/// Player 0 has a Recruit on the board, player 1 has none, and player 0 can cast the Stray Shot
/// in hand.
fn stray_shot_facing_an_empty_board() -> Game {
    let deck0 = deck_with_top(&[RECRUIT, STRAY_SHOT]);
    let mut game = Game::with_deck_order(0, [deck0, deck_with_top(&[])]);
    let [p0, _] = players(&game);
    turn_with_mana(&mut game, p0, 2);
    summon(&mut game, p0, RECRUIT);
    turn_with_mana(&mut game, p0, 1);
    game
}

#[test]
fn stray_shot_is_playable_with_no_enemy_minion() {
    let game = stray_shot_facing_an_empty_board();
    let [p0, _] = players(&game);

    let stray_shot = in_hand(&game, p0, STRAY_SHOT);

    assert!(game.legal_actions(p0).contains(&play(stray_shot)));
}

#[test]
fn stray_shot_with_no_enemy_minion_does_nothing() {
    let mut game = stray_shot_facing_an_empty_board();
    let [p0, _] = players(&game);
    let stray_shot = in_hand(&game, p0, STRAY_SHOT);

    let events = observe(&mut game, p0, play(stray_shot)).events();

    assert_eq!(
        events,
        [Event::Played {
            player_id: p0,
            object_id: stray_shot
        }],
        "the friendly Recruit and both heroes are not candidates"
    );
}
