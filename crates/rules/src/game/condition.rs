use enumset::EnumSet;

use crate::cards::definition::{CardKind, CharacterKind};
use crate::cards::filter::{CardFilter, CharacterFilter, fulfill_side_filter};
use crate::cards::object::ObjectId;
use crate::game::Game;
use crate::ids::PlayerId;

impl Game {
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
}
