use enumset::EnumSet;

use crate::cards::definition::CardKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardPickOptions {
    pub zone: CardPickZone,
    pub kind: EnumSet<CardKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardPickZone {
    YourDeck(DeckWindow),
}

/// The cards of a deck a pick looks at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeckWindow {
    /// Up to this many from the top, top first.
    Top(u8),
}
