use crate::{
    cards::amount::{HistoryQuery, HistoryQueryKind, PlayerFilter, TurnFilter},
    event::Event,
    game::Game,
    ids::PlayerId,
    object::Object,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum HistoryKind {
    CardPlayed { object: Object },
    CharacterDied { object: Object },
    CharacterEntered { object: Object },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct HistoryEntry {
    pub player_id: PlayerId,
    pub kind: HistoryKind,
    pub turn: u32,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) struct History {
    pub entries: Vec<HistoryEntry>,
}

impl Game {
    pub(super) fn history_from_event(&mut self, event: &Event) {
        let turn = self.turn_order.turn_count();
        if let Some(entry) = match event {
            Event::Died { object_id } => Some(HistoryEntry {
                player_id: self.objects.get(*object_id).player_id,
                kind: HistoryKind::CharacterDied {
                    object: self.objects.get(*object_id).clone(),
                },
                turn,
            }),
            Event::Played {
                player_id,
                object_id,
            } => Some(HistoryEntry {
                player_id: *player_id,
                kind: HistoryKind::CardPlayed {
                    object: self.objects.get(*object_id).clone(),
                },
                turn,
            }),
            Event::BoardEntered {
                player_id,
                object_id,
            } => Some(HistoryEntry {
                player_id: *player_id,
                kind: HistoryKind::CharacterEntered {
                    object: self.objects.get(*object_id).clone(),
                },
                turn,
            }),
            _ => None,
        } {
            self.history.entries.push(entry);
        }
    }

    pub(super) fn history_query(&self, query: HistoryQuery, player_id: PlayerId) -> u8 {
        let entries = self.history.entries.iter();
        let targets = match query.scope {
            PlayerFilter::All => self.players.iter().map(|p| p.id).collect(),
            PlayerFilter::Owner => vec![player_id],
            PlayerFilter::Active => vec![self.turn_order.get_active_player_id()],
        };
        let count = match query.kind {
            HistoryQueryKind::SpellsPlayed => entries
                .filter(|entry| targets.contains(&entry.player_id))
                .filter(|entry| match query.turn {
                    TurnFilter::Current => {
                        entry.turn == self.turn_order.turn_count()
                    }
                    TurnFilter::All => true,
                })
                .filter(|entry| {
                    matches!(
                        &entry.kind,
                        HistoryKind::CardPlayed { object } if self.binder.is_spell(object.def_id)
                    )
                })
                .count(),
            HistoryQueryKind::MinionsDied => entries
                .filter(|entry| targets.contains(&entry.player_id))
                .filter(|entry| match query.turn {
                    TurnFilter::Current => {
                        entry.turn == self.turn_order.turn_count()
                    }
                    TurnFilter::All => true,
                })
                .filter(|entry| matches!(&entry.kind, HistoryKind::CharacterDied { object } if self.binder.is_minion(object.def_id)))
                .count(),
        };
        u8::try_from(count).unwrap_or(u8::MAX)
    }
}
