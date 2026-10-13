use crate::cards::{amount::EffectAmount, filter::CharacterFilter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmountBound {
    Exactly,
    // AtMost,
    // AtLeast,
}

impl AmountBound {
    pub fn is_satisfied<T: PartialOrd>(&self, chosen: T, target: T) -> bool {
        match self {
            Self::Exactly => chosen == target,
        }
    }

    pub fn upper_bound<T>(&self, target: T) -> Option<T> {
        match self {
            Self::Exactly => Some(target),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChoiceId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharacterChoice {
    pub id: ChoiceId,
    pub filter: CharacterFilter,
    pub count: EffectAmount,
    pub bound: AmountBound,
    pub different: bool,
}
