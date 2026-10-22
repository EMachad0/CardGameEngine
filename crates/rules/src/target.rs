use crate::cards::filter::CharacterFilter;
use crate::effect::EffectAmount;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmountBound {
    Exactly,
    // AtMost,
    // AtLeast,
}

impl AmountBound {
    pub(crate) fn is_satisfied<T: PartialOrd>(&self, count: T, required: T) -> bool {
        match self {
            Self::Exactly => count == required,
        }
    }

    pub(crate) fn is_full<T: PartialOrd>(&self, count: T, required: T) -> bool {
        self.upper_bound(required).is_some_and(|max| count >= max)
    }

    fn upper_bound<T>(&self, required: T) -> Option<T> {
        match self {
            Self::Exactly => Some(required),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetId(pub usize);

/// It has no values, so a `Targeted` selector that holds it can't be written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoTarget {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharacterTarget {
    pub filter: CharacterFilter,
    pub count: EffectAmount,
    pub bound: AmountBound,
    pub different: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exactly_is_full_at_the_required_count() {
        assert!(AmountBound::Exactly.is_full(2, 2));
    }

    #[test]
    fn exactly_is_not_full_below_the_required_count() {
        assert!(!AmountBound::Exactly.is_full(1, 2));
    }
}
