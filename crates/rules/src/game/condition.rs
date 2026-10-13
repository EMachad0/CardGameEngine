use enumset::EnumSet;

use crate::{
    cards::{
        choice::{CharacterChoice, ChoiceId},
        filter::{CardFilter, CardKindFilter, CharacterFilter, CharacterKindFilter, Side},
    },
    game::Game,
    ids::PlayerId,
    object::ObjectId,
};

impl Game {
    pub(crate) fn scan_for_valid_choices(
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

    pub(crate) fn check_choice_count_fulfilled(
        &self,
        object_id: ObjectId,
        choice_id: ChoiceId,
        chosen_count: u8,
    ) -> bool {
        let obj = self.objects.get(object_id);
        let choice = self.binder.choice(obj.def_id, choice_id).unwrap();
        let count = self.effect_amount(&choice.count, obj.player_id);
        choice.bound.is_satisfied(chosen_count, count)
    }

    pub(crate) fn scan_characters_with_filter(
        &self,
        asking: PlayerId,
        filter: &CharacterFilter,
    ) -> Vec<ObjectId> {
        let mut characters = Vec::new();
        characters.extend(
            self.players
                .iter()
                .filter(|player| {
                    self.fulfill_character_filter(asking, player.id, player.zones.hero, filter)
                })
                .map(|player| player.zones.hero),
        );
        characters.extend(self.players.iter().flat_map(|player| {
            player.zones.board.as_slice().iter().filter(|object_id| {
                self.fulfill_character_filter(asking, player.id, **object_id, filter)
            })
        }));
        characters
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
        kind: EnumSet<CharacterKindFilter>,
    ) -> bool {
        let def_id = self.objects.get(object_id).def_id;
        kind.contains(CharacterKindFilter::Heroes) && self.binder.is_hero(def_id)
            || kind.contains(CharacterKindFilter::Minions) && self.binder.is_minion(def_id)
    }

    fn fulfill_card_kind_filter(&self, object_id: ObjectId, kind: EnumSet<CardKindFilter>) -> bool {
        let def_id = self.objects.get(object_id).def_id;
        kind.contains(CardKindFilter::Heroes) && self.binder.is_hero(def_id)
            || kind.contains(CardKindFilter::Minions) && self.binder.is_minion(def_id)
            || kind.contains(CardKindFilter::Spells) && self.binder.is_spell(def_id)
    }

    pub(super) fn can_finish_choices(
        &self,
        object_id: ObjectId,
        chosen: &mut Vec<(ObjectId, ChoiceId)>,
    ) -> bool {
        let mut any_unfulfiled = false;
        for choice in self.binder.choices(self.def_id(object_id)) {
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

pub(super) fn fulfill_side_filter(side: EnumSet<Side>, asking: PlayerId, owner: PlayerId) -> bool {
    (side.contains(Side::Enemy) && owner != asking)
        || (side.contains(Side::Friendly) && owner == asking)
}
