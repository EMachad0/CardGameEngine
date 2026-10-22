//! What the game awaits from each player, and what they targeted and picked. The game awaits at
//! most one thing from a player: targeting while playing a card, or picking while a card
//! resolves. A revealed card sits in a picking's options until picked, then in its picks.

use std::marker::PhantomData;

use crate::cards::object::ObjectId;
use crate::effect::PickId;
use crate::ids::PlayerId;
use crate::target::TargetId;

/// What the game awaits from one player, as read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Awaiting<'a> {
    Nothing(AwaitsNothing<'a>),
    Picking(AwaitsPicking<'a>),
    Targeting(AwaitsTargeting<'a>),
}

/// That the game awaits nothing from a player. It borrows the player input, so nothing is awaited
/// from them while this lives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct AwaitsNothing<'a> {
    player_id: PlayerId,
    input: PhantomData<&'a PlayerInput>,
}

/// That the game awaits a player's picking. It borrows the player input, so the picking stays open
/// while this lives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct AwaitsPicking<'a> {
    player_id: PlayerId,
    picking: &'a Picking,
}

impl<'a> AwaitsPicking<'a> {
    /// In reveal order.
    pub(super) fn options(self) -> impl Iterator<Item = WasAwaitingPicking> + 'a {
        let player_id = self.player_id;
        self.picking
            .options
            .iter()
            .map(move |&object_id| WasAwaitingPicking {
                player_id,
                object_id,
            })
    }
}

/// That the game awaits a player's targeting. It borrows the player input, so the targeting stays
/// open while this lives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct AwaitsTargeting<'a> {
    player_id: PlayerId,
    targeting: &'a Targeting,
}

impl<'a> AwaitsTargeting<'a> {
    pub(super) fn targeting(self) -> &'a Targeting {
        self.targeting
    }
}

/// That the game was awaiting nothing from a player. It holds no borrow, so the game may have
/// started waiting on them since.
pub(super) struct WasAwaitingNothing {
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
pub(super) struct WasAwaitingTargeting {
    player_id: PlayerId,
    object_id: ObjectId,
}

impl From<AwaitsTargeting<'_>> for WasAwaitingTargeting {
    fn from(awaits_targeting: AwaitsTargeting<'_>) -> Self {
        Self {
            player_id: awaits_targeting.player_id,
            object_id: awaits_targeting.targeting.object_id,
        }
    }
}

impl WasAwaitingTargeting {
    /// The card being targeted for.
    pub(super) fn object_id(&self) -> ObjectId {
        self.object_id
    }
}

/// That the game was awaiting a player's picking, and one of its options. It holds no borrow, so
/// the picking may have closed since.
pub(super) struct WasAwaitingPicking {
    player_id: PlayerId,
    object_id: ObjectId,
}

