use crate::cards::definition::CardKind;
use crate::effect::EffectAmount;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmountBound {
    Exactly,
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
pub struct Choice<O> {
    pub options: O,
    pub count: EffectAmount,
    pub bound: AmountBound,
    pub different: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardTargetOptions {
    pub zone: CardTargetZone,
    pub kind: enumset::EnumSet<CardKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardTargetZone {
    Hand,
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
