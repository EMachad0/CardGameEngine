//! One player's record inside `Game`: resources.
//!
//! Fields are `pub(super)`, so `Game` can change them and nothing outside
//! `game` can see them.

use crate::ids::PlayerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Player {
    pub(super) id: PlayerId,
    pub(super) mana: u8,
    pub(super) max_mana: u8,
    pub(super) playing: bool,
}

impl Player {
    pub(super) fn new(id: PlayerId) -> Self {
        Self {
            id,
            mana: 0,
            max_mana: 0,
            playing: true,
        }
    }
}
