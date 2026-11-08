use crate::cards::filter::{CardFilter, CharacterFilter};
use crate::cards::object::ObjectId;
use crate::cards::parsed::TriggerDef;

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
    Summoned(TriggerSubject),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerSubject {
    Itself,
    Character(CharacterFilter),
}
