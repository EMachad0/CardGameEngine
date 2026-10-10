//! Card text that reacts to events, and when it resolves.

use pretty_assertions::assert_eq;
use rules::static_card_definition::{
    BLAST, BOLT, BOMBER, DRILLMASTER, MEDIC, MOURNER, PING, RALLY_BANNER, RECRUIT, SQUIRE,
};
use rules::{Event, ObjectId};

use crate::support::*;

/// How many times `target` was hit in `events`.
fn hits_on(events: &[Event], target: ObjectId) -> usize {
    events
        .iter()
        .filter(|e| matches!(e, Event::Damaged { target: t, .. } if *t == target))
        .count()
}

/// The index of the first event in `events` that matches `pred`.
fn first(events: &[Event], pred: impl Fn(&Event) -> bool) -> usize {
    events
        .iter()
        .position(pred)
        .unwrap_or_else(|| panic!("no matching event in:\n{}", listing(events)))
}

/// The index of the last event in `events` that matches `pred`.
fn last(events: &[Event], pred: impl Fn(&Event) -> bool) -> usize {
    events
        .iter()
        .rposition(pred)
        .unwrap_or_else(|| panic!("no matching event in:\n{}", listing(events)))
}

fn is_hit_from(e: &Event, from: ObjectId) -> bool {
    matches!(e, Event::Damaged { source, .. } if *source == from)
}

fn is_draw(e: &Event) -> bool {
    matches!(e, Event::Drew { .. })
}

/// Pings `target` with the Ping in P0's hand and returns what the play reported.
fn ping(b: &mut Boards, target: ObjectId) -> Vec<Event> {
    let ping = in_hand(&b.game, b.p0, PING);
    apply_all(&mut b.game, b.p0, &[draft(ping), choose(0, target)]);
    observe(&mut b.game, b.p0, play(ping)).events()
}

#[test]
fn medics_deathrattle_gives_its_surviving_friendly_minions_2_health() {
    let mut b = boards(&[MEDIC, MOURNER], &[], &[BLAST], 3);
    let mourner = b.mine[1];

    play_def(&mut b.game, b.p0, BLAST);

    assert_eq!(
        b.game.health(mourner),
        Some(3),
        "3 base, plus 2 from the dead Medic, minus 2 from Blast"
    );
}

#[test]
fn medics_deathrattle_does_not_reach_enemy_minions() {
    let mut b = boards(&[MEDIC], &[DRILLMASTER], &[BLAST], 3);
    let drillmaster = b.theirs[0];

    play_def(&mut b.game, b.p0, BLAST);

    assert_eq!(b.game.health(drillmaster), Some(1));
}

#[test]
fn mourner_draws_a_card_for_each_friendly_minion_that_dies() {
    let mut b = boards(&[MOURNER, RECRUIT, RECRUIT], &[], &[BLAST], 3);
    let blast = in_hand(&b.game, b.p0, BLAST);

    let events = observe(&mut b.game, b.p0, play(blast)).events();

    assert_eq!(draws(&events, b.p0), 2, "events:\n{}", listing(&events));
}

#[test]
fn mourner_does_not_draw_for_enemy_minions_dying() {
    let mut b = boards(&[MOURNER], &[RECRUIT], &[BLAST], 3);
    let blast = in_hand(&b.game, b.p0, BLAST);

    let events = observe(&mut b.game, b.p0, play(blast)).events();

    assert_eq!(draws(&events, b.p0), 0, "events:\n{}", listing(&events));
}

#[test]
fn a_mourner_dying_in_the_same_pass_draws_nothing() {
    let mut b = boards(&[MOURNER, RECRUIT], &[], &[BLAST, RECRUIT, BLAST], 3);
    let mourner = b.mine[0];
    play_def(&mut b.game, b.p0, BLAST);
    assert_eq!(
        b.game.health(mourner),
        Some(1),
        "the first Blast leaves it at 1"
    );
    turn_with_mana(&mut b.game, b.p0, 5);
    summon(&mut b.game, b.p0, RECRUIT);
    let blast = in_hand(&b.game, b.p0, BLAST);

    let events = observe(&mut b.game, b.p0, play(blast)).events();

    assert_eq!(b.game.health(mourner), None, "the second Blast kills it");
    assert_eq!(
        draws(&events, b.p0),
        0,
        "a removed minion hears no death from its own pass:\n{}",
        listing(&events)
    );
}

#[test]
fn drillmaster_summons_a_squire_when_its_owner_casts_a_spell() {
    let mut b = boards(&[DRILLMASTER], &[], &[], 2);

    play_def(&mut b.game, b.p0, BOLT);

    assert_eq!(board_defs(&b.game, b.p0), [DRILLMASTER, SQUIRE]);
}

#[test]
fn drillmaster_ignores_minions_its_owner_plays() {
    let mut b = boards(&[DRILLMASTER], &[], &[RECRUIT], 2);

    play_def(&mut b.game, b.p0, RECRUIT);

    assert_eq!(board_defs(&b.game, b.p0), [DRILLMASTER, RECRUIT]);
}

#[test]
fn drillmaster_ignores_the_opponents_spells() {
    let mut b = boards(&[DRILLMASTER], &[], &[], 0);
    turn_with_mana(&mut b.game, b.p1, 2);

    play_def(&mut b.game, b.p1, BOLT);

    assert_eq!(board_defs(&b.game, b.p0), [DRILLMASTER]);
}

