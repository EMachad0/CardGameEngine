use crate::{
    DefId, ObjectId, PickId,
    cards::{
        definition::{CharacterSelector, PlayerSelector},
        modifier::ModifierEffect,
    },
    choice::EffectAmount,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneMove {
    AddToHand,
    Bury,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
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
        selector: CharacterSelector,
        amount: EffectAmount,
    },
    Buff {
        selector: CharacterSelector,
        effect: ModifierEffect,
    },
    AddFriendlyAura {
        selector: CharacterSelector,
        effect: ModifierEffect,
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

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EffectSequence(pub Vec<Effect>);

impl EffectSequence {
    pub fn as_slice(&self) -> &[Effect] {
        self.0.as_slice()
    }
}

impl From<Vec<Effect>> for EffectSequence {
    fn from(value: Vec<Effect>) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PendingEffect {
    pub object_id: ObjectId,
    pub effects: EffectSequence,
}

impl PendingEffect {
    pub(crate) fn new<I: Into<EffectSequence>>(object_id: ObjectId, effects: I) -> Self {
        Self {
            object_id,
            effects: effects.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvingEffect {
    pub object_id: ObjectId,
    pub effects: EffectSequence,
    pub idx: usize,
}

impl ResolvingEffect {
    pub(crate) fn new<I: Into<EffectSequence>>(object_id: ObjectId, effects: I) -> Self {
        Self {
            object_id,
            effects: effects.into(),
            idx: 0,
        }
    }

    pub(crate) fn curr_effect(&self) -> Option<&Effect> {
        self.effects.as_slice().get(self.idx)
    }

    pub(crate) fn is_finished(&self) -> bool {
        self.effects.as_slice().len() <= self.idx
    }

    pub(crate) fn advance(&mut self) {
        self.idx += 1;
    }
}

impl From<PendingEffect> for ResolvingEffect {
    fn from(value: PendingEffect) -> Self {
        Self::new(value.object_id, value.effects)
    }
}
