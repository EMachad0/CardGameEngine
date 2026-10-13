use enumset::EnumSet;

use crate::cards::{
    choice::ChoiceId,
    filter::{CharacterFilter, Side},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerSelectorFilter {
    pub side: EnumSet<Side>,
}

impl PlayerSelectorFilter {
    pub fn all() -> Self {
        Self {
            side: EnumSet::all(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerSelector {
    All(PlayerSelectorFilter),
    Owner,
    Random(PlayerSelectorFilter),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterSelector {
    All(CharacterFilter),
    Itself,
    OwnerHero,
    Random(CharacterFilter),
    Chosen(ChoiceId),
}
