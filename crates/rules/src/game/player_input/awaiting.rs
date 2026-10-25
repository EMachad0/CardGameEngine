//! What the game awaits from each player: their targeting while they play a card, or the
//! resolving effect's picking if they choose for it. The game awaits at most one thing from a
//! player.

use std::marker::PhantomData;

use crate::cards::object::ObjectId;
use crate::game::cards_ref::CardsRef;
use crate::game::effect_stack::{AwaitsPicking, EffectStack};
use crate::game::player_input::targeting::Targeting;
use crate::ids::PlayerId;
use crate::target::TargetId;

/// What the game awaits from one player, as read.
#[derive(Debug, Clone, Copy)]
pub(in crate::game) enum Awaiting<'a> {
    Nothing(AwaitsNothing<'a>),
    Picking(AwaitsPicking<'a>),
    Targeting(AwaitsTargeting<'a>),
}

/// That the game awaits nothing from a player. It borrows the player input and the effect stack,
/// so nothing is awaited from them while this lives.
#[derive(Debug, Clone, Copy)]
pub(in crate::game) struct AwaitsNothing<'a> {
    player_id: PlayerId,
    input: PhantomData<(&'a PlayerInput, &'a EffectStack)>,
}

/// That the game awaits a player's targeting. It borrows the player input, so the targeting stays
/// open while this lives.
#[derive(Debug, Clone, Copy)]
pub(in crate::game) struct AwaitsTargeting<'a> {
    player_id: PlayerId,
    targeting: &'a Targeting,
}

impl<'a> AwaitsTargeting<'a> {
    pub(in crate::game) fn targeting(self) -> &'a Targeting {
        self.targeting
    }
}

/// That the game was awaiting nothing from a player. It holds no borrow, so the game may have
/// started waiting on them since.
pub(in crate::game) struct WasAwaitingNothing {
    player_id: PlayerId,
}

impl From<AwaitsNothing<'_>> for WasAwaitingNothing {
    fn from(awaits_nothing: AwaitsNothing<'_>) -> Self {
        Self {
            player_id: awaits_nothing.player_id,
        }
    }
}

/// That the game was awaiting a player's targeting of a card. It holds no borrow, so the targeting
/// may have closed since.
pub(in crate::game) struct WasAwaitingTargeting {
    player_id: PlayerId,
    object_id: ObjectId,
}

impl From<AwaitsTargeting<'_>> for WasAwaitingTargeting {
    fn from(awaits_targeting: AwaitsTargeting<'_>) -> Self {
        Self {
            player_id: awaits_targeting.player_id,
            object_id: awaits_targeting.targeting.object_id(),
        }
    }
}

impl WasAwaitingTargeting {
    pub(in crate::game) fn player_id(&self) -> PlayerId {
        self.player_id
    }

    /// The card being targeted for.
    pub(in crate::game) fn object_id(&self) -> ObjectId {
        self.object_id
    }
}

/// Each player's targeting.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(in crate::game) struct PlayerInput {
    /// Sorted by player.
    targetings: Vec<(PlayerId, Targeting)>,
}

pub(in crate::game) fn awaiting<'a>(
    cards: CardsRef,
    player_input: &'a PlayerInput,
    effect_stack: &'a EffectStack,
    player_id: PlayerId,
) -> Awaiting<'a> {
    if let Some(awaits_picking) = effect_stack.awaited_picking()
        && cards.chooser(awaits_picking.source()) == player_id
    {
        return Awaiting::Picking(awaits_picking);
    }
    player_input.awaiting_targeting(player_id).map_or(
        Awaiting::Nothing(AwaitsNothing {
            player_id,
            input: PhantomData,
        }),
        Awaiting::Targeting,
    )
}

/// Whether the game awaits anything from any player.
pub(in crate::game) fn any_awaiting(
    player_input: &PlayerInput,
    effect_stack: &EffectStack,
) -> bool {
    player_input.any_targeting() || effect_stack.awaited_picking().is_some()
}

impl PlayerInput {
    pub(in crate::game) fn any_targeting(&self) -> bool {
        !self.targetings.is_empty()
    }

