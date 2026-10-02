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
    pub(crate) fn new(object_id: ObjectId, def_id: DefId, player_id: PlayerId) -> Self {
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
    pub(crate) fn next_id(&mut self) -> ObjectId {
        let id = self.next_id;
        self.next_id += 1;
        ObjectId(id)
    }

    pub(crate) fn insert(&mut self, object: Object) -> ObjectId {
        let object_id = object.object_id;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::SPARK;

    #[test]
    fn an_inserted_object_is_stored_under_its_own_id() {
        let mut bag = ObjectBag::default();
        let id = bag.next_id();

        let key = bag.insert(Object::new(id, SPARK, PlayerId::new(0)));

        assert_eq!(key, id, "insert must not allocate a second id");
        assert_eq!(bag.get(id).map(|object| object.object_id), Some(id));
    }

    #[test]
    fn each_new_id_is_distinct() {
        let mut bag = ObjectBag::default();

        let first = bag.next_id();
        let second = bag.next_id();

        assert_ne!(first, second);
    }
}
