use crate::{PlayerId, cards::object::Object};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EventKind {
    CardPlayed { object: Object },
    MonsterDied { object: Object },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EventLog {
    player_id: PlayerId,
    event_kind: EventKind,
    turn: u32,
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

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct History {
    pub logs: Vec<EventLog>,
}