    /// Finds the targeting again. Panics unless the player is still targeting the card.
    pub(in crate::game) fn still_targeting(
        &self,
        awaiting_targeting: &WasAwaitingTargeting,
    ) -> &Targeting {
        &self.targetings[self.targeting_idx(awaiting_targeting)].1
    }

    /// With nothing targeted yet. Panics if the player is targeting.
    pub(in crate::game) fn start_targeting(
        &mut self,
        awaiting_nothing: WasAwaitingNothing,
        object_id: ObjectId,
    ) {
        let player_id = awaiting_nothing.player_id;
        let Err(idx) = entry_idx(&self.targetings, player_id) else {
            panic!("the game should await nothing from the player until their targeting starts");
        };
        self.targetings
            .insert(idx, (player_id, Targeting::new(object_id)));
    }

    /// Panics unless the player is still targeting the card.
    pub(in crate::game) fn target(
        &mut self,
        awaiting_targeting: WasAwaitingTargeting,
        object_id: ObjectId,
        target_id: TargetId,
    ) {
        let idx = self.targeting_idx(&awaiting_targeting);
        self.targetings[idx].1.target(object_id, target_id);
    }

    /// Panics unless the player is still targeting the card.
    pub(in crate::game) fn cancel(&mut self, awaiting_targeting: WasAwaitingTargeting) {
        let idx = self.targeting_idx(&awaiting_targeting);
        self.targetings.remove(idx);
    }

    /// Closes the targeting. Panics unless the player is still targeting the card.
    pub(in crate::game) fn commit_targeting(&mut self, awaiting_targeting: WasAwaitingTargeting) {
        let idx = self.targeting_idx(&awaiting_targeting);
        self.targetings.remove(idx);
    }

    fn awaiting_targeting(&self, player_id: PlayerId) -> Option<AwaitsTargeting<'_>> {
        let idx = entry_idx(&self.targetings, player_id).ok()?;
        let targeting = &self.targetings[idx].1;
        Some(AwaitsTargeting {
            player_id,
            targeting,
        })
    }

    fn targeting_idx(&self, awaiting_targeting: &WasAwaitingTargeting) -> usize {
        entry_idx(&self.targetings, awaiting_targeting.player_id)
            .ok()
            .filter(|&idx| self.targetings[idx].1.object_id() == awaiting_targeting.object_id)
            .expect("a targeting found open should stay open until used")
    }
}

