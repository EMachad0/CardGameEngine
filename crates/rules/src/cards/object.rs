use std::collections::BTreeMap;

use crate::{DefId, PlayerId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObjectId(u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Object {
    pub def_id: DefId,
    pub object_id: ObjectId,
    pub player_id: PlayerId,
    pub modifiers: Modifiers,
    pub damage: u8,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Modifiers(Vec<Modifier>);

impl Modifiers {
    pub(crate) fn add(&mut self, modifier: Modifier) {
        self.0.push(modifier);
    }

    pub(crate) fn remove(&mut self, modifier: Modifier) {
        if let Some(idx) = self.0.iter().position(|m| *m == modifier) {
            let _ = self.0.remove(idx);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Modifier {
    pub source: ObjectId,
    pub effect: ModifierEffect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ModifierEffect {
    BuffAtk { amount: i32 },
    BuffHealth { amount: i32 },
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct ObjectBag {
    next_id: u64,
    objects: BTreeMap<ObjectId, Object>,
}

impl ObjectBag {
    pub(crate) fn next_id(&mut self) -> ObjectId {
        let id = self.next_id;
        self.next_id += 1;
        ObjectId(id)
    }

    pub(crate) fn insert(&mut self, object: Object) -> ObjectId {
        let object_id = self.next_id();
        self.objects.insert(object_id, object);
        object_id
    }

    pub(crate) fn insert_all<I: IntoIterator<Item = Object>>(
        &mut self,
        objects: I,
    ) -> Vec<ObjectId> {
        objects
            .into_iter()
            .map(|object| self.insert(object))
            .collect()
    }

    pub(crate) fn get(&self, object_id: ObjectId) -> Option<&Object> {
        self.objects.get(&object_id)
    }

    pub(crate) fn get_mut(&mut self, object_id: ObjectId) -> Option<&mut Object> {
        self.objects.get_mut(&object_id)
    }
}
