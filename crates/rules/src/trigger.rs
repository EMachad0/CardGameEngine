use crate::cards::definition::TriggerDef;
use crate::cards::filter::{CardFilter, CharacterFilter};
use crate::cards::object::ObjectId;
use crate::target::NoTarget;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Trigger {
    pub granted_by: ObjectId,
    pub def: TriggerDef<NoTarget>,
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
