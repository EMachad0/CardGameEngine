use crate::cards::definition::{CharacterSelector, DefId, PlayerSelector};
use crate::cards::modifier::ModifierEffect;
use crate::history::HistoryQuery;
use crate::target::TargetId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectAmount {
    Static(u8),
    History(HistoryQuery),
}

impl From<u8> for EffectAmount {
    fn from(value: u8) -> Self {
        Self::Static(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PickId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneMove {
    AddToHand,
    Bury,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect<T = TargetId, G = T> {
    Draw {
        selector: PlayerSelector,
        amount: EffectAmount,
    },
    Reveal {
        pick_id: PickId,
        selector: PlayerSelector,
        amount: EffectAmount,
    },
    Damage {
        selector: CharacterSelector<T>,
        amount: EffectAmount,
    },
    Buff {
        selector: CharacterSelector<T>,
        effect: ModifierEffect<G>,
    },
    AddFriendlyAura {
        selector: CharacterSelector<T>,
        effect: ModifierEffect<G>,
    },
    Summon {
        selector: PlayerSelector,
        def_id: DefId,
    },
    ResolvePick {
        pick_id: PickId,
        picked_move: ZoneMove,
        other_move: ZoneMove,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectSequence<T = TargetId>(pub Vec<Effect<T>>);

impl<T> EffectSequence<T> {
    pub fn as_slice(&self) -> &[Effect<T>] {
        self.0.as_slice()
    }
}

impl<T> Default for EffectSequence<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T> From<Vec<Effect<T>>> for EffectSequence<T> {
    fn from(value: Vec<Effect<T>>) -> Self {
        Self(value)
    }
}
