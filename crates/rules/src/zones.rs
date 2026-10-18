//! Which zone holds each object, and order of play. Every player has a hero and
//! four zones: a hand, a deck, a board and a graveyard. `Deck` is ordered top
//! first, `Hand` oldest first and `Board` left to right. An object is in one zone
//! at a time, or in none. Order of play orders objects by when each last entered
//! a zone, or took its place as a hero.

use std::collections::VecDeque;
use std::marker::PhantomData;

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

/// A card found in a hand. It borrows the zones, so the card stays there while this lives.
#[derive(Debug, Clone, Copy)]
pub(crate) struct InHand<'a> {
    object_id: ObjectId,
    zones: PhantomData<&'a Zones>,
}

impl InHand<'_> {
    fn new(object_id: ObjectId) -> Self {
        Self {
            object_id,
            zones: PhantomData,
        }
    }

    pub(crate) fn object_id(self) -> ObjectId {
        self.object_id
    }
}

/// A card found on a board. It borrows the zones, so the card stays there while this lives.
#[derive(Debug, Clone, Copy)]
pub(crate) struct OnBoard<'a> {
    object_id: ObjectId,
    zones: PhantomData<&'a Zones>,
}

impl OnBoard<'_> {
    fn new(object_id: ObjectId) -> Self {
        Self {
            object_id,
            zones: PhantomData,
        }
    }

    pub(crate) fn object_id(self) -> ObjectId {
        self.object_id
    }
}

/// A character found in play: a hero, or a card on a board. It borrows the zones, so the
/// character stays in play while this lives.
#[derive(Debug, Clone, Copy)]
pub(crate) struct InPlay<'a> {
    object_id: ObjectId,
    zones: PhantomData<&'a Zones>,
}

impl InPlay<'_> {
    fn new(object_id: ObjectId) -> Self {
        Self {
            object_id,
            zones: PhantomData,
        }
    }

    pub(crate) fn object_id(self) -> ObjectId {
        self.object_id
    }
}

impl<'a> From<OnBoard<'a>> for InPlay<'a> {
    fn from(card: OnBoard<'a>) -> Self {
        Self::new(card.object_id)
    }
}

/// Every player's zones, by `PlayerId`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Zones {
    players: Vec<PlayerZones>,
    /// The value of `next_stamp` when each object last entered a zone, sorted by
    /// object.
    entered: Vec<(ObjectId, u64)>,
    next_stamp: u64,
}

impl Zones {
    /// One hero and one deck, top first, per player in `PlayerId` order.
    pub(crate) fn new(players: impl IntoIterator<Item = (ObjectId, Vec<ObjectId>)>) -> Self {
        let mut zones = Self {
            players: Vec::new(),
            entered: Vec::new(),
            next_stamp: 0,
        };
        for (hero, deck) in players {
            zones.stamp_entry(hero);
            for &card in &deck {
                zones.stamp_entry(card);
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

    /// Oldest first.
    pub(crate) fn hand_cards(&self, player_id: PlayerId) -> impl Iterator<Item = InHand<'_>> {
        self.hand(player_id).iter().copied().map(InHand::new)
    }

    /// Left to right.
    pub(crate) fn board_cards(&self, player_id: PlayerId) -> impl Iterator<Item = OnBoard<'_>> {
        self.board(player_id).iter().copied().map(OnBoard::new)
    }

    pub(crate) fn hero_in_play(&self, player_id: PlayerId) -> InPlay<'_> {
        InPlay::new(self.hero(player_id))
    }

