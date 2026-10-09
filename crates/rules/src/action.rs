//! What a player sends in, and what comes back when it isn't legal.

use crate::{ObjectId, choice::ChoiceId, ids::PlayerId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Play {
        object_id: ObjectId,
    },
    Pick {
        object_id: ObjectId,
    },
    Draft {
        object_id: ObjectId,
    },
    Choose {
        choice_id: ChoiceId,
        object_id: ObjectId,
    },
    Cancel {
        object_id: ObjectId,
    },
    EndTurn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IllegalAction {
    pub player_id: PlayerId,
    pub action: Action,
}

impl IllegalAction {
    pub(crate) fn new(player_id: PlayerId, action: Action) -> Self {
        Self { player_id, action }
    }
}
