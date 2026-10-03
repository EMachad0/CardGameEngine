//! One player's record inside `Game`: resources, zones, and any pending choice.
//!
//! Fields are `pub(super)`, so `Game` can change them and nothing outside
//! `game` can see them.

use crate::ObjectId;
use crate::ids::PlayerId;
use crate::zones::Zones;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) enum PlayerInteractionState {
    #[default]
    Idle,
    PendingPick {
        options: Vec<ObjectId>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Player {
    pub(super) id: PlayerId,
    pub(super) mana: u8,
    pub(super) max_mana: u8,
    pub(super) health: i32,
    pub(super) interaction_state: PlayerInteractionState,
    pub(super) zones: Zones,
}

impl Player {
    pub(super) fn new(id: PlayerId) -> Self {
        Self {
            id,
            mana: 0,
            max_mana: 0,
            health: 10,
            interaction_state: PlayerInteractionState::default(),
            zones: Zones::default(),
        }
    }
}
