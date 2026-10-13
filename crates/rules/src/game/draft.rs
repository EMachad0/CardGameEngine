use crate::{
    cards::choice::{CharacterChoice, ChoiceId},
    game::Game,
    object::ObjectId,
};

impl Game {
    pub(super) fn scan_for_valid_choices(
        &self,
        chooser_id: ObjectId,
        choice: &CharacterChoice,
        chosen: &[(ObjectId, ChoiceId)],
    ) -> Vec<ObjectId> {
        let chosen_for_choice = chosen
            .iter()
            .filter(|(_, choice_id)| choice.id == *choice_id)
            .map(|(o, _)| *o)
            .collect::<Vec<_>>();
        if self.check_choice_count_fulfilled(chooser_id, choice.id, chosen_for_choice.len() as u8) {
            return Vec::new();
        }

        let obj = self.objects.get(chooser_id);
        self.scan_characters_with_filter(obj.player_id, &choice.filter)
            .into_iter()
            .filter(|o| !choice.different || chosen.iter().all(|(o2, _)| o != o2))
            .filter(|o| !chosen_for_choice.contains(o))
            .collect()
    }

    pub(super) fn check_choice_count_fulfilled(
        &self,
        object_id: ObjectId,
        choice_id: ChoiceId,
        chosen_count: u8,
    ) -> bool {
        let obj = self.objects.get(object_id);
        let choice = self
            .card_def(object_id)
            .choices
            .iter()
            .find(|c| c.id == choice_id)
            .unwrap();
        let count = self.effect_amount(&choice.count, obj.player_id);
        choice.bound.is_satisfied(chosen_count, count)
    }

    pub(super) fn can_finish_choices(
        &self,
        object_id: ObjectId,
        chosen: &mut Vec<(ObjectId, ChoiceId)>,
    ) -> bool {
        let mut any_unfulfiled = false;
        for choice in &self.card_def(object_id).choices {
            let chosen_for_choice_count = chosen
                .iter()
                .filter(|(_, choice_id)| choice.id == *choice_id)
                .count();
            if !self.check_choice_count_fulfilled(
                object_id,
                choice.id,
                chosen_for_choice_count as u8,
            ) {
                let candidates = self.scan_for_valid_choices(object_id, choice, chosen);
                for candidate in candidates.into_iter() {
                    chosen.push((candidate, choice.id));
                    let can_finish = self.can_finish_choices(object_id, chosen);
                    if can_finish {
                        return true;
                    }
                    chosen.pop();
                }
                any_unfulfiled = true;
                break;
            }
        }
        !any_unfulfiled
    }
}
