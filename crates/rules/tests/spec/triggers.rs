//! Card text that reacts to events, and when it resolves.

use std::collections::BTreeSet;

use pretty_assertions::assert_eq;
use rules::prelude::static_card_definition::{
    BARRACKS, BLAST, BOLT, BOMBER, COURIER, DISPATCH, DRILLMASTER, GRAVEDIGGER, HERALD,
    JUGGLING_ACT, JUGGLING_LINE, KNIFE_JUGGLER, MEDIC, MIRROR_IMAGE, MOURNER, PING, RALLY_BANNER,
    RECRUIT, RINGMASTER, SQUIRE, TREMOR, TWIN_SHOT,
};
use rules::prelude::{Event, Game, ObjectId, Outcome};

use crate::support::*;

/// How many times `target` was hit in `events`.
fn hits_on(events: &[Event], target: ObjectId) -> usize {
    events
        .iter()
        .filter(|e| matches!(e, Event::Damaged { target: t, .. } if *t == target))
        .count()
}

fn is_hit_from(e: &Event, from: ObjectId) -> bool {
    matches!(e, Event::Damaged { source, .. } if *source == from)
}

/// How many knives `juggler` threw in `events`.
fn knives_from(events: &[Event], juggler: ObjectId) -> usize {
    events.iter().filter(|e| is_hit_from(e, juggler)).count()
}

fn is_draw(e: &Event) -> bool {
    matches!(e, Event::Drew { .. })
}

/// Whether `e` is the 2 damage of `courier`'s play effect.
fn is_couriers_play_effect(e: &Event, courier: ObjectId) -> bool {
    matches!(e, Event::Damaged { amount: 2, source, .. } if *source == courier)
}

/// Whether `e` is the 3 damage of `courier`'s reaction to its own summon.
fn is_couriers_summon_reaction(e: &Event, courier: ObjectId) -> bool {
    matches!(e, Event::Damaged { amount: 3, source, .. } if *source == courier)
}

/// Plays the Courier in P0's hand and returns it, with what the play reported.
fn play_courier(b: &mut Boards) -> (ObjectId, Vec<Event>) {
    let courier = in_hand(&b.game, b.p0, COURIER);
    let events = observe(&mut b.game, b.p0, play(courier)).events();
    (courier, events)
}

