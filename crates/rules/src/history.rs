use crate::{PlayerId, cards::object::Object};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HistoryKind {
    CardPlayed { object: Object },
    MinionDied { object: Object },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HistoryEntry {
    pub player_id: PlayerId,
    pub kind: HistoryKind,
    pub turn: u32,
}

impl HistoryEntry {
    pub(crate) fn new(player_id: PlayerId, kind: HistoryKind, turn: u32) -> Self {
        Self {
            player_id,
            kind,
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
    MinionDied,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct History {
    pub entries: Vec<HistoryEntry>,
}
