//! What the game awaits from each player, and what they targeted and picked. The game awaits at
//! most one thing from a player: targeting while playing a card, or a pending pick while a card
//! resolves. A revealed card sits in a pending pick's options until picked, then in its picks.

use crate::cards::object::ObjectId;
use crate::effect::PickId;
use crate::ids::PlayerId;
use crate::target::TargetId;

/// What the game awaits from one player, as read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Awaiting<'a> {
    Pick(AwaitedPick<'a>),
    Targets(AwaitedTargeting<'a>),
}

/// A pending pick, with where it is kept. It borrows the player input, so the pick stays pending
/// while this lives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct AwaitedPick<'a> {
    idx: usize,
    pick: &'a PendingPick,
}

impl<'a> AwaitedPick<'a> {
    /// In reveal order.
    pub(super) fn options(self) -> impl Iterator<Item = PickOption> + 'a {
        let pick = self.idx;
        self.pick
            .options
            .iter()
            .enumerate()
            .map(move |(idx, &object_id)| PickOption {
                pick,
                idx,
                object_id,
            })
    }
}

/// A targeting, with where it is kept. It borrows the player input, so the targeting stays open
/// while this lives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct AwaitedTargeting<'a> {
    idx: usize,
    targeting: &'a Targeting,
}

impl<'a> AwaitedTargeting<'a> {
    pub(super) fn targeting(self) -> &'a Targeting {
        self.targeting
    }

    pub(super) fn slot(self) -> TargetingSlot {
        TargetingSlot { idx: self.idx }
    }
}

/// Where a targeting is kept. It holds no borrow, so it stays true only until what the game
/// awaits changes.
pub(super) struct TargetingSlot {
    idx: usize,
}

/// One option of a pending pick, by where it is kept. It holds no borrow, so it stays true only
/// until what the game awaits changes.
pub(super) struct PickOption {
    pick: usize,
    idx: usize,
    object_id: ObjectId,
}

impl PickOption {
    pub(super) fn object_id(&self) -> ObjectId {
        self.object_id
    }
}

/// That the game awaited nothing from any player when asked. It holds no borrow, so it stays true
/// only until a player is awaited.
#[derive(Clone, Copy)]
pub(super) struct NothingAwaited(());

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingPick {
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
    /// Sorted by player. No player is in both this and `pending_picks`.
    targeting: Vec<(PlayerId, Targeting)>,
    /// Sorted by player.
    pending_picks: Vec<(PlayerId, PendingPick)>,
    /// Sorted by card, then in the order stored.
    targets: Vec<StoredTargets>,
    /// Sorted by source, then in the order picked.
    picks: Vec<Picks>,
}

