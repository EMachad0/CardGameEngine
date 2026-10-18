use enumset::EnumSet;

use crate::{
    cards::{
        definition::{CardKind, CharacterKind},
        filter::{CardFilter, CharacterFilter, fulfill_side_filter},
        object::ObjectId,
    },
    game::Game,
    ids::PlayerId,
    target::{CharacterTarget, TargetId},
};

impl Game {
    pub(crate) fn scan_for_valid_targets(
        &self,
        object_id: ObjectId,
        target: &CharacterTarget,
        targeted: &[(ObjectId, TargetId)],
    ) -> Vec<ObjectId> {
        let objects_for_target = targeted
            .iter()
            .filter(|(_, target_id)| target.id == *target_id)
            .map(|(o, _)| *o)
            .collect::<Vec<_>>();
        if self.check_target_count_fulfilled(object_id, target.id, objects_for_target.len() as u8) {
            return Vec::new();
        }

        let obj = self.objects.get(object_id);
        self.scan_characters_with_filter(obj.player_id, &target.filter)
            .into_iter()
            .filter(|o| !target.different || targeted.iter().all(|(o2, _)| o != o2))
            .filter(|o| !objects_for_target.contains(o))
            .collect()
    }

    pub(crate) fn check_target_count_fulfilled(
        &self,
        object_id: ObjectId,
        target_id: TargetId,
        targeted_count: u8,
    ) -> bool {
        let obj = self.objects.get(object_id);
        let target = self.card_def(object_id).target(target_id).unwrap();
        let count = self.effect_amount(&target.count, obj.player_id);
        target.bound.is_satisfied(targeted_count, count)
    }

    pub(crate) fn scan_characters_with_filter(
        &self,
        asking: PlayerId,
        filter: &CharacterFilter,
    ) -> Vec<ObjectId> {
        self.zones
            .characters_in_play()
            .filter(|&object_id| {
                let owner = self.objects.get(object_id).player_id;
                self.fulfill_character_filter(asking, owner, object_id, filter)
            })
            .collect()
    }

    pub(crate) fn fulfill_character_filter(
        &self,
        asking: PlayerId,
        owner: PlayerId,
        object_id: ObjectId,
        &CharacterFilter { kind, side }: &CharacterFilter,
    ) -> bool {
        fulfill_side_filter(side, asking, owner)
            && self.fulfill_character_kind_filter(object_id, kind)
    }

    pub(crate) fn fulfill_card_filter(
        &self,
        asking: PlayerId,
        owner: PlayerId,
        object_id: ObjectId,
        &CardFilter { kind, side }: &CardFilter,
    ) -> bool {
        fulfill_side_filter(side, asking, owner) && self.fulfill_card_kind_filter(object_id, kind)
    }

    fn fulfill_character_kind_filter(
        &self,
        object_id: ObjectId,
        kind: EnumSet<CharacterKind>,
    ) -> bool {
        self.card_def(object_id)
            .character_kind()
            .is_some_and(|character_kind| kind.contains(character_kind))
    }

    fn fulfill_card_kind_filter(&self, object_id: ObjectId, kind: EnumSet<CardKind>) -> bool {
        kind.contains(self.card_def(object_id).card_kind())
    }

    pub(super) fn can_finish_targeting(
        &self,
        object_id: ObjectId,
        targeted: &mut Vec<(ObjectId, TargetId)>,
    ) -> bool {
        let mut any_unfulfiled = false;
        for target in &self.card_def(object_id).targets {
            let targeted_count = targeted
                .iter()
                .filter(|(_, target_id)| target.id == *target_id)
                .count();
            if !self.check_target_count_fulfilled(object_id, target.id, targeted_count as u8) {
                let candidates = self.scan_for_valid_targets(object_id, target, targeted);
                for candidate in candidates.into_iter() {
                    targeted.push((candidate, target.id));
                    let can_finish = self.can_finish_targeting(object_id, targeted);
                    if can_finish {
                        return true;
                    }
                    targeted.pop();
                }
                any_unfulfiled = true;
                break;
            }
        }
        !any_unfulfiled
    }
}
