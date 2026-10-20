//! The cards of one game, borrowed together for the questions card text asks: what each object
//! is, and which characters in play a filter matches.

use crate::cards::binder::store::Binder;
use crate::cards::definition::{CardDef, DefId};
use crate::cards::filter::CharacterFilter;
use crate::cards::object::{ObjectBag, ObjectId};
use crate::effect::Effect;
use crate::game::state::Game;
use crate::history::History;
use crate::ids::PlayerId;
use crate::turn::TurnOrder;
use crate::zones::Zones;

#[derive(Clone, Copy)]
pub(super) struct Cards<'g> {
    binder: &'g Binder,
    pub(super) objects: &'g ObjectBag,
    pub(super) zones: &'g Zones,
    pub(super) history: &'g History,
    pub(super) turn_order: &'g TurnOrder,
}

impl Game {
    pub(super) fn cards(&self) -> Cards<'_> {
        Cards {
            binder: &self.binder,
            objects: &self.objects,
            zones: &self.zones,
            history: &self.history,
            turn_order: &self.turn_order,
        }
    }

    pub fn def_id(&self, object_id: ObjectId) -> DefId {
        self.cards().def_id(object_id)
    }
}

impl<'g> Cards<'g> {
    fn def_id(&self, object_id: ObjectId) -> DefId {
        self.objects.get(object_id).def_id
    }

    pub(super) fn card_def(&self, object_id: ObjectId) -> &'g CardDef {
        self.binder.get(self.def_id(object_id))
    }

    pub(super) fn play_effects(&self, object_id: ObjectId) -> &'g [Effect] {
        self.card_def(object_id).play_effects.as_slice()
    }

    pub(super) fn scan_characters_with_filter(
        &self,
        asking: PlayerId,
        filter: &CharacterFilter,
    ) -> Vec<ObjectId> {
        self.zones
            .characters_in_play()
            .filter(|&object_id| {
                let owner = self.objects.get(object_id).player_id;
                filter.matches(asking, owner, self.card_def(object_id).character_kind())
            })
            .collect()
    }
}
