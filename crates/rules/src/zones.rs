//! A player's card containers. `Deck` is ordered top first and `Hand` oldest first.

use std::collections::VecDeque;

use crate::ObjectId;
use crate::rng::Rng;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Deck(VecDeque<ObjectId>);

impl Deck {
    /// Index 0 is the top.
    pub(crate) fn new<I: Into<VecDeque<ObjectId>>>(deck: I) -> Self {
        Self(deck.into())
    }

    /// A copy, top first. Through `&self`, a `VecDeque` can't lend one contiguous slice.
    pub(crate) fn to_vec(&self) -> Vec<ObjectId> {
        self.0.iter().copied().collect()
    }

    pub(crate) fn shuffle(&mut self, rng: &mut Rng) {
        rng.shuffle(self.0.make_contiguous());
    }

    /// Takes the top card.
    pub(crate) fn pop_front(&mut self) -> Option<ObjectId> {
        self.0.pop_front()
    }

    /// Puts a card on the bottom.
    pub(crate) fn push_back(&mut self, card: ObjectId) {
        self.0.push_back(card);
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Hand(Vec<ObjectId>);

impl Hand {
    /// Appends to the end.
    pub(crate) fn add(&mut self, object_id: ObjectId) {
        self.0.push(object_id);
    }

    /// Removes one object_id. The others keep their order.
    pub(crate) fn remove(&mut self, object_id: ObjectId) -> Option<ObjectId> {
        self.position(object_id).map(|idx| self.0.remove(idx))
    }

    pub(crate) fn as_slice(&self) -> &[ObjectId] {
        self.0.as_slice()
    }

    pub(crate) fn contains(&self, object_id: &ObjectId) -> bool {
        self.0.contains(object_id)
    }

    fn position(&self, object_id: ObjectId) -> Option<usize> {
        self.0.iter().position(|o| *o == object_id)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Board {
    monsters: Vec<ObjectId>,
}

impl Board {
    /// Appends to the end.
    pub(crate) fn add(&mut self, object_id: ObjectId) {
        self.monsters.push(object_id);
    }

    /// Removes one card. The others keep their order.
    pub(crate) fn remove(&mut self, object_id: ObjectId) -> Option<ObjectId> {
        self.position(object_id)
            .map(|idx| self.monsters.remove(idx))
    }

    pub(crate) fn as_slice(&self) -> &[ObjectId] {
        self.monsters.as_slice()
    }

    pub(crate) fn contains(&self, object_id: &ObjectId) -> bool {
        self.monsters.contains(object_id)
    }

    fn position(&self, object_id: ObjectId) -> Option<usize> {
        self.monsters.iter().position(|o| *o == object_id)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Graveyard(Vec<ObjectId>);

impl Graveyard {
    /// Appends to the top.
    pub(super) fn add(&mut self, object_id: ObjectId) {
        self.0.push(object_id);
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Zones {
    pub hand: Hand,
    pub deck: Deck,
    pub board: Board,
    pub graveyard: Graveyard,
}

#[cfg(test)]
mod tests {
    use crate::cards::object::ObjectBag;

    use super::*;

    fn ids<const N: usize>() -> [ObjectId; N] {
        let mut bag = ObjectBag::default();
        std::array::from_fn(|_| bag.next_id())
    }

    fn hand_of(cards: &[ObjectId]) -> Hand {
        let mut hand = Hand::default();
        for &card in cards {
            hand.add(card);
        }
        hand
    }

    #[test]
    fn a_deck_takes_from_the_top_and_puts_on_the_bottom() {
        let [a, b, c, d] = ids();
        let mut deck = Deck::new(vec![a, b, c]);

        assert_eq!(deck.pop_front(), Some(a));
        deck.push_back(d);

        assert_eq!(deck.to_vec(), [b, c, d]);
    }

    #[test]
    fn an_empty_deck_gives_nothing() {
        assert_eq!(Deck::default().pop_front(), None);
    }

    #[test]
    fn a_shuffle_is_a_function_of_the_rng() {
        let cards: [ObjectId; 10] = ids();
        let mut a = Deck::new(cards.to_vec());
        let mut b = Deck::new(cards.to_vec());

        a.shuffle(&mut Rng::new(7));
        b.shuffle(&mut Rng::new(7));

        assert_eq!(a, b);
    }

    #[test]
    fn a_shuffle_keeps_every_card() {
        let cards: [ObjectId; 10] = ids();
        let mut deck = Deck::new(cards.to_vec());

        deck.shuffle(&mut Rng::new(7));

        let mut shuffled = deck.to_vec();
        shuffled.sort();
        assert_eq!(shuffled, cards);
    }

    #[test]
    fn a_hand_adds_to_the_end() {
        let [a, b] = ids();
        let mut hand = Hand::default();

        hand.add(a);
        hand.add(b);

        assert_eq!(hand.as_slice(), [a, b]);
    }

    #[test]
    fn removing_from_a_hand_keeps_the_rest_in_order() {
        let [a, b, c, d] = ids();
        let mut hand = hand_of(&[a, b, c, d]);

        assert_eq!(hand.remove(b), Some(b));

        assert_eq!(hand.as_slice(), [a, c, d]);
    }

    #[test]
    fn removing_a_card_not_in_the_hand_changes_nothing() {
        let [a, b] = ids();
        let mut hand = hand_of(&[a]);

        assert_eq!(hand.remove(b), None);

        assert_eq!(hand.as_slice(), [a]);
    }
}