/// Where the player's entry is, or would go, in entries sorted by player.
fn entry_idx<T>(entries: &[(PlayerId, T)], player_id: PlayerId) -> Result<usize, usize> {
    entries.binary_search_by_key(&player_id.idx(), |(p, _)| p.idx())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::cards::binder::sets::base::static_card_definition::SPARK;
    use crate::cards::object::ObjectBag;

    fn ids<const N: usize>() -> [ObjectId; N] {
        let mut bag = ObjectBag::default();
        std::array::from_fn(|_| bag.insert(SPARK, PlayerId::new(0)))
    }

    fn targeting(input: &PlayerInput, player: PlayerId) -> Option<ObjectId> {
        input
            .awaiting_targeting(player)
            .map(|awaits_targeting| awaits_targeting.targeting().object_id())
    }

    /// Checks the targetings alone, since the player input holds no picking.
    fn was_awaiting_nothing(input: &PlayerInput, player: PlayerId) -> WasAwaitingNothing {
        assert_eq!(
            targeting(input, player),
            None,
            "the player must not be targeting"
        );
        WasAwaitingNothing { player_id: player }
    }

    fn start_targeting(input: &mut PlayerInput, player: PlayerId, card: ObjectId) {
        let awaiting_nothing = was_awaiting_nothing(input, player);
        input.start_targeting(awaiting_nothing, card);
    }

    fn was_awaiting_targeting(input: &PlayerInput, player: PlayerId) -> WasAwaitingTargeting {
        let awaits_targeting = input
            .awaiting_targeting(player)
            .expect("the player must be targeting");
        WasAwaitingTargeting::from(awaits_targeting)
    }

    fn target_object(
        input: &mut PlayerInput,
        player: PlayerId,
        object: ObjectId,
        target_id: TargetId,
    ) {
        let awaiting_targeting = was_awaiting_targeting(input, player);
        input.target(awaiting_targeting, object, target_id);
    }

    fn commit_targeting(input: &mut PlayerInput, player: PlayerId) {
        let awaiting_targeting = was_awaiting_targeting(input, player);
        input.commit_targeting(awaiting_targeting);
    }

    fn cancel(input: &mut PlayerInput, player: PlayerId) {
        let awaiting_targeting = was_awaiting_targeting(input, player);
        input.cancel(awaiting_targeting);
    }

    #[test]
    #[should_panic(
        expected = "the game should await nothing from the player until their targeting starts"
    )]
    fn a_targeting_cannot_start_while_the_player_is_targeting() {
        let [first, second] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();
        let earlier = was_awaiting_nothing(&input, player);
        start_targeting(&mut input, player, first);

        input.start_targeting(earlier, second);
    }

    #[test]
    fn each_player_keeps_their_own_targeting() {
        let [first, second] = ids();
        let mut input = PlayerInput::default();

        start_targeting(&mut input, PlayerId::new(1), first);
        start_targeting(&mut input, PlayerId::new(0), second);

        assert_eq!(
            [PlayerId::new(0), PlayerId::new(1)].map(|player| targeting(&input, player)),
            [Some(second), Some(first)],
        );
    }

    #[test]
    #[should_panic(expected = "a targeting found open should stay open until used")]
    fn a_targeting_cannot_be_used_once_it_closes() {
        let [card] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();
        start_targeting(&mut input, player, card);
        let first = was_awaiting_targeting(&input, player);
        let second = was_awaiting_targeting(&input, player);
        input.cancel(first);

        input.cancel(second);
    }

    #[test]
    #[should_panic(expected = "a targeting found open should stay open until used")]
    fn a_targeting_of_one_card_cannot_be_used_for_another() {
        let [first, second] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();
        start_targeting(&mut input, player, first);
        let earlier = was_awaiting_targeting(&input, player);
        cancel(&mut input, player);
        start_targeting(&mut input, player, second);

        input.cancel(earlier);
    }

    #[test]
    fn a_targeting_is_used_after_another_players_targeting_closes() {
        let [first, second] = ids();
        let (player, other) = (PlayerId::new(0), PlayerId::new(1));
        let mut input = PlayerInput::default();
        start_targeting(&mut input, player, first);
        start_targeting(&mut input, other, second);
        let awaiting_targeting = was_awaiting_targeting(&input, other);
        cancel(&mut input, player);

        input.cancel(awaiting_targeting);

        assert_eq!(targeting(&input, other), None);
    }

    #[test]
    fn a_targeting_found_again_holds_what_was_targeted() {
        let [card, target] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();
        start_targeting(&mut input, player, card);
        target_object(&mut input, player, target, TargetId(0));
        let awaiting_targeting = was_awaiting_targeting(&input, player);

        let found = input.still_targeting(&awaiting_targeting);

        assert_eq!(found.object_id(), card);
        assert_eq!(
            found.targeted_for(TargetId(0)).collect::<Vec<_>>(),
            [target]
        );
    }

    #[test]
    #[should_panic(expected = "a targeting found open should stay open until used")]
    fn a_targeting_cannot_be_found_again_once_it_closes() {
        let [card] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();
        start_targeting(&mut input, player, card);
        let awaiting_targeting = was_awaiting_targeting(&input, player);
        cancel(&mut input, player);

        input.still_targeting(&awaiting_targeting);
    }

    #[test]
    fn committed_targeting_closes_the_targeting() {
        let [card] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();
        start_targeting(&mut input, player, card);

        commit_targeting(&mut input, player);

        assert_eq!(targeting(&input, player), None);
    }

    #[test]
    fn targeting_names_its_card() {
        let [card] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();

        start_targeting(&mut input, player, card);

        assert_eq!(targeting(&input, player), Some(card));
    }

    #[test]
    fn no_one_is_targeting_before_a_targeting_starts() {
        let input = PlayerInput::default();

        assert!(!input.any_targeting());
    }

    #[test]
    fn a_targeting_by_any_player_is_a_targeting_open() {
        let [card] = ids();
        let mut input = PlayerInput::default();

        start_targeting(&mut input, PlayerId::new(1), card);

        assert!(input.any_targeting());
        assert_eq!(targeting(&input, PlayerId::new(0)), None);
    }
}
