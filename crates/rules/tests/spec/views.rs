//! What `view` shows each player.

use pretty_assertions::assert_eq;
use rules::prelude::static_card_definition::{
    BOLT, CAPTAIN, HERO, MUSTER, PING, RECRUIT, RECYCLE, ROUSE, SPARK, TWIN_SHOT,
};
use rules::prelude::{
    BoardCard, ChooserPicking, DefId, Face, Game, HeroCard, ObjectId, PickingCard, PickingView,
    PlayerId, TargetingView,
};

use crate::support::*;

/// Player 0 holds Bolt, Recruit, Captain and a drawn Bolt. Player 1 holds Spark, Spark, Recruit.
fn after_setup() -> Game {
    let deck0 = deck_with_top(&[BOLT, RECRUIT, CAPTAIN]);
    let deck1 = deck_with_top(&[SPARK, SPARK, RECRUIT]);
    Game::with_deck_order(0, [deck0, deck1])
}

fn face(def_id: DefId, mana_cost: u8) -> Option<Face> {
    Some(Face { def_id, mana_cost })
}

#[test]
fn a_view_names_its_viewer_whose_turn_it_is_and_each_player_by_id() {
    let game = after_setup();
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
    let game = after_setup();
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
    let game = after_setup();
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
    let game = after_setup();

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

/// Player 0 casts Muster over a Bolt, a Recruit and a Captain. Returns the game and the three.
fn muster_over_bolt_recruit_and_captain() -> (Game, [ObjectId; 3]) {
    let deck0 = vec![MUSTER, SPARK, SPARK, SPARK, BOLT, RECRUIT, CAPTAIN, BOLT];
    let mut game = Game::with_deck_order(0, [deck0, vec![SPARK; 6]]);
    let [p0, _] = players(&game);
    let deck = game.deck(p0);
    play_def(&mut game, p0, MUSTER);
    (game, [deck[0], deck[1], deck[2]])
}

#[test]
fn the_chooser_sees_every_card_a_pick_covers_face_up_in_the_order_shown() {
    let (game, [bolt, recruit, captain]) = muster_over_bolt_recruit_and_captain();
    let [p0, _] = players(&game);

    assert_eq!(
        player_view(&game.view(p0), p0).picking,
        Some(PickingView::Chooser(ChooserPicking::Cards(vec![
            PickingCard {
                object_id: bolt,
                def_id: BOLT,
                pickable: false
            },
            PickingCard {
                object_id: recruit,
                def_id: RECRUIT,
                pickable: true
            },
            PickingCard {
                object_id: captain,
                def_id: CAPTAIN,
                pickable: true
            },
        ])))
    );
}

#[test]
fn every_other_player_sees_only_how_many_options_a_pick_has() {
    let (game, _) = muster_over_bolt_recruit_and_captain();
    let [p0, p1] = players(&game);

    assert_eq!(
        player_view(&game.view(p1), p0).picking,
        Some(PickingView::Other { option_count: 2 })
    );
}

/// Player 0 casts Rouse with two Recruits on their board and one on player 1's.
fn rouse_over_three_recruits() -> Table {
    let mut t = table(&[ROUSE], 2, 1);
    play_def(&mut t.game, t.p0, ROUSE);
    t
}

#[test]
fn the_chooser_sees_a_character_picks_options_in_the_order_shown() {
    let t = rouse_over_three_recruits();

    assert_eq!(
        player_view(&t.game.view(t.p0), t.p0).picking,
        Some(PickingView::Chooser(ChooserPicking::Characters(vec![
            t.friendly[0],
            t.friendly[1],
            t.enemy[0],
        ])))
    );
}

#[test]
fn every_other_player_sees_only_how_many_options_a_character_pick_has() {
    let t = rouse_over_three_recruits();

    assert_eq!(
        player_view(&t.game.view(t.p1), t.p0).picking,
        Some(PickingView::Other { option_count: 3 })
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
fn both_players_see_the_card_in_targeting_and_what_it_targeted_so_far() {
    let mut t = table(&[TWIN_SHOT], 0, 2);
    let twin_shot = in_hand(&t.game, t.p0, TWIN_SHOT);
    apply_all(
        &mut t.game,
        t.p0,
        &[start_targeting(twin_shot), target(0, t.enemy[1])],
    );

    for viewer in players(&t.game) {
        let view = t.game.view(viewer);
        assert_eq!(
            player_view(&view, t.p0).targeting,
            Some(TargetingView {
                object_id: twin_shot,
                targeted: vec![t.enemy[1]],
            }),
            "{viewer:?}"
        );
        assert_eq!(player_view(&view, t.p1).targeting, None, "{viewer:?}");
    }
}

#[test]
fn the_opponent_still_sees_no_face_on_the_card_in_targeting() {
    let mut t = table(&[PING], 0, 1);
    let ping = in_hand(&t.game, t.p0, PING);
    apply_all(&mut t.game, t.p0, &[start_targeting(ping)]);

    let view = t.game.view(t.p1);
    let card = player_view(&view, t.p0)
        .hand
        .iter()
        .find(|card| card.object_id == ping)
        .expect("the card stays in the hand while targeting");
    assert_eq!(card.face, None);
}

#[test]
fn cancelled_targeting_no_longer_shows() {
    let mut t = table(&[PING], 0, 1);
    let ping = in_hand(&t.game, t.p0, PING);
    apply_all(
        &mut t.game,
        t.p0,
        &[start_targeting(ping), target(0, t.enemy[0]), cancel(ping)],
    );

    for viewer in players(&t.game) {
        assert_eq!(
            player_view(&t.game.view(viewer), t.p0).targeting,
            None,
            "{viewer:?}"
        );
    }
}

#[test]
fn targeting_no_longer_shows_once_the_card_is_played() {
    let mut t = table(&[PING], 0, 1);
    let ping = in_hand(&t.game, t.p0, PING);
    apply_all(
        &mut t.game,
        t.p0,
        &[start_targeting(ping), target(0, t.enemy[0]), play(ping)],
    );

    for viewer in players(&t.game) {
        assert_eq!(
            player_view(&t.game.view(viewer), t.p0).targeting,
            None,
            "{viewer:?}"
        );
    }
}

#[test]
fn both_players_see_the_hand_card_recycle_targeted() {
    let mut t = table(&[RECYCLE, RECRUIT], 0, 0);
    let recycle = in_hand(&t.game, t.p0, RECYCLE);
    let recruit = in_hand(&t.game, t.p0, RECRUIT);
    apply_all(
        &mut t.game,
        t.p0,
        &[start_targeting(recycle), target(0, recruit)],
    );

    for viewer in players(&t.game) {
        assert_eq!(
            player_view(&t.game.view(viewer), t.p0).targeting,
            Some(TargetingView {
                object_id: recycle,
                targeted: vec![recruit],
            }),
            "{viewer:?}"
        );
    }
}

#[test]
fn the_opponent_sees_no_face_on_a_targeted_hand_card() {
    let mut t = table(&[RECYCLE, RECRUIT], 0, 0);
    let recycle = in_hand(&t.game, t.p0, RECYCLE);
    let recruit = in_hand(&t.game, t.p0, RECRUIT);
    apply_all(
        &mut t.game,
        t.p0,
        &[start_targeting(recycle), target(0, recruit)],
    );

    let view = t.game.view(t.p1);
    let card = player_view(&view, t.p0)
        .hand
        .iter()
        .find(|card| card.object_id == recruit)
        .expect("the targeted card stays in the hand while targeting");
    assert_eq!(card.face, None);
}

#[test]
fn the_owner_still_sees_the_face_of_a_targeted_hand_card() {
    let mut t = table(&[RECYCLE, RECRUIT], 0, 0);
    let recycle = in_hand(&t.game, t.p0, RECYCLE);
    let recruit = in_hand(&t.game, t.p0, RECRUIT);
    apply_all(
        &mut t.game,
        t.p0,
        &[start_targeting(recycle), target(0, recruit)],
    );

    let view = t.game.view(t.p0);
    let card = player_view(&view, t.p0)
        .hand
        .iter()
        .find(|card| card.object_id == recruit)
        .expect("the targeted card stays in the hand while targeting");
    assert_eq!(card.face, face(RECRUIT, 2));
}
