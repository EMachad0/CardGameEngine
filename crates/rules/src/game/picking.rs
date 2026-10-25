//! An open pick: the cards it covers in the order shown, which of them are options, what the
//! player has picked, and the verbs that run once the pick is complete.

use crate::cards::object::ObjectId;
use crate::choice::AmountBound;
use crate::effect::CardVerb;
use crate::game::zone_move::CardOriginZone;
use crate::ids::PlayerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Picking {
    Cards(CardPicking),
}

/// The cards a pick covers, and the zone that holds them.
pub(super) struct PickWindow {
    pub(super) holder: PlayerId,
    pub(super) from: CardOriginZone,
    /// In the order shown, each with whether the pick's kind filter lets the player pick it.
    pub(super) cards: Vec<(ObjectId, bool)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CardPicking {
    holder: PlayerId,
    from: CardOriginZone,
    /// In the order shown.
    window: Vec<WindowCard>,
    /// In the order picked.
    picked: Vec<ObjectId>,
    count: u8,
    bound: AmountBound,
    verbs: Vec<CardVerb>,
    others: Vec<CardVerb>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WindowCard {
    object_id: ObjectId,
    /// Whether the pick's kind filter lets the player pick it.
    pickable: bool,
}

/// A complete pick's objects, by subject.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum PickedObjects {
    Cards(PickedCards),
}

/// A complete card pick's cards, each group with the verbs that act on it.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct PickedCards {
    /// Whose `from` zone holds the cards.
    pub(super) holder: PlayerId,
    pub(super) from: CardOriginZone,
    /// In the order picked.
    pub(super) picked: Vec<ObjectId>,
    pub(super) verbs: Vec<CardVerb>,
    /// Every other card the pick covers, in the order shown.
    pub(super) rest: Vec<ObjectId>,
    pub(super) others: Vec<CardVerb>,
}

impl Picking {
    /// The options not picked yet, in the order shown.
    pub(super) fn options_left(&self) -> impl Iterator<Item = ObjectId> {
        match self {
            Picking::Cards(cards) => cards.options_left(),
        }
    }

    /// Picked or not.
    pub(super) fn option_count(&self) -> usize {
        match self {
            Picking::Cards(cards) => cards.option_count(),
        }
    }

    /// Every object the pick covers, in the order shown, each with whether it is an option.
    pub(super) fn covered(&self) -> impl Iterator<Item = (ObjectId, bool)> {
        match self {
            Picking::Cards(cards) => cards.covered(),
        }
    }

    /// Whether it holds its count, or every option if there are fewer.
    pub(super) fn is_complete(&self) -> bool {
        match self {
            Picking::Cards(cards) => cards.is_complete(),
        }
    }

    /// Panics unless `object_id` is an option not picked yet, in a pick that isn't complete.
    pub(super) fn pick(&mut self, object_id: ObjectId) {
        match self {
            Picking::Cards(cards) => cards.pick(object_id),
        }
    }

    /// Its objects, if it is complete. Otherwise the pick, still open.
    pub(super) fn finish(self) -> Result<PickedObjects, Self> {
        match self {
            Picking::Cards(cards) => cards
                .finish()
                .map(PickedObjects::Cards)
                .map_err(Picking::Cards),
        }
    }
}

impl CardPicking {
    pub(super) fn new(
        window: PickWindow,
        count: u8,
        bound: AmountBound,
        verbs: Vec<CardVerb>,
        others: Vec<CardVerb>,
    ) -> Self {
        let PickWindow {
            holder,
            from,
            cards,
        } = window;
        Self {
            holder,
            from,
            window: cards
                .into_iter()
                .map(|(object_id, pickable)| WindowCard {
                    object_id,
                    pickable,
                })
                .collect(),
            picked: Vec::new(),
            count,
            bound,
            verbs,
            others,
        }
    }

    fn finish(self) -> Result<PickedCards, Self> {
        if !self.is_complete() {
            return Err(self);
        }
        let rest = self
            .window
            .iter()
            .map(|card| card.object_id)
            .filter(|object_id| !self.picked.contains(object_id))
            .collect();
        Ok(PickedCards {
            holder: self.holder,
            from: self.from,
            picked: self.picked,
            verbs: self.verbs,
            rest,
            others: self.others,
        })
    }

    fn options(&self) -> impl Iterator<Item = ObjectId> {
        self.window
            .iter()
            .filter(|card| card.pickable)
            .map(|card| card.object_id)
    }

    fn options_left(&self) -> impl Iterator<Item = ObjectId> {
        self.options()
            .filter(|object_id| !self.picked.contains(object_id))
    }

    fn option_count(&self) -> usize {
        self.options().count()
    }

    fn covered(&self) -> impl Iterator<Item = (ObjectId, bool)> {
        self.window
            .iter()
            .map(|card| (card.object_id, card.pickable))
    }

