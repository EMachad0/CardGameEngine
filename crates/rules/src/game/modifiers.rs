//! The modifiers that reach an object: its printed ones, those granted it, and the auras of the
//! other cards on the boards.

use crate::{
    cards::{modifier::SortedModifiers, object::ObjectId},
    game::Game,
};

impl Game {
    pub(super) fn modifiers(&self, object_id: ObjectId) -> SortedModifiers<'_> {
        let object = self.objects.get(object_id);
        let card_def = self.card_def(object_id);
        let mut modifiers = SortedModifiers::default();
        for effect in &card_def.modifier_effects {
            modifiers.add(object_id, effect);
        }
        for modifier in object.modifiers() {
            modifiers.add(modifier.granted_by, &modifier.effect);
        }

        if card_def.receives_auras() {
            for other_player_id in self.players().into_iter() {
                for other_object_id in self.player_board(other_player_id).iter().copied() {
                    if object_id == other_object_id {
                        continue;
                    }

                    if object.player_id == other_player_id {
                        self.add_friendly_aura(other_object_id, &mut modifiers);
                    } else {
                        self.add_hostile_aura(other_object_id, &mut modifiers);
                    }
                }
            }
        }

        modifiers
    }

    fn add_friendly_aura<'a>(&'a self, object_id: ObjectId, modifiers: &mut SortedModifiers<'a>) {
        for effect in self.card_def(object_id).friendly_aura_effects() {
            modifiers.add(object_id, effect);
        }
        for modifier in self.objects.get(object_id).friendly_aura() {
            modifiers.add(modifier.granted_by, &modifier.effect);
        }
    }

    fn add_hostile_aura<'a>(&'a self, object_id: ObjectId, modifiers: &mut SortedModifiers<'a>) {
        for effect in self.card_def(object_id).hostile_aura_effects() {
            modifiers.add(object_id, effect);
        }
    }
}
