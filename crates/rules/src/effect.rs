use crate::{
    cards::effect::{Effect, EffectSequence},
    object::ObjectId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PendingEffect {
    pub source: ObjectId,
    pub effects: EffectSequence,
}

impl PendingEffect {
    pub(crate) fn new<I: Into<EffectSequence>>(source: ObjectId, effects: I) -> Self {
        Self {
            source,
            effects: effects.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvingEffect {
    pub source: ObjectId,
    pub effects: EffectSequence,
    pub idx: usize,
}

impl ResolvingEffect {
    pub(crate) fn new<I: Into<EffectSequence>>(source: ObjectId, effects: I) -> Self {
        Self {
            source,
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
        Self::new(value.source, value.effects)
    }
}
