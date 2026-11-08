//! Setup: the shuffle, the first draws and the first turn.

use pretty_assertions::{assert_eq, assert_ne};
use rules::prelude::{
    Action, Binder, CardDef, CardKindDef, Deck, DeckError, DefCode, Face, Game, HeroCardDef,
    ObjectId, Set, SetCard,
};

use crate::support::*;
use crate::test_set::{BLAST, BOLT, CAPTAIN, FORAGE, GIANT, HERO, RECRUIT, SPARK, WILD_BOLT};

fn sample_deck() -> Vec<DefCode> {
    vec![
        SPARK, BOLT, FORAGE, RECRUIT, WILD_BOLT, CAPTAIN, BLAST, GIANT, SPARK, RECRUIT,
    ]
}

fn after_setup() -> Game {
    let deck0 = vec![SPARK, BOLT, RECRUIT, CAPTAIN, BLAST];
    let deck1 = vec![SPARK, SPARK, SPARK, BOLT, BOLT];
    game_with_deck_order(0, [deck0, deck1])
}

#[test]
fn setup_draws_three_cards_each_from_the_top() {
    let game = after_setup();
    let [p0, p1] = players(&game);

    assert_eq!(hand_defs(&game, p1), [SPARK, SPARK, SPARK]);
    assert_eq!(deck_defs(&game, p1), [BOLT, BOLT]);
    assert_eq!(
        hand_defs(&game, p0),
        [SPARK, BOLT, RECRUIT, CAPTAIN],
        "three drawn, then player 0's first turn draws the fourth"
    );
    assert_eq!(deck_defs(&game, p0), [BLAST]);
}

#[test]
fn setup_starts_player_0s_turn() {
    let game = after_setup();
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
    let game = after_setup();

    for p in players(&game) {
        assert_eq!(game.def_code(game.hero_id(p)), HERO, "{p:?}");
    }
}

#[test]
fn each_hero_is_an_object_apart_from_every_card() {
    let game = after_setup();
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
    let game = after_setup();
    let [p0, p1] = players(&game);

    assert_eq!(game.hero_health(p0), 10);
    assert_eq!(game.hero_health(p1), 10);
}

#[test]
fn a_new_game_has_empty_boards_and_no_pick_open() {
    let game = after_setup();

    for p in players(&game) {
        assert!(game.player_board(p).is_empty(), "{p:?}'s board");
        assert!(game.pick_options(p).is_empty(), "{p:?}'s pick options");
    }
}

#[test]
fn copies_of_one_card_are_distinct_objects() {
    let deck = vec![RECRUIT; 8];
    let game = game_with_deck_order(0, [deck.clone(), deck]);

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
    let mut game = game_with_deck_order(0, [deck.clone(), deck]);
    let [p0, _] = players(&game);
    turn_with_mana(&mut game, p0, 2);

    let mut expected: Vec<Action> = game.hand(p0).iter().map(|&o| play(o)).collect();
    assert_eq!(expected.len(), 5, "three drawn plus two turn draws");
    expected.push(Action::EndTurn);
    assert_actions(&game, p0, &expected);
}

#[test]
fn new_shuffles_the_decks() {
    let unshuffled = game_with_deck_order(0, [sample_deck(), sample_deck()]);
    let [p0, _] = players(&unshuffled);
    let differs = (0..20).any(|seed| {
        let g = new_game(seed, [sample_deck(), sample_deck()]);
        let [g0, _] = players(&g);
        hand_defs(&g, g0) != hand_defs(&unshuffled, p0)
            || deck_defs(&g, g0) != deck_defs(&unshuffled, p0)
    });
    assert!(differs, "20 seeds and never a different deck order");
}

const MISSING: DefCode = DefCode {
    set: "test",
    name: "missing",
    version: 0,
};

fn hero_card(name: &'static str) -> SetCard {
    SetCard {
        name,
        version: 0,
        def: CardDef {
            kind: CardKindDef::Hero(HeroCardDef { health: 10 }),
            ..Default::default()
        },
    }
}

fn start(decks: [Deck; 2]) -> Result<Game, DeckError> {
    Game::new(0, test_binder(), decks)
}

#[test]
fn a_deck_card_that_names_no_card_fails_to_start_a_game() {
    let result = start([deck(vec![BOLT, MISSING]), deck(vec![BOLT])]);

    assert_eq!(result.map(drop), Err(DeckError::NotFound(MISSING)));
}

#[test]
fn a_hero_that_names_no_card_fails_to_start_a_game() {
    let no_hero = Deck {
        hero: MISSING,
        cards: vec![BOLT],
    };

    let result = start([deck(vec![BOLT]), no_hero]);

    assert_eq!(result.map(drop), Err(DeckError::NotFound(MISSING)));
}

#[test]
fn a_hero_that_names_a_minion_or_a_spell_fails_to_start_a_game() {
    for not_a_hero in [RECRUIT, BOLT] {
        let deck_with_hero = Deck {
            hero: not_a_hero,
            cards: vec![BOLT],
        };

        let result = start([deck_with_hero, deck(vec![BOLT])]);

        assert_eq!(
            result.map(drop),
            Err(DeckError::NotAHero(not_a_hero)),
            "{not_a_hero}"
        );
    }
}

#[test]
fn a_deck_built_for_one_binder_fails_against_a_binder_without_its_codes() {
    let other = Binder::new([Set {
        prefix: "other",
        cards: vec![hero_card("hero")],
    }])
    .expect("the other set should parse");

    let result = Game::new(0, other, [deck(vec![BOLT]), deck(vec![BOLT])]);

    assert_eq!(result.map(drop), Err(DeckError::NotFound(HERO)));
}

#[test]
fn each_player_gets_the_hero_their_deck_names() {
    let rival = DefCode {
        set: "p",
        name: "rival",
        version: 0,
    };
    let binder = binder_with(Set {
        prefix: "p",
        cards: vec![hero_card("rival")],
    });
    let rival_deck = Deck {
        hero: rival,
        cards: vec![BOLT; 4],
    };

    let game = Game::with_deck_order(0, binder, [deck(vec![BOLT; 4]), rival_deck])
        .expect("the decks name cards in the binder");

    let [p0, p1] = players(&game);
    assert_eq!(
        [p0, p1].map(|p| game.def_code(game.hero_id(p))),
        [HERO, rival],
    );
}

#[test]
fn a_card_in_hand_is_viewed_with_the_code_its_deck_named() {
    let game = game_with_deck_order(0, [vec![SPARK; 4], vec![BOLT; 4]]);
    let [p0, _] = players(&game);

    let view = game.view(p0);

    assert_eq!(
        player_view(&view, p0).hand[0].face,
        Some(Face {
            def_code: SPARK,
            mana_cost: 1,
        }),
    );
}
