//! What the game awaits from each player, and what they picked. The game awaits at most one thing
//! from a player: targeting while playing a card, or picking while a card resolves.

use std::marker::PhantomData;

use crate::cards::object::ObjectId;
use crate::effect::PickId;
use crate::game::player_input::picks::{Picking, Picks, StoredPicks};
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

/// That the game awaits nothing from a player. It borrows the player input, so nothing is awaited
/// from them while this lives.
#[derive(Debug, Clone, Copy)]
pub(in crate::game) struct AwaitsNothing<'a> {
    player_id: PlayerId,
    input: PhantomData<&'a PlayerInput>,
}

/// That the game awaits a player's picking. It borrows the player input, so the picking stays open
/// while this lives.
#[derive(Debug, Clone, Copy)]
pub(in crate::game) struct AwaitsPicking<'a> {
    player_id: PlayerId,
    picking: &'a Picking,
}

impl<'a> AwaitsPicking<'a> {
    /// In reveal order.
    pub(in crate::game) fn options(self) -> impl Iterator<Item = WasAwaitingPicking> + 'a {
        let player_id = self.player_id;
        self.picking
            .options()
            .iter()
            .map(move |&object_id| WasAwaitingPicking {
                player_id,
                object_id,
            })
    }
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

/// That the game was awaiting a player's picking, and one of its options. It holds no borrow, so
/// the picking may have closed since.
pub(in crate::game) struct WasAwaitingPicking {
    player_id: PlayerId,
    object_id: ObjectId,
}