    fn is_complete(&self) -> bool {
        let required = usize::from(self.count).min(self.option_count());
        self.bound.is_full(self.picked.len(), required)
    }

    fn pick(&mut self, object_id: ObjectId) {
        assert!(
            !self.is_complete(),
            "a picking should take no pick once it is complete",
        );
        assert!(
            self.options_left().any(|option| option == object_id),
            "a card found among a picking's options should stay there until picked",
        );
        self.picked.push(object_id);
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::cards::binder::sets::base::static_card_definition::SPARK;
    use crate::cards::object::ObjectBag;
    use crate::effect::CardDestinationZone;

    fn ids<const N: usize>() -> [ObjectId; N] {
        let mut bag = ObjectBag::default();
        std::array::from_fn(|_| bag.insert(SPARK, PlayerId::new(0)))
    }

    fn picking(window: &[(ObjectId, bool)], count: u8) -> CardPicking {
        CardPicking::new(
            PickWindow {
                holder: PlayerId::new(1),
                from: CardOriginZone::Deck,
                cards: window.to_vec(),
            },
            count,
            AmountBound::Exactly,
            vec![CardVerb::Move(CardDestinationZone::Hand)],
            vec![CardVerb::Move(CardDestinationZone::DeckBottom)],
        )
    }

    #[test]
    fn the_options_are_the_pickable_cards_in_the_order_shown() {
        let [a, b, c] = ids();

        let picking = picking(&[(a, true), (b, false), (c, true)], 1);

        assert_eq!(picking.options_left().collect::<Vec<_>>(), [a, c]);
        assert_eq!(picking.option_count(), 2);
    }

    #[test]
    fn a_picked_card_is_no_longer_an_option_left() {
        let [a, b, c] = ids();
        let mut picking = picking(&[(a, true), (b, true), (c, true)], 2);

        picking.pick(b);

        assert_eq!(picking.options_left().collect::<Vec<_>>(), [a, c]);
        assert_eq!(
            picking.option_count(),
            3,
            "a picked card is still an option"
        );
    }

    #[test]
    fn a_pick_is_complete_once_it_holds_its_count() {
        let [a, b, c] = ids();
        let mut picking = picking(&[(a, true), (b, true), (c, true)], 2);

        picking.pick(a);
        assert!(!picking.is_complete(), "one of two picked");

        picking.pick(c);
        assert!(picking.is_complete());
    }

    #[test]
    fn a_pick_with_fewer_options_than_its_count_is_complete_once_it_holds_them_all() {
        let [a, b] = ids();
        let mut picking = picking(&[(a, true), (b, false)], 2);

        picking.pick(a);

        assert!(picking.is_complete());
    }

    #[test]
    fn a_pick_with_no_options_is_complete_from_the_start() {
        let [a] = ids();

        assert!(picking(&[(a, false)], 1).is_complete());
        assert!(picking(&[], 1).is_complete());
    }

    #[test]
    fn finishing_gives_the_picked_cards_in_the_order_picked_and_the_rest_in_the_order_shown() {
        let [a, b, c, d] = ids();
        let mut picking = picking(&[(a, false), (b, true), (c, true), (d, true)], 2);
        picking.pick(d);
        picking.pick(b);

        assert_eq!(
            picking.finish(),
            Ok(PickedCards {
                holder: PlayerId::new(1),
                from: CardOriginZone::Deck,
                picked: vec![d, b],
                verbs: vec![CardVerb::Move(CardDestinationZone::Hand)],
                rest: vec![a, c],
                others: vec![CardVerb::Move(CardDestinationZone::DeckBottom)],
            }),
        );
    }

    #[test]
    fn an_incomplete_pick_stays_open_when_it_finishes() {
        let [a, b] = ids();
        let open = picking(&[(a, true), (b, true)], 1);

        assert_eq!(open.clone().finish(), Err(open));
    }

    #[test]
    #[should_panic(
        expected = "a card found among a picking's options should stay there until picked"
    )]
    fn a_card_the_filter_left_out_cannot_be_picked() {
        let [a, b] = ids();

        picking(&[(a, false), (b, true)], 1).pick(a);
    }

    #[test]
    #[should_panic(expected = "a picking should take no pick once it is complete")]
    fn a_complete_pick_takes_no_more() {
        let [a, b] = ids();
        let mut picking = picking(&[(a, true), (b, true)], 1);
        picking.pick(a);

        picking.pick(b);
    }

    #[test]
    #[should_panic(
        expected = "a card found among a picking's options should stay there until picked"
    )]
    fn a_card_cannot_be_picked_twice() {
        let [a, b] = ids();
        let mut picking = picking(&[(a, true), (b, true)], 2);
        picking.pick(a);

        picking.pick(a);
    }
}
