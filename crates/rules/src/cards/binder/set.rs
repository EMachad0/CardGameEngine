use crate::cards::definition::CardDef;

/// Cards released together under one prefix, which the binder puts first in each card's code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Set {
    pub prefix: &'static str,
    pub cards: Vec<SetCard>,
}

/// A card's code within its set, and its definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetCard {
    pub name: &'static str,
    pub version: u16,
    pub def: CardDef,
}
