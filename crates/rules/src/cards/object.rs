use std::collections::BTreeMap;

use crate::{DefId, PlayerId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObjectId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Object {
    pub def_id: DefId,
    pub object_id: ObjectId,
    pub player_id: PlayerId,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ObjectBag {
    next_id: u64,
    objects: BTreeMap<ObjectId, Object>,
}

impl ObjectBag {
    pub fn next_id(&mut self) -> ObjectId {
        let id = self.next_id;
        self.next_id += 1;
        ObjectId(id)
    }

    pub fn insert(&mut self, object: Object) -> ObjectId {
        let object_id = self.next_id();
        self.objects.insert(object_id, object);
        object_id
    }

    pub fn insert_all<I: IntoIterator<Item = Object>>(&mut self, objects: I) -> Vec<ObjectId> {
        objects
            .into_iter()
            .map(|object| self.insert(object))
            .collect()
    }

    pub fn get(&self, object_id: ObjectId) -> Option<&Object> {
        self.objects.get(&object_id)
    }
}
