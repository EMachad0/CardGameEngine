use crate::{ObjectId, history::HistoryQuery};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Modifiers(Vec<Modifier>);

impl Modifiers {
    pub(crate) fn new(modifiers: Vec<Modifier>) -> Self {
        Self(modifiers)
    }

    pub(crate) fn add(&mut self, modifier: Modifier) {
        self.0.push(modifier);
    }

    pub(crate) fn extend(&mut self, modifiers: Self) {
        self.0.extend(modifiers.0);
    }

    pub(crate) fn as_slice(&self) -> &[Modifier] {
        self.0.as_slice()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Modifier {
    pub source: ObjectId,
    pub effect: ModifierEffect,
}

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
pub enum ModifierEffect {
    BuffAtk { amount: EffectAmount },
    BuffHealth { amount: EffectAmount },
    ReduceManaCost { amount: EffectAmount },
}
