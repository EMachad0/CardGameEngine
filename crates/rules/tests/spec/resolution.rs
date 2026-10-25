//! A card or trigger that pauses for a pick, and what waits for it.

use pretty_assertions::assert_eq;
use rules::prelude::static_card_definition::{
    AFTERSHOCK, BOMBER, DRUMMER, FORAGE, HOARDER, MOURNER, PACKRAT, RALLY_BANNER, RECRUIT, SPARK,
    SWEEP,
};
use rules::prelude::{Action, Event, Game, PlayerId};

use crate::support::*;

/// A pick of each of `p`'s pick options.
fn picks(game: &Game, p: PlayerId) -> Vec<Action> {
    game.pick_options(p).into_iter().map(pick).collect()
}

/// Plays the Sweep in P0's hand and returns what the play reported.
fn sweep(b: &mut Boards) -> Vec<Event> {
    let sweep = in_hand(&b.game, b.p0, SWEEP);
    observe(&mut b.game, b.p0, play(sweep)).events()
}

/// Picks the first of `p`'s pick options and returns what the pick reported.
fn pick_first(game: &mut Game, p: PlayerId) -> Vec<Event> {
    let card = *game
        .pick_options(p)
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
fn a_pick_that_is_its_cards_last_effect_stays_open_until_answered() {
    let deck0 = vec![FORAGE, SPARK, SPARK, SPARK, RECRUIT, RECRUIT];
    let mut game = Game::with_deck_order(0, [deck0, vec![SPARK; 6]]);
    let [p0, _] = players(&game);

    play_def(&mut game, p0, FORAGE);

    assert_eq!(game.pick_options(p0).len(), 2, "Forage's pick is open");
    assert!(
        !game.legal_actions(p0).contains(&Action::EndTurn),
        "the turn can't end while the pick waits"
    );
}

#[test]
fn a_skipped_pick_lets_the_rest_of_its_card_resolve() {
    let deck0 = vec![SWEEP, SPARK, SPARK, SPARK];
    let mut game = Game::with_deck_order(0, [deck0, vec![SPARK; 20]]);
    let [p0, _] = players(&game);
    turn_with_mana(&mut game, p0, 2);
    let sweep = in_hand(&game, p0, SWEEP);

    let events = observe(&mut game, p0, play(sweep)).events();

    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::FatigueDamaged { player_id, .. } if *player_id == p0)),
        "Sweep's draw, from the empty deck its pick skipped:\n{}",
        listing(&events)
    );
    assert!(game.pick_options(p0).is_empty());
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
    let pick_at = first(&picked, |e| matches!(e, Event::AddedToHand { .. }));
    let draw_at = first(&picked, |e| matches!(e, Event::Drew { .. }));
    assert!(
        pick_at < draw_at,
        "the card picked joins the hand, then the draw:\n{}",
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
        b.game.pick_options(b.p1).len(),
        2,
        "the Hoarder's deathrattle offers P1's top 2"
    );
    assert_actions(&b.game, b.p0, &[]);
    assert_actions(&b.game, b.p1, &picks(&b.game, b.p1));
    assert_unlisted_rejected(&b.game);
}

#[test]
fn hoarders_death_rattle_picks_from_its_owners_deck() {
    let mut b = hoarder_waiting_for_its_owner();
    let top_two = b.game.deck(b.p1)[..2].to_vec();

    assert_eq!(b.game.pick_options(b.p1), top_two);
    b.game.apply(b.p1, pick(top_two[0]), &mut ()).unwrap();
    assert!(
        b.game.hand(b.p1).ends_with(&top_two[..1]),
        "the card picked joins P1's hand"
    );
}

#[test]
fn two_resolutions_of_one_sources_trigger_each_move_their_own_picks() {
    let mut b = boards(&[], &[PACKRAT], &[AFTERSHOCK], 2);
    let aftershock = in_hand(&b.game, b.p0, AFTERSHOCK);
    b.game.apply(b.p0, play(aftershock), &mut ()).unwrap();

    let (mut taken, mut left) = (Vec::new(), Vec::new());
    for resolution in 0..2 {
        let options = b.game.pick_options(b.p1);
        let &[first, second] = options.as_slice() else {
            panic!("resolution {resolution} of the Packrat's trigger offers {options:?}");
        };
        b.game.apply(b.p1, pick(first), &mut ()).unwrap();
        taken.push(first);
        left.push(second);
    }

    assert!(
        b.game.hand(b.p1).ends_with(&taken),
        "each resolution puts its own pick into P1's hand"
    );
    assert!(
        b.game.deck(b.p1).ends_with(&left),
        "each resolution buries the card it left"
    );
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
    ];
    assert!(
        order.is_sorted(),
        "Sweep's draw, the Hoarder's death, then the Mourner's draw:\n{}",
        listing(&picked)
    );
    assert_eq!(
        b.game.pick_options(b.p1).len(),
        2,
        "the Hoarder's pick opens after the Mourner's draw, and the game waits on it"
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
    let card = b.game.pick_options(b.p0)[0];

    b.game.apply(b.p0, pick(card), &mut ()).unwrap();
    clone.apply(b.p0, pick(card), &mut ()).unwrap();

    assert_eq!(clone.health(bomber), None, "the clone finished Sweep");
    assert_eq!(clone, b.game);
}
