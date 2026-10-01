//! What a player sends in, and what comes back when it isn't legal.

use crate::{ObjectId, ids::PlayerId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Play { object_id: ObjectId },
    Pick { object_id: ObjectId },
    EndTurn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IllegalAction {
    pub player_id: PlayerId,
    pub action: Action,
}

impl IllegalAction {
    pub(crate) fn new(player_id: PlayerId, action: Action) -> Self {
        Self { player_id, action }
    }
}
