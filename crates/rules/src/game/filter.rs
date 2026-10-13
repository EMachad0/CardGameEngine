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
        let card_def = self.card_def(object_id);
        kind.contains(CharacterKindFilter::Heroes) && card_def.is_hero()
            || kind.contains(CharacterKindFilter::Minions) && card_def.is_minion()
    }

    fn fulfill_card_kind_filter(&self, object_id: ObjectId, kind: EnumSet<CardKindFilter>) -> bool {
        let card_def = self.card_def(object_id);
        kind.contains(CardKindFilter::Heroes) && card_def.is_hero()
            || kind.contains(CardKindFilter::Minions) && card_def.is_minion()
            || kind.contains(CardKindFilter::Spells) && card_def.is_spell()
    }
}

pub(super) fn fulfill_side_filter(side: EnumSet<Side>, asking: PlayerId, owner: PlayerId) -> bool {
    (side.contains(Side::Enemy) && owner != asking)
        || (side.contains(Side::Friendly) && owner == asking)
}
