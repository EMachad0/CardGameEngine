//! Setup: the shuffle, the opening deal and the first turn.

use rules::cards::{BLAST, BOLT, CAPTAIN, FORAGE, GIANT, RECRUIT, SPARK, WILD_BOLT};
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

    assert_eq!(hand_defs(&game, P1), [SPARK, SPARK, SPARK]);
    assert_eq!(deck_defs(&game, P1), [BOLT, BOLT]);
    assert_eq!(
        hand_defs(&game, P0),
        [SPARK, BOLT, RECRUIT, CAPTAIN],
        "three dealt, then player 0's first turn draws the fourth"
    );
    assert_eq!(deck_defs(&game, P0), [BLAST]);
}

#[test]
fn setup_starts_player_0s_turn() {
    let game = opening();

    assert_eq!(game.outcome(), None);
    assert_eq!(game.mana(P0), 1, "player 0's first turn start gives 1 mana");
    assert_eq!(game.mana(P1), 0, "player 1's first turn hasn't started");
    let spark = in_hand(&game, P0, SPARK);
    assert_actions(&game, P0, &[play(spark), Action::EndTurn]);
    assert_actions(&game, P1, &[]);
}

#[test]
fn heroes_start_at_ten_health() {
    let game = opening();

    assert_eq!(game.hero_health(P0), 10);
    assert_eq!(game.hero_health(P1), 10);
}

#[test]
fn a_new_game_has_empty_boards_and_nothing_revealed() {
    let game = opening();

    for p in PLAYERS {
        assert!(game.board(p).is_empty(), "{p:?}'s board");
        assert!(game.revealed(p).is_empty(), "{p:?}'s reveal");
    }
}

#[test]
fn copies_of_one_card_are_distinct_objects() {
    let deck = vec![RECRUIT; 8];
    let game = Game::with_deck_order(0, [deck.clone(), deck]);

    let mut ids: Vec<ObjectId> = PLAYERS
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
    turn_with_mana(&mut game, P0, 2);

    let mut expected: Vec<Action> = game.hand(P0).iter().map(|&o| play(o)).collect();
    assert_eq!(expected.len(), 5, "three dealt plus two turn draws");
    expected.push(Action::EndTurn);
    assert_actions(&game, P0, &expected);
}

#[test]
fn new_shuffles_the_decks() {
    let unshuffled = Game::with_deck_order(0, [sample_deck(), sample_deck()]);
    let differs = (0..20).any(|seed| {
        let g = Game::new(seed, [sample_deck(), sample_deck()]);
        hand_defs(&g, P0) != hand_defs(&unshuffled, P0)
            || deck_defs(&g, P0) != deck_defs(&unshuffled, P0)
    });
    assert!(differs, "20 seeds and never a different deck order");
}
