//! An object's definition, a player's hero, and amounts computed from history.

use crate::cards::definition::{CardDef, CardKind, CharacterKind, DefId};
use crate::cards::object::{Object, ObjectId};
use crate::effect::{Effect, EffectAmount};
use crate::game::Game;
use crate::history::{HistoryQuery, HistoryQueryKind, PlayerFilter, TurnFilter};
use crate::ids::PlayerId;

impl Game {
    fn object(&self, object_id: ObjectId) -> &Object {
        self.objects.get(object_id)
    }

    pub fn def_id(&self, object_id: ObjectId) -> DefId {
        self.object(object_id).def_id
    }

    pub(super) fn card_def(&self, object_id: ObjectId) -> &CardDef {
        self.binder.get(self.def_id(object_id))
    }

    pub fn hero_id(&self, player_id: PlayerId) -> ObjectId {
        self.zones.hero(player_id)
    }

    pub(crate) fn play_effects(&self, object_id: ObjectId) -> &[Effect] {
        self.card_def(object_id).play_effects.as_slice()
    }

    pub(super) fn effect_amount(&self, amount: &EffectAmount, player_id: PlayerId) -> u8 {
        match amount {
            EffectAmount::Static(value) => *value,
            EffectAmount::History(query) => self.history_query(*query, player_id),
        }
    }

    fn history_query(&self, query: HistoryQuery, player_id: PlayerId) -> u8 {
        let scope = match query.scope {
            PlayerFilter::All => None,
            PlayerFilter::Owner => Some(player_id),
            PlayerFilter::Active => Some(self.turn_order.get_active_player_id()),
        };
        let turn = match query.turn {
            TurnFilter::Current => Some(self.turn_order.turn_count()),
            TurnFilter::All => None,
        };
        let count = match query.kind {
            HistoryQueryKind::SpellsPlayed => self
                .history
                .played(scope, turn)
                .filter(|&object_id| self.card_def(object_id).card_kind() == CardKind::Spell)
                .count(),
            HistoryQueryKind::MinionsDied => self
                .history
                .died(scope, turn)
                .filter(|&object_id| {
                    self.card_def(object_id).character_kind() == Some(CharacterKind::Minion)
                })
                .count(),
        };
        u8::try_from(count).unwrap_or(u8::MAX)
    }
}
