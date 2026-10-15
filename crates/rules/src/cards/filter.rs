use enumset::EnumSet;

use crate::{
    cards::definition::{CardKind, CharacterKind},
    ids::PlayerId,
};

#[derive(Debug, enumset::EnumSetType)]
pub enum Side {
    Friendly,
    Enemy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardFilter {
    pub kind: EnumSet<CardKind>,
    pub side: EnumSet<Side>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharacterFilter {
    pub kind: EnumSet<CharacterKind>,
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

pub(crate) fn fulfill_side_filter(side: EnumSet<Side>, asking: PlayerId, owner: PlayerId) -> bool {
    (side.contains(Side::Enemy) && owner != asking)
        || (side.contains(Side::Friendly) && owner == asking)
}
