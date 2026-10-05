use enumset::EnumSet;

use crate::{
    DefId, Game, ObjectId, PlayerId,
    cards::definition::{CharacterKindFilter, CharacterSelectorFilter, CharacterSideFilter},
    choice::ChoiceId,
};

impl Game {
    pub(crate) fn can_fill_preconditions(&self, _object_id: ObjectId) -> bool {
        true
    }

    pub(crate) fn scan_for_valid_choices(&self, chooser_id: ObjectId, choice_id: ChoiceId) -> Vec<ObjectId> {
        let mut valid_choices = Vec::new();
        for object_id in self
            .players
            .iter()
            .flat_map(|p| p.zones.board.as_slice().iter().copied())
        {
            if self.is_valid_choice(chooser_id, choice_id, object_id) {
                valid_choices.push(object_id);
            }
        }
        for object_id in self.players.iter().map(|p| p.zones.hero) {
            if self.is_valid_choice(chooser_id, choice_id, object_id) {
                valid_choices.push(object_id);
            }
        }
        valid_choices
    }

    pub(crate) fn choice_fulfilled(&self, object_id: ObjectId, choice_id: ChoiceId) -> bool {
        let obj = self.objects.get(object_id);
        let choice = self.binder.choice(obj.def_id, choice_id).unwrap();
        let Some(targets) = obj
            .choice_targets
            .iter()
            .find(|c| c.choice_id == choice_id)
            .map(|c| &c.targets)
        else {
            return false;
        };

        let count = self.effect_amount(choice.count, obj.player_id);
        let chosen_count = targets.len() as u8;
        choice.bound.is_satisfied(chosen_count, count)
    }

    fn is_valid_choice(
        &self,
        chooser_id: ObjectId,
        choice_id: ChoiceId,
        object_id: ObjectId,
    ) -> bool {
        let obj = self.objects.get(chooser_id);
        let choice = self.binder.choice(obj.def_id, choice_id).unwrap();
        let targets = obj
            .choice_targets
            .iter()
            .find(|c| c.choice_id == choice_id)
            .map(|c| c.targets.clone())
            .unwrap_or_default();

        let count = self.effect_amount(choice.count, obj.player_id);
        let chosen_count = targets.len() as u8;
        let fulfill_count =
            (chosen_count + 1) <= choice.bound.upper_bound(count).unwrap_or(u8::MAX);
        let fulfill_unique = !targets.contains(&object_id);
        let fulfill_filter =
            self.fulfill_character_selection_filter(object_id, obj.player_id, choice.filter);

        fulfill_count && fulfill_unique && fulfill_filter
    }

    fn fulfill_character_selection_filter(
        &self,
        object_id: ObjectId,
        asking: PlayerId,
        CharacterSelectorFilter { kind, side }: CharacterSelectorFilter,
    ) -> bool {
        let obj = self.objects.get(object_id);
        self.fulfill_chacter_kind_filter(kind, obj.def_id)
            && fulfill_character_side_filter(side, obj.player_id, asking)
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

    pub(crate) fn fulfill_chacter_kind_filter(
        &self,
        kind: EnumSet<CharacterKindFilter>,
        def_id: DefId,
    ) -> bool {
        self.binder.is_hero(def_id) && kind.contains(CharacterKindFilter::Heroes)
            || self.binder.is_minion(def_id) && kind.contains(CharacterKindFilter::Minions)
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
