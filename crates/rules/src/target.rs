use std::fmt::Debug;

use crate::cards::definition::CardKind;
use crate::cards::filter::CharacterFilter;
use crate::cards::object::ObjectId;
use crate::choice::{AmountBound, Choice};
use crate::effect::EffectAmount;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetId(pub usize);

/// It has no values, so a `Targeted` selector that holds it can't be written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoTarget {}

/// The state an effect's targets are in.
pub trait TargetForm {
    type CharacterTarget: Debug + Clone + PartialEq + Eq;
    type CardTarget: Debug + Clone + PartialEq + Eq;
}

pub(crate) trait MapTargets<S: TargetForm, R: TargetForm> {
    fn map_character(&mut self, target: S::CharacterTarget) -> R::CharacterTarget;
    fn map_card(&mut self, target: S::CardTarget) -> R::CardTarget;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TargetChoiceRef<'t, S: TargetForm> {
    Character(&'t S::CharacterTarget),
    Card(&'t S::CardTarget),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardTargetOptions {
    pub zone: CardTargetZone,
    pub kind: enumset::EnumSet<CardKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardTargetZone {
    Hand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Declared {}

impl TargetForm for Declared {
    type CharacterTarget = Choice<CharacterFilter>;
    type CardTarget = Choice<CardTargetOptions>;
}

impl TargetChoiceRef<'_, Declared> {
    pub(crate) fn count(&self) -> EffectAmount {
        match self {
            TargetChoiceRef::Character(c) => c.count,
            TargetChoiceRef::Card(c) => c.count,
        }
    }

    pub(crate) fn bound(&self) -> AmountBound {
        match self {
            TargetChoiceRef::Character(c) => c.bound,
            TargetChoiceRef::Card(c) => c.bound,
        }
    }

    pub(crate) fn different(&self) -> bool {
        match self {
            TargetChoiceRef::Character(c) => c.different,
            TargetChoiceRef::Card(c) => c.different,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Triggered {}

impl TargetForm for Triggered {
    type CharacterTarget = NoTarget;
    type CardTarget = NoTarget;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BoundCards {
    pub zone: CardTargetZone,
    pub object_ids: Vec<ObjectId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Bound {}

impl TargetForm for Bound {
    type CharacterTarget = Vec<ObjectId>;
    type CardTarget = BoundCards;
}
