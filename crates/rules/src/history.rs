use crate::{ids::PlayerId, object::Object};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HistoryKind {
    CardPlayed { object: Object },
    CharacterDied { object: Object },
    CharacterEntered { object: Object },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HistoryEntry {
    pub player_id: PlayerId,
    pub kind: HistoryKind,
    pub turn: u32,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct History {
    pub entries: Vec<HistoryEntry>,
}
