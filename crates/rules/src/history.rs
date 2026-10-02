use crate::{PlayerId, cards::object::Object};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EventKind {
    CardPlayed { object: Object },
    MonsterDied { object: Object },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EventLog {
    pub player_id: PlayerId,
    pub event_kind: EventKind,
    pub turn: u32,
}

impl EventLog {
    pub(crate) fn new(player_id: PlayerId, event_kind: EventKind, turn: u32) -> Self {
        Self {
            player_id,
            event_kind,
            turn,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerFilter {
    Current,
    All,
    Owner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnFilter {
    Current,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HistoryQuery {
    pub kind: HistoryQueryKind,
    pub scope: PlayerFilter,
    pub turn: TurnFilter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryQueryKind {
    SpellsPlayed,
    MonsterDied,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct History {
    pub logs: Vec<EventLog>,
}
