use crate::cards::{
    definition::TriggerDef,
    filter::{CardFilter, CharacterFilter},
    object::ObjectId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Trigger {
    pub granted_by: ObjectId,
    pub def: TriggerDef,
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
