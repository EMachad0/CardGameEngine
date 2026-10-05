use enumset::EnumSet;

use crate::{
    Game, ObjectId, PlayerId,
    cards::definition::{CharacterKindFilter, CharacterSelectorFilter, CharacterSideFilter},
    choice::{CharacterChoice, ChoiceId},
};

impl Game {
    pub(crate) fn scan_for_valid_choices(
        &self,
        chooser_id: ObjectId,
        choice: &CharacterChoice,
        chosen: &[ObjectId],
    ) -> Vec<ObjectId> {
        if self.check_choice_count_fulfilled(chooser_id, choice.id, chosen.len() as u8) {
            return Vec::new();
        }

        let obj = self.objects.get(chooser_id);
        self.scan_characters_with_filter(obj.player_id, choice.filter)
            .into_iter()
            .filter(|o| !chosen.contains(o))
            .collect()
    }

    pub(crate) fn check_choice_count_fulfilled(
        &self,
        object_id: ObjectId,
        choice_id: ChoiceId,
        chosen_count: u8,
    ) -> bool {
        let obj = self.objects.get(object_id);
        let choice = self.binder.choice(obj.def_id, choice_id).unwrap();
        let count = self.effect_amount(choice.count, obj.player_id);
        choice.bound.is_satisfied(chosen_count, count)
    }

    pub(crate) fn scan_characters_with_filter(
        &self,
        asking: PlayerId,
        CharacterSelectorFilter { kind, side }: CharacterSelectorFilter,
    ) -> Vec<ObjectId> {
        let mut characters = Vec::new();
        if kind.contains(CharacterKindFilter::Heroes) {
            characters.extend(
                self.players
                    .iter()
                    .filter(|player| fulfill_character_side_filter(side, asking, player.id))
                    .map(|player| player.zones.hero),
            );
        }
        if kind.contains(CharacterKindFilter::Minions) {
            characters.extend(
                self.players
                    .iter()
                    .filter(|player| fulfill_character_side_filter(side, asking, player.id))
                    .flat_map(|player| player.zones.board.as_slice()),
            );
        }
        characters
    }

    pub(super) fn can_finish_choices(
        &self,
        object_id: ObjectId,
        chosen: &mut Vec<(ObjectId, ChoiceId)>,
    ) -> bool {
        let mut any_unfulfiled = false;
        for choice in self.binder.choices(self.def_id(object_id)) {
            let chosen_for_choice = chosen
                .iter()
                .filter(|(_, choice_id)| choice.id == *choice_id)
                .map(|(o, _)| *o)
                .collect::<Vec<_>>();
            if !self.check_choice_count_fulfilled(
                object_id,
                choice.id,
                chosen_for_choice.len() as u8,
            ) {
                let candidates = self.scan_for_valid_choices(object_id, choice, &chosen_for_choice);
                for candidate in candidates.into_iter() {
                    chosen.push((candidate, choice.id));
                    let can_finish = self.can_finish_choices(object_id, chosen);
                    if can_finish {
                        return true;
                    }
                    chosen.pop();
                }
                any_unfulfiled = true;
            }
        }
        !any_unfulfiled
    }
}

pub(crate) fn fulfill_character_side_filter(
    side: EnumSet<CharacterSideFilter>,
    asking: PlayerId,
    player_id: PlayerId,
) -> bool {
    (side.contains(CharacterSideFilter::Enemy) && player_id != asking)
        || (side.contains(CharacterSideFilter::Friendly) && player_id == asking)
}
