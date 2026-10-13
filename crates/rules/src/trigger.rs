use crate::{
    cards::{effect::EffectSequence, trigger::TriggerPattern},
    object::ObjectId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Trigger {
    pub source: ObjectId,
    pub pattern: TriggerPattern,
    pub effects: EffectSequence,
}
