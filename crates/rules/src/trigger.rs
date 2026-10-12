use crate::{
    ObjectId,
    cards::filter::{CardFilter, CharacterFilter},
    effect::EffectSequence,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Trigger {
    pub source: ObjectId,
    pub pattern: TriggerPattern,
    pub effects: EffectSequence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerPattern {
    Died(TriggerSubject),
    Damaged(TriggerSubject),
    Played(CardFilter),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerSubject {
    Itself,
    Character(CharacterFilter),
}
