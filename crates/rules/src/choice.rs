use crate::{ObjectId, cards::definition::CharacterSelectorFilter, history::HistoryQuery};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectAmount {
    Static(u8),
    History(HistoryQuery),
}

impl From<u8> for EffectAmount {
    fn from(value: u8) -> Self {
        Self::Static(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmountBound {
    Exactly,
    AtMost,
    AtLeast,
}

impl AmountBound {
    pub fn is_satisfied<T: PartialOrd>(&self, chosen: T, target: T) -> bool {
        match self {
            Self::Exactly => chosen == target,
            Self::AtMost => chosen <= target,
            Self::AtLeast => chosen >= target,
        }
    }

    pub fn upper_bound<T>(&self, target: T) -> Option<T> {
        match self {
            Self::Exactly | Self::AtMost => Some(target),
            Self::AtLeast => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChoiceId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharacterChoice {
    pub id: ChoiceId,
    pub filter: CharacterSelectorFilter,
    pub count: EffectAmount,
    pub bound: AmountBound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChoiceTarget {
    pub choice_id: ChoiceId,
    pub targets: Vec<ObjectId>,
}

impl ChoiceTarget {
    pub(crate) fn new(choice_id: ChoiceId) -> Self {
        Self {
            choice_id,
            targets: Default::default(),
        }
    }
}
