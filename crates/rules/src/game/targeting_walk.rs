//! Whether a card's targeting can be finished, which objects the player may target next, and
//! whether they may stop.

use crate::cards::object::ObjectId;
use crate::game::amount::effect_amount;
use crate::game::cards_ref::CardsRef;
use crate::game::player_input::Targeting;
use crate::target::{CharacterTarget, TargetId};

/// Whether the card's targeting can be finished from nothing targeted.
pub(super) fn can_start_targeting(cards: CardsRef, object_id: ObjectId) -> bool {
    can_finish_targeting(cards, &mut Targeting::new(object_id))
}

/// Whether the player may stop at every target and play the card.
pub(super) fn may_stop_targeting(cards: CardsRef, targeting: &Targeting) -> bool {
    first_unsatisfied_target(cards, targeting).is_none()
}

/// The objects the player may target next, for the first target they may not stop at. Each one
/// leaves the targeting finishable.
pub(super) fn next_candidates(cards: CardsRef, targeting: &Targeting) -> Vec<(TargetId, ObjectId)> {
    let Some(target) = first_unsatisfied_target(cards, targeting) else {
        return Vec::new();
    };
    finishing_candidates(cards, target, &mut targeting.clone())
        .map(|object_id| (target.id, object_id))
        .collect()
}

fn can_finish_targeting(cards: CardsRef, targeting: &mut Targeting) -> bool {
    match first_unsatisfied_target(cards, targeting) {
        None => true,
        Some(target) => finishing_candidates(cards, target, targeting)
            .next()
            .is_some(),
    }
}

fn finishing_candidates<'a>(
    cards: CardsRef<'a>,
    target: &'a CharacterTarget,
    targeting: &'a mut Targeting,
) -> impl Iterator<Item = ObjectId> + 'a {
    let candidates = untargeted_matches(cards, target, targeting);
    candidates.into_iter().filter(move |&candidate| {
        targeting.with_targeted(candidate, target.id, |targeting| {
            can_finish_targeting(cards, targeting)
        })
    })
}

fn first_unsatisfied_target<'g>(
    cards: CardsRef<'g>,
    targeting: &Targeting,
) -> Option<&'g CharacterTarget> {
    let object_id = targeting.object_id();
    cards.card_def(object_id).targets().iter().find(|target| {
        let count = targeting.targeted_for(target.id).count() as u8;
        !target
            .bound
            .is_satisfied(count, required_count(cards, object_id, target))
    })
}

fn untargeted_matches(
    cards: CardsRef,
    target: &CharacterTarget,
    targeting: &Targeting,
) -> Vec<ObjectId> {
    let object_id = targeting.object_id();
    let objects_for_target = targeting.targeted_for(target.id).collect::<Vec<_>>();
    let count = objects_for_target.len() as u8;
    if target
        .bound
        .is_full(count, required_count(cards, object_id, target))
    {
        return Vec::new();
    }

    let owner = cards.objects.get(object_id).player_id;
    cards
        .scan_characters_with_filter(owner, &target.filter)
        .into_iter()
        .filter(|&candidate| !target.different || !targeting.is_targeted(candidate))
        .filter(|candidate| !objects_for_target.contains(candidate))
        .collect()
}

fn required_count(cards: CardsRef, object_id: ObjectId, target: &CharacterTarget) -> u8 {
    let owner = cards.objects.get(object_id).player_id;
    effect_amount(cards, &target.count, owner)
}
