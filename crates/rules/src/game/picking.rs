//! An open pick: the objects it covers in the order shown, which of them are options, what the
//! player has picked, and the verbs that run once the pick is complete.

use crate::cards::object::ObjectId;
use crate::choice::AmountBound;
use crate::effect::{CardVerb, CharacterVerb};
use crate::game::zone_move::CardOriginZone;
use crate::ids::PlayerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Picking {
    Cards(CardPicking),
    Characters(CharacterPicking),
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
    objects: ObjectPicking<CardVerb>,
}

/// A pick of characters in play. It has no window, so every character it covers is an option.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CharacterPicking(ObjectPicking<CharacterVerb>);

#[derive(Debug, Clone, PartialEq, Eq)]
struct ObjectPicking<V> {
    /// In the order shown.
    covered: Vec<CoveredObject>,
    /// In the order picked.
    picked: Vec<ObjectId>,
    count: u8,
    bound: AmountBound,
    verbs: Vec<V>,
    others: Vec<V>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CoveredObject {
    object_id: ObjectId,
    /// Whether the player may pick it.
    pickable: bool,
}

/// A complete pick's objects, by subject.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum PickedObjects {
    Cards(PickedCards),
    Characters(PickedCharacters),
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct PickedCards {
    holder: PlayerId,
    from: CardOriginZone,
    groups: PickedGroups<CardVerb>,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct PickedCharacters(PickedGroups<CharacterVerb>);

/// A complete pick's objects, each group with the verbs that act on it.
#[derive(Debug, PartialEq, Eq)]
struct PickedGroups<V> {
    /// In the order picked.
    picked: Vec<ObjectId>,
    verbs: Vec<V>,
    /// Every other object the pick covers, in the order shown.
    rest: Vec<ObjectId>,
    others: Vec<V>,
}

type ObjectsWithVerbs<V> = (Vec<ObjectId>, Vec<V>);

impl Picking {
    /// The options not picked yet, in the order shown.
    pub(super) fn options_left(&self) -> impl Iterator<Item = ObjectId> {
        let options_left: Vec<_> = match self {
            Picking::Cards(cards) => cards.objects.options_left().collect(),
            Picking::Characters(characters) => characters.0.options_left().collect(),
        };
        options_left.into_iter()
    }

    /// Picked or not.
    pub(super) fn option_count(&self) -> usize {
        match self {
            Picking::Cards(cards) => cards.objects.option_count(),
            Picking::Characters(characters) => characters.0.option_count(),
        }
    }

    /// Whether it holds its count, or every option if there are fewer.
    pub(super) fn is_complete(&self) -> bool {
        match self {
            Picking::Cards(cards) => cards.objects.is_complete(),
            Picking::Characters(characters) => characters.0.is_complete(),
        }
    }

    /// Panics unless `object_id` is an option not picked yet, in a pick that isn't complete.
    pub(super) fn pick(&mut self, object_id: ObjectId) {
        match self {
            Picking::Cards(cards) => cards.objects.pick(object_id),
            Picking::Characters(characters) => characters.0.pick(object_id),
        }
    }

    /// Its objects, if it is complete. Otherwise the pick, still open.
    pub(super) fn finish(self) -> Result<PickedObjects, Self> {
        match self {
            Picking::Cards(cards) => cards
                .finish()
                .map(PickedObjects::Cards)
                .map_err(Picking::Cards),
            Picking::Characters(characters) => characters
                .finish()
                .map(PickedObjects::Characters)
                .map_err(Picking::Characters),
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
            objects: ObjectPicking::new(cards, count, bound, verbs, others),
        }
    }

    /// Every card it covers, in the order shown, each with whether it is an option.
    pub(super) fn covered(&self) -> impl Iterator<Item = (ObjectId, bool)> {
        self.objects.covered()
    }

    fn finish(self) -> Result<PickedCards, Self> {
        let Self {
            holder,
            from,
            objects,
        } = self;
        objects
            .finish()
            .map(|groups| PickedCards {
                holder,
                from,
                groups,
            })
            .map_err(|objects| Self {
                holder,
                from,
                objects,
            })
    }
}

impl CharacterPicking {
    pub(super) fn new(
        options: Vec<ObjectId>,
        count: u8,
        bound: AmountBound,
        verbs: Vec<CharacterVerb>,
        others: Vec<CharacterVerb>,
    ) -> Self {
        let covered = options.into_iter().map(|object_id| (object_id, true));
        Self(ObjectPicking::new(covered, count, bound, verbs, others))
    }

    /// Picked or not, in the order shown.
    pub(super) fn options(&self) -> impl Iterator<Item = ObjectId> {
        self.0.options()
    }

    fn finish(self) -> Result<PickedCharacters, Self> {
        self.0.finish().map(PickedCharacters).map_err(Self)
    }
}

impl PickedCards {
    /// The player whose `from` zone holds the cards, then each group of cards with the verbs that
    /// act on it, in the order the verbs run: the cards picked, then the rest.
    pub(super) fn into_groups(self) -> (PlayerId, CardOriginZone, [ObjectsWithVerbs<CardVerb>; 2]) {
        (self.holder, self.from, self.groups.into_groups())
    }
}

impl PickedCharacters {
    /// Each group of characters with the verbs that act on it, in the order the verbs run: the
    /// characters picked, then the rest.
    pub(super) fn into_groups(self) -> [ObjectsWithVerbs<CharacterVerb>; 2] {
        self.0.into_groups()
    }
}

impl<V> PickedGroups<V> {
    fn into_groups(self) -> [ObjectsWithVerbs<V>; 2] {
        [(self.picked, self.verbs), (self.rest, self.others)]
    }
}

impl<V> ObjectPicking<V> {
    /// `covered` in the order shown, each with whether the player may pick it.
    fn new(
        covered: impl IntoIterator<Item = (ObjectId, bool)>,
        count: u8,
        bound: AmountBound,
        verbs: Vec<V>,
        others: Vec<V>,
    ) -> Self {
        Self {
            covered: covered
                .into_iter()
                .map(|(object_id, pickable)| CoveredObject {
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

    fn finish(self) -> Result<PickedGroups<V>, Self> {
        if !self.is_complete() {
            return Err(self);
        }
        let rest = self
            .covered
            .iter()
            .map(|object| object.object_id)
            .filter(|object_id| !self.picked.contains(object_id))
            .collect();
        Ok(PickedGroups {
            picked: self.picked,
            verbs: self.verbs,
            rest,
            others: self.others,
        })
    }

    fn options(&self) -> impl Iterator<Item = ObjectId> {
        self.covered
            .iter()
            .filter(|object| object.pickable)
            .map(|object| object.object_id)
    }

    fn options_left(&self) -> impl Iterator<Item = ObjectId> {
        self.options()
            .filter(|object_id| !self.picked.contains(object_id))
    }

    fn option_count(&self) -> usize {
        self.options().count()
    }

    fn covered(&self) -> impl Iterator<Item = (ObjectId, bool)> {
        self.covered
            .iter()
            .map(|object| (object.object_id, object.pickable))
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
            "an object found among a picking's options should stay there until picked",
        );
        self.picked.push(object_id);
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::cards::binder::sets::base::static_card_definition::SPARK;
    use crate::cards::modifier::ModifierEffect;
    use crate::cards::object::ObjectBag;
    use crate::effect::CardDestinationZone;

    fn ids<const N: usize>() -> [ObjectId; N] {
        let mut bag = ObjectBag::default();
        std::array::from_fn(|_| bag.insert(SPARK, PlayerId::new(0)))
    }

    /// A pick of `count` from `covered`, with tags for verbs.
    fn picking(covered: &[(ObjectId, bool)], count: u8) -> ObjectPicking<&'static str> {
        ObjectPicking::new(
            covered.iter().copied(),
            count,
            AmountBound::Exactly,
            vec!["verb"],
            vec!["other"],
        )
    }

    fn card_picking(window: &[(ObjectId, bool)], count: u8) -> CardPicking {
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

    fn buff(amount: u8) -> CharacterVerb {
        CharacterVerb::Modify(ModifierEffect::BuffAtk {
            amount: amount.into(),
        })
    }

    #[test]
    fn the_options_are_the_pickable_objects_in_the_order_shown() {
        let [a, b, c] = ids();

        let picking = picking(&[(a, true), (b, false), (c, true)], 1);

        assert_eq!(picking.options_left().collect::<Vec<_>>(), [a, c]);
        assert_eq!(picking.option_count(), 2);
    }

    #[test]
    fn a_picked_object_is_no_longer_an_option_left() {
        let [a, b, c] = ids();
        let mut picking = picking(&[(a, true), (b, true), (c, true)], 2);

        picking.pick(b);

        assert_eq!(picking.options_left().collect::<Vec<_>>(), [a, c]);
        assert_eq!(
            picking.option_count(),
            3,
            "a picked object is still an option"
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
    fn finishing_gives_the_picked_objects_in_the_order_picked_and_the_rest_in_the_order_shown() {
        let [a, b, c, d] = ids();
        let mut picking = picking(&[(a, false), (b, true), (c, true), (d, true)], 2);
        picking.pick(d);
        picking.pick(b);

        assert_eq!(
            picking.finish().map(PickedGroups::into_groups),
            Ok([(vec![d, b], vec!["verb"]), (vec![a, c], vec!["other"])]),
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
        expected = "an object found among a picking's options should stay there until picked"
    )]
    fn an_object_the_filter_left_out_cannot_be_picked() {
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
        expected = "an object found among a picking's options should stay there until picked"
    )]
    fn an_object_cannot_be_picked_twice() {
        let [a, b] = ids();
        let mut picking = picking(&[(a, true), (b, true)], 2);
        picking.pick(a);

        picking.pick(a);
    }

    #[test]
    fn a_finished_card_pick_names_the_zone_that_holds_its_cards() {
        let [a, b] = ids();
        let mut picking = Picking::Cards(card_picking(&[(a, true), (b, true)], 1));
        picking.pick(b);

        let Ok(PickedObjects::Cards(picked)) = picking.finish() else {
            panic!("a complete card pick finishes with its cards");
        };

        assert_eq!(
            picked.into_groups(),
            (
                PlayerId::new(1),
                CardOriginZone::Deck,
                [
                    (vec![b], vec![CardVerb::Move(CardDestinationZone::Hand)]),
                    (
                        vec![a],
                        vec![CardVerb::Move(CardDestinationZone::DeckBottom)]
                    ),
                ],
            ),
        );
    }

    #[test]
    fn every_character_a_character_pick_covers_is_an_option() {
        let [a, b] = ids();

        let picking = CharacterPicking::new(vec![a, b], 1, AmountBound::Exactly, vec![], vec![]);

        assert_eq!(picking.options().collect::<Vec<_>>(), [a, b]);
        assert_eq!(
            picking.0.covered().collect::<Vec<_>>(),
            [(a, true), (b, true)]
        );
    }

    #[test]
    fn a_finished_character_pick_gives_the_picked_characters_their_verbs_and_the_rest_its_others() {
        let [a, b, c] = ids();
        let mut picking = Picking::Characters(CharacterPicking::new(
            vec![a, b, c],
            1,
            AmountBound::Exactly,
            vec![buff(2)],
            vec![buff(1)],
        ));
        picking.pick(b);

        let Ok(PickedObjects::Characters(picked)) = picking.finish() else {
            panic!("a complete character pick finishes with its characters");
        };

        assert_eq!(
            picked.into_groups(),
            [(vec![b], vec![buff(2)]), (vec![a, c], vec![buff(1)])],
        );
    }
}
