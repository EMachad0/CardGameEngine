//! Which objects a player may target next for a card, so that its targeting can still be finished.

use crate::{
    cards::object::ObjectId,
    game::{Game, player_input::Targeting},
    target::{CharacterTarget, TargetId},
};

impl Game {
    /// Whether the card's targeting can be finished from nothing targeted.
    pub(super) fn can_start_targeting(&self, object_id: ObjectId) -> bool {
        self.can_finish_targeting(object_id, &mut Vec::new())
    }

    /// Whether the player may stop at every target and play the card.
    pub(super) fn may_stop_targeting(&self, targeting: &Targeting) -> bool {
        self.first_unsatisfied_target(targeting.object_id(), targeting.targeted())
            .is_none()
    }

    /// The objects the player may target next, for the first target they may not stop at. Each one
    /// leaves the targeting finishable.
    pub(super) fn next_candidates(&self, targeting: &Targeting) -> Vec<(TargetId, ObjectId)> {
        let object_id = targeting.object_id();
        let mut targeted = targeting.targeted().to_vec();
        let Some(target) = self.first_unsatisfied_target(object_id, &targeted) else {
            return Vec::new();
        };
        self.finishing_candidates(object_id, target, &mut targeted)
            .map(|candidate| (target.id, candidate))
            .collect()
    }

    fn can_finish_targeting(
        &self,
        object_id: ObjectId,
        targeted: &mut Vec<(ObjectId, TargetId)>,
    ) -> bool {
        match self.first_unsatisfied_target(object_id, targeted) {
            None => true,
            Some(target) => self
                .finishing_candidates(object_id, target, targeted)
                .next()
                .is_some(),
        }
    }

    fn finishing_candidates<'a>(
        &'a self,
        object_id: ObjectId,
        target: &'a CharacterTarget,
        targeted: &'a mut Vec<(ObjectId, TargetId)>,
    ) -> impl Iterator<Item = ObjectId> + 'a {
        let candidates = self.untargeted_matches(object_id, target, targeted);
        candidates.into_iter().filter(move |&candidate| {
            targeted.push((candidate, target.id));
            let can_finish = self.can_finish_targeting(object_id, targeted);
            targeted.pop();
            can_finish
        })
    }

    fn first_unsatisfied_target(
        &self,
        object_id: ObjectId,
        targeted: &[(ObjectId, TargetId)],
    ) -> Option<&CharacterTarget> {
        self.card_def(object_id).targets.iter().find(|target| {
            let count = targeted_for(targeted, target.id).count() as u8;
            !target
                .bound
                .is_satisfied(count, self.required_count(object_id, target))
        })
    }

    fn untargeted_matches(
        &self,
        object_id: ObjectId,
        target: &CharacterTarget,
        targeted: &[(ObjectId, TargetId)],
    ) -> Vec<ObjectId> {
        let objects_for_target = targeted_for(targeted, target.id).collect::<Vec<_>>();
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
            .filter(|candidate| {
                !target.different || targeted.iter().all(|(object, _)| object != candidate)
            })
            .filter(|candidate| !objects_for_target.contains(candidate))
            .collect()
    }

    fn required_count(&self, object_id: ObjectId, target: &CharacterTarget) -> u8 {
        let owner = self.objects.get(object_id).player_id;
        self.effect_amount(&target.count, owner)
    }
}

fn targeted_for(
    targeted: &[(ObjectId, TargetId)],
    target_id: TargetId,
) -> impl Iterator<Item = ObjectId> + '_ {
    targeted
        .iter()
        .filter(move |(_, id)| *id == target_id)
        .map(|(object_id, _)| *object_id)
}
