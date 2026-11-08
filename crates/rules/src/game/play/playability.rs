//! When a card is playable, and which targeting it is played from.

use crate::cards::object::ObjectId;
use crate::game::cards_ref::CardsRef;
use crate::game::mana::Mana;
use crate::game::player_input::awaiting::WasAwaitingTargeting;
use crate::game::stats::cost;
use crate::zones::{InHand, WasInHand};

/// A card its player could play when asked, and their targeting of it if they play it from there.
/// It holds no borrow, so the card may have become unplayable since.
pub(in crate::game) struct WasPlayable {
    card: WasInHand,
    awaiting_targeting: Option<WasAwaitingTargeting>,
}

/// How its player could play a card they could pay for when asked.
pub(in crate::game) enum Playability {
    WithoutTargeting(WasPlayable),
    FromTargeting(WasPlayableFromTargeting),
}

/// A card that declares targets and that its player could pay for when asked. It holds no borrow,
/// so the card may have become unplayable since.
pub(in crate::game) struct WasPlayableFromTargeting {
    card: WasInHand,
}

impl Playability {
    /// `None` unless the mana pays the card's cost now.
    pub(in crate::game) fn new(cards: CardsRef, mana: &Mana, card: InHand) -> Option<Self> {
        if !mana.can_pay(cost(cards, card)) {
            return None;
        }
        let declares_targets = cards.card_def(card.object_id()).targets().next().is_some();
        let card = WasInHand::from(card);
        Some(match declares_targets {
            true => Self::FromTargeting(WasPlayableFromTargeting { card }),
            false => Self::WithoutTargeting(WasPlayable {
                card,
                awaiting_targeting: None,
            }),
        })
    }
}

impl WasPlayable {
    pub(in crate::game) fn object_id(&self) -> ObjectId {
        self.card.object_id()
    }

    pub(super) fn into_parts(self) -> (WasInHand, Option<WasAwaitingTargeting>) {
        (self.card, self.awaiting_targeting)
    }
}

impl WasPlayableFromTargeting {
    pub(in crate::game) fn with_targeting(
        self,
        awaiting_targeting: WasAwaitingTargeting,
    ) -> WasPlayable {
        assert!(
            awaiting_targeting.player_id() == self.card.player_id()
                && awaiting_targeting.object_id() == self.card.object_id(),
            "a card should be paired with the targeting it was found through",
        );
        WasPlayable {
            card: self.card,
            awaiting_targeting: Some(awaiting_targeting),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::binder::sets::base::static_card_definition::{PING, SPARK};
    use crate::cards::binder::store::Binder;
    use crate::cards::object::ObjectBag;
    use crate::game::effect_stack::EffectStack;
    use crate::game::player_input::awaiting::{
        Awaiting, PlayerInput, WasAwaitingNothing, awaiting,
    };
    use crate::history::History;
    use crate::ids::PlayerId;
    use crate::turn::TurnOrder;
    use crate::zones::Zones;

    fn started_targeting(
        input: &mut PlayerInput,
        objects: &ObjectBag,
        zones: &Zones,
        player: PlayerId,
        card: ObjectId,
    ) -> WasAwaitingTargeting {
        let (binder, history) = (Binder::built_in(), History::default());
        let turn_order = TurnOrder::new(vec![player]);
        let cards = CardsRef::new(&binder, objects, zones, &history, &turn_order);
        let effect_stack = EffectStack::default();
        let Awaiting::Nothing(awaits_nothing) = awaiting(cards, input, &effect_stack, player)
        else {
            panic!("the game must await nothing from the player");
        };
        input.start_targeting(WasAwaitingNothing::from(awaits_nothing), card);
        let Awaiting::Targeting(awaits_targeting) = awaiting(cards, input, &effect_stack, player)
        else {
            panic!("the player must be targeting");
        };
        WasAwaitingTargeting::from(awaits_targeting)
    }

    fn playability_with_one_mana(
        objects: &ObjectBag,
        zones: &Zones,
        player: PlayerId,
        card: ObjectId,
    ) -> Option<Playability> {
        let (binder, history) = (Binder::built_in(), History::default());
        let turn_order = TurnOrder::new(vec![player]);
        let cards = CardsRef::new(&binder, objects, zones, &history, &turn_order);
        let mut mana = Mana::default();
        mana.refill();
        Playability::new(cards, &mana, zones.find_in_hand(player, card).unwrap())
    }

    fn playable_from_targeting_without_checks(
        zones: &Zones,
        player: PlayerId,
        card: ObjectId,
    ) -> WasPlayableFromTargeting {
        WasPlayableFromTargeting {
            card: WasInHand::from(zones.find_in_hand(player, card).unwrap()),
        }
    }

    #[test]
    fn a_card_that_declares_targets_is_playable_only_from_its_targeting() {
        let player = PlayerId::new(0);
        let mut bag = ObjectBag::default();
        let hero = bag.insert(SPARK, player);
        let card = bag.insert(PING, player);
        let mut zones = Zones::new([(hero, vec![card])]);
        zones.draw(player);

        assert!(
            matches!(
                playability_with_one_mana(&bag, &zones, player, card),
                Some(Playability::FromTargeting(_)),
            ),
            "Ping declares a target, so it is playable only from its targeting",
        );
    }

    #[test]
    fn a_card_without_targets_is_playable_without_targeting() {
        let player = PlayerId::new(0);
        let mut bag = ObjectBag::default();
        let [hero, card] = std::array::from_fn(|_| bag.insert(SPARK, player));
        let mut zones = Zones::new([(hero, vec![card])]);
        zones.draw(player);

        assert!(
            matches!(
                playability_with_one_mana(&bag, &zones, player, card),
                Some(Playability::WithoutTargeting(_)),
            ),
            "Spark declares no target, so it is playable without a targeting",
        );
    }

    #[test]
    #[should_panic(expected = "a card should be paired with the targeting it was found through")]
    fn a_card_cannot_be_played_from_another_cards_targeting() {
        let player = PlayerId::new(0);
        let mut bag = ObjectBag::default();
        let [hero, card, other] = std::array::from_fn(|_| bag.insert(SPARK, player));
        let mut zones = Zones::new([(hero, vec![card, other])]);
        zones.draw(player);
        let mut input = PlayerInput::default();
        let awaiting_targeting = started_targeting(&mut input, &bag, &zones, player, other);

        playable_from_targeting_without_checks(&zones, player, card)
            .with_targeting(awaiting_targeting);
    }

    #[test]
    #[should_panic(expected = "a card should be paired with the targeting it was found through")]
    fn a_card_cannot_be_played_from_another_players_targeting() {
        let (player, other) = (PlayerId::new(0), PlayerId::new(1));
        let mut bag = ObjectBag::default();
        let [hero, card] = std::array::from_fn(|_| bag.insert(SPARK, player));
        let other_hero = bag.insert(SPARK, other);
        let mut zones = Zones::new([(hero, vec![card]), (other_hero, Vec::new())]);
        zones.draw(player);
        let mut input = PlayerInput::default();
        let awaiting_targeting = started_targeting(&mut input, &bag, &zones, other, card);

        playable_from_targeting_without_checks(&zones, player, card)
            .with_targeting(awaiting_targeting);
    }
}