#[test]
fn the_spell_that_made_drillmaster_summon_hits_the_squire() {
    let mut b = boards(&[DRILLMASTER], &[], &[BLAST], 3);
    let blast = in_hand(&b.game, b.p0, BLAST);

    let events = observe(&mut b.game, b.p0, play(blast)).events();

    let squire = events
        .iter()
        .find_map(|e| match e {
            Event::BoardEntered { object_id, .. } => Some(*object_id),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no Squire entered:\n{}", listing(&events)));
    assert!(
        events.contains(&Event::Damaged {
            target: squire,
            amount: 2,
            source: blast,
        }),
        "the Squire is on the board before Blast's effects:\n{}",
        listing(&events)
    );
    assert_eq!(board_defs(&b.game, b.p0), [DRILLMASTER]);
}

#[test]
fn a_minion_played_after_rally_banner_draws_when_it_takes_damage() {
    let mut b = boards(&[RALLY_BANNER, MOURNER], &[], &[BLAST], 3);
    let blast = in_hand(&b.game, b.p0, BLAST);

    let events = observe(&mut b.game, b.p0, play(blast)).events();

    assert_eq!(
        draws(&events, b.p0),
        1,
        "the Mourner has the Banner's trigger though it entered after the Banner:\n{}",
        listing(&events)
    );
}

#[test]
fn a_minion_brought_to_zero_health_still_draws_from_rally_banner() {
    let mut b = boards(&[RALLY_BANNER, RECRUIT], &[], &[BLAST], 3);
    let blast = in_hand(&b.game, b.p0, BLAST);

    let events = observe(&mut b.game, b.p0, play(blast)).events();

    assert_eq!(
        draws(&events, b.p0),
        1,
        "the Recruit is still on the board when its hit is matched:\n{}",
        listing(&events)
    );
}

#[test]
fn rally_banner_does_not_grant_its_trigger_to_itself() {
    let mut b = boards(&[RALLY_BANNER], &[], &[BLAST], 3);
    let blast = in_hand(&b.game, b.p0, BLAST);

    let events = observe(&mut b.game, b.p0, play(blast)).events();

    assert_eq!(draws(&events, b.p0), 0, "events:\n{}", listing(&events));
}

#[test]
fn rally_banner_does_not_grant_its_trigger_to_enemy_minions() {
    let mut b = boards(&[RALLY_BANNER], &[MOURNER], &[BLAST], 3);
    let blast = in_hand(&b.game, b.p0, BLAST);

    let events = observe(&mut b.game, b.p0, play(blast)).events();

    assert_eq!(draws(&events, b.p1), 0, "events:\n{}", listing(&events));
}

#[test]
fn bombers_deathrattle_deals_1_damage_to_every_minion() {
    let mut b = boards(&[BOMBER], &[RECRUIT], &[PING], 1);
    let (bomber, recruit) = (b.mine[0], b.theirs[0]);

    ping(&mut b, bomber);

    assert_eq!(b.game.health(recruit), Some(1));
}

#[test]
fn a_minion_killed_by_the_first_deathrattle_is_gone_before_the_second() {
    let mut b = boards(&[BOMBER, BOMBER, MOURNER], &[], &[BLAST], 3);
    let mourner = b.mine[2];
    let blast = in_hand(&b.game, b.p0, BLAST);

    let events = observe(&mut b.game, b.p0, play(blast)).events();

    assert_eq!(b.game.health(mourner), None);
    assert_eq!(
        hits_on(&events, mourner),
        2,
        "Blast leaves it at 1, the first Bomber kills it, and the state check removes it \
         before the second Bomber's deathrattle:\n{}",
        listing(&events)
    );
}

#[test]
fn a_trigger_resolves_after_its_source_has_died() {
    let mut b = boards(&[BOMBER, MOURNER], &[], &[BLAST], 3);
    let mourner = b.mine[1];
    let blast = in_hand(&b.game, b.p0, BLAST);

    let events = observe(&mut b.game, b.p0, play(blast)).events();

    assert_eq!(b.game.health(mourner), None);
    assert_eq!(
        draws(&events, b.p0),
        1,
        "the Mourner's draw for the Bomber was queued before the Bomber's deathrattle \
         killed it:\n{}",
        listing(&events)
    );
}

#[test]
fn an_older_minions_deathrattle_resolves_before_a_younger_minions_trigger() {
    let mut b = boards(&[BOMBER, MOURNER], &[], &[BLAST], 3);
    let bomber = b.mine[0];
    let blast = in_hand(&b.game, b.p0, BLAST);

    let events = observe(&mut b.game, b.p0, play(blast)).events();

    assert!(
        first(&events, |e| is_hit_from(e, bomber)) < first(&events, is_draw),
        "the Bomber entered first:\n{}",
        listing(&events)
    );
}

#[test]
fn an_older_minions_trigger_resolves_before_a_younger_minions_deathrattle() {
    let mut b = boards(&[MOURNER, BOMBER], &[], &[BLAST], 3);
    let bomber = b.mine[1];
    let blast = in_hand(&b.game, b.p0, BLAST);

    let events = observe(&mut b.game, b.p0, play(blast)).events();

    assert!(
        first(&events, is_draw) < first(&events, |e| is_hit_from(e, bomber)),
        "the Mourner entered first:\n{}",
        listing(&events)
    );
}

#[test]
fn a_deathrattles_consequences_resolve_before_triggers_already_waiting() {
    let mut b = boards(&[BOMBER, BOMBER, MOURNER], &[], &[PING], 1);
    let [first_bomber, second_bomber, _] = b.mine[..] else {
        unreachable!();
    };

    let events = ping(&mut b, first_bomber);

    assert!(
        last(&events, |e| is_hit_from(e, second_bomber)) < first(&events, is_draw),
        "the first deathrattle kills the second Bomber, whose deathrattle resolves before \
         the Mourner's draw that was already waiting:\n{}",
        listing(&events)
    );
}
