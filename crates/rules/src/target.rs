use crate::{cards::filter::CharacterFilter, effect::EffectAmount};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmountBound {
    Exactly,
    // AtMost,
    // AtLeast,
}

impl AmountBound {
    pub fn is_satisfied<T: PartialOrd>(&self, count: T, required: T) -> bool {
        match self {
            Self::Exactly => count == required,
        }
    }

    pub fn upper_bound<T>(&self, required: T) -> Option<T> {
        match self {
            Self::Exactly => Some(required),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharacterTarget {
    pub id: TargetId,
    pub filter: CharacterFilter,
    pub count: EffectAmount,
    pub bound: AmountBound,
    pub different: bool,
}
