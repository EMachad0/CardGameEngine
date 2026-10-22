//! How a targeting is built: its card, and each object targeted for one of the card's targets, in
//! the order targeted.

use crate::cards::object::ObjectId;
use crate::target::TargetId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::game) struct Targeting {
    object_id: ObjectId,
    targeted: Vec<(ObjectId, TargetId)>,
}

impl Targeting {
    /// With nothing targeted yet.
    pub(in crate::game) fn new(object_id: ObjectId) -> Self {
        Self {
            object_id,
            targeted: Vec::new(),
        }
    }

    pub(in crate::game) fn object_id(&self) -> ObjectId {
        self.object_id
    }

    /// In the order targeted.
    pub(in crate::game) fn targeted_objects(&self) -> impl Iterator<Item = ObjectId> {
        self.targeted.iter().map(|&(object_id, _)| object_id)
    }

    /// In the order targeted.
    pub(in crate::game) fn targeted_for(
        &self,
        target_id: TargetId,
    ) -> impl Iterator<Item = ObjectId> {
        self.targeted
            .iter()
            .filter(move |&&(_, id)| id == target_id)
            .map(|&(object_id, _)| object_id)
    }

    /// Whether `object_id` is targeted for any target.
    pub(in crate::game) fn is_targeted(&self, object_id: ObjectId) -> bool {
        self.targeted_objects()
            .any(|targeted| targeted == object_id)
    }

    pub(super) fn target(&mut self, object_id: ObjectId, target_id: TargetId) {
        self.targeted.push((object_id, target_id));
    }

    /// Runs `f` on this targeting with `object_id` also targeted for `target_id`, then takes it
    /// back out.
    pub(in crate::game) fn with_targeted<R>(
        &mut self,
        object_id: ObjectId,
        target_id: TargetId,
        f: impl FnOnce(&mut Self) -> R,
    ) -> R {
        self.target(object_id, target_id);
        let result = f(self);
        self.targeted.pop();
        result
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::cards::binder::sets::base::static_card_definition::SPARK;
    use crate::cards::object::ObjectBag;
    use crate::ids::PlayerId;

    fn ids<const N: usize>() -> [ObjectId; N] {
        let mut bag = ObjectBag::default();
        std::array::from_fn(|_| bag.insert(SPARK, PlayerId::new(0)))
    }

    #[test]
    fn targeted_for_lists_one_targets_objects_in_the_order_targeted() {
        let [card, first, other, second] = ids();
        let mut targeting = Targeting::new(card);
        targeting.target(first, TargetId(0));
        targeting.target(other, TargetId(1));
        targeting.target(second, TargetId(0));

        assert_eq!(
            targeting.targeted_for(TargetId(0)).collect::<Vec<_>>(),
            [first, second]
        );
    }

    #[test]
    fn an_object_targeted_for_any_target_is_targeted() {
        let [card, object] = ids();
        let mut targeting = Targeting::new(card);

        targeting.target(object, TargetId(1));

        assert!(targeting.is_targeted(object));
    }

    #[test]
    fn an_object_not_yet_targeted_is_not_targeted() {
        let [card, object, other] = ids();
        let mut targeting = Targeting::new(card);

        targeting.target(object, TargetId(0));

        assert!(!targeting.is_targeted(other));
    }

    #[test]
    fn targeting_lists_its_targeted_objects_in_the_order_targeted() {
        let [card, first, second] = ids();
        let mut targeting = Targeting::new(card);
        targeting.target(first, TargetId(1));
        targeting.target(second, TargetId(0));

        assert_eq!(
            targeting.targeted_objects().collect::<Vec<_>>(),
            [first, second]
        );
    }

    #[test]
    fn with_targeted_runs_f_with_the_object_also_targeted() {
        let [card, first, second] = ids();
        let mut targeting = Targeting::new(card);
        targeting.target(first, TargetId(0));

        let targeted = targeting.with_targeted(second, TargetId(0), |targeting| {
            targeting.targeted_for(TargetId(0)).collect::<Vec<_>>()
        });

        assert_eq!(targeted, [first, second]);
    }

    #[test]
    fn with_targeted_leaves_the_targeting_as_it_found_it() {
        let [card, first, second] = ids();
        let mut targeting = Targeting::new(card);
        targeting.target(first, TargetId(0));
        let before = targeting.clone();

        targeting.with_targeted(second, TargetId(0), |_| {});

        assert_eq!(targeting, before);
    }
}
