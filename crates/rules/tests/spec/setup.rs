//! Setup: the shuffle, the opening deal and the first turn.

use rules::static_card_definition::{
    BLAST, BOLT, CAPTAIN, FORAGE, GIANT, HERO, RECRUIT, SPARK, WILD_BOLT,
};
use rules::{Action, DefId, Game, ObjectId};

use crate::support::*;

fn sample_deck() -> Vec<DefId> {
    vec![
        SPARK, BOLT, FORAGE, RECRUIT, WILD_BOLT, CAPTAIN, BLAST, GIANT, SPARK, RECRUIT,
    ]
}

fn opening() -> Game {
    let deck0 = vec![SPARK, BOLT, RECRUIT, CAPTAIN, BLAST];
    let deck1 = vec![SPARK, SPARK, SPARK, BOLT, BOLT];
    Game::with_deck_order(0, [deck0, deck1])
}

#[test]
fn setup_deals_three_cards_each_from_the_top() {
    let game = opening();
    let [p0, p1] = players(&game);

    assert_eq!(hand_defs(&game, p1), [SPARK, SPARK, SPARK]);
    assert_eq!(deck_defs(&game, p1), [BOLT, BOLT]);
    assert_eq!(
        hand_defs(&game, p0),
        [SPARK, BOLT, RECRUIT, CAPTAIN],
        "three dealt, then player 0's first turn draws the fourth"
    );
    assert_eq!(deck_defs(&game, p0), [BLAST]);
}

#[test]
fn setup_starts_player_0s_turn() {
    let game = opening();
    let [p0, p1] = players(&game);

    assert_eq!(game.outcome(), None);
    assert_eq!(game.mana(p0), 1, "player 0's first turn start gives 1 mana");
    assert_eq!(game.mana(p1), 0, "player 1's first turn hasn't started");
    let spark = in_hand(&game, p0, SPARK);
    assert_actions(&game, p0, &[play(spark), Action::EndTurn]);
    assert_actions(&game, p1, &[]);
}

#[test]
fn each_player_starts_with_a_hero() {
    let game = opening();

    for p in players(&game) {
        assert_eq!(game.def_id(game.hero_id(p)), HERO, "{p:?}");
    }
}

#[test]
fn each_hero_is_an_object_apart_from_every_card() {
    let game = opening();
    let [p0, p1] = players(&game);

    let cards: Vec<ObjectId> = [p0, p1]
        .into_iter()
        .flat_map(|p| [game.hand(p), &game.deck(p)].concat())
        .collect();
    assert_ne!(game.hero_id(p0), game.hero_id(p1));
    for p in [p0, p1] {
        assert!(
            !cards.contains(&game.hero_id(p)),
            "{p:?}'s hero shares an id with a card"
        );
    }
}

#[test]
fn heroes_start_at_ten_health() {
    let game = opening();
    let [p0, p1] = players(&game);

    assert_eq!(game.hero_health(p0), 10);
    assert_eq!(game.hero_health(p1), 10);
}

#[test]
fn a_new_game_has_empty_boards_and_nothing_revealed() {
    let game = opening();

    for p in players(&game) {
        assert!(game.board(p).is_empty(), "{p:?}'s board");
        assert!(game.revealed(p).is_empty(), "{p:?}'s reveal");
    }
}

#[test]
fn copies_of_one_card_are_distinct_objects() {
    let deck = vec![RECRUIT; 8];
    let game = Game::with_deck_order(0, [deck.clone(), deck]);

    let mut ids: Vec<ObjectId> = players(&game)
        .into_iter()
        .flat_map(|p| [game.hand(p), &game.deck(p)].concat())
        .collect();
    assert_eq!(ids.len(), 16);
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 16, "two cards share an ObjectId");
}

#[test]
fn each_copy_in_hand_is_its_own_play_action() {
    let deck = vec![RECRUIT; 8];
    let mut game = Game::with_deck_order(0, [deck.clone(), deck]);
    let [p0, _] = players(&game);
    turn_with_mana(&mut game, p0, 2);

    let mut expected: Vec<Action> = game.hand(p0).iter().map(|&o| play(o)).collect();
    assert_eq!(expected.len(), 5, "three dealt plus two turn draws");
    expected.push(Action::EndTurn);
    assert_actions(&game, p0, &expected);
}

#[test]
fn new_shuffles_the_decks() {
    let unshuffled = Game::with_deck_order(0, [sample_deck(), sample_deck()]);
    let [p0, _] = players(&unshuffled);
    let differs = (0..20).any(|seed| {
        let g = Game::new(seed, [sample_deck(), sample_deck()]);
        let [g0, _] = players(&g);
        hand_defs(&g, g0) != hand_defs(&unshuffled, p0)
            || deck_defs(&g, g0) != deck_defs(&unshuffled, p0)
    });
    assert!(differs, "20 seeds and never a different deck order");
}
