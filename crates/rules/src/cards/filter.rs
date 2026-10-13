use enumset::EnumSet;

#[derive(Debug, enumset::EnumSetType)]
pub enum CharacterKindFilter {
    Heroes,
    Minions,
}

#[derive(Debug, enumset::EnumSetType)]
pub enum Side {
    Friendly,
    Enemy,
}

#[derive(Debug, enumset::EnumSetType)]
pub enum CardKindFilter {
    Spells,
    Minions,
    Heroes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardFilter {
    pub kind: EnumSet<CardKindFilter>,
    pub side: EnumSet<Side>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharacterFilter {
    pub kind: EnumSet<CharacterKindFilter>,
    pub side: EnumSet<Side>,
}

impl CharacterFilter {
    pub fn all() -> Self {
        Self {
            kind: EnumSet::all(),
            side: EnumSet::all(),
        }
    }
}
