//! The cards of one game, borrowed together for the questions asked about them.

use enumset::EnumSet;

use crate::cards::binder::def_id::DefId;
use crate::cards::binder::store::Binder;
use crate::cards::definition::{CardDef, CardKind};
use crate::cards::filter::CharacterFilter;
use crate::cards::object::{ObjectBag, ObjectId};
use crate::game::picking::PickWindow;
use crate::game::zone_move::CardOriginZone;
use crate::history::History;
use crate::ids::PlayerId;
use crate::pick::{CardPickOptions, CardPickZone, DeckWindow};
use crate::target::{CardTargetOptions, CardTargetZone};
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

    /// The player who answers the choices in `source`'s text.
    pub(super) fn chooser(&self, source: ObjectId) -> PlayerId {
        self.objects.get(source).player_id
    }

    pub(super) fn scan_characters_with_filter(
        &self,
        asking: PlayerId,
        filter: &CharacterFilter,
    ) -> Vec<ObjectId> {
        self.zones
            .characters_in_play()
            .filter(|&object_id| self.object_passes_character_filter(asking, filter, object_id))
            .collect()
    }

    pub(super) fn scan_cards_with_options(
        &self,
        asking: PlayerId,
        filter: &CardTargetOptions,
    ) -> Vec<ObjectId> {
        let objects = match filter.zone {
            CardTargetZone::Hand => self.zones.hand(asking),
        };
        objects
            .iter()
            .copied()
            .filter(|&object_id| self.card_is_kind(filter.kind, object_id))
            .collect()
    }

    pub(super) fn scan_pick_window(
        &self,
        asking: PlayerId,
        options: &CardPickOptions,
    ) -> PickWindow {
        let (holder, from, covered) = match options.zone {
            CardPickZone::YourDeck(DeckWindow::Top(count)) => {
                let mut deck = self.zones.deck(asking);
                deck.truncate(usize::from(count));
                (asking, CardOriginZone::Deck, deck)
            }
        };
        PickWindow {
            holder,
            from,
            cards: covered
                .into_iter()
                .map(|object_id| (object_id, self.card_is_kind(options.kind, object_id)))
                .collect(),
        }
    }

    fn card_is_kind(&self, kind: EnumSet<CardKind>, object_id: ObjectId) -> bool {
        kind.contains(self.card_def(object_id).card_kind())
    }

    pub(super) fn object_passes_character_filter(
        &self,
        asking: PlayerId,
        filter: &CharacterFilter,
        object_id: ObjectId,
    ) -> bool {
        let owner = self.objects.get(object_id).player_id;
        filter.passes(asking, owner, self.card_def(object_id).character_kind())
    }
}
