//! Targeting: a card that needs targets starts targeting, its targets are named one at a time, and
//! then it is played.

use pretty_assertions::assert_eq;
use rules::prelude::Action;
use rules::prelude::static_card_definition::{CROSSFIRE, PING, SHOVE, TWIN_SHOT};

use crate::support::*;

#[test]
fn a_card_with_targets_is_offered_as_targeting_and_not_as_a_play() {
    let t = table(&[PING], 0, 1);
    let ping = in_hand(&t.game, t.p0, PING);
    let legal = t.game.legal_actions(t.p0);

    assert!(legal.contains(&start_targeting(ping)), "{legal:?}");
    assert!(!legal.contains(&play(ping)), "{legal:?}");
}

#[test]
fn ping_is_not_offered_with_no_minion_on_the_board() {
    let t = table(&[PING], 0, 0);
    let ping = in_hand(&t.game, t.p0, PING);

    assert!(!t.game.legal_actions(t.p0).contains(&start_targeting(ping)));
}

#[test]
fn ping_can_target_a_friendly_or_an_enemy_minion() {
    let mut t = table(&[PING], 1, 1);
    let ping = in_hand(&t.game, t.p0, PING);
    apply_all(&mut t.game, t.p0, &[start_targeting(ping)]);

    assert_actions(
        &t.game,
        t.p0,
        &[
            target(0, t.friendly[0]),
            target(0, t.enemy[0]),
            cancel(ping),
        ],
    );
}

#[test]
fn twin_shot_is_not_offered_with_one_minion_on_the_board() {
    let t = table(&[TWIN_SHOT], 0, 1);
    let twin_shot = in_hand(&t.game, t.p0, TWIN_SHOT);

    assert!(
        !t.game
            .legal_actions(t.p0)
            .contains(&start_targeting(twin_shot))
    );
}

#[test]
fn twin_shot_is_offered_with_two_minions_on_the_board() {
    let t = table(&[TWIN_SHOT], 0, 2);
    let twin_shot = in_hand(&t.game, t.p0, TWIN_SHOT);

    assert!(
        t.game
            .legal_actions(t.p0)
            .contains(&start_targeting(twin_shot))
    );
}

#[test]
fn twin_shots_second_target_skips_the_minion_already_targeted() {
    let mut t = table(&[TWIN_SHOT], 0, 2);
    let twin_shot = in_hand(&t.game, t.p0, TWIN_SHOT);
    apply_all(
        &mut t.game,
        t.p0,
        &[start_targeting(twin_shot), target(0, t.enemy[0])],
    );

    assert_actions(&t.game, t.p0, &[target(0, t.enemy[1]), cancel(twin_shot)]);
}

#[test]
fn shove_can_start_targeting_with_an_enemy_minion_and_one_friendly_minion() {
    let t = table(&[SHOVE], 1, 1);
    let shove = in_hand(&t.game, t.p0, SHOVE);

    assert!(
        t.game.legal_actions(t.p0).contains(&start_targeting(shove)),
        "the enemy Recruit can fill the first target and P0's Recruit the second",
    );
}

#[test]
fn shove_is_not_offered_with_one_friendly_minion_and_no_other() {
    let t = table(&[SHOVE], 1, 0);
    let shove = in_hand(&t.game, t.p0, SHOVE);

    assert!(
        !t.game.legal_actions(t.p0).contains(&start_targeting(shove)),
        "the second target needs a friendly minion other than the first target",
    );
}

#[test]
fn shoves_first_target_skips_a_minion_that_would_leave_the_second_target_empty() {
    let mut t = table(&[SHOVE], 1, 1);
    let shove = in_hand(&t.game, t.p0, SHOVE);
    apply_all(&mut t.game, t.p0, &[start_targeting(shove)]);

    assert_actions(&t.game, t.p0, &[target(0, t.enemy[0]), cancel(shove)]);
}

#[test]
fn shoves_second_target_offers_a_friendly_minion_different_from_the_first() {
    let mut t = table(&[SHOVE], 1, 1);
    let shove = in_hand(&t.game, t.p0, SHOVE);
    apply_all(
        &mut t.game,
        t.p0,
        &[start_targeting(shove), target(0, t.enemy[0])],
    );

    assert_actions(&t.game, t.p0, &[target(1, t.friendly[0]), cancel(shove)]);
}

#[test]
fn crossfires_second_target_offers_either_hero_after_the_minion() {
    let mut t = table(&[CROSSFIRE], 0, 1);
    let crossfire = in_hand(&t.game, t.p0, CROSSFIRE);
    apply_all(
        &mut t.game,
        t.p0,
        &[start_targeting(crossfire), target(0, t.enemy[0])],
    );

    assert_actions(
        &t.game,
        t.p0,
        &[
            target(1, t.game.hero_id(t.p0)),
            target(1, t.game.hero_id(t.p1)),
            cancel(crossfire),
        ],
    );
}

#[test]
fn play_is_offered_once_every_target_is_filled() {
    let mut t = table(&[PING], 0, 1);
    let ping = in_hand(&t.game, t.p0, PING);
    apply_all(
        &mut t.game,
        t.p0,
        &[start_targeting(ping), target(0, t.enemy[0])],
    );

    assert_actions(&t.game, t.p0, &[play(ping), cancel(ping)]);
}

