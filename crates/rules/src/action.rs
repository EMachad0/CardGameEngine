//! What a player sends in, and what comes back when it isn't legal.

use crate::ids::PlayerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Play { hand_index: usize },
    Pick { index: usize },
    EndTurn,
}

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
