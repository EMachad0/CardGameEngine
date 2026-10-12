use crate::{
    DefId, ObjectId,
    cards::{
        definition::{CharacterSelector, PlayerSelector},
        modifier::ModifierEffect,
    },
    choice::EffectAmount,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    Draw {
        selector: PlayerSelector,
        amount: EffectAmount,
    },
    Reveal {
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
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EffectSequence(pub Vec<Effect>);

impl EffectSequence {
    pub fn as_slice(&self) -> &[Effect] {
        self.0.as_slice()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PendingEffect {
    pub object_id: ObjectId,
    pub effects: EffectSequence,
}
