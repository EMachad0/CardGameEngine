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

impl WasPlayable {
    /// `None` unless the mana pays the card's cost now.
    pub(in crate::game) fn new(cards: CardsRef, mana: &Mana, card: InHand) -> Option<Self> {
        mana.can_pay(cost(cards, card)).then(|| Self {
            card: WasInHand::from(card),
            awaiting_targeting: None,
        })
    }

    pub(in crate::game) fn with_targeting(self, awaiting_targeting: WasAwaitingTargeting) -> Self {
        assert!(
            awaiting_targeting.player_id() == self.card.player_id()
                && awaiting_targeting.object_id() == self.card.object_id(),
            "a card should be paired with the targeting it was found through",
        );
        Self {
            awaiting_targeting: Some(awaiting_targeting),
            ..self
        }
    }

    pub(in crate::game) fn object_id(&self) -> ObjectId {
        self.card.object_id()
    }

    pub(super) fn into_parts(self) -> (WasInHand, Option<WasAwaitingTargeting>) {
        (self.card, self.awaiting_targeting)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::binder::sets::base::static_card_definition::SPARK;
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

    fn playable_without_mana_check(zones: &Zones, player: PlayerId, card: ObjectId) -> WasPlayable {
        WasPlayable {
            card: WasInHand::from(zones.find_in_hand(player, card).unwrap()),
            awaiting_targeting: None,
        }
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

        playable_without_mana_check(&zones, player, card).with_targeting(awaiting_targeting);
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

        playable_without_mana_check(&zones, player, card).with_targeting(awaiting_targeting);
    }
}
