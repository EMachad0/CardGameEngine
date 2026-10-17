//! Which zone holds each object. Every player has a hero, a hand, a deck, a board
//! and a graveyard. `Deck` is ordered top first, `Hand` oldest first and `Board`
//! left to right. An object is in one zone at a time, or in none.

use std::collections::VecDeque;

use crate::cards::object::ObjectId;
use crate::ids::PlayerId;
use crate::rng::Rng;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct Deck(VecDeque<ObjectId>);

impl Deck {
    /// Index 0 is the top.
    fn new<I: Into<VecDeque<ObjectId>>>(deck: I) -> Self {
        Self(deck.into())
    }

    /// A copy, top first. Through `&self`, a `VecDeque` can't lend one contiguous slice.
    fn to_vec(&self) -> Vec<ObjectId> {
        self.0.iter().copied().collect()
    }

    fn shuffle(&mut self, rng: &mut Rng) {
        rng.shuffle(self.0.make_contiguous());
    }

    /// Takes the top card.
    fn pop_front(&mut self) -> Option<ObjectId> {
        self.0.pop_front()
    }

    /// Puts a card on the bottom.
    fn push_back(&mut self, card: ObjectId) {
        self.0.push_back(card);
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn contains(&self, object_id: &ObjectId) -> bool {
        self.0.contains(object_id)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct Hand(Vec<ObjectId>);

impl Hand {
    /// Appends to the end.
    fn add(&mut self, object_id: ObjectId) {
        self.0.push(object_id);
    }

    /// Removes one object_id. The others keep their order.
    fn remove(&mut self, object_id: ObjectId) -> Option<ObjectId> {
        self.position(object_id).map(|idx| self.0.remove(idx))
    }

    fn as_slice(&self) -> &[ObjectId] {
        self.0.as_slice()
    }

    fn contains(&self, object_id: &ObjectId) -> bool {
        self.0.contains(object_id)
    }

    fn position(&self, object_id: ObjectId) -> Option<usize> {
        self.0.iter().position(|o| *o == object_id)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct Board {
    minions: Vec<ObjectId>,
}

impl Board {
    /// Appends to the end.
    fn add(&mut self, object_id: ObjectId) {
        self.minions.push(object_id);
    }

    /// Removes one card. The others keep their order.
    fn remove(&mut self, object_id: ObjectId) -> Option<ObjectId> {
        self.position(object_id).map(|idx| self.minions.remove(idx))
    }

    fn as_slice(&self) -> &[ObjectId] {
        self.minions.as_slice()
    }

    fn contains(&self, object_id: &ObjectId) -> bool {
        self.minions.contains(object_id)
    }

    fn position(&self, object_id: ObjectId) -> Option<usize> {
        self.minions.iter().position(|o| *o == object_id)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct Graveyard(Vec<ObjectId>);

impl Graveyard {
    /// Appends to the top.
    fn add(&mut self, object_id: ObjectId) {
        self.0.push(object_id);
    }

    fn contains(&self, object_id: &ObjectId) -> bool {
        self.0.contains(object_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlayerZones {
    hero: ObjectId,
    hand: Hand,
    deck: Deck,
    board: Board,
    graveyard: Graveyard,
}

impl PlayerZones {
    fn holds(&self, object_id: &ObjectId) -> bool {
        self.hero == *object_id
            || self.hand.contains(object_id)
            || self.deck.contains(object_id)
            || self.board.contains(object_id)
            || self.graveyard.contains(object_id)
    }
}

/// Every player's zones, by `PlayerId`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Zones {
    players: Vec<PlayerZones>,
}

impl Zones {
    /// One hero and one deck, top first, per player in `PlayerId` order.
    pub(crate) fn new(players: impl IntoIterator<Item = (ObjectId, Vec<ObjectId>)>) -> Self {
        let mut zones = Self {
            players: Vec::new(),
        };
        for (hero, deck) in players {
            zones.assert_in_no_zone(hero);
            for &card in &deck {
                zones.assert_in_no_zone(card);
            }
            zones.players.push(PlayerZones {
                hero,
                hand: Hand::default(),
                deck: Deck::new(deck),
                board: Board::default(),
                graveyard: Graveyard::default(),
            });
        }
        zones
    }

    pub(crate) fn hero(&self, player_id: PlayerId) -> ObjectId {
        self.player(player_id).hero
    }

    pub(crate) fn hand(&self, player_id: PlayerId) -> &[ObjectId] {
        self.player(player_id).hand.as_slice()
    }

    /// A copy, top first.
    pub(crate) fn deck(&self, player_id: PlayerId) -> Vec<ObjectId> {
        self.player(player_id).deck.to_vec()
    }

    pub(crate) fn deck_size(&self, player_id: PlayerId) -> usize {
        self.player(player_id).deck.len()
    }

    /// Left to right.
    pub(crate) fn board(&self, player_id: PlayerId) -> &[ObjectId] {
        self.player(player_id).board.as_slice()
    }

    pub(crate) fn in_hand(&self, player_id: PlayerId, object_id: ObjectId) -> bool {
        self.player(player_id).hand.contains(&object_id)
    }

    pub(crate) fn on_board(&self, player_id: PlayerId, object_id: ObjectId) -> bool {
        self.player(player_id).board.contains(&object_id)
    }

    /// Every hero, then every player's board, both by `PlayerId`.
    pub(crate) fn characters_in_play(&self) -> impl Iterator<Item = ObjectId> {
        let heroes = self.players.iter().map(|player| player.hero);
        let minions = self
            .players
            .iter()
            .flat_map(|player| player.board.as_slice().iter().copied());
        heroes.chain(minions)
    }

    pub(crate) fn shuffle_deck(&mut self, player_id: PlayerId, rng: &mut Rng) {
        self.player_mut(player_id).deck.shuffle(rng);
    }

    /// Moves the top card of the deck to the end of the hand. `None` if the deck is empty.
    pub(crate) fn draw(&mut self, player_id: PlayerId) -> Option<ObjectId> {
        let player = self.player_mut(player_id);
        let card = player.deck.pop_front()?;
        player.hand.add(card);
        Some(card)
    }

    /// Takes up to `count` cards off the top of the deck, top first, into no zone.
    pub(crate) fn reveal(&mut self, player_id: PlayerId, count: u8) -> Vec<ObjectId> {
        let deck = &mut self.player_mut(player_id).deck;
        (0..count).map_while(|_| deck.pop_front()).collect()
    }

    /// Takes a card out of the hand into no zone. `None` if it isn't there.
    pub(crate) fn take_from_hand(
        &mut self,
        player_id: PlayerId,
        object_id: ObjectId,
    ) -> Option<ObjectId> {
        self.player_mut(player_id).hand.remove(object_id)
    }

    /// Takes a card off the board into no zone. `None` if it isn't there.
    pub(crate) fn leave_board(
        &mut self,
        player_id: PlayerId,
        object_id: ObjectId,
    ) -> Option<ObjectId> {
        self.player_mut(player_id).board.remove(object_id)
    }

    /// Puts a card in no zone at the end of the hand.
    pub(crate) fn add_to_hand(&mut self, player_id: PlayerId, object_id: ObjectId) {
        self.assert_in_no_zone(object_id);
        self.player_mut(player_id).hand.add(object_id);
    }

    /// Puts a card in no zone on the bottom of the deck.
    pub(crate) fn bury(&mut self, player_id: PlayerId, object_id: ObjectId) {
        self.assert_in_no_zone(object_id);
        self.player_mut(player_id).deck.push_back(object_id);
    }

    /// Puts a card in no zone at the right end of the board.
    pub(crate) fn enter_board(&mut self, player_id: PlayerId, object_id: ObjectId) {
        self.assert_in_no_zone(object_id);
        self.player_mut(player_id).board.add(object_id);
    }

    /// Puts a card in no zone on top of the graveyard.
    pub(crate) fn entomb(&mut self, player_id: PlayerId, object_id: ObjectId) {
        self.assert_in_no_zone(object_id);
        self.player_mut(player_id).graveyard.add(object_id);
    }

    fn assert_in_no_zone(&self, object_id: ObjectId) {
        assert!(
            !self.players.iter().any(|player| player.holds(&object_id)),
            "an object enters a zone only from no zone",
        );
    }

    fn player(&self, player_id: PlayerId) -> &PlayerZones {
        &self.players[player_id.idx()]
    }

    fn player_mut(&mut self, player_id: PlayerId) -> &mut PlayerZones {
        &mut self.players[player_id.idx()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    use crate::cards::binder::sets::base::static_card_definition::SPARK;
    use crate::cards::object::ObjectBag;
    use crate::ids::PlayerId;

    fn ids<const N: usize>() -> [ObjectId; N] {
        let mut bag = ObjectBag::default();
        std::array::from_fn(|_| bag.insert(SPARK, PlayerId::new(0)))
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
