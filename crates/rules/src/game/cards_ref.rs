//! The cards of one game, borrowed together for the questions asked about them: what each object
//! is, and which characters pass a filter.

use crate::cards::binder::def_id::DefId;
use crate::cards::binder::store::Binder;
use crate::cards::definition::CardDef;
use crate::cards::filter::CharacterFilter;
use crate::cards::object::{ObjectBag, ObjectId};
use crate::history::History;
use crate::ids::PlayerId;
use crate::turn::TurnOrder;
use crate::zones::Zones;

#[derive(Clone, Copy)]
pub(super) struct CardsRef<'g> {
    binder: &'g Binder,
    pub(super) objects: &'g ObjectBag,
    pub(super) zones: &'g Zones,
    pub(super) history: &'g History,
    pub(super) turn_order: &'g TurnOrder,
}

impl<'g> CardsRef<'g> {
    pub(super) fn new(
        binder: &'g Binder,
        objects: &'g ObjectBag,
        zones: &'g Zones,
        history: &'g History,
        turn_order: &'g TurnOrder,
    ) -> Self {
        Self {
            binder,
            objects,
            zones,
            history,
            turn_order,
        }
    }

    pub(super) fn def_id(&self, object_id: ObjectId) -> DefId {
        self.objects.get(object_id).def_id
    }

    pub(super) fn card_def(&self, object_id: ObjectId) -> &'g CardDef {
        self.binder.get(self.def_id(object_id))
    }

    pub(super) fn scan_characters_with_filter(
        &self,
        asking: PlayerId,
        filter: &CharacterFilter,
    ) -> Vec<ObjectId> {
        self.zones
            .characters_in_play()
            .filter(|&object_id| self.object_passes(asking, filter, object_id))
            .collect()
    }

    pub(super) fn object_passes(
        &self,
        asking: PlayerId,
        filter: &CharacterFilter,
        object_id: ObjectId,
    ) -> bool {
        let owner = self.objects.get(object_id).player_id;
        filter.passes(asking, owner, self.card_def(object_id).character_kind())
    }
}
