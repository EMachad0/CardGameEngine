//! What the game awaits from each player, and the targets and picks they gave. The game awaits at
//! most one thing from a player: a draft while playing a card, or a pending pick while a card
//! resolves. A revealed card sits in a pending pick's options until picked, then in its picks.

use crate::cards::object::ObjectId;
use crate::choice::{ChoiceId, PickId};
use crate::ids::PlayerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Awaiting {
    Pick(PendingPick),
    Draft(Draft),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PendingPick {
    source: ObjectId,
    pick_id: PickId,
    options: Vec<ObjectId>,
}

impl PendingPick {
    /// In reveal order.
    pub(super) fn options(&self) -> &[ObjectId] {
        &self.options
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Draft {
    object_id: ObjectId,
    chosen: Vec<(ObjectId, ChoiceId)>,
}

impl Draft {
    pub(super) fn object_id(&self) -> ObjectId {
        self.object_id
    }

    /// In the order chosen.
    pub(super) fn chosen(&self) -> &[(ObjectId, ChoiceId)] {
        &self.chosen
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Targets {
    object_id: ObjectId,
    choice_id: ChoiceId,
    chosen: Vec<ObjectId>,
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
    /// Sorted by player.
    awaiting: Vec<(PlayerId, Awaiting)>,
    /// Sorted by card, then in the order stored.
    targets: Vec<Targets>,
    /// Sorted by source, then in the order picked.
    picks: Vec<Picks>,
}

impl PlayerInput {
    pub(super) fn awaiting(&self, player_id: PlayerId) -> Option<&Awaiting> {
        self.slot(player_id).ok().map(|idx| &self.awaiting[idx].1)
    }

    pub(super) fn any_awaiting(&self) -> bool {
        !self.awaiting.is_empty()
    }

    /// A pending pick's options. Empty while awaiting anything else, or nothing.
    pub(super) fn revealed(&self, player_id: PlayerId) -> &[ObjectId] {
        match self.awaiting(player_id) {
            Some(Awaiting::Pick(pick)) => pick.options(),
            Some(Awaiting::Draft(_)) | None => &[],
        }
    }

    pub(super) fn draft(&self, player_id: PlayerId) -> Option<&Draft> {
        match self.awaiting(player_id) {
            Some(Awaiting::Draft(draft)) => Some(draft),
            Some(Awaiting::Pick(_)) | None => None,
        }
    }

    /// `None` if the card was never played from a draft that declared `choice_id`.
    pub(super) fn targets(&self, object_id: ObjectId, choice_id: ChoiceId) -> Option<&[ObjectId]> {
        self.targets
            .iter()
            .find(|entry| entry.object_id == object_id && entry.choice_id == choice_id)
            .map(|entry| entry.chosen.as_slice())
    }

    /// The first picks `source` stored for `pick_id`.
    pub(super) fn picks(&self, source: ObjectId, pick_id: PickId) -> Option<&Picks> {
        self.picks
            .iter()
            .find(|entry| entry.source == source && entry.pick_id == pick_id)
    }

    /// Replaces any draft the player has. Panics if they have a pending pick.
    pub(super) fn open_draft(&mut self, player_id: PlayerId, object_id: ObjectId) {
        self.ask(
            player_id,
            Awaiting::Draft(Draft {
                object_id,
                chosen: Vec::new(),
            }),
        );
    }

    pub(super) fn choose(&mut self, player_id: PlayerId, object_id: ObjectId, choice_id: ChoiceId) {
        let Some(Awaiting::Draft(draft)) = self.awaiting_mut(player_id) else {
            unreachable!();
        };
        draft.chosen.push((object_id, choice_id));
    }

    /// Drops the player's draft, if any. Panics if they have a pending pick.
    pub(super) fn cancel(&mut self, player_id: PlayerId) {
        if let Ok(idx) = self.slot(player_id) {
            self.assert_droppable(idx);
            self.awaiting.remove(idx);
        }
    }

    /// Closes the player's draft and stores the drafted card's targets for each choice in
    /// `choice_ids`, replacing any it had.
    pub(super) fn commit_draft(&mut self, player_id: PlayerId, choice_ids: &[ChoiceId]) {
        let Some(Awaiting::Draft(Draft { object_id, chosen })) = self.take(player_id) else {
            unreachable!();
        };
        self.targets.retain(|entry| entry.object_id != object_id);
        let idx = self
            .targets
            .partition_point(|entry| entry.object_id <= object_id);
        let entries = choice_ids.iter().map(|&choice_id| Targets {
            object_id,
            choice_id,
            chosen: chosen
                .iter()
                .filter(|(_, c)| *c == choice_id)
                .map(|(o, _)| *o)
                .collect(),
        });
        self.targets.splice(idx..idx, entries);
    }

    /// Asks the player to pick one of `options`, replacing any draft they have. With no options,
    /// asks nothing and returns `false`. Panics if they have a pending pick.
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
        self.ask(
            player_id,
            Awaiting::Pick(PendingPick {
                source,
                pick_id,
                options,
            }),
        );
        true
    }

    /// Closes the player's pending pick and stores `object_id` as picked and the other options
    /// as the others.
    pub(super) fn pick(&mut self, player_id: PlayerId, object_id: ObjectId) {
        let Some(Awaiting::Pick(PendingPick {
            source,
            pick_id,
            mut options,
        })) = self.take(player_id)
        else {
            unreachable!();
        };
        let idx = options
            .iter()
            .position(|id| *id == object_id)
            .expect("a legal pick should name one of the options");
        let picked = options.remove(idx);
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
    }

    fn awaiting_mut(&mut self, player_id: PlayerId) -> Option<&mut Awaiting> {
        self.slot(player_id)
            .ok()
            .map(|idx| &mut self.awaiting[idx].1)
    }

    fn ask(&mut self, player_id: PlayerId, awaiting: Awaiting) {
        match self.slot(player_id) {
            Ok(idx) => {
                self.assert_droppable(idx);
                self.awaiting[idx].1 = awaiting;
            }
            Err(idx) => self.awaiting.insert(idx, (player_id, awaiting)),
        }
    }

    fn take(&mut self, player_id: PlayerId) -> Option<Awaiting> {
        self.slot(player_id)
            .ok()
            .map(|idx| self.awaiting.remove(idx).1)
    }

    fn assert_droppable(&self, idx: usize) {
        let holds_revealed_cards = match &self.awaiting[idx].1 {
            Awaiting::Pick(_) => true,
            Awaiting::Draft(_) => false,
        };
        assert!(
            !holds_revealed_cards,
            "a pending pick closes only when the player picks",
        );
    }

    fn slot(&self, player_id: PlayerId) -> Result<usize, usize> {
        self.awaiting
            .binary_search_by_key(&player_id.idx(), |(p, _)| p.idx())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    use crate::cards::binder::sets::base::static_card_definition::SPARK;
    use crate::cards::object::ObjectBag;

    fn ids<const N: usize>() -> [ObjectId; N] {
        let mut bag = ObjectBag::default();
        std::array::from_fn(|_| bag.insert(SPARK, PlayerId::new(0)))
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

        input.pick(player, b);

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

        input.pick(player, a);

        assert_eq!(input.awaiting(player), None);
    }

    #[test]
    #[should_panic(expected = "a pending pick closes only when the player picks")]
    fn a_pending_pick_cannot_be_cancelled() {
        let [source, a] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();
        input.ask_pick(player, source, PickId(0), vec![a]);

        input.cancel(player);
    }

    #[test]
    #[should_panic(expected = "a pending pick closes only when the player picks")]
    fn a_pending_pick_cannot_be_replaced() {
        let [source, card, a] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();
        input.ask_pick(player, source, PickId(0), vec![a]);

        input.open_draft(player, card);
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
    fn a_committed_draft_stores_targets_for_every_declared_choice() {
        let [card, target] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();
        input.open_draft(player, card);
        input.choose(player, target, ChoiceId(0));

        input.commit_draft(player, &[ChoiceId(0), ChoiceId(1)]);

        assert_eq!(input.targets(card, ChoiceId(0)), Some(&[target][..]));
        assert_eq!(
            input.targets(card, ChoiceId(1)),
            Some(&[][..]),
            "a declared choice with no target must still have its targets stored",
        );
    }

    #[test]
    fn a_committed_draft_closes_the_draft() {
        let [card] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();
        input.open_draft(player, card);

        input.commit_draft(player, &[ChoiceId(0)]);

        assert_eq!(input.awaiting(player), None);
    }

    #[test]
    fn committing_a_cards_draft_again_replaces_its_targets() {
        let [card, first, second] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();
        input.open_draft(player, card);
        input.choose(player, first, ChoiceId(0));
        input.commit_draft(player, &[ChoiceId(0)]);
        input.open_draft(player, card);
        input.choose(player, second, ChoiceId(0));

        input.commit_draft(player, &[ChoiceId(0)]);

        assert_eq!(input.targets(card, ChoiceId(0)), Some(&[second][..]));
    }

    #[test]
    fn a_cancelled_draft_stores_no_targets() {
        let [card, target] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();
        input.open_draft(player, card);
        input.choose(player, target, ChoiceId(0));

        input.cancel(player);

        assert_eq!(input.targets(card, ChoiceId(0)), None);
    }

    #[test]
    fn an_open_draft_shows_no_revealed_cards() {
        let [card] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();

        input.open_draft(player, card);

        assert_eq!(input.revealed(player), []);
    }

    #[test]
    fn an_open_draft_names_its_card() {
        let [card] = ids();
        let player = PlayerId::new(0);
        let mut input = PlayerInput::default();

        input.open_draft(player, card);

        assert_eq!(input.draft(player).map(Draft::object_id), Some(card));
    }

    #[test]
    fn awaiting_one_player_keeps_the_game_waiting() {
        let [card] = ids();
        let mut input = PlayerInput::default();

        input.open_draft(PlayerId::new(1), card);

        assert!(input.any_awaiting());
        assert_eq!(input.awaiting(PlayerId::new(0)), None);
    }
}
