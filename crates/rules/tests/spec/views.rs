//! What `view` shows each player.

use rules::static_card_definition::{BOLT, CAPTAIN, FORAGE, HERO, PING, RECRUIT, SPARK, TWIN_SHOT};
use rules::{BoardCard, DefId, DraftView, Face, Game, HeroCard, ObjectId, PlayerId, RevealedCard};

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
    let [p0, _] = players(&game);

    for viewer in players(&game) {
        let view = game.view(viewer);
        assert_eq!(view.viewer, viewer);
        assert_eq!(view.active_player, p0, "{viewer:?}");
        let ids: Vec<PlayerId> = view.players.iter().map(|p| p.player_id).collect();
        assert_eq!(ids.len(), 2, "{viewer:?}");
        for p in players(&game) {
            assert!(ids.contains(&p), "{viewer:?} sees no entry for {p:?}");
        }
        assert_eq!(view.outcome, None, "{viewer:?}");
    }
}

#[test]
fn a_player_sees_their_own_hand_with_each_cards_definition_and_cost() {
    let game = opening();
    let [p0, _] = players(&game);

    let view = game.view(p0);
    let hand = &player_view(&view, p0).hand;

    let ids: Vec<ObjectId> = hand.iter().map(|card| card.object_id).collect();
    assert_eq!(ids, game.hand(p0));
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
    let [p0, p1] = players(&game);

    let view = game.view(p1);
    let hand = &player_view(&view, p0).hand;

    let ids: Vec<ObjectId> = hand.iter().map(|card| card.object_id).collect();
    assert_eq!(ids, game.hand(p0));
    assert!(
        hand.iter().all(|card| card.face.is_none()),
        "player 1 sees a face in player 0's hand: {hand:?}"
    );
}

#[test]
fn heroes_mana_and_deck_sizes_are_public() {
    let game = opening();

    for viewer in players(&game) {
        let view = game.view(viewer);
        let rows: Vec<(i32, u8, u8, usize)> = players(&game)
            .iter()
            .map(|&p| player_view(&view, p))
            .map(|p| (p.hero.health, p.mana, p.max_mana, p.deck_size))
            .collect();
        assert_eq!(
            rows,
            [(10, 1, 1, 19), (10, 0, 0, 20)],
            "(health, mana, max mana, deck size) as {viewer:?} sees them"
        );
    }
}

#[test]
fn both_players_see_each_hero_with_its_definition_and_current_health() {
    let mut game = Game::with_deck_order(0, [deck_with_top(&[SPARK]), deck_with_top(&[])]);
    let [p0, p1] = players(&game);
    play_def(&mut game, p0, SPARK);

    for viewer in players(&game) {
        let view = game.view(viewer);
        for (p, health) in [(p0, 10), (p1, 9)] {
            assert_eq!(
                player_view(&view, p).hero,
                HeroCard {
                    object_id: game.hero_id(p),
                    def_id: HERO,
                    health
                },
                "{p:?}'s hero as {viewer:?} sees it"
            );
        }
    }
}

#[test]
fn both_players_see_each_minions_definition_and_current_stats() {
    let deck0 = deck_with_top(&[RECRUIT, CAPTAIN]);
    let mut game = Game::with_deck_order(0, [deck0, deck_with_top(&[])]);
    let [p0, _] = players(&game);
    turn_with_mana(&mut game, p0, 2);
    let recruit = summon(&mut game, p0, RECRUIT);
    turn_with_mana(&mut game, p0, 3);
    let captain = summon(&mut game, p0, CAPTAIN);

    for viewer in players(&game) {
        assert_eq!(
            player_view(&game.view(viewer), p0).board,
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
    let [p0, p1] = players(&game);
    play_def(&mut game, p0, FORAGE);
    let &[recruit, captain] = game.revealed(p0) else {
        panic!("Forage revealed {:?}", game.revealed(p0));
    };

    let shown = |viewer: PlayerId| player_view(&game.view(viewer), p0).revealed.clone();

    assert_eq!(
        shown(p0),
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
        shown(p1),
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

    for viewer in players(&game) {
        assert_eq!(game.view(viewer).outcome, game.outcome(), "{viewer:?}");
    }
}

#[test]
fn both_players_see_the_drafted_card_and_the_targets_chosen_so_far() {
    let mut t = table(&[TWIN_SHOT], 0, 2);
    let twin_shot = in_hand(&t.game, t.p0, TWIN_SHOT);
    apply_all(
        &mut t.game,
        t.p0,
        &[draft(twin_shot), choose(0, t.enemy[1])],
    );

    for viewer in players(&t.game) {
        let view = t.game.view(viewer);
        assert_eq!(
            player_view(&view, t.p0).draft,
            Some(DraftView {
                object_id: twin_shot,
                chosen: vec![t.enemy[1]],
            }),
            "{viewer:?}"
        );
        assert_eq!(player_view(&view, t.p1).draft, None, "{viewer:?}");
    }
}

#[test]
fn the_opponent_still_sees_no_face_on_the_drafted_card() {
    let mut t = table(&[PING], 0, 1);
    let ping = in_hand(&t.game, t.p0, PING);
    apply_all(&mut t.game, t.p0, &[draft(ping)]);

    let view = t.game.view(t.p1);
    let card = player_view(&view, t.p0)
        .hand
        .iter()
        .find(|card| card.object_id == ping)
        .expect("the drafted card stays in the hand");
    assert_eq!(card.face, None);
}

#[test]
fn a_cancelled_draft_no_longer_shows() {
    let mut t = table(&[PING], 0, 1);
    let ping = in_hand(&t.game, t.p0, PING);
    apply_all(
        &mut t.game,
        t.p0,
        &[draft(ping), choose(0, t.enemy[0]), cancel(ping)],
    );

    for viewer in players(&t.game) {
        assert_eq!(
            player_view(&t.game.view(viewer), t.p0).draft,
            None,
            "{viewer:?}"
        );
    }
}

#[test]
fn a_played_draft_no_longer_shows() {
    let mut t = table(&[PING], 0, 1);
    let ping = in_hand(&t.game, t.p0, PING);
    apply_all(
        &mut t.game,
        t.p0,
        &[draft(ping), choose(0, t.enemy[0]), play(ping)],
    );

    for viewer in players(&t.game) {
        assert_eq!(
            player_view(&t.game.view(viewer), t.p0).draft,
            None,
            "{viewer:?}"
        );
    }
}
