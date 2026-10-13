use crate::cards::{
    effect::EffectSequence,
    filter::{CardFilter, CharacterFilter},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerDef {
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
