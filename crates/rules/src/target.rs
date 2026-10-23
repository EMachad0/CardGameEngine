use std::fmt::Debug;

use crate::cards::filter::CharacterFilter;
use crate::cards::object::ObjectId;
use crate::choice::Choice;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetId(pub usize);

/// It has no values, so a `Targeted` selector that holds it can't be written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoTarget {}

/// A form of effect text.
pub trait TargetForm {
    type CharacterTarget: Debug + Clone + PartialEq + Eq;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Declared {}

impl TargetForm for Declared {
    type CharacterTarget = Choice<CharacterFilter>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Triggered {}

impl TargetForm for Triggered {
    type CharacterTarget = NoTarget;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Bound {}

impl TargetForm for Bound {
    type CharacterTarget = Vec<ObjectId>;
}
