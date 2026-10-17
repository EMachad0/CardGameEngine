//! One player's record inside `Game`: resources and any pending choice.
//!
//! Fields are `pub(super)`, so `Game` can change them and nothing outside
//! `game` can see them.

use crate::cards::object::ObjectId;
use crate::choice::{ChoiceId, PickId};
use crate::ids::PlayerId;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) enum PlayerInteractionState {
    #[default]
    Idle,
    PendingPick {
        source: ObjectId,
        pick_id: PickId,
        options: Vec<ObjectId>,
    },
    Draft {
        of_object_id: ObjectId,
        chosen: Vec<(ObjectId, ChoiceId)>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Player {
    pub(super) id: PlayerId,
    pub(super) mana: u8,
    pub(super) max_mana: u8,
    pub(super) interaction_state: PlayerInteractionState,
    pub(super) playing: bool,
}

impl Player {
    pub(super) fn new(id: PlayerId) -> Self {
        Self {
            id,
            mana: 0,
            max_mana: 0,
            interaction_state: PlayerInteractionState::default(),
            playing: true,
        }
    }
}
