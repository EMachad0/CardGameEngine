//! What an effect amount comes to: its printed value, or a count of what history recorded.

use crate::cards::definition::{CardKind, CharacterKind};
use crate::effect::EffectAmount;
use crate::game::cards_ref::CardsRef;
use crate::history::{HistoryQuery, HistoryQueryKind, PlayerFilter, TurnFilter};
use crate::ids::PlayerId;

pub(super) fn effect_amount(cards: CardsRef, amount: &EffectAmount, player_id: PlayerId) -> u8 {
    match amount {
        EffectAmount::Static(value) => *value,
        EffectAmount::History(query) => history_query(cards, *query, player_id),
    }
}

fn history_query(cards: CardsRef, query: HistoryQuery, player_id: PlayerId) -> u8 {
    let scope = match query.scope {
        PlayerFilter::All => None,
        PlayerFilter::Owner => Some(player_id),
        PlayerFilter::Active => Some(cards.turn_order.get_active_player_id()),
    };
    let turn = match query.turn {
        TurnFilter::Current => Some(cards.turn_order.turn_count()),
        TurnFilter::All => None,
    };
    let count = match query.kind {
        HistoryQueryKind::SpellsPlayed => cards
            .history
            .played(scope, turn)
            .filter(|&object_id| cards.card_def(object_id).kind().card_kind() == CardKind::Spell)
            .count(),
        HistoryQueryKind::MinionsDied => cards
            .history
            .died(scope, turn)
            .filter(|&object_id| {
                cards.card_def(object_id).kind().character_kind() == Some(CharacterKind::Minion)
            })
            .count(),
    };
    u8::try_from(count).unwrap_or(u8::MAX)
}