/// Pings `minion` with the Ping in P0's hand and returns what the play reported.
fn ping(b: &mut Boards, minion: ObjectId) -> Vec<Event> {
    let ping = in_hand(&b.game, b.p0, PING);
    apply_all(
        &mut b.game,
        b.p0,
        &[start_targeting(ping), target(0, minion)],
    );
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
fn a_mourner_does_not_draw_for_its_own_death() {
    let mut b = boards(&[MOURNER], &[], &[PING, PING], 2);
    let mourner = b.mine[0];
    ping(&mut b, mourner);

    let events = ping(&mut b, mourner);

    assert_eq!(b.game.health(mourner), None, "the second Ping kills it");
    assert_eq!(
        draws(&events, b.p0),
        0,
        "the Mourner has left the board when its own death is matched:\n{}",
        listing(&events)
    );
}

#[test]
fn a_mourner_dying_in_the_same_pass_as_a_younger_minion_draws_nothing() {
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
fn a_mourner_dying_in_the_same_pass_as_an_older_minion_draws_nothing() {
    let mut b = boards(&[RECRUIT, MOURNER], &[], &[PING, BLAST], 4);
    let [recruit, mourner] = [b.mine[0], b.mine[1]];
    ping(&mut b, mourner);
    let blast = in_hand(&b.game, b.p0, BLAST);

    let events = observe(&mut b.game, b.p0, play(blast)).events();

    assert_eq!(
        [b.game.health(recruit), b.game.health(mourner)],
        [None, None],
        "Blast kills both in one pass"
    );
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
            Event::Summoned { object_id, .. } => Some(*object_id),
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
fn a_knife_juggler_damages_the_enemy_hero_when_barracks_summons_a_squire() {
    let mut b = boards(&[KNIFE_JUGGLER], &[], &[BARRACKS], 2);

    play_def(&mut b.game, b.p0, BARRACKS);

    assert_eq!(
        b.game.hero_health(b.p1),
        9,
        "the Juggler hears the Squire's summon, and the enemy hero is the only enemy character",
    );
}

#[test]
fn a_knife_juggler_does_not_hear_its_own_summon() {
    let mut b = boards(&[], &[], &[KNIFE_JUGGLER], 2);
    let juggler = in_hand(&b.game, b.p0, KNIFE_JUGGLER);

    let events = observe(&mut b.game, b.p0, play(juggler)).events();

    assert_eq!(
        knives_from(&events, juggler),
        0,
        "events:\n{}",
        listing(&events)
    );
}

#[test]
fn a_knife_juggler_does_not_hear_an_enemy_minions_summon() {
    let deck0 = deck_with_top(&[KNIFE_JUGGLER]);
    let mut game = Game::with_deck_order(0, [deck0, deck_with_top(&[RECRUIT])]);
    let [p0, p1] = players(&game);
    turn_with_mana(&mut game, p0, 2);
    let juggler = summon(&mut game, p0, KNIFE_JUGGLER);
    turn_with_mana(&mut game, p1, 2);
    let recruit = in_hand(&game, p1, RECRUIT);

    let events = observe(&mut game, p1, play(recruit)).events();

    assert_eq!(
        knives_from(&events, juggler),
        0,
        "events:\n{}",
        listing(&events)
    );
}

/// P0 has a Recruit and a Knife Juggler, P1 has a Recruit, and P0's Barracks summons a Squire, in
/// a game seeded with `seed`. Returns the characters the Juggler's knives hit, and P1's hero and
/// Recruit.
fn knife_targets(seed: u64) -> (Vec<ObjectId>, [ObjectId; 2]) {
    let mut b = seeded_boards(seed, &[RECRUIT, KNIFE_JUGGLER], &[RECRUIT], &[BARRACKS], 2);
    let juggler = b.mine[1];
    let barracks = in_hand(&b.game, b.p0, BARRACKS);

    let events = observe(&mut b.game, b.p0, play(barracks)).events();

    let hit = events
        .iter()
        .filter_map(|e| match e {
            Event::Damaged { target, source, .. } if *source == juggler => Some(*target),
            _ => None,
        })
        .collect();
    (hit, [b.game.hero_id(b.p1), b.theirs[0]])
}

#[test]
fn a_knife_jugglers_knife_hits_only_enemy_characters() {
    for seed in 0..100 {
        let (hit, enemies) = knife_targets(seed);

        assert!(
            matches!(hit[..], [target] if enemies.contains(&target)),
            "seed {seed}: the knives hit {hit:?}, and the enemy characters are {enemies:?}",
        );
    }
}

#[test]
fn a_knife_jugglers_knife_can_hit_any_enemy_character() {
    let hit: BTreeSet<ObjectId> = (0..100).flat_map(|seed| knife_targets(seed).0).collect();

    assert_eq!(
        hit,
        BTreeSet::from(knife_targets(0).1),
        "in 100 seeds, the knife hit the enemy hero and the enemy Recruit",
    );
}

#[test]
fn with_a_knife_juggler_on_the_board_mirror_image_throws_two_knives() {
    let mut b = boards(&[KNIFE_JUGGLER], &[], &[MIRROR_IMAGE], 1);
    let juggler = b.mine[0];
    let mirror_image = in_hand(&b.game, b.p0, MIRROR_IMAGE);

    let events = observe(&mut b.game, b.p0, play(mirror_image)).events();

    assert_eq!(
        knives_from(&events, juggler),
        2,
        "one knife for each Image's summon:\n{}",
        listing(&events)
    );
}

#[test]
fn juggling_act_throws_a_knife_from_each_juggler() {
    let mut b = boards(&[], &[], &[JUGGLING_ACT], 2);
    let juggling_act = in_hand(&b.game, b.p0, JUGGLING_ACT);

    let events = observe(&mut b.game, b.p0, play(juggling_act)).events();

    let &[first, second] = b.game.player_board(b.p0) else {
        panic!("Juggling Act left {:?}", board_defs(&b.game, b.p0));
    };
    assert_eq!(
        [knives_from(&events, first), knives_from(&events, second)],
        [1, 1],
        "one effect summons both Jugglers, so each hears the other's summon:\n{}",
        listing(&events)
    );
}

#[test]
fn juggling_line_throws_one_knife_from_the_first_juggler() {
    let mut b = boards(&[], &[], &[JUGGLING_LINE], 2);
    let juggling_line = in_hand(&b.game, b.p0, JUGGLING_LINE);

    let events = observe(&mut b.game, b.p0, play(juggling_line)).events();

    let &[first, second] = b.game.player_board(b.p0) else {
        panic!("Juggling Line left {:?}", board_defs(&b.game, b.p0));
    };
    assert_eq!(
        [knives_from(&events, first), knives_from(&events, second)],
        [1, 0],
        "each effect summons one Juggler, so the first hears the second's summon, and the second \
         was not on the board when the first's summon was matched:\n{}",
        listing(&events)
    );
}

#[test]
fn with_a_knife_juggler_on_the_board_couriers_play_effect_resolves_before_the_knife() {
    let mut b = boards(&[KNIFE_JUGGLER], &[], &[COURIER], 2);
    let juggler = b.mine[0];

    let (courier, events) = play_courier(&mut b);

    assert!(
        first(&events, |e| is_couriers_play_effect(e, courier))
            < first(&events, |e| is_hit_from(e, juggler)),
        "the knife reacts to Courier's summon, which goes under Courier's play effect:\n{}",
        listing(&events)
    );
}

#[test]
fn couriers_play_effect_resolves_before_its_own_summon_reaction() {
    let mut b = boards(&[], &[], &[COURIER], 2);

    let (courier, events) = play_courier(&mut b);

    assert!(
        first(&events, |e| is_couriers_play_effect(e, courier))
            < first(&events, |e| is_couriers_summon_reaction(e, courier)),
        "the reactions to a played minion's summon go under its play effects:\n{}",
        listing(&events)
    );
}

#[test]
fn with_a_knife_juggler_on_the_board_couriers_own_summon_reaction_resolves_before_the_knife() {
    let mut b = boards(&[KNIFE_JUGGLER], &[], &[COURIER], 2);
    let juggler = b.mine[0];

    let (courier, events) = play_courier(&mut b);

    assert!(
        first(&events, |e| is_couriers_summon_reaction(e, courier))
            < first(&events, |e| is_hit_from(e, juggler)),
        "both react to Courier's summon, and Courier is the newer, so its reaction resolves \
         first:\n{}",
        listing(&events)
    );
}

#[test]
fn a_courier_on_the_board_does_not_react_to_another_minions_summon() {
    let mut b = boards(&[COURIER], &[], &[RECRUIT], 2);
    let courier = b.mine[0];
    let recruit = in_hand(&b.game, b.p0, RECRUIT);

    let events = observe(&mut b.game, b.p0, play(recruit)).events();

    assert!(
        !events.iter().any(|e| is_hit_from(e, courier)),
        "Courier reacts only to its own summon:\n{}",
        listing(&events)
    );
}

#[test]
fn with_herald_on_the_board_heralds_draw_resolves_before_couriers_play_effect() {
    let mut b = boards(&[HERALD], &[], &[COURIER], 2);

    let (courier, events) = play_courier(&mut b);

    assert!(
        first(&events, is_draw) < first(&events, |e| is_couriers_play_effect(e, courier)),
        "Herald's draw reacts to Courier's play, so it goes on top of Courier's play effect:\n{}",
        listing(&events)
    );
}

#[test]
fn dispatch_runs_couriers_summon_reaction_and_not_its_play_effect() {
    let mut b = boards(&[], &[], &[DISPATCH], 2);
    let dispatch = in_hand(&b.game, b.p0, DISPATCH);

    let events = observe(&mut b.game, b.p0, play(dispatch)).events();

    let &[courier] = b.game.player_board(b.p0) else {
        panic!("Dispatch left {:?}", board_defs(&b.game, b.p0));
    };
    let hits = events
        .iter()
        .filter_map(|e| match e {
            Event::Damaged { amount, source, .. } if *source == courier => Some(*amount),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        hits,
        [3],
        "a summoned Courier is not played, so only its summon reaction deals damage:\n{}",
        listing(&events)
    );
}

#[test]
fn ringmaster_played_on_an_empty_board_throws_no_knife() {
    let mut b = boards(&[], &[], &[RINGMASTER], 3);
    let ringmaster = in_hand(&b.game, b.p0, RINGMASTER);

    let events = observe(&mut b.game, b.p0, play(ringmaster)).events();

    let &[_, juggler] = b.game.player_board(b.p0) else {
        panic!("Ringmaster left {:?}", board_defs(&b.game, b.p0));
    };
    assert_eq!(
        knives_from(&events, juggler),
        0,
        "the Juggler was not on the board when Ringmaster's summon was matched:\n{}",
        listing(&events)
    );
}

#[test]
fn a_knife_juggler_that_tremors_play_effect_kills_still_throws_its_knife() {
    let mut b = boards(&[KNIFE_JUGGLER], &[], &[TREMOR], 2);
    let juggler = b.mine[0];
    let tremor = in_hand(&b.game, b.p0, TREMOR);

    let events = observe(&mut b.game, b.p0, play(tremor)).events();

    assert!(
        first(&events, |e| *e == Event::Died { object_id: juggler })
            < first(&events, |e| is_hit_from(e, juggler)),
        "the knife was matched at Tremor's summon, and resolves after Tremor's play effect kills \
         the Juggler:\n{}",
        listing(&events)
    );
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
fn a_minion_killed_by_one_deathrattle_is_gone_before_the_next() {
    let mut b = boards(&[BOMBER, BOMBER, MOURNER], &[], &[BLAST], 3);
    let mourner = b.mine[2];
    let blast = in_hand(&b.game, b.p0, BLAST);

    let events = observe(&mut b.game, b.p0, play(blast)).events();

    assert_eq!(b.game.health(mourner), None);
    assert_eq!(
        hits_on(&events, mourner),
        2,
        "Blast leaves it at 1, one Bomber's deathrattle kills it, and the state check removes it \
         before the other Bomber's deathrattle:\n{}",
        listing(&events)
    );
}

#[test]
fn a_trigger_resolves_after_its_source_has_died() {
    let mut b = boards(&[RALLY_BANNER, RECRUIT], &[], &[BLAST], 3);
    let recruit = b.mine[1];
    let blast = in_hand(&b.game, b.p0, BLAST);

    let events = observe(&mut b.game, b.p0, play(blast)).events();

    assert!(
        first(&events, |e| *e == Event::Died { object_id: recruit }) < first(&events, is_draw),
        "the Recruit's draw was matched at Blast's hit, and the state check removes the Recruit \
         before any trigger resolves:\n{}",
        listing(&events)
    );
}

#[test]
fn the_triggers_reacting_to_a_death_resolve_before_the_dead_minions_own() {
    let mut b = boards(&[BOMBER, MOURNER], &[], &[BLAST], 3);
    let bomber = b.mine[0];
    let blast = in_hand(&b.game, b.p0, BLAST);

    let events = observe(&mut b.game, b.p0, play(blast)).events();

    assert!(
        first(&events, is_draw) < first(&events, |e| is_hit_from(e, bomber)),
        "the Mourner's draw goes on top of the Bomber's deathrattle, though the Bomber entered \
         first:\n{}",
        listing(&events)
    );
}

#[test]
fn a_deathrattles_consequences_resolve_before_triggers_already_waiting() {
    let mut b = boards(&[BOMBER, BOMBER, BOMBER], &[RALLY_BANNER], &[TWIN_SHOT], 2);
    let [one, two, three] = b.mine[..] else {
        unreachable!();
    };
    let twin_shot = in_hand(&b.game, b.p0, TWIN_SHOT);
    apply_all(
        &mut b.game,
        b.p0,
        &[start_targeting(twin_shot), target(0, one), target(0, two)],
    );

    let events = observe(&mut b.game, b.p0, play(twin_shot)).events();

    let waiting =
        if first(&events, |e| is_hit_from(e, one)) < first(&events, |e| is_hit_from(e, two)) {
            two
        } else {
            one
        };
    assert!(
        first(&events, |e| is_hit_from(e, three)) < first(&events, |e| is_hit_from(e, waiting)),
        "Twin Shot kills two Bombers. Whichever deathrattle resolves first kills the third Bomber, \
         and the third's deathrattle resolves before the other one, which was already \
         waiting:\n{}",
        listing(&events)
    );
}

#[test]
fn gravedigger_hits_the_enemy_hero_when_any_minion_dies() {
    let mut b = boards(&[GRAVEDIGGER], &[RECRUIT], &[PING], 1);
    let recruit = b.theirs[0];

    ping(&mut b, recruit);

    assert_eq!(b.game.hero_health(b.p1), 9);
}

#[test]
fn the_newest_of_the_triggers_matched_by_one_event_resolves_first() {
    let mut b = boards(&[RECRUIT, GRAVEDIGGER], &[GRAVEDIGGER], &[PING], 1);
    let [recruit, friendly] = b.mine[..] else {
        unreachable!();
    };
    let enemy = b.theirs[0];

    let events = ping(&mut b, recruit);

    assert!(
        first(&events, |e| is_hit_from(e, friendly)) < first(&events, |e| is_hit_from(e, enemy)),
        "the enemy Gravedigger entered play before the friendly one, so the friendly one's \
         trigger is on top of the stack:\n{}",
        listing(&events)
    );
}

#[test]
fn nothing_resolves_after_the_game_ends() {
    let deck0 = vec![BOMBER, MOURNER, BLAST, BLAST];
    let mut game = Game::with_deck_order(0, [deck0, deck_with_top(&[])]);
    let [p0, p1] = players(&game);
    turn_with_mana(&mut game, p0, 2);
    summon(&mut game, p0, BOMBER);
    turn_with_mana(&mut game, p0, 3);
    summon(&mut game, p0, MOURNER);
    turn_with_mana(&mut game, p0, 9);
    assert_eq!(game.hero_health(p0), 2, "fatigue leaves player 0 at 2");
    let blast = in_hand(&game, p0, BLAST);

    let events = observe(&mut game, p0, play(blast)).events();

    assert_eq!(
        events.last(),
        Some(&Event::GameEnded {
            outcome: Outcome::Won(p1)
        }),
        "the Mourner's draw and the Bomber's deathrattle were waiting when the hero died:\n{}",
        listing(&events)
    );
}
