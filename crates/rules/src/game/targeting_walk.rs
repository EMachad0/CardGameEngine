//! Whether a card's targeting can be finished, which objects the player may target next, and
//! whether they may stop.

use crate::{
    cards::object::ObjectId,
    game::{Game, player_input::Targeting},
    target::{CharacterTarget, TargetId},
};

impl Game {
    /// Whether the card's targeting can be finished from nothing targeted.
    pub(super) fn can_start_targeting(&self, object_id: ObjectId) -> bool {
        self.can_finish_targeting(&mut Targeting::new(object_id))
    }

    /// Whether the player may stop at every target and play the card.
    pub(super) fn may_stop_targeting(&self, targeting: &Targeting) -> bool {
        self.first_unsatisfied_target(targeting).is_none()
    }

    /// The objects the player may target next, for the first target they may not stop at. Each one
    /// leaves the targeting finishable.
    pub(super) fn next_candidates(&self, targeting: &Targeting) -> Vec<(TargetId, ObjectId)> {
        let Some(target) = self.first_unsatisfied_target(targeting) else {
            return Vec::new();
        };
        self.finishing_candidates(target, &mut targeting.clone())
            .map(|object_id| (target.id, object_id))
            .collect()
    }

    fn can_finish_targeting(&self, targeting: &mut Targeting) -> bool {
        match self.first_unsatisfied_target(targeting) {
            None => true,
            Some(target) => self
                .finishing_candidates(target, targeting)
                .next()
                .is_some(),
        }
    }

    fn finishing_candidates<'a>(
        &'a self,
        target: &'a CharacterTarget,
        targeting: &'a mut Targeting,
    ) -> impl Iterator<Item = ObjectId> + 'a {
        let candidates = self.untargeted_matches(target, targeting);
        candidates.into_iter().filter(move |&candidate| {
            targeting.with_targeted(candidate, target.id, |targeting| {
                self.can_finish_targeting(targeting)
            })
        })
    }

    fn first_unsatisfied_target(&self, targeting: &Targeting) -> Option<&CharacterTarget> {
        let object_id = targeting.object_id();
        self.card_def(object_id).targets.iter().find(|target| {
            let count = targeting.targeted_for(target.id).count() as u8;
            !target
                .bound
                .is_satisfied(count, self.required_count(object_id, target))
        })
    }

    fn untargeted_matches(&self, target: &CharacterTarget, targeting: &Targeting) -> Vec<ObjectId> {
        let object_id = targeting.object_id();
        let objects_for_target = targeting.targeted_for(target.id).collect::<Vec<_>>();
        let count = objects_for_target.len() as u8;
        if target
            .bound
            .is_full(count, self.required_count(object_id, target))
        {
            return Vec::new();
        }

        let owner = self.objects.get(object_id).player_id;
        self.scan_characters_with_filter(owner, &target.filter)
            .into_iter()
            .filter(|&candidate| !target.different || !targeting.is_targeted(candidate))
            .filter(|candidate| !objects_for_target.contains(candidate))
            .collect()
    }

    fn required_count(&self, object_id: ObjectId, target: &CharacterTarget) -> u8 {
        let owner = self.objects.get(object_id).player_id;
        self.effect_amount(&target.count, owner)
    }
}
