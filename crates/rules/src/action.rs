//! What a player sends in, and what comes back when it isn't legal.

use crate::ids::PlayerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Play { hand_index: usize },
    Pick { index: usize },
    EndTurn,
}

/// Returned by `Game::apply` for any action not in `legal_actions`.
/// The game is unchanged when you get one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Illegal {
    pub player: PlayerId,
    pub action: Action,
}

impl Illegal {
    pub(crate) fn new(player: PlayerId, action: Action) -> Self {
        Self { player, action }
    }
}
