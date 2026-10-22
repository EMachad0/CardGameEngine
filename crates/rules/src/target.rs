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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DeclaredTarget(TargetId);

impl DeclaredTarget {
    pub(crate) fn id(self) -> TargetId {
        self.0
    }
}

/// What a `Targeted` selector holds in text that resolves with no targeting, such as a trigger's.
/// It has no values, so that text can't name a target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoTarget {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharacterTarget {
    pub filter: CharacterFilter,
    pub count: EffectAmount,
    pub bound: AmountBound,
    pub different: bool,
}

/// A card's targets, in the order declared. Each one's id is its place in the list.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Targets(Vec<CharacterTarget>);

impl Targets {
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (TargetId, &CharacterTarget)> {
        self.0
            .iter()
            .enumerate()
            .map(|(idx, target)| (TargetId(idx), target))
    }

    pub(crate) fn declared(&self, id: TargetId) -> Option<DeclaredTarget> {
        (id.0 < self.0.len()).then_some(DeclaredTarget(id))
    }
}

impl From<Vec<CharacterTarget>> for Targets {
    fn from(targets: Vec<CharacterTarget>) -> Self {
        Self(targets)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn target(different: bool) -> CharacterTarget {
        CharacterTarget {
            filter: CharacterFilter::all(),
            count: 1.into(),
            bound: AmountBound::Exactly,
            different,
        }
    }

    #[test]
    fn exactly_is_full_at_the_required_count() {
        assert!(AmountBound::Exactly.is_full(2, 2));
    }

    #[test]
    fn exactly_is_not_full_below_the_required_count() {
        assert!(!AmountBound::Exactly.is_full(1, 2));
    }

    #[test]
    fn each_targets_id_is_its_place_in_the_order_declared() {
        let (first, second) = (target(true), target(false));
        let targets = Targets::from(vec![first, second]);

        assert_eq!(
            targets.iter().collect::<Vec<_>>(),
            [(TargetId(0), &first), (TargetId(1), &second)],
        );
    }

    #[test]
    fn the_id_of_a_target_on_the_list_is_declared() {
        let targets = Targets::from(vec![target(true), target(false)]);

        assert_eq!(
            targets.declared(TargetId(1)).map(DeclaredTarget::id),
            Some(TargetId(1))
        );
    }

    #[test]
    fn an_id_past_the_last_target_is_not_declared() {
        let targets = Targets::from(vec![target(true), target(false)]);

        assert_eq!(targets.declared(TargetId(2)), None);
    }
}