    pub(crate) fn find_in_hand(
        &self,
        player_id: PlayerId,
        object_id: ObjectId,
    ) -> Option<InHand<'_>> {
        self.player(player_id)
            .hand
            .contains(&object_id)
            .then(|| InHand::new(object_id))
    }

    pub(crate) fn find_on_board(
        &self,
        player_id: PlayerId,
        object_id: ObjectId,
    ) -> Option<OnBoard<'_>> {
        self.player(player_id)
            .board
            .contains(&object_id)
            .then(|| OnBoard::new(object_id))
    }

    pub(crate) fn find_in_play(&self, object_id: ObjectId) -> Option<InPlay<'_>> {
        self.characters_in_play()
            .any(|character| character == object_id)
            .then(|| InPlay::new(object_id))
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

    /// Where a character's triggers come among those one event matches.
    pub(crate) fn match_order(&self, character: ObjectId) -> impl Ord + use<> {
        self.order_of_play(character)
    }

    /// Where a death comes among the deaths of one state check pass.
    pub(crate) fn death_order(&self, dead: ObjectId) -> impl Ord + use<> {
        self.order_of_play(dead)
    }

    /// Where a trigger comes among one object's triggers, by the object that granted it.
    pub(crate) fn grant_order(&self, granted_by: ObjectId) -> impl Ord + use<> {
        self.order_of_play(granted_by)
    }

    pub(crate) fn shuffle_deck(&mut self, player_id: PlayerId, rng: &mut Rng) {
        self.player_mut(player_id).deck.shuffle(rng);
    }

    /// Moves the top card of the deck to the end of the hand. `None` if the deck is empty.
    pub(crate) fn draw(&mut self, player_id: PlayerId) -> Option<ObjectId> {
        let player = self.player_mut(player_id);
        let card = player.deck.pop_front()?;
        self.stamp_entry(card);
        self.player_mut(player_id).hand.add(card);
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

    /// Puts a card that is in no zone at the end of the hand.
    pub(crate) fn add_to_hand(&mut self, player_id: PlayerId, object_id: ObjectId) {
        self.stamp_entry(object_id);
        self.player_mut(player_id).hand.add(object_id);
    }

    /// Puts a card that is in no zone on the bottom of the deck.
    pub(crate) fn bury(&mut self, player_id: PlayerId, object_id: ObjectId) {
        self.stamp_entry(object_id);
        self.player_mut(player_id).deck.push_back(object_id);
    }

    /// Puts a card that is in no zone at the right end of the board.
    pub(crate) fn enter_board(&mut self, player_id: PlayerId, object_id: ObjectId) {
        self.stamp_entry(object_id);
        self.player_mut(player_id).board.add(object_id);
    }

    /// Puts a card that is in no zone on top of the graveyard.
    pub(crate) fn entomb(&mut self, player_id: PlayerId, object_id: ObjectId) {
        self.stamp_entry(object_id);
        self.player_mut(player_id).graveyard.add(object_id);
    }

    /// Stamps an object about to enter a zone. Panics if it is in one already.
    fn stamp_entry(&mut self, object_id: ObjectId) {
        assert!(
            !self.players.iter().any(|player| player.holds(&object_id)),
            "an object enters a zone only from no zone",
        );
        match self.entered.binary_search_by_key(&object_id, |&(o, _)| o) {
            Ok(idx) => self.entered[idx].1 = self.next_stamp,
            Err(idx) => self.entered.insert(idx, (object_id, self.next_stamp)),
        }
        self.next_stamp += 1;
    }

    fn order_of_play(&self, object_id: ObjectId) -> u64 {
        let idx = self
            .entered
            .binary_search_by_key(&object_id, |&(o, _)| o)
            .expect("every object enters a zone when it is made");
        self.entered[idx].1
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

    #[test]
    fn order_of_play_follows_each_objects_latest_entry_into_a_zone() {
        let [hero, first, second] = ids();
        let player = PlayerId::new(0);
        let mut zones = Zones::new([(hero, vec![first, second])]);

        zones.draw(player);
        zones.draw(player);
        for card in [second, first] {
            zones.take_from_hand(player, card);
            zones.enter_board(player, card);
        }

        let mut deaths = vec![first, hero, second];
        deaths.sort_by_key(|&o| zones.death_order(o));
        assert_eq!(
            deaths,
            [hero, second, first],
            "the card that entered the board last must come last",
        );
    }

    /// Player 0 has `summoned` on the board, `held` in hand and `in_deck` in the deck. Player 1
    /// has only a hero.
    fn one_of_each() -> (Zones, [ObjectId; 5]) {
        let [hero, other_hero, summoned, held, in_deck] = ids();
        let player = PlayerId::new(0);
        let mut zones = Zones::new([
            (hero, vec![summoned, held, in_deck]),
            (other_hero, Vec::new()),
        ]);
        zones.draw(player);
        zones.draw(player);
        zones.take_from_hand(player, summoned);
        zones.enter_board(player, summoned);
        (zones, [hero, other_hero, summoned, held, in_deck])
    }

    #[test]
    fn a_card_in_a_hand_is_found_there() {
        let (zones, [_, _, _, held, _]) = one_of_each();

        let found = zones.find_in_hand(PlayerId::new(0), held);

        assert_eq!(found.map(InHand::object_id), Some(held));
    }

    #[test]
    fn nothing_outside_a_hand_is_found_in_it() {
        let (zones, [hero, _, summoned, _, in_deck]) = one_of_each();

        let found = [hero, summoned, in_deck].map(|o| zones.find_in_hand(PlayerId::new(0), o));

        assert!(found.iter().all(Option::is_none));
    }

    #[test]
    fn a_card_on_a_board_is_found_there() {
        let (zones, [_, _, summoned, _, _]) = one_of_each();

        let found = zones.find_on_board(PlayerId::new(0), summoned);

        assert_eq!(found.map(OnBoard::object_id), Some(summoned));
    }

    #[test]
    fn nothing_off_a_board_is_found_on_it() {
        let (zones, [hero, _, _, held, in_deck]) = one_of_each();

        let found = [hero, held, in_deck].map(|o| zones.find_on_board(PlayerId::new(0), o));

        assert!(found.iter().all(Option::is_none));
    }

    #[test]
    fn heroes_and_cards_on_a_board_are_in_play() {
        let (zones, [hero, other_hero, summoned, _, _]) = one_of_each();

        let found =
            [hero, other_hero, summoned].map(|o| zones.find_in_play(o).map(InPlay::object_id));

        assert_eq!(found, [Some(hero), Some(other_hero), Some(summoned)]);
    }

    #[test]
    fn cards_in_a_hand_or_a_deck_are_not_in_play() {
        let (zones, [_, _, _, held, in_deck]) = one_of_each();

        let found = [held, in_deck].map(|o| zones.find_in_play(o));

        assert!(found.iter().all(Option::is_none));
    }

    #[test]
    fn a_players_hero_is_in_play() {
        let [hero, other_hero] = ids();
        let zones = Zones::new([(hero, Vec::new()), (other_hero, Vec::new())]);

        assert_eq!(zones.hero_in_play(PlayerId::new(1)).object_id(), other_hero);
    }

    /// Player 0 has drawn `[a, b, c, d]` and played `c`, then `a`.
    fn c_then_a_played() -> (Zones, [ObjectId; 4]) {
        let [hero, a, b, c, d] = ids();
        let player = PlayerId::new(0);
        let mut zones = Zones::new([(hero, vec![a, b, c, d])]);
        for _ in 0..4 {
            zones.draw(player);
        }
        for card in [c, a] {
            zones.take_from_hand(player, card);
            zones.enter_board(player, card);
        }
        (zones, [a, b, c, d])
    }

    #[test]
    fn walking_a_hand_goes_oldest_first() {
        let (zones, [_, b, _, d]) = c_then_a_played();

        let walked = zones.hand_cards(PlayerId::new(0)).map(InHand::object_id);

        assert_eq!(walked.collect::<Vec<_>>(), [b, d]);
    }

    #[test]
    fn walking_a_board_goes_left_to_right() {
        let (zones, [a, _, c, _]) = c_then_a_played();

        let walked = zones.board_cards(PlayerId::new(0)).map(OnBoard::object_id);

        assert_eq!(
            walked.collect::<Vec<_>>(),
            [c, a],
            "the card played first must come first",
        );
    }

    #[test]
    #[should_panic(expected = "an object enters a zone only from no zone")]
    fn an_object_in_a_zone_cannot_enter_another() {
        let [hero, card] = ids();
        let player = PlayerId::new(0);
        let mut zones = Zones::new([(hero, vec![card])]);

        zones.draw(player);
        zones.enter_board(player, card);
    }

    #[test]
    #[expect(clippy::disallowed_methods, reason = "tests the reveal itself")]
    fn a_revealed_card_leaves_its_deck_for_no_zone() {
        let [hero, top, next] = ids();
        let player = PlayerId::new(0);
        let mut zones = Zones::new([(hero, vec![top, next])]);

        assert_eq!(zones.reveal(player, 1), [top]);

        assert_eq!(zones.deck(player), [next]);
        zones.add_to_hand(player, top);
        assert_eq!(
            zones.hand(player),
            [top],
            "a card in no zone must be free to enter a hand",
        );
    }
}
