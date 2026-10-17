//! What each player is asked, and what each card was answered. A player has at most one
//! question: a draft while playing a card, or a pending pick while a card resolves. A revealed card
//! sits in a pending pick's options until picked, then in its pick's answer.

use crate::cards::object::ObjectId;
use crate::choice::{ChoiceId, PickId};
use crate::ids::PlayerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Question {
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
struct ChoiceAnswer {
    object_id: ObjectId,
    choice_id: ChoiceId,
    targets: Vec<ObjectId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PickAnswer {
    source: ObjectId,
    pick_id: PickId,
    picked: Vec<ObjectId>,
    others: Vec<ObjectId>,
}

impl PickAnswer {
    pub(super) fn picked(&self) -> &[ObjectId] {
        &self.picked
    }

    /// The options not picked, in reveal order.
    pub(super) fn others(&self) -> &[ObjectId] {
        &self.others
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) struct Questions {
    /// Sorted by player.
    pending: Vec<(PlayerId, Question)>,
    /// Sorted by card, then in the order stored.
    choice_answers: Vec<ChoiceAnswer>,
    /// Sorted by source, then in the order picked.
    pick_answers: Vec<PickAnswer>,
}

impl Questions {
    pub(super) fn question(&self, player_id: PlayerId) -> Option<&Question> {
        self.slot(player_id).ok().map(|idx| &self.pending[idx].1)
    }

    pub(super) fn any_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    /// A pending pick's options. Empty for any other question, or none.
    pub(super) fn revealed(&self, player_id: PlayerId) -> &[ObjectId] {
        match self.question(player_id) {
            Some(Question::Pick(pick)) => pick.options(),
            Some(Question::Draft(_)) | None => &[],
        }
    }

    pub(super) fn draft(&self, player_id: PlayerId) -> Option<&Draft> {
        match self.question(player_id) {
            Some(Question::Draft(draft)) => Some(draft),
            Some(Question::Pick(_)) | None => None,
        }
    }

    /// `None` if the card was never played from a draft that declared `choice_id`.
    pub(super) fn choice_answer(
        &self,
        object_id: ObjectId,
        choice_id: ChoiceId,
    ) -> Option<&[ObjectId]> {
        self.choice_answers
            .iter()
            .find(|answer| answer.object_id == object_id && answer.choice_id == choice_id)
            .map(|answer| answer.targets.as_slice())
    }

    /// The first answer `source` stored for `pick_id`.
    pub(super) fn pick_answer(&self, source: ObjectId, pick_id: PickId) -> Option<&PickAnswer> {
        self.pick_answers
            .iter()
            .find(|answer| answer.source == source && answer.pick_id == pick_id)
    }

    /// Replaces any draft the player has. Panics if they have a pending pick.
    pub(super) fn open_draft(&mut self, player_id: PlayerId, object_id: ObjectId) {
        self.ask(
            player_id,
            Question::Draft(Draft {
                object_id,
                chosen: Vec::new(),
            }),
        );
    }

    pub(super) fn choose(&mut self, player_id: PlayerId, object_id: ObjectId, choice_id: ChoiceId) {
        let Some(Question::Draft(draft)) = self.question_mut(player_id) else {
            unreachable!();
        };
        draft.chosen.push((object_id, choice_id));
    }

    /// Drops the player's draft, if any. Panics if they have a pending pick.
    pub(super) fn cancel(&mut self, player_id: PlayerId) {
        if let Ok(idx) = self.slot(player_id) {
            self.assert_droppable(idx);
            self.pending.remove(idx);
        }
    }

    /// Closes the player's draft and stores one answer per choice in `choice_ids` for the
    /// drafted card, replacing any it had.
    pub(super) fn commit_draft(&mut self, player_id: PlayerId, choice_ids: &[ChoiceId]) {
        let Some(Question::Draft(Draft { object_id, chosen })) = self.take(player_id) else {
            unreachable!();
        };
        self.choice_answers
            .retain(|answer| answer.object_id != object_id);
        let idx = self
            .choice_answers
            .partition_point(|answer| answer.object_id <= object_id);
        let answers = choice_ids.iter().map(|&choice_id| ChoiceAnswer {
            object_id,
            choice_id,
            targets: chosen
                .iter()
                .filter(|(_, c)| *c == choice_id)
                .map(|(o, _)| *o)
                .collect(),
        });
        self.choice_answers.splice(idx..idx, answers);
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
            Question::Pick(PendingPick {
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
        let Some(Question::Pick(PendingPick {
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
        let idx = self
            .pick_answers
            .partition_point(|answer| answer.source <= source);
        self.pick_answers.insert(
            idx,
            PickAnswer {
                source,
                pick_id,
                picked: vec![picked],
                others: options,
            },
        );
    }

    fn question_mut(&mut self, player_id: PlayerId) -> Option<&mut Question> {
        self.slot(player_id)
            .ok()
            .map(|idx| &mut self.pending[idx].1)
    }

    fn ask(&mut self, player_id: PlayerId, question: Question) {
        match self.slot(player_id) {
            Ok(idx) => {
                self.assert_droppable(idx);
                self.pending[idx].1 = question;
            }
            Err(idx) => self.pending.insert(idx, (player_id, question)),
        }
    }

    fn take(&mut self, player_id: PlayerId) -> Option<Question> {
        self.slot(player_id)
            .ok()
            .map(|idx| self.pending.remove(idx).1)
    }

    fn assert_droppable(&self, idx: usize) {
        let holds_revealed_cards = match &self.pending[idx].1 {
            Question::Pick(_) => true,
            Question::Draft(_) => false,
        };
        assert!(
            !holds_revealed_cards,
            "a pending pick closes only when the player picks",
        );
    }

    fn slot(&self, player_id: PlayerId) -> Result<usize, usize> {
        self.pending
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
        let mut questions = Questions::default();

        assert!(!questions.ask_pick(player, source, PickId(0), Vec::new()));

        assert_eq!(questions.question(player), None);
        assert!(!questions.any_pending());
    }

    #[test]
    fn a_pick_keeps_the_options_left_as_the_others() {
        let [source, a, b, c] = ids();
        let player = PlayerId::new(0);
        let mut questions = Questions::default();
        questions.ask_pick(player, source, PickId(0), vec![a, b, c]);

        questions.pick(player, b);

        let answer = questions.pick_answer(source, PickId(0)).unwrap();
        assert_eq!(answer.picked(), [b]);
        assert_eq!(answer.others(), [a, c]);
    }

    #[test]
    fn a_pick_closes_the_pending_pick() {
        let [source, a] = ids();
        let player = PlayerId::new(0);
        let mut questions = Questions::default();
        questions.ask_pick(player, source, PickId(0), vec![a]);

        questions.pick(player, a);

        assert_eq!(questions.question(player), None);
    }

    #[test]
    #[should_panic(expected = "a pending pick closes only when the player picks")]
    fn a_pending_pick_cannot_be_cancelled() {
        let [source, a] = ids();
        let player = PlayerId::new(0);
        let mut questions = Questions::default();
        questions.ask_pick(player, source, PickId(0), vec![a]);

        questions.cancel(player);
    }

    #[test]
    #[should_panic(expected = "a pending pick closes only when the player picks")]
    fn a_pending_pick_cannot_be_replaced() {
        let [source, card, a] = ids();
        let player = PlayerId::new(0);
        let mut questions = Questions::default();
        questions.ask_pick(player, source, PickId(0), vec![a]);

        questions.open_draft(player, card);
    }

    #[test]
    fn a_pending_pick_shows_its_options_as_revealed() {
        let [source, a, b] = ids();
        let player = PlayerId::new(0);
        let mut questions = Questions::default();

        questions.ask_pick(player, source, PickId(0), vec![a, b]);

        assert_eq!(questions.revealed(player), [a, b]);
    }

    #[test]
    fn a_committed_draft_answers_every_declared_choice() {
        let [card, target] = ids();
        let player = PlayerId::new(0);
        let mut questions = Questions::default();
        questions.open_draft(player, card);
        questions.choose(player, target, ChoiceId(0));

        questions.commit_draft(player, &[ChoiceId(0), ChoiceId(1)]);

        assert_eq!(
            questions.choice_answer(card, ChoiceId(0)),
            Some(&[target][..])
        );
        assert_eq!(
            questions.choice_answer(card, ChoiceId(1)),
            Some(&[][..]),
            "a declared choice with no target must still have an answer",
        );
    }

    #[test]
    fn a_committed_draft_closes_the_draft() {
        let [card] = ids();
        let player = PlayerId::new(0);
        let mut questions = Questions::default();
        questions.open_draft(player, card);

        questions.commit_draft(player, &[ChoiceId(0)]);

        assert_eq!(questions.question(player), None);
    }

    #[test]
    fn committing_a_cards_draft_again_replaces_its_answers() {
        let [card, first, second] = ids();
        let player = PlayerId::new(0);
        let mut questions = Questions::default();
        questions.open_draft(player, card);
        questions.choose(player, first, ChoiceId(0));
        questions.commit_draft(player, &[ChoiceId(0)]);
        questions.open_draft(player, card);
        questions.choose(player, second, ChoiceId(0));

        questions.commit_draft(player, &[ChoiceId(0)]);

        assert_eq!(
            questions.choice_answer(card, ChoiceId(0)),
            Some(&[second][..])
        );
    }

    #[test]
    fn a_cancelled_draft_leaves_no_answer() {
        let [card, target] = ids();
        let player = PlayerId::new(0);
        let mut questions = Questions::default();
        questions.open_draft(player, card);
        questions.choose(player, target, ChoiceId(0));

        questions.cancel(player);

        assert_eq!(questions.choice_answer(card, ChoiceId(0)), None);
    }

    #[test]
    fn an_open_draft_shows_no_revealed_cards() {
        let [card] = ids();
        let player = PlayerId::new(0);
        let mut questions = Questions::default();

        questions.open_draft(player, card);

        assert_eq!(questions.revealed(player), []);
    }

    #[test]
    fn an_open_draft_names_its_card() {
        let [card] = ids();
        let player = PlayerId::new(0);
        let mut questions = Questions::default();

        questions.open_draft(player, card);

        assert_eq!(questions.draft(player).map(Draft::object_id), Some(card));
    }

    #[test]
    fn one_players_question_keeps_the_game_waiting() {
        let [card] = ids();
        let mut questions = Questions::default();

        questions.open_draft(PlayerId::new(1), card);

        assert!(questions.any_pending());
        assert_eq!(questions.question(PlayerId::new(0)), None);
    }
}
