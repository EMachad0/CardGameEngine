//! What an effect amount comes to: its printed value, or a count of what history recorded.

use crate::cards::definition::{CardKind, CharacterKind};
use crate::effect::EffectAmount;
use crate::game::cards::Cards;
use crate::history::{HistoryQuery, HistoryQueryKind, PlayerFilter, TurnFilter};
use crate::ids::PlayerId;

impl Cards<'_> {
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