impl PlayerInput {
    pub(super) fn awaiting(&self, player_id: PlayerId) -> Option<Awaiting<'_>> {
        if let Ok(idx) = slot(&self.targeting, player_id) {
            let targeting = &self.targeting[idx].1;
            return Some(Awaiting::Targets(AwaitedTargeting { idx, targeting }));
        }
        slot(&self.pending_picks, player_id).ok().map(|idx| {
            let pick = &self.pending_picks[idx].1;
            Awaiting::Pick(AwaitedPick { idx, pick })
        })
    }

    pub(super) fn any_awaiting(&self) -> bool {
        !self.targeting.is_empty() || !self.pending_picks.is_empty()
    }

    /// `None` while the game awaits anything from any player.
    pub(super) fn nothing_awaited(&self) -> Option<NothingAwaited> {
        (!self.any_awaiting()).then_some(NothingAwaited(()))
    }

    /// A pending pick's options. Empty while awaiting anything else, or nothing.
    pub(super) fn revealed(&self, player_id: PlayerId) -> &[ObjectId] {
        match slot(&self.pending_picks, player_id) {
            Ok(idx) => self.pending_picks[idx].1.options.as_slice(),
            Err(_) => &[],
        }
    }

    pub(super) fn targeting(&self, player_id: PlayerId) -> Option<&Targeting> {
        slot(&self.targeting, player_id)
            .ok()
            .map(|idx| &self.targeting[idx].1)
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

    /// Starts the player's targeting of `object_id`, with nothing targeted.
    pub(super) fn start_targeting(
        &mut self,
        _: NothingAwaited,
        player_id: PlayerId,
        object_id: ObjectId,
    ) {
        self.targeting.push((player_id, Targeting::new(object_id)));
    }

    pub(super) fn target(
        &mut self,
        targeting: TargetingSlot,
        object_id: ObjectId,
        target_id: TargetId,
    ) {
        self.targeting[targeting.idx]
            .1
            .targeted
            .push((object_id, target_id));
    }

    /// Drops the targeting.
    pub(super) fn cancel(&mut self, targeting: TargetingSlot) {
        self.targeting.remove(targeting.idx);
    }

    /// Closes the targeting and stores its card's targeted objects for each target in
    /// `target_ids`, replacing any it had.
    pub(super) fn commit_targeting(&mut self, targeting: TargetingSlot, target_ids: &[TargetId]) {
        let (_, targeting) = self.targeting.remove(targeting.idx);
        let object_id = targeting.object_id;
        self.targets.retain(|entry| entry.object_id != object_id);
        let idx = self
            .targets
            .partition_point(|entry| entry.object_id <= object_id);
        let entries = target_ids.iter().map(|&target_id| StoredTargets {
            object_id,
            target_id,
            targeted: targeting.targeted_for(target_id).collect(),
        });
        self.targets.splice(idx..idx, entries);
    }

    /// Asks the player to pick one of `options`, replacing any targeting they have. With no
    /// options, asks nothing and returns `false`. Panics if they have a pending pick.
    pub(super) fn ask_pick(
        &mut self,
        player_id: PlayerId,
        source: ObjectId,
        pick_id: PickId,
        options: Vec<ObjectId>,
    ) -> bool {
        if options.is_empty() {
            return false;
        }
        let Err(idx) = slot(&self.pending_picks, player_id) else {
            panic!("a pending pick closes only when the player picks");
        };
        if let Ok(targeting) = slot(&self.targeting, player_id) {
            self.targeting.remove(targeting);
        }
        let pick = PendingPick {
            source,
            pick_id,
            options,
        };
        self.pending_picks.insert(idx, (player_id, pick));
        true
    }

    /// Closes the option's pending pick, stores the option as picked and the other options as
    /// the others, and returns the picked object.
    pub(super) fn pick(&mut self, option: PickOption) -> ObjectId {
        let (
            _,
            PendingPick {
                source,
                pick_id,
                mut options,
            },
        ) = self.pending_picks.remove(option.pick);
        let picked = options.remove(option.idx);
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
        picked
    }
}

