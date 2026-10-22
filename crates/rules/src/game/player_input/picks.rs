//! Where revealed cards wait. A revealed card sits in a picking's options until picked, then in
//! its picks.

use crate::cards::object::ObjectId;
use crate::effect::PickId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Picking {
    source: ObjectId,
    pick_id: PickId,
    options: Vec<ObjectId>,
}

impl Picking {
    pub(super) fn new(source: ObjectId, pick_id: PickId, options: Vec<ObjectId>) -> Self {
        Self {
            source,
            pick_id,
            options,
        }
    }

    /// In reveal order.
    pub(super) fn options(&self) -> &[ObjectId] {
        &self.options
    }

    /// Closes the picking with the option at `option_idx` as picked and the other options as the
    /// others. Panics unless `option_idx` names an option.
    pub(super) fn pick(mut self, option_idx: usize) -> Picks {
        let picked = self.options.remove(option_idx);
        Picks {
            source: self.source,
            pick_id: self.pick_id,
            picked: vec![picked],
            others: self.options,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::game) struct Picks {
    source: ObjectId,
    pick_id: PickId,
    picked: Vec<ObjectId>,
    others: Vec<ObjectId>,
}

impl Picks {
    pub(in crate::game) fn picked(&self) -> &[ObjectId] {
        &self.picked
    }

    /// The options not picked, in reveal order.
    pub(in crate::game) fn others(&self) -> &[ObjectId] {
        &self.others
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) struct StoredPicks {
    /// Sorted by source, then in the order picked.
    picks: Vec<Picks>,
}

impl StoredPicks {
    pub(super) fn store(&mut self, picks: Picks) {
        let idx = self
            .picks
            .partition_point(|entry| entry.source <= picks.source);
        self.picks.insert(idx, picks);
    }

    /// The first picks `source` stored for `pick_id`.
    pub(super) fn find(&self, source: ObjectId, pick_id: PickId) -> Option<&Picks> {
        self.picks
            .iter()
            .find(|entry| entry.source == source && entry.pick_id == pick_id)
    }
}
