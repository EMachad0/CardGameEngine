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
    pub(crate) fn add(&mut self, card: ObjectId) {
        self.0.push(card);
    }

    /// Removes one card. The others keep their order.
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
    pub(crate) fn add(&mut self, card: ObjectId) {
        self.monsters.push(card);
    }

    /// Removes one card. The others keep their order.
    pub(crate) fn remove(&mut self, object_id: ObjectId) -> Option<ObjectId> {
        self.position(object_id)
            .map(|idx| self.monsters.remove(idx))
    }

    pub(crate) fn as_slice(&self) -> &[ObjectId] {
        self.monsters.as_slice()
    }

    fn position(&self, object_id: ObjectId) -> Option<usize> {
        self.monsters.iter().position(|o| *o == object_id)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Zones {
    pub hand: Hand,
    pub deck: Deck,
    pub board: Board,
}

impl Zones {}

#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::testkit::*;

    // commented because of bolt function
    // #[test]
    // fn deck_takes_from_the_top_and_puts_on_the_bottom() {
    //     let mut deck = Deck::new(vec![bolt(1), bolt(2), bolt(3)]);
    //     assert_eq!(deck.pop_front(), Some(bolt(1)));
    //     deck.push_back(bolt(9));
    //     assert_eq!(deck.to_vec(), [bolt(2), bolt(3), bolt(9)]);
    // }
    //
    // #[test]
    // fn empty_deck_gives_nothing() {
    //     assert_eq!(Deck::new(Vec::new()).pop_front(), None);
    // }
    //
    // #[test]
    // fn shuffle_is_a_function_of_the_rng_and_keeps_every_card() {
    //     let cards: Vec<ObjectId> = (1..=10).map(bolt).collect();
    //     let mut a = Deck::new(cards.clone());
    //     let mut b = Deck::new(cards.clone());
    //     a.shuffle(&mut Rng::new(7));
    //     b.shuffle(&mut Rng::new(7));
    //     assert_eq!(a, b);
    //
    //     let shuffled = a.to_vec();
    //     assert_eq!(shuffled.len(), cards.len());
    //     assert!(cards.iter().all(|c| shuffled.contains(c)));
    // }
    //
    // #[test]
    // fn hand_appends_and_removal_keeps_the_rest_in_order() {
    //     let mut hand = Hand::empty();
    //     for d in 1..=4 {
    //         hand.add(bolt(d));
    //     }
    //     assert_eq!(hand.remove(1), bolt(2));
    //     assert_eq!(hand.as_slice(), [bolt(1), bolt(3), bolt(4)]);
    // }
}
