//! What `view` shows each player (node D3).

use rules::cards::{BOLT, CAPTAIN, FORAGE, RECRUIT, SPARK};
use rules::{BoardCard, DefId, Face, Game, ObjectId, PlayerId, RevealedCard};

use crate::support::*;

/// Player 0 holds Bolt, Recruit, Captain and a drawn Bolt. Player 1 holds Spark, Spark, Recruit.
fn opening() -> Game {
    let deck0 = deck_with_top(&[BOLT, RECRUIT, CAPTAIN]);
    let deck1 = deck_with_top(&[SPARK, SPARK, RECRUIT]);
    Game::with_deck_order(0, [deck0, deck1])
}

fn face(def_id: DefId, mana_cost: u8) -> Option<Face> {
    Some(Face { def_id, mana_cost })
}

#[test]
fn a_view_names_its_viewer_whose_turn_it_is_and_each_player_by_id() {
    let game = opening();

    for viewer in PLAYERS {
        let view = game.view(viewer);
        assert_eq!(view.viewer, viewer);
        assert_eq!(view.active_player, P0, "{viewer:?}");
        let ids: Vec<PlayerId> = view.players.iter().map(|p| p.player_id).collect();
        assert_eq!(ids, PLAYERS, "{viewer:?}");
        assert_eq!(view.outcome, None, "{viewer:?}");
    }
}

#[test]
fn a_player_sees_their_own_hand_with_each_cards_definition_and_cost() {
    let game = opening();

    let hand = &game.view(P0).players[P0.idx()].hand;

    let ids: Vec<ObjectId> = hand.iter().map(|card| card.object_id).collect();
    assert_eq!(ids, game.hand(P0));
    let faces: Vec<Option<Face>> = hand.iter().map(|card| card.face).collect();
    assert_eq!(
        faces,
        [
            face(BOLT, 2),
            face(RECRUIT, 2),
            face(CAPTAIN, 3),
            face(BOLT, 2)
        ]
    );
}

#[test]
fn a_player_sees_only_the_ids_of_the_opponents_hand() {
    let game = opening();

    let hand = &game.view(P1).players[P0.idx()].hand;

    let ids: Vec<ObjectId> = hand.iter().map(|card| card.object_id).collect();
    assert_eq!(ids, game.hand(P0));
    assert!(
        hand.iter().all(|card| card.face.is_none()),
        "player 1 sees a face in player 0's hand: {hand:?}"
    );
}

#[test]
fn heroes_mana_and_deck_sizes_are_public() {
    let game = opening();

    for viewer in PLAYERS {
        let rows: Vec<(i32, u8, u8, usize)> = game
            .view(viewer)
            .players
            .iter()
            .map(|p| (p.hero_health, p.mana, p.max_mana, p.deck_size))
            .collect();
        assert_eq!(
            rows,
            [(10, 1, 1, 19), (10, 0, 0, 20)],
            "(health, mana, max mana, deck size) as {viewer:?} sees them"
        );
    }
}

#[test]
fn both_players_see_each_minions_definition_and_current_stats() {
    let deck0 = deck_with_top(&[RECRUIT, CAPTAIN]);
    let mut game = Game::with_deck_order(0, [deck0, deck_with_top(&[])]);
    turn_with_mana(&mut game, P0, 2);
    let recruit = summon(&mut game, P0, RECRUIT);
    turn_with_mana(&mut game, P0, 3);
    let captain = summon(&mut game, P0, CAPTAIN);

    for viewer in PLAYERS {
        assert_eq!(
            game.view(viewer).players[P0.idx()].board,
            [
                BoardCard {
                    object_id: recruit,
                    def_id: RECRUIT,
                    attack: 3,
                    health: 3
                },
                BoardCard {
                    object_id: captain,
                    def_id: CAPTAIN,
                    attack: 1,
                    health: 1
                },
            ],
            "the Recruit carries the Captain's buff, as {viewer:?} sees it"
        );
    }
}

#[test]
fn only_the_player_choosing_sees_a_pending_forages_options() {
    let deck0 = vec![FORAGE, SPARK, SPARK, SPARK, RECRUIT, CAPTAIN, BOLT];
    let mut game = Game::with_deck_order(0, [deck0, vec![SPARK; 6]]);
    play_def(&mut game, P0, FORAGE);
    let &[recruit, captain] = game.revealed(P0) else {
        panic!("Forage revealed {:?}", game.revealed(P0));
    };

    let shown = |viewer: PlayerId| game.view(viewer).players[P0.idx()].revealed.clone();

    assert_eq!(
        shown(P0),
        [
            RevealedCard {
                object_id: recruit,
                def_id: Some(RECRUIT)
            },
            RevealedCard {
                object_id: captain,
                def_id: Some(CAPTAIN)
            },
        ]
    );
    assert_eq!(
        shown(P1),
        [
            RevealedCard {
                object_id: recruit,
                def_id: None
            },
            RevealedCard {
                object_id: captain,
                def_id: None
            },
        ]
    );
}

#[test]
fn a_view_reports_the_outcome_once_the_game_is_over() {
    let mut game = Game::with_deck_order(0, [vec![SPARK; 3], vec![SPARK; 3]]);

    end_turns_until_over(&mut game);

    for viewer in PLAYERS {
        assert_eq!(game.view(viewer).outcome, game.outcome(), "{viewer:?}");
    }
}
