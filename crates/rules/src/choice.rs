use crate::{
    cards::choice::{ChoiceId, PickId},
    object::ObjectId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChoiceTarget {
    pub choice_id: ChoiceId,
    pub targets: Vec<ObjectId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PickTarget {
    pub pick_id: PickId,
    pub picked: Vec<ObjectId>,
    pub others: Vec<ObjectId>,
}
