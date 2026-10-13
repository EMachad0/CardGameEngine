use enumset::EnumSet;

use crate::{
    cards::filter::{CardFilter, CardKindFilter, CharacterFilter, CharacterKindFilter, Side},
    game::Game,
    ids::PlayerId,
    object::ObjectId,
};

impl Game {
    pub(super) fn scan_characters_with_filter(
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

    pub(super) fn fulfill_character_filter(
        &self,
        asking: PlayerId,
        owner: PlayerId,
        object_id: ObjectId,
        &CharacterFilter { kind, side }: &CharacterFilter,
    ) -> bool {
        fulfill_side_filter(side, asking, owner)
            && self.fulfill_character_kind_filter(object_id, kind)
    }

    pub(super) fn fulfill_card_filter(
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
}

pub(super) fn fulfill_side_filter(side: EnumSet<Side>, asking: PlayerId, owner: PlayerId) -> bool {
    (side.contains(Side::Enemy) && owner != asking)
        || (side.contains(Side::Friendly) && owner == asking)
}
