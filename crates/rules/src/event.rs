//! What `apply` reports as it resolves.
//! Events name objects by `ObjectId` only:
//! a card's identity and current values reach a viewer through its `View`.

use crate::{ObjectId, Outcome, PlayerId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    TurnStarted {
        player_id: PlayerId,
    },
    TurnEnded {
        player_id: PlayerId,
    },
    Drew {
        player_id: PlayerId,
        object_id: ObjectId,
    },
    Played {
        player_id: PlayerId,
        object_id: ObjectId,
    },
    Damaged {
        target: Target,
        amount: u8,
        source: ObjectId,
    },
    Revealed {
        player_id: PlayerId,
        object_ids: Vec<ObjectId>,
    },
    Picked {
        player_id: PlayerId,
        object_id: ObjectId,
    },
    /// One per card, in the order they go to the bottom.
    Buried {
        player_id: PlayerId,
        object_id: ObjectId,
    },
    BoardEntered {
        player_id: PlayerId,
        object_id: ObjectId,
    },
    /// A draw from an empty deck.
    FatigueDamaged {
        amount: u8,
        player_id: PlayerId,
    },
    Died {
        object_id: ObjectId,
    },
    GameEnded {
        outcome: Outcome,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Hero(PlayerId),
    Minion(ObjectId),
}
