#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectAmount {
    Static(u8),
    History(HistoryQuery),
}

impl From<u8> for EffectAmount {
    fn from(value: u8) -> Self {
        Self::Static(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerFilter {
    Active,
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
    MinionsDied,
}
