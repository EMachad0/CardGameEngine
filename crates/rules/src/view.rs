//! What one player can see. `Game::view` is the one definition of it.

use crate::{DefId, ObjectId, Outcome, PlayerId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct View {
    pub viewer: PlayerId,
    pub active_player: PlayerId,
    /// Indexed by `PlayerId::idx`.
    pub players: Vec<PlayerView>,
    pub outcome: Option<Outcome>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerView {
    pub player_id: PlayerId,
    pub hero_health: i32,
    pub mana: u8,
    pub max_mana: u8,
    /// Oldest first.
    pub hand: Vec<HandCard>,
    pub deck_size: usize,
    /// Left to right.
    pub board: Vec<BoardCard>,
    /// A pending Forage's options, in reveal order. Empty if none is pending.
    pub revealed: Vec<RevealedCard>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandCard {
    pub object_id: ObjectId,
    /// `None` if not currently visible.
    pub face: Option<Face>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Face {
    pub def_id: DefId,
    pub mana_cost: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RevealedCard {
    pub object_id: ObjectId,
    /// `None` unless the viewer is the one choosing.
    pub def_id: Option<DefId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardCard {
    pub object_id: ObjectId,
    pub def_id: DefId,
    pub attack: i32,
    pub health: i32,
}