/// Where the player's entry is, or would go, in entries sorted by player.
fn slot<T>(entries: &[(PlayerId, T)], player_id: PlayerId) -> Result<usize, usize> {
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

    fn start_targeting(input: &mut PlayerInput, player: PlayerId, card: ObjectId) {
        let nothing_awaited = input.nothing_awaited().unwrap();
        input.start_targeting(nothing_awaited, player, card);
    }

    fn targeting_slot(input: &PlayerInput, player: PlayerId) -> TargetingSlot {
        let Some(Awaiting::Targets(awaited)) = input.awaiting(player) else {
            panic!("the player must be targeting");
        };
        awaited.slot()
    }

    fn target_object(
        input: &mut PlayerInput,
        player: PlayerId,
        object: ObjectId,
        target_id: TargetId,
    ) {
        let targeting = targeting_slot(input, player);
        input.target(targeting, object, target_id);
    }

    fn commit_targeting(input: &mut PlayerInput, player: PlayerId, target_ids: &[TargetId]) {
        let targeting = targeting_slot(input, player);
        input.commit_targeting(targeting, target_ids);
    }

    fn cancel(input: &mut PlayerInput, player: PlayerId) {
        let targeting = targeting_slot(input, player);
        input.cancel(targeting);
    }

    fn pick_options(input: &PlayerInput, player: PlayerId) -> Vec<PickOption> {
        let Some(Awaiting::Pick(pick)) = input.awaiting(player) else {
            panic!("the player must have a pending pick");
        };
        pick.options().collect()
    }

    fn pick(input: &mut PlayerInput, player: PlayerId, object: ObjectId) -> ObjectId {
        let option = pick_options(input, player)
            .into_iter()
            .find(|option| option.object_id() == object)
            .unwrap();
        input.pick(option)
    }

    #[test]
    fn a_reveal_with_no_cards_asks_no_pick() {
        let [source] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();

        assert!(!input.ask_pick(player, source, PickId(0), Vec::new()));

        assert_eq!(input.awaiting(player), None);
        assert!(!input.any_awaiting());
    }

    #[test]
    fn a_pick_keeps_the_options_left_as_the_others() {
        let [source, a, b, c] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();
        input.ask_pick(player, source, PickId(0), vec![a, b, c]);

        pick(&mut input, player, b);

        let picks = input.picks(source, PickId(0)).unwrap();
        assert_eq!(picks.picked(), [b]);
        assert_eq!(picks.others(), [a, c]);
    }

    #[test]
    fn a_pick_closes_the_pending_pick() {
        let [source, a] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();
        input.ask_pick(player, source, PickId(0), vec![a]);

        pick(&mut input, player, a);

        assert_eq!(input.awaiting(player), None);
    }

    #[test]
    fn a_pick_names_the_object_picked() {
        let [source, a, b] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();
        input.ask_pick(player, source, PickId(0), vec![a, b]);

        assert_eq!(pick(&mut input, player, b), b);
    }

    #[test]
    fn a_pending_picks_options_come_in_reveal_order() {
        let [source, a, b, c] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();

        input.ask_pick(player, source, PickId(0), vec![b, c, a]);

        let options = pick_options(&input, player);
        assert_eq!(
            options
                .iter()
                .map(PickOption::object_id)
                .collect::<Vec<_>>(),
            [b, c, a]
        );
    }

    #[test]
    #[should_panic(expected = "a pending pick closes only when the player picks")]
    fn a_pending_pick_cannot_be_replaced() {
        let [source, a, b] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();
        input.ask_pick(player, source, PickId(0), vec![a]);

        input.ask_pick(player, source, PickId(1), vec![b]);
    }

    #[test]
    fn a_pending_pick_replaces_the_players_targeting() {
        let [card, source, a] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();
        start_targeting(&mut input, player, card);

        input.ask_pick(player, source, PickId(0), vec![a]);

        assert_eq!(input.targeting(player), None);
        assert_eq!(input.revealed(player), [a]);
    }

    #[test]
    fn a_pending_pick_shows_its_options_as_revealed() {
        let [source, a, b] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();

        input.ask_pick(player, source, PickId(0), vec![a, b]);

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

        assert_eq!(input.awaiting(player), None);
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

        assert!(input.nothing_awaited().is_some());
    }

    #[test]
    fn a_pending_pick_for_any_player_is_something_awaited() {
        let [source, a] = ids();
        let mut input = PlayerInput::default();

        input.ask_pick(PlayerId::new(1), source, PickId(0), vec![a]);

        assert!(input.nothing_awaited().is_none());
    }

    #[test]
    fn awaiting_one_player_keeps_the_game_waiting() {
        let [card] = ids();
        let mut input = PlayerInput::default();

        start_targeting(&mut input, PlayerId::new(1), card);

        assert!(input.any_awaiting());
        assert_eq!(input.awaiting(PlayerId::new(0)), None);
    }
}
