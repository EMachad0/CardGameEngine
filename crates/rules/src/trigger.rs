use crate::{
    ObjectId,
    cards::definition::{EffectSequence, TriggerPattern},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PendingEffect {
    pub object_id: ObjectId,
    pub effects: EffectSequence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Trigger {
    pub source: ObjectId,
    pub pattern: TriggerPattern,
    pub effects: EffectSequence,
}