#[test]
fn the_opponent_has_no_actions_during_targeting() {
    let mut t = table(&[PING], 0, 1);
    let ping = in_hand(&t.game, t.p0, PING);
    apply_all(&mut t.game, t.p0, &[start_targeting(ping)]);

    assert_actions(&t.game, t.p1, &[]);
}

#[test]
fn starting_targeting_and_targeting_pay_nothing_and_keep_the_card_in_hand() {
    let mut t = table(&[PING], 0, 1);
    let ping = in_hand(&t.game, t.p0, PING);
    let mana = t.game.mana(t.p0);
    apply_all(
        &mut t.game,
        t.p0,
        &[start_targeting(ping), target(0, t.enemy[0])],
    );

    assert_eq!(t.game.mana(t.p0), mana);
    assert!(t.game.hand(t.p0).contains(&ping));
}

#[test]
fn cancel_right_after_start_targeting_leaves_the_game_as_it_was() {
    let mut t = table(&[PING], 0, 1);
    let ping = in_hand(&t.game, t.p0, PING);
    let before = t.game.clone();
    apply_all(&mut t.game, t.p0, &[start_targeting(ping), cancel(ping)]);

    assert_eq!(t.game, before);
}

#[test]
fn cancel_after_a_target_leaves_the_game_as_it_was_before_targeting() {
    let mut t = table(&[TWIN_SHOT], 0, 2);
    let twin_shot = in_hand(&t.game, t.p0, TWIN_SHOT);
    let before = t.game.clone();
    apply_all(
        &mut t.game,
        t.p0,
        &[
            start_targeting(twin_shot),
            target(0, t.enemy[0]),
            cancel(twin_shot),
        ],
    );

    assert_eq!(t.game, before);
}

#[test]
fn playing_a_card_after_targeting_pays_its_cost_and_takes_it_from_the_hand() {
    let mut t = table(&[TWIN_SHOT], 0, 2);
    let twin_shot = in_hand(&t.game, t.p0, TWIN_SHOT);
    let mana = t.game.mana(t.p0);
    apply_all(
        &mut t.game,
        t.p0,
        &[
            start_targeting(twin_shot),
            target(0, t.enemy[0]),
            target(0, t.enemy[1]),
            play(twin_shot),
        ],
    );

    assert_eq!(t.game.mana(t.p0), mana - 2);
    assert!(!t.game.hand(t.p0).contains(&twin_shot));
}

#[test]
fn after_a_card_with_targets_is_played_its_owner_acts_freely_again() {
    let mut t = table(&[PING], 0, 1);
    let ping = in_hand(&t.game, t.p0, PING);
    apply_all(
        &mut t.game,
        t.p0,
        &[start_targeting(ping), target(0, t.enemy[0]), play(ping)],
    );

    assert!(t.game.legal_actions(t.p0).contains(&Action::EndTurn));
}

#[test]
fn ping_deals_2_damage_to_the_targeted_minion_only() {
    let mut t = table(&[PING], 0, 2);
    let ping = in_hand(&t.game, t.p0, PING);
    apply_all(
        &mut t.game,
        t.p0,
        &[start_targeting(ping), target(0, t.enemy[0]), play(ping)],
    );

    assert_eq!(
        t.game.player_board(t.p1),
        [t.enemy[1]],
        "the targeted 2/2 Recruit dies and the other stays"
    );
    assert_eq!(t.game.health(t.enemy[1]), Some(2));
}

#[test]
fn twin_shot_deals_1_damage_to_each_targeted_minion() {
    let mut t = table(&[TWIN_SHOT], 0, 2);
    let twin_shot = in_hand(&t.game, t.p0, TWIN_SHOT);
    apply_all(
        &mut t.game,
        t.p0,
        &[
            start_targeting(twin_shot),
            target(0, t.enemy[1]),
            target(0, t.enemy[0]),
            play(twin_shot),
        ],
    );

    assert_eq!(t.game.health(t.enemy[0]), Some(1));
    assert_eq!(t.game.health(t.enemy[1]), Some(1));
}

#[test]
fn shove_deals_1_damage_to_each_of_its_two_targeted_minions() {
    let mut t = table(&[SHOVE], 1, 1);
    let shove = in_hand(&t.game, t.p0, SHOVE);
    apply_all(
        &mut t.game,
        t.p0,
        &[
            start_targeting(shove),
            target(0, t.enemy[0]),
            target(1, t.friendly[0]),
            play(shove),
        ],
    );

    assert_eq!(t.game.health(t.enemy[0]), Some(1));
    assert_eq!(t.game.health(t.friendly[0]), Some(1));
}

#[test]
fn crossfire_deals_1_damage_to_the_targeted_minion_and_1_to_the_targeted_hero() {
    let mut t = table(&[CROSSFIRE], 0, 1);
    let crossfire = in_hand(&t.game, t.p0, CROSSFIRE);
    let own_hero = t.game.hero_id(t.p0);
    let enemy_health = t.game.hero_health(t.p1);
    let own_health = t.game.hero_health(t.p0);
    apply_all(
        &mut t.game,
        t.p0,
        &[
            start_targeting(crossfire),
            target(0, t.enemy[0]),
            target(1, own_hero),
            play(crossfire),
        ],
    );

    assert_eq!(t.game.health(t.enemy[0]), Some(1));
    assert_eq!(t.game.hero_health(t.p0), own_health - 1);
    assert_eq!(
        t.game.hero_health(t.p1),
        enemy_health,
        "only the targeted hero is hit"
    );
}
