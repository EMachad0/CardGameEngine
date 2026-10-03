//! P on exact positions. An unlisted action returns the exact `IllegalAction` and
//! leaves the game unchanged.

use rules::cards::{BLAST, BOLT, CAPTAIN, FORAGE, GIANT, RECRUIT, SPARK};
use rules::{Action, ApplyError, Game, IllegalAction, Outcome};

use crate::support::*;

#[test]
fn acting_on_the_opponents_turn_is_rejected_and_changes_nothing() {
    let deck = vec![SPARK; 6];
    let game = Game::with_deck_order(0, [deck.clone(), deck]);
    let [p0, p1] = players(&game);
    let own_card = game.hand(p1)[0];
    let opponents_card = game.hand(p0)[0];

    for a in [Action::EndTurn, play(own_card), play(opponents_card)] {
        let mut g = game.clone();
        assert_eq!(
            g.apply(p1, a.clone(), &mut ()),
            Err(ApplyError::IllegalAction(IllegalAction {
                player_id: p1,
                action: a
            }))
        );
        assert_eq!(g, game);
    }
}

#[test]
fn only_affordable_cards_in_the_hand_can_be_played() {
    let deck = vec![SPARK, BOLT, CAPTAIN, BLAST, GIANT];
    let game = Game::with_deck_order(0, [deck.clone(), deck]);
    let [p0, _] = players(&game);

    assert_actions(
        &game,
        p0,
        &[play(in_hand(&game, p0, SPARK)), Action::EndTurn],
    );
    assert_unlisted_rejected(&game);
}

#[test]
fn a_pending_pick_rejects_everything_else() {
    let deck0 = vec![FORAGE, SPARK, SPARK, SPARK, RECRUIT, CAPTAIN, BOLT];
    let mut game = Game::with_deck_order(0, [deck0, vec![SPARK; 6]]);
    let [p0, _] = players(&game);
    game.apply(p0, play(in_hand(&game, p0, FORAGE)), &mut ())
        .unwrap();

    let revealed = game.revealed(p0);
    assert_actions(&game, p0, &[pick(revealed[0]), pick(revealed[1])]);
    assert_unlisted_rejected(&game);
}

#[test]
fn a_card_that_left_every_zone_cant_be_played_again() {
    let deck = vec![SPARK; 8];
    let mut game = Game::with_deck_order(0, [deck.clone(), deck]);
    let [p0, _] = players(&game);
    let spark = in_hand(&game, p0, SPARK);
    game.apply(p0, play(spark), &mut ()).unwrap();
    end_turn(&mut game);
    end_turn(&mut game);

    let before = game.clone();
    assert_eq!(
        game.apply(p0, play(spark), &mut ()),
        Err(ApplyError::IllegalAction(IllegalAction {
            player_id: p0,
            action: play(spark)
        }))
    );
    assert_eq!(game, before);
}

#[test]
fn a_finished_game_rejects_everything() {
    let mut game = Game::with_deck_order(0, [vec![BOLT; 30], vec![BOLT; 3]]);
    let [p0, _] = players(&game);
    end_turns_until_over(&mut game);

    assert_eq!(game.outcome(), Some(Outcome::Won(p0)));
    assert_unlisted_rejected(&game);
}

#[test]
fn applied_returns_the_next_game_without_changing_the_original() {
    let deck = vec![SPARK; 6];
    let game = Game::with_deck_order(0, [deck.clone(), deck]);
    let [p0, p1] = players(&game);
    let snapshot = game.clone();

    let next = game.applied(p0, play(in_hand(&game, p0, SPARK))).unwrap();

    assert_eq!(game, snapshot);
    assert_eq!(next.hero_health(p1), 9, "the Spark resolved in the copy");
}

#[test]
fn applied_rejects_an_unlisted_action() {
    let deck = vec![SPARK; 6];
    let game = Game::with_deck_order(0, [deck.clone(), deck]);
    let [_, p1] = players(&game);

    assert_eq!(
        game.applied(p1, Action::EndTurn),
        Err(ApplyError::IllegalAction(IllegalAction {
            player_id: p1,
            action: Action::EndTurn
        }))
    );
}
