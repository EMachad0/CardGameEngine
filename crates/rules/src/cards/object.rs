use std::collections::BTreeMap;

use crate::{DefId, PlayerId, cards::modifier::Modifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObjectId(u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Object {
    pub def_id: DefId,
    pub object_id: ObjectId,
    pub player_id: PlayerId,
    pub damage: u8,
    pub modifiers: Modifiers,
    pub friendly_aura: Modifiers,
}

impl Object {
    fn new(object_id: ObjectId, def_id: DefId, player_id: PlayerId) -> Self {
        Self {
            object_id,
            def_id,
            player_id,
            damage: 0,
            modifiers: Modifiers::default(),
            friendly_aura: Modifiers::default(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct ObjectBag {
    next_id: u64,
    objects: BTreeMap<ObjectId, Object>,
}

impl ObjectBag {
    fn next_id(&mut self) -> ObjectId {
        let id = self.next_id;
        self.next_id += 1;
        ObjectId(id)
    }

    pub(crate) fn insert(&mut self, def_id: DefId, player_id: PlayerId) -> ObjectId {
        let object_id = self.next_id();
        let object = Object::new(object_id, def_id, player_id);
        self.objects.insert(object_id, object);
        object_id
    }

    pub(crate) fn get(&self, object_id: ObjectId) -> &Object {
        self.objects
            .get(&object_id)
            .expect("ObjectId is only created on object insert")
    }

    pub(crate) fn get_mut(&mut self, object_id: ObjectId) -> &mut Object {
        self.objects
            .get_mut(&object_id)
            .expect("ObjectId is only created on object insert")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_new_id_is_distinct() {
        let mut bag = ObjectBag::default();

        let first = bag.next_id();
        let second = bag.next_id();

        assert_ne!(first, second);
    }
}
