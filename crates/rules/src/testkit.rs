//! Helpers shared by the in-file tests. Built only under `cfg(test)`.

use crate::action::Action;
use crate::cards::Card;
use crate::game::Game;
use crate::ids::PlayerId;

pub(crate) const P0: PlayerId = PlayerId::new(0);
pub(crate) const P1: PlayerId = PlayerId::new(1);

pub(crate) fn bolt(damage: u8) -> Card {
    Card::Bolt { damage }
}

pub(crate) fn play(hand_index: usize) -> Action {
    Action::Play { hand_index }
}

pub(crate) fn pick(index: usize) -> Action {
    Action::Pick { index }
}

/// Asserts that `legal_actions(p)` holds the same actions as `expected`, in any order.
pub(crate) fn assert_actions(game: &Game, p: PlayerId, expected: &[Action]) {
    let actual = game.legal_actions(p);
    let missing: Vec<&Action> = expected.iter().filter(|a| !actual.contains(a)).collect();
    let extra: Vec<&Action> = actual.iter().filter(|a| !expected.contains(a)).collect();
    assert!(
        missing.is_empty() && extra.is_empty() && actual.len() == expected.len(),
        "legal_actions({p:?}) = {actual:?}\n  expected {expected:?}\n  missing {missing:?}\n  extra {extra:?}"
    );
}
