//! An effect's verbs, each run in list order on the one selection its selector reads.

use std::collections::BTreeSet;

use pretty_assertions::assert_eq;
use rules::prelude::static_card_definition::{AFTERSHOCK, FRENZY, GOAD, MOURNER, RECRUIT};
use rules::prelude::{Event, Game, ObjectId};

use crate::support::*;

/// Each hit in `events`, as its target and amount.
fn hits(events: &[Event]) -> Vec<(ObjectId, u8)> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::Damaged { target, amount, .. } => Some((*target, *amount)),
            _ => None,
        })
        .collect()
}

/// P0 has two Recruits on the board and can cast the Frenzy in hand, in a game seeded with `seed`.
/// Returns the game and the Recruits, left to right.
fn frenzy_over_two_recruits(seed: u64) -> (Game, [ObjectId; 2]) {
    let deck0 = deck_with_top(&[RECRUIT, RECRUIT, FRENZY]);
    let mut game = Game::with_deck_order(seed, [deck0, deck_with_top(&[])]);
    let [p0, _] = players(&game);
    turn_with_mana(&mut game, p0, 4);
    let recruits = [
        summon(&mut game, p0, RECRUIT),
        summon(&mut game, p0, RECRUIT),
    ];
    turn_with_mana(&mut game, p0, 1);
    (game, recruits)
}

#[test]
fn two_verbs_on_one_random_selection_act_on_the_same_character() {
    let hit = (Some(1), Some(4));
    let untouched = (Some(2), Some(2));
    let mut selected = BTreeSet::new();
    for seed in 0..50 {
        let (mut game, [left, right]) = frenzy_over_two_recruits(seed);
        let [p0, _] = players(&game);

        play_def(&mut game, p0, FRENZY);

        let stats = [left, right].map(|id| (game.health(id), game.attack(id)));
        assert!(
            stats == [hit, untouched] || stats == [untouched, hit],
            "seed {seed}: one Recruit should take both the 1 damage and the +2 Attack, \
             but (health, attack) are {stats:?}",
        );
        selected.insert(if stats[0] == hit { left } else { right });
    }
    assert_eq!(
        selected.len(),
        2,
        "the random selection should land on each Recruit in some seed, or reading it twice would \
         agree in every seed anyway",
    );
}

#[test]
fn two_verbs_on_one_targeted_selection_act_on_the_targeted_character() {
    let mut b = boards(&[RECRUIT], &[MOURNER], &[GOAD], 1);
    let (recruit, mourner) = (b.mine[0], b.theirs[0]);
    let goad = in_hand(&b.game, b.p0, GOAD);

    apply_all(
        &mut b.game,
        b.p0,
        &[start_targeting(goad), target(0, mourner), play(goad)],
    );

    assert_eq!(
        [mourner, recruit].map(|id| (b.game.health(id), b.game.attack(id))),
        [(Some(1), Some(3)), (Some(2), Some(2))],
        "the targeted Mourner takes the 2 damage and the +2 Attack, and the Recruit neither",
    );
}

/// P1 has two Recruits on the board, and P0 can cast the Aftershock in hand.
fn aftershock_facing_two_recruits() -> Boards {
    boards(&[], &[RECRUIT, RECRUIT], &[AFTERSHOCK], 2)
}

#[test]
fn each_verb_runs_on_the_whole_selection_in_list_order() {
    let mut b = aftershock_facing_two_recruits();
    let (left, right) = (b.theirs[0], b.theirs[1]);
    let aftershock = in_hand(&b.game, b.p0, AFTERSHOCK);

    let events = observe(&mut b.game, b.p0, play(aftershock)).events();

    assert_eq!(
        hits(&events),
        [(left, 2), (right, 2), (left, 1), (right, 1)],
        "the first verb hits each Recruit for 2, then the second hits each for 1:\n{}",
        listing(&events),
    );
}

#[test]
fn a_verb_after_one_that_brings_a_minion_to_0_health_still_acts_on_it() {
    let mut b = boards(&[], &[RECRUIT], &[AFTERSHOCK], 2);
    let recruit = b.theirs[0];
    let aftershock = in_hand(&b.game, b.p0, AFTERSHOCK);

    let events = observe(&mut b.game, b.p0, play(aftershock)).events();

    assert_eq!(
        hits(&events),
        [(recruit, 2), (recruit, 1)],
        "the 2 damage leaves the Recruit at 0, and it dies only at the state check:\n{}",
        listing(&events),
    );
}

#[test]
fn an_effect_with_several_verbs_takes_one_checkpoint() {
    let mut b = aftershock_facing_two_recruits();
    let aftershock = in_hand(&b.game, b.p0, AFTERSHOCK);

    let recorder = observe(&mut b.game, b.p0, play(aftershock));

    let hits_per_step: Vec<usize> = recorder
        .steps
        .iter()
        .map(|step| hits(&step.events).len())
        .filter(|&count| count > 0)
        .collect();
    assert_eq!(
        hits_per_step,
        [4],
        "both verbs' hits land before the effect's one checkpoint",
    );
}
