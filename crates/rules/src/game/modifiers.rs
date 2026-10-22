//! The modifiers that reach an object: its printed ones, those granted it, and the auras of the
//! other cards on the boards.

use crate::cards::modifier::SortedModifiers;
use crate::cards::object::ObjectId;
use crate::game::cards_ref::CardsRef;

pub(super) fn modifiers(cards: CardsRef<'_>, object_id: ObjectId) -> SortedModifiers<'_> {
    let object = cards.objects.get(object_id);
    let card_def = cards.card_def(object_id);
    let mut modifiers = SortedModifiers::default();
    for effect in card_def.modifier_effects() {
        modifiers.add(object_id, effect);
    }
    for modifier in object.modifiers() {
        modifiers.add(modifier.granted_by, &modifier.effect);
    }

    if card_def.kind().receives_auras() {
        for (other_player_id, board) in cards.zones.boards() {
            for other_object_id in board.iter().copied() {
                if object_id == other_object_id {
                    continue;
                }

                if object.player_id == other_player_id {
                    add_friendly_aura(cards, other_object_id, &mut modifiers);
                } else {
                    add_hostile_aura(cards, other_object_id, &mut modifiers);
                }
            }
        }
    }

    modifiers
}

fn add_friendly_aura<'g>(
    cards: CardsRef<'g>,
    object_id: ObjectId,
    modifiers: &mut SortedModifiers<'g>,
) {
    for effect in cards.card_def(object_id).kind().friendly_aura_effects() {
        modifiers.add(object_id, effect);
    }
    for modifier in cards.objects.get(object_id).friendly_aura() {
        modifiers.add(modifier.granted_by, &modifier.effect);
    }
}

fn add_hostile_aura<'g>(
    cards: CardsRef<'g>,
    object_id: ObjectId,
    modifiers: &mut SortedModifiers<'g>,
) {
    for effect in cards.card_def(object_id).kind().hostile_aura_effects() {
        modifiers.add(object_id, effect);
    }
}
