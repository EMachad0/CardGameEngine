//! A card or trigger that pauses for a pick, and what waits for it.

use pretty_assertions::assert_eq;
use rules::prelude::static_card_definition::{
    BOMBER, DRUMMER, HOARDER, MOURNER, RALLY_BANNER, RECRUIT, SWEEP,
};
use rules::prelude::{Action, Event, Game, PlayerId};

use crate::support::*;

/// A pick of each card `p` has revealed.
fn picks(game: &Game, p: PlayerId) -> Vec<Action> {
    game.revealed(p).iter().map(|&id| pick(id)).collect()
}

/// Plays the Sweep in P0's hand and returns what the play reported.
fn sweep(b: &mut Boards) -> Vec<Event> {
    let sweep = in_hand(&b.game, b.p0, SWEEP);
    observe(&mut b.game, b.p0, play(sweep)).events()
}

/// Picks the first card `p` has revealed and returns what the pick reported.
fn pick_first(game: &mut Game, p: PlayerId) -> Vec<Event> {
    let card = *game
        .revealed(p)
        .first()
        .unwrap_or_else(|| panic!("{p:?} has nothing to pick"));
    observe(game, p, pick(card)).events()
}

/// P1's Hoarder died to P0's Sweep, and its deathrattle waits for P1's pick.
fn hoarder_waiting_for_its_owner() -> Boards {
    let mut b = boards(&[], &[HOARDER], &[SWEEP], 2);
    sweep(&mut b);
    pick_first(&mut b.game, b.p0);
    b
}

#[test]
fn sweeps_draw_waits_for_its_pick() {
    let mut b = boards(&[], &[], &[SWEEP], 2);

    let played = sweep(&mut b);
    let picked = pick_first(&mut b.game, b.p0);

    assert_eq!(
        draws(&played, b.p0),
        0,
        "nothing is drawn before the pick:\n{}",
        listing(&played)
    );
    let pick_at = first(&picked, |e| matches!(e, Event::Picked { .. }));
    let draw_at = first(&picked, |e| matches!(e, Event::Drew { .. }));
    assert!(
        pick_at < draw_at,
        "the pick, then the draw:\n{}",
        listing(&picked)
    );
}

#[test]
fn a_minion_at_0_health_stays_on_the_board_while_the_card_that_hit_it_waits() {
    let mut b = boards(&[BOMBER], &[RECRUIT], &[SWEEP], 2);
    let bomber = b.mine[0];

    sweep(&mut b);

    assert_eq!(
        b.game.health(bomber),
        Some(0),
        "Sweep left it at 0, and it dies only once Sweep has finished"
    );
}

#[test]
fn a_deathrattle_resolves_after_the_rest_of_the_card_that_killed_its_minion() {
    let mut b = boards(&[BOMBER], &[RECRUIT], &[SWEEP], 2);
    let bomber = b.mine[0];
    sweep(&mut b);

    let picked = pick_first(&mut b.game, b.p0);

    let draw_at = first(&picked, |e| matches!(e, Event::Drew { .. }));
    let deathrattle_at = first(
        &picked,
        |e| matches!(e, Event::Damaged { source, .. } if *source == bomber),
    );
    assert!(
        draw_at < deathrattle_at,
        "Sweep's draw, then the Bomber's deathrattle:\n{}",
        listing(&picked)
    );
}

#[test]
fn a_trigger_a_paused_card_set_off_waits_until_the_card_finishes() {
    let mut b = boards(&[RALLY_BANNER, RECRUIT], &[], &[SWEEP], 2);

    let played = sweep(&mut b);
    let picked = pick_first(&mut b.game, b.p0);

    assert_eq!(
        draws(&played, b.p0),
        0,
        "the Recruit's granted draw waits behind the paused Sweep:\n{}",
        listing(&played)
    );
    assert_eq!(
        draws(&picked, b.p0),
        2,
        "Sweep's draw, then the Recruit's:\n{}",
        listing(&picked)
    );
}

#[test]
fn only_the_player_a_pick_waits_for_has_actions() {
    let b = hoarder_waiting_for_its_owner();

    assert_eq!(
        b.game.revealed(b.p1).len(),
        2,
        "the Hoarder's deathrattle revealed P1's top 2"
    );
    assert_actions(&b.game, b.p0, &[]);
    assert_actions(&b.game, b.p1, &picks(&b.game, b.p1));
    assert_unlisted_rejected(&b.game);
}

#[test]
fn the_active_player_acts_again_once_the_opponents_pick_is_made() {
    let mut b = hoarder_waiting_for_its_owner();

    pick_first(&mut b.game, b.p1);

    assert!(
        b.game.legal_actions(b.p0).contains(&Action::EndTurn),
        "P0's turn goes on"
    );
    assert_actions(&b.game, b.p1, &[]);
}

#[test]
fn a_reaction_to_a_death_resolves_before_the_deathrattle_that_pauses() {
    let mut b = boards(&[], &[MOURNER, HOARDER], &[SWEEP], 2);
    let hoarder = b.theirs[1];
    sweep(&mut b);

    let picked = pick_first(&mut b.game, b.p0);

    let order = [
        first(
            &picked,
            |e| matches!(e, Event::Drew { player_id, .. } if *player_id == b.p0),
        ),
        first(
            &picked,
            |e| matches!(e, Event::Died { object_id } if *object_id == hoarder),
        ),
        first(
            &picked,
            |e| matches!(e, Event::Drew { player_id, .. } if *player_id == b.p1),
        ),
        first(
            &picked,
            |e| matches!(e, Event::Revealed { player_id, .. } if *player_id == b.p1),
        ),
    ];
    assert!(
        order.is_sorted(),
        "Sweep's draw, the Hoarder's death, the Mourner's draw, then the Hoarder's reveal:\n{}",
        listing(&picked)
    );
}

#[test]
fn a_minion_is_on_the_board_when_its_play_effects_resolve() {
    let mut b = boards(&[], &[], &[DRUMMER], 2);

    let drummer = summon(&mut b.game, b.p0, DRUMMER);

    assert_eq!(
        b.game.health(drummer),
        Some(2),
        "Drummer's +1 Health reaches Drummer itself"
    );
}

#[test]
fn a_game_cloned_during_a_pick_resolves_like_the_original() {
    let mut b = boards(&[BOMBER], &[RECRUIT], &[SWEEP], 2);
    let bomber = b.mine[0];
    sweep(&mut b);
    let mut clone = b.game.clone();
    let card = b.game.revealed(b.p0)[0];

    b.game.apply(b.p0, pick(card), &mut ()).unwrap();
    clone.apply(b.p0, pick(card), &mut ()).unwrap();

    assert_eq!(clone.health(bomber), None, "the clone finished Sweep");
    assert_eq!(clone, b.game);
}