impl WasAwaitingPicking {
    /// The option.
    pub(in crate::game) fn object_id(&self) -> ObjectId {
        self.object_id
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(in crate::game) struct PlayerInput {
    /// Sorted by player. No player is in both this and `pickings`.
    targetings: Vec<(PlayerId, Targeting)>,
    /// Sorted by player.
    pickings: Vec<(PlayerId, Picking)>,
    stored_picks: StoredPicks,
}

impl PlayerInput {
    pub(in crate::game) fn awaiting(&self, player_id: PlayerId) -> Awaiting<'_> {
        self.awaiting_targeting(player_id)
            .map(Awaiting::Targeting)
            .or_else(|| self.awaiting_picking(player_id).map(Awaiting::Picking))
            .unwrap_or(Awaiting::Nothing(AwaitsNothing {
                player_id,
                input: PhantomData,
            }))
    }

    pub(in crate::game) fn any_awaiting(&self) -> bool {
        !self.targetings.is_empty() || !self.pickings.is_empty()
    }

    /// A picking's options. Empty while awaiting anything else, or nothing.
    pub(in crate::game) fn revealed(&self, player_id: PlayerId) -> &[ObjectId] {
        self.awaiting_picking(player_id)
            .map_or(&[], |awaits_picking| awaits_picking.picking.options())
    }

    pub(in crate::game) fn targeting(&self, player_id: PlayerId) -> Option<&Targeting> {
        self.awaiting_targeting(player_id)
            .map(AwaitsTargeting::targeting)
    }

    /// Finds the targeting again. Panics unless the player is still targeting the card.
    pub(in crate::game) fn still_targeting(
        &self,
        awaiting_targeting: &WasAwaitingTargeting,
    ) -> &Targeting {
        &self.targetings[self.targeting_idx(awaiting_targeting)].1
    }

    /// The first picks `source` stored for `pick_id`.
    pub(in crate::game) fn picks(&self, source: ObjectId, pick_id: PickId) -> Option<&Picks> {
        self.stored_picks.find(source, pick_id)
    }

    /// With nothing targeted yet. Panics if the game awaits anything from the player.
    pub(in crate::game) fn start_targeting(
        &mut self,
        awaiting_nothing: WasAwaitingNothing,
        object_id: ObjectId,
    ) {
        let player_id = awaiting_nothing.player_id;
        let (Err(idx), Err(_)) = (
            entry_idx(&self.targetings, player_id),
            entry_idx(&self.pickings, player_id),
        ) else {
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

    /// Starts the player's picking from `options`, replacing any targeting they have. With no
    /// options, starts nothing and returns `false`. Panics if they are picking.
    pub(in crate::game) fn start_picking(
        &mut self,
        player_id: PlayerId,
        source: ObjectId,
        pick_id: PickId,
        options: Vec<ObjectId>,
    ) -> bool {
        if options.is_empty() {
            return false;
        }
        let Err(idx) = entry_idx(&self.pickings, player_id) else {
            panic!("a player who is picking should pick before their next picking starts");
        };
        if let Ok(replaced) = entry_idx(&self.targetings, player_id) {
            self.targetings.remove(replaced);
        }
        let picking = Picking::new(source, pick_id, options);
        self.pickings.insert(idx, (player_id, picking));
        true
    }

    /// Closes the option's picking and stores its picks. Returns the player who picked and the
    /// picked object. Panics unless the card is still among the picking's options.
    pub(in crate::game) fn pick(
        &mut self,
        awaiting_picking: WasAwaitingPicking,
    ) -> (PlayerId, ObjectId) {
        let found = entry_idx(&self.pickings, awaiting_picking.player_id)
            .ok()
            .and_then(|picking_idx| {
                let options = self.pickings[picking_idx].1.options();
                let option_idx = options
                    .iter()
                    .position(|&o| o == awaiting_picking.object_id)?;
                Some((picking_idx, option_idx))
            });
        let (picking_idx, option_idx) =
            found.expect("a card found among a picking's options should stay there until picked");
        let (player_id, picking) = self.pickings.remove(picking_idx);
        self.stored_picks.store(picking.pick(option_idx));
        (player_id, awaiting_picking.object_id)
    }

    fn awaiting_targeting(&self, player_id: PlayerId) -> Option<AwaitsTargeting<'_>> {
        let idx = entry_idx(&self.targetings, player_id).ok()?;
        let targeting = &self.targetings[idx].1;
        Some(AwaitsTargeting {
            player_id,
            targeting,
        })
    }

    fn awaiting_picking(&self, player_id: PlayerId) -> Option<AwaitsPicking<'_>> {
        let idx = entry_idx(&self.pickings, player_id).ok()?;
        let picking = &self.pickings[idx].1;
        Some(AwaitsPicking { player_id, picking })
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

    fn is_awaiting_nothing(input: &PlayerInput, player: PlayerId) -> bool {
        matches!(input.awaiting(player), Awaiting::Nothing(_))
    }

    fn was_awaiting_nothing(input: &PlayerInput, player: PlayerId) -> WasAwaitingNothing {
        let Awaiting::Nothing(awaits_nothing) = input.awaiting(player) else {
            panic!("the game must await nothing from the player");
        };
        WasAwaitingNothing::from(awaits_nothing)
    }

    fn start_targeting(input: &mut PlayerInput, player: PlayerId, card: ObjectId) {
        let awaiting_nothing = was_awaiting_nothing(input, player);
        input.start_targeting(awaiting_nothing, card);
    }

    fn was_awaiting_targeting(input: &PlayerInput, player: PlayerId) -> WasAwaitingTargeting {
        let Awaiting::Targeting(awaits_targeting) = input.awaiting(player) else {
            panic!("the player must be targeting");
        };
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

    fn pick_options(input: &PlayerInput, player: PlayerId) -> Vec<WasAwaitingPicking> {
        let Awaiting::Picking(awaits_picking) = input.awaiting(player) else {
            panic!("the player must be picking");
        };
        awaits_picking.options().collect()
    }

    fn pick(input: &mut PlayerInput, player: PlayerId, object: ObjectId) -> (PlayerId, ObjectId) {
        let awaiting_picking = pick_options(input, player)
            .into_iter()
            .find(|awaiting_picking| awaiting_picking.object_id() == object)
            .unwrap();
        input.pick(awaiting_picking)
    }

    #[test]
    fn a_reveal_with_no_cards_starts_no_picking() {
        let [source] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();

        assert!(!input.start_picking(player, source, PickId(0), Vec::new()));

        assert!(is_awaiting_nothing(&input, player));
        assert!(!input.any_awaiting());
    }

    #[test]
    fn a_pick_keeps_the_options_left_as_the_others() {
        let [source, a, b, c] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();
        input.start_picking(player, source, PickId(0), vec![a, b, c]);

        pick(&mut input, player, b);

        let picks = input.picks(source, PickId(0)).unwrap();
        assert_eq!(picks.picked(), [b]);
        assert_eq!(picks.others(), [a, c]);
    }

    #[test]
    fn a_pick_closes_the_picking() {
        let [source, a] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();
        input.start_picking(player, source, PickId(0), vec![a]);

        pick(&mut input, player, a);

        assert!(is_awaiting_nothing(&input, player));
    }

    #[test]
    fn a_pick_names_the_player_and_the_object_picked() {
        let [source, a, b] = ids();
        let player = PlayerId::new(1);
        let mut input = PlayerInput::default();
        input.start_picking(player, source, PickId(0), vec![a, b]);

        assert_eq!(pick(&mut input, player, b), (player, b));
    }

    #[test]
    fn a_pickings_options_come_in_reveal_order() {
        let [source, a, b, c] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();

        input.start_picking(player, source, PickId(0), vec![b, c, a]);

        let options = pick_options(&input, player);
        assert_eq!(
            options
                .iter()
                .map(WasAwaitingPicking::object_id)
                .collect::<Vec<_>>(),
            [b, c, a]
        );
    }

    #[test]
    #[should_panic(
        expected = "a player who is picking should pick before their next picking starts"
    )]
    fn a_picking_cannot_be_replaced_by_another() {
        let [source, a, b] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();
        input.start_picking(player, source, PickId(0), vec![a]);

        input.start_picking(player, source, PickId(1), vec![b]);
    }

    #[test]
    #[should_panic(
        expected = "the game should await nothing from the player until their targeting starts"
    )]
    fn a_targeting_cannot_start_once_the_player_is_picking() {
        let [source, card, a] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();
        let awaiting_nothing = was_awaiting_nothing(&input, player);
        input.start_picking(player, source, PickId(0), vec![a]);

        input.start_targeting(awaiting_nothing, card);
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
            [PlayerId::new(0), PlayerId::new(1)]
                .map(|player| input.targeting(player).map(Targeting::object_id)),
            [Some(second), Some(first)],
        );
    }

    #[test]
    #[should_panic(
        expected = "a card found among a picking's options should stay there until picked"
    )]
    fn a_card_cannot_be_picked_once_its_picking_closes() {
        let [source, a, b] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();
        input.start_picking(player, source, PickId(0), vec![a, b]);
        let mut options = pick_options(&input, player).into_iter();
        let (first, second) = (options.next().unwrap(), options.next().unwrap());
        input.pick(first);

        input.pick(second);
    }

    #[test]
    fn a_card_is_picked_after_another_players_picking_closes() {
        let [source, a, b] = ids();
        let (player, other) = (PlayerId::new(0), PlayerId::new(1));
        let mut input = PlayerInput::default();
        input.start_picking(player, source, PickId(0), vec![a]);
        input.start_picking(other, source, PickId(1), vec![b]);
        let awaiting_picking = pick_options(&input, other).pop().unwrap();
        pick(&mut input, player, a);

        assert_eq!(input.pick(awaiting_picking), (other, b));
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

        assert!(is_awaiting_nothing(&input, other));
    }

    #[test]
    fn a_picking_replaces_the_players_targeting() {
        let [card, source, a] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();
        start_targeting(&mut input, player, card);

        input.start_picking(player, source, PickId(0), vec![a]);

        assert_eq!(input.targeting(player), None);
        assert_eq!(input.revealed(player), [a]);
    }

    #[test]
    fn a_picking_shows_its_options_as_revealed() {
        let [source, a, b] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();

        input.start_picking(player, source, PickId(0), vec![a, b]);

        assert_eq!(input.revealed(player), [a, b]);
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

        assert!(is_awaiting_nothing(&input, player));
    }

    #[test]
    fn targeting_shows_no_revealed_cards() {
        let [card] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();

        start_targeting(&mut input, player, card);

        assert_eq!(input.revealed(player), []);
    }

    #[test]
    fn targeting_names_its_card() {
        let [card] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();

        start_targeting(&mut input, player, card);

        assert_eq!(
            input.targeting(player).map(Targeting::object_id),
            Some(card)
        );
    }

    #[test]
    fn nothing_is_awaited_before_anything_starts() {
        let input = PlayerInput::default();

        assert!(!input.any_awaiting());
    }

    #[test]
    fn a_picking_by_any_player_is_something_awaited() {
        let [source, a] = ids();
        let mut input = PlayerInput::default();

        input.start_picking(PlayerId::new(1), source, PickId(0), vec![a]);

        assert!(input.any_awaiting());
    }

    #[test]
    fn awaiting_one_player_keeps_the_game_waiting() {
        let [card] = ids();
        let mut input = PlayerInput::default();

        start_targeting(&mut input, PlayerId::new(1), card);

        assert!(input.any_awaiting());
        assert!(is_awaiting_nothing(&input, PlayerId::new(0)));
    }
}
