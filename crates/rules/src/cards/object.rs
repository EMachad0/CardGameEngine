use std::collections::BTreeMap;

use crate::{
    cards::{
        definition::DefId,
        modifier::{Modifier, Modifiers},
    },
    ids::PlayerId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObjectId(u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Object {
    pub def_id: DefId,
    pub object_id: ObjectId,
    pub player_id: PlayerId,
    damage: u8,
    modifiers: Modifiers,
    friendly_aura: Modifiers,
}

impl Object {
    pub(crate) fn damage(&self) -> u8 {
        self.damage
    }

    /// Modifiers granted after the object was made, oldest first.
    pub(crate) fn modifiers(&self) -> &[Modifier] {
        self.modifiers.as_slice()
    }

    /// Modifiers granted to the object to give its friendly cards, oldest first.
    pub(crate) fn friendly_aura(&self) -> &[Modifier] {
        self.friendly_aura.as_slice()
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
        let object = Object {
            object_id,
            def_id,
            player_id,
            damage: 0,
            modifiers: Modifiers::default(),
            friendly_aura: Modifiers::default(),
        };
        self.objects.insert(object_id, object);
        object_id
    }

    pub(crate) fn get(&self, object_id: ObjectId) -> &Object {
        self.objects
            .get(&object_id)
            .expect("ObjectId is only created on object insert")
    }

    pub(crate) fn add_damage(&mut self, object_id: ObjectId, amount: u8) {
        self.get_mut(object_id).damage += amount;
    }

    pub(crate) fn grant(&mut self, object_id: ObjectId, modifier: Modifier) {
        self.get_mut(object_id).modifiers.add(modifier);
    }

    pub(crate) fn grant_friendly_aura(&mut self, object_id: ObjectId, modifier: Modifier) {
        self.get_mut(object_id).friendly_aura.add(modifier);
    }

    fn get_mut(&mut self, object_id: ObjectId) -> &mut Object {
        self.objects
            .get_mut(&object_id)
            .expect("ObjectId is only created on object insert")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::{assert_eq, assert_ne};

    use crate::cards::binder::sets::base::static_card_definition::{RECRUIT, SPARK};
    use crate::cards::modifier::ModifierEffect;

    fn buff(granted_by: ObjectId, amount: u8) -> Modifier {
        Modifier {
            granted_by,
            effect: ModifierEffect::BuffAtk {
                amount: amount.into(),
            },
        }
    }

    #[test]
    fn each_new_id_is_distinct() {
        let mut bag = ObjectBag::default();

        let first = bag.next_id();
        let second = bag.next_id();

        assert_ne!(first, second);
    }

    #[test]
    fn a_new_object_starts_with_no_damage_or_grants() {
        let mut bag = ObjectBag::default();
        let recruit = bag.insert(RECRUIT, PlayerId::new(0));

        let object = bag.get(recruit);

        assert_eq!(object.damage(), 0);
        assert_eq!(object.modifiers(), []);
        assert_eq!(object.friendly_aura(), []);
    }

    #[test]
    fn damage_adds_up() {
        let mut bag = ObjectBag::default();
        let recruit = bag.insert(RECRUIT, PlayerId::new(0));

        bag.add_damage(recruit, 2);
        bag.add_damage(recruit, 3);

        assert_eq!(bag.get(recruit).damage(), 5);
    }

    #[test]
    fn granted_modifiers_keep_their_order() {
        let mut bag = ObjectBag::default();
        let recruit = bag.insert(RECRUIT, PlayerId::new(0));
        let spark = bag.insert(SPARK, PlayerId::new(0));

        bag.grant(recruit, buff(spark, 1));
        bag.grant(recruit, buff(recruit, 2));

        assert_eq!(
            bag.get(recruit).modifiers(),
            [buff(spark, 1), buff(recruit, 2)]
        );
    }

    #[test]
    fn a_granted_friendly_aura_is_not_a_modifier_of_the_object() {
        let mut bag = ObjectBag::default();
        let recruit = bag.insert(RECRUIT, PlayerId::new(0));

        bag.grant_friendly_aura(recruit, buff(recruit, 1));

        assert_eq!(
            (
                bag.get(recruit).modifiers(),
                bag.get(recruit).friendly_aura()
            ),
            (&[][..], &[buff(recruit, 1)][..]),
            "a friendly aura goes to the object's other friendly cards, not to the object",
        );
    }
}