impl WasAwaitingPicking {
    pub(super) fn object_id(&self) -> ObjectId {
        self.object_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Picking {
    source: ObjectId,
    pick_id: PickId,
    options: Vec<ObjectId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Targeting {
    object_id: ObjectId,
    targeted: Vec<(ObjectId, TargetId)>,
}

impl Targeting {
    /// With nothing targeted yet.
    pub(super) fn new(object_id: ObjectId) -> Self {
        Self {
            object_id,
            targeted: Vec::new(),
        }
    }

    pub(super) fn object_id(&self) -> ObjectId {
        self.object_id
    }

    /// In the order targeted.
    pub(super) fn targeted_objects(&self) -> impl Iterator<Item = ObjectId> {
        self.targeted.iter().map(|&(object_id, _)| object_id)
    }

    /// In the order targeted.
    pub(super) fn targeted_for(&self, target_id: TargetId) -> impl Iterator<Item = ObjectId> {
        self.targeted
            .iter()
            .filter(move |&&(_, id)| id == target_id)
            .map(|&(object_id, _)| object_id)
    }

    /// Whether `object_id` is targeted for any target.
    pub(super) fn is_targeted(&self, object_id: ObjectId) -> bool {
        self.targeted_objects()
            .any(|targeted| targeted == object_id)
    }

    /// Runs `f` on this targeting with `object_id` also targeted for `target_id`, then takes it
    /// back out.
    pub(super) fn with_targeted<R>(
        &mut self,
        object_id: ObjectId,
        target_id: TargetId,
        f: impl FnOnce(&mut Self) -> R,
    ) -> R {
        self.targeted.push((object_id, target_id));
        let result = f(self);
        self.targeted.pop();
        result
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StoredTargets {
    object_id: ObjectId,
    target_id: TargetId,
    targeted: Vec<ObjectId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Picks {
    source: ObjectId,
    pick_id: PickId,
    picked: Vec<ObjectId>,
    others: Vec<ObjectId>,
}

impl Picks {
    pub(super) fn picked(&self) -> &[ObjectId] {
        &self.picked
    }

    /// The options not picked, in reveal order.
    pub(super) fn others(&self) -> &[ObjectId] {
        &self.others
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) struct PlayerInput {
    /// Sorted by player. No player is in both this and `pickings`.
    targetings: Vec<(PlayerId, Targeting)>,
    /// Sorted by player.
    pickings: Vec<(PlayerId, Picking)>,
    /// Sorted by card, then in the order stored.
    targets: Vec<StoredTargets>,
    /// Sorted by source, then in the order picked.
    picks: Vec<Picks>,
}

impl PlayerInput {
    pub(super) fn awaiting(&self, player_id: PlayerId) -> Awaiting<'_> {
        self.awaiting_targeting(player_id)
            .map(Awaiting::Targeting)
            .or_else(|| self.awaiting_picking(player_id).map(Awaiting::Picking))
            .unwrap_or(Awaiting::Nothing(AwaitsNothing {
                player_id,
                input: PhantomData,
            }))
    }

    pub(super) fn any_awaiting(&self) -> bool {
        !self.targetings.is_empty() || !self.pickings.is_empty()
    }

    /// A picking's options. Empty while awaiting anything else, or nothing.
    pub(super) fn revealed(&self, player_id: PlayerId) -> &[ObjectId] {
        self.awaiting_picking(player_id)
            .map_or(&[], |awaits_picking| {
                awaits_picking.picking.options.as_slice()
            })
    }

    pub(super) fn targeting(&self, player_id: PlayerId) -> Option<&Targeting> {
        self.awaiting_targeting(player_id)
            .map(AwaitsTargeting::targeting)
    }

    /// `None` unless the card was played after targeting and declares `target_id`.
    pub(super) fn targeted(&self, object_id: ObjectId, target_id: TargetId) -> Option<&[ObjectId]> {
        self.targets
            .iter()
            .find(|entry| entry.object_id == object_id && entry.target_id == target_id)
            .map(|entry| entry.targeted.as_slice())
    }

    /// The first picks `source` stored for `pick_id`.
    pub(super) fn picks(&self, source: ObjectId, pick_id: PickId) -> Option<&Picks> {
        self.picks
            .iter()
            .find(|entry| entry.source == source && entry.pick_id == pick_id)
    }

    /// With nothing targeted yet. Panics if the game awaits anything from the player.
    pub(super) fn start_targeting(
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
    pub(super) fn target(
        &mut self,
        awaiting_targeting: WasAwaitingTargeting,
        object_id: ObjectId,
        target_id: TargetId,
    ) {
        let idx = self.targeting_idx(&awaiting_targeting);
        self.targetings[idx].1.targeted.push((object_id, target_id));
    }

    /// Panics unless the player is still targeting the card.
    pub(super) fn cancel(&mut self, awaiting_targeting: WasAwaitingTargeting) {
        let idx = self.targeting_idx(&awaiting_targeting);
        self.targetings.remove(idx);
    }

    /// Closes the targeting and stores its card's targeted objects for each target in
    /// `target_ids`, replacing any it had. Panics unless the player is still targeting the card.
    pub(super) fn commit_targeting(
        &mut self,
        awaiting_targeting: WasAwaitingTargeting,
        target_ids: &[TargetId],
    ) {
        let (_, closed) = self
            .targetings
            .remove(self.targeting_idx(&awaiting_targeting));
        let object_id = closed.object_id;
        self.targets.retain(|entry| entry.object_id != object_id);
        let idx = self
            .targets
            .partition_point(|entry| entry.object_id <= object_id);
        let entries = target_ids.iter().map(|&target_id| StoredTargets {
            object_id,
            target_id,
            targeted: closed.targeted_for(target_id).collect(),
        });
        self.targets.splice(idx..idx, entries);
    }

    /// Starts the player's picking from `options`, replacing any targeting they have. With no
    /// options, starts nothing and returns `false`. Panics if they are picking.
    pub(super) fn start_picking(
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
        let picking = Picking {
            source,
            pick_id,
            options,
        };
        self.pickings.insert(idx, (player_id, picking));
        true
    }

    /// Closes the option's picking, stores the option as picked and the other options as
    /// the others, and returns the player who picked and the picked object. Panics unless the
    /// option is still pending.
    pub(super) fn pick(&mut self, awaiting_picking: WasAwaitingPicking) -> (PlayerId, ObjectId) {
        let found = entry_idx(&self.pickings, awaiting_picking.player_id)
            .ok()
            .and_then(|picking_idx| {
                let options = &self.pickings[picking_idx].1.options;
                let option_idx = options
                    .iter()
                    .position(|&o| o == awaiting_picking.object_id)?;
                Some((picking_idx, option_idx))
            });
        let (picking_idx, option_idx) =
            found.expect("a card found among a picking's options should stay there until picked");
        let (
            player_id,
            Picking {
                source,
                pick_id,
                mut options,
            },
        ) = self.pickings.remove(picking_idx);
        let picked = options.remove(option_idx);
        let idx = self.picks.partition_point(|entry| entry.source <= source);
        self.picks.insert(
            idx,
            Picks {
                source,
                pick_id,
                picked: vec![picked],
                others: options,
            },
        );
        (player_id, picked)
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
            .filter(|&idx| self.targetings[idx].1.object_id == awaiting_targeting.object_id)
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

    fn commit_targeting(input: &mut PlayerInput, player: PlayerId, target_ids: &[TargetId]) {
        let awaiting_targeting = was_awaiting_targeting(input, player);
        input.commit_targeting(awaiting_targeting, target_ids);
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
    fn committed_targeting_stores_targeted_objects_for_every_declared_target() {
        let [card, target] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();
        start_targeting(&mut input, player, card);
        target_object(&mut input, player, target, TargetId(0));

        commit_targeting(&mut input, player, &[TargetId(0), TargetId(1)]);

        assert_eq!(input.targeted(card, TargetId(0)), Some(&[target][..]));
        assert_eq!(
            input.targeted(card, TargetId(1)),
            Some(&[][..]),
            "a declared target with nothing targeted must still store an empty list",
        );
    }

    #[test]
    fn committed_targeting_closes_the_targeting() {
        let [card] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();
        start_targeting(&mut input, player, card);

        commit_targeting(&mut input, player, &[TargetId(0)]);

        assert!(is_awaiting_nothing(&input, player));
    }

    #[test]
    fn committing_a_cards_targeting_again_replaces_what_it_targeted() {
        let [card, first, second] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();
        start_targeting(&mut input, player, card);
        target_object(&mut input, player, first, TargetId(0));
        commit_targeting(&mut input, player, &[TargetId(0)]);
        start_targeting(&mut input, player, card);
        target_object(&mut input, player, second, TargetId(0));

        commit_targeting(&mut input, player, &[TargetId(0)]);

        assert_eq!(input.targeted(card, TargetId(0)), Some(&[second][..]));
    }

    #[test]
    fn cancelled_targeting_stores_nothing_targeted() {
        let [card, target] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();
        start_targeting(&mut input, player, card);
        target_object(&mut input, player, target, TargetId(0));

        cancel(&mut input, player);

        assert_eq!(input.targeted(card, TargetId(0)), None);
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
    fn targeted_for_lists_one_targets_objects_in_the_order_targeted() {
        let [card, first, other, second] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();
        start_targeting(&mut input, player, card);
        target_object(&mut input, player, first, TargetId(0));
        target_object(&mut input, player, other, TargetId(1));
        target_object(&mut input, player, second, TargetId(0));

        let targeting = input.targeting(player).unwrap();

        assert_eq!(
            targeting.targeted_for(TargetId(0)).collect::<Vec<_>>(),
            [first, second]
        );
    }

    #[test]
    fn an_object_targeted_for_any_target_is_targeted() {
        let [card, object] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();
        start_targeting(&mut input, player, card);

        target_object(&mut input, player, object, TargetId(1));

        assert!(input.targeting(player).unwrap().is_targeted(object));
    }

    #[test]
    fn an_object_not_yet_targeted_is_not_targeted() {
        let [card, object, other] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();
        start_targeting(&mut input, player, card);

        target_object(&mut input, player, object, TargetId(0));

        assert!(!input.targeting(player).unwrap().is_targeted(other));
    }

    #[test]
    fn targeting_lists_its_targeted_objects_in_the_order_targeted() {
        let [card, first, second] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();
        start_targeting(&mut input, player, card);
        target_object(&mut input, player, first, TargetId(1));
        target_object(&mut input, player, second, TargetId(0));

        let targeting = input.targeting(player).unwrap();

        assert_eq!(
            targeting.targeted_objects().collect::<Vec<_>>(),
            [first, second]
        );
    }

    #[test]
    fn with_targeted_runs_f_with_the_object_also_targeted() {
        let [card, first, second] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();
        start_targeting(&mut input, player, card);
        target_object(&mut input, player, first, TargetId(0));
        let mut targeting = input.targeting(player).unwrap().clone();

        let targeted = targeting.with_targeted(second, TargetId(0), |targeting| {
            targeting.targeted_for(TargetId(0)).collect::<Vec<_>>()
        });

        assert_eq!(targeted, [first, second]);
    }

    #[test]
    fn with_targeted_leaves_the_targeting_as_it_found_it() {
        let [card, first, second] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();
        start_targeting(&mut input, player, card);
        target_object(&mut input, player, first, TargetId(0));
        let mut targeting = input.targeting(player).unwrap().clone();

        targeting.with_targeted(second, TargetId(0), |_| {});

        assert_eq!(&targeting, input.targeting(player).unwrap());
    }

    #[test]
    fn nothing_is_awaited_before_any_player_is_asked() {
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
