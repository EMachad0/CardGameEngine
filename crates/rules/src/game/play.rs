//! Playing a card from its owner's hand.

use crate::cards::object::ObjectId;
use crate::event::Event;
use crate::game::cards_ref::CardsRef;
use crate::game::mana::Mana;
use crate::game::player_input::WasAwaitingTargeting;
use crate::game::reporting::Report;
use crate::game::state::Game;
use crate::game::stats::cost;
use crate::ids::PlayerId;
use crate::observer::{Observer, Views};
use crate::zones::{InHand, WasInHand};

pub(super) struct Play {
    player_id: PlayerId,
    object_id: ObjectId,
}

/// A card its player could play when asked, and their targeting of it if they play it from there.
/// It holds no borrow, so the card may have become unplayable since.
pub(super) struct WasPlayable {
    card: WasInHand,
    awaiting_targeting: Option<WasAwaitingTargeting>,
}

impl WasPlayable {
    /// `None` unless the mana pays the card's cost now.
    pub(super) fn new(cards: CardsRef, mana: &Mana, card: InHand) -> Option<Self> {
        mana.can_pay(cost(cards, card)).then(|| Self {
            card: WasInHand::from(card),
            awaiting_targeting: None,
        })
    }

    pub(super) fn with_targeting(self, awaiting_targeting: WasAwaitingTargeting) -> Self {
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

    pub(super) fn object_id(&self) -> ObjectId {
        self.card.object_id()
    }
}

impl From<Play> for Event {
    fn from(play: Play) -> Self {
        Event::Played {
            player_id: play.player_id,
            object_id: play.object_id,
        }
    }
}

impl Game {
    /// If the card is played from the player's targeting, closes it before reporting the play.
    pub(super) fn play(&mut self, playable: WasPlayable, obs: &mut impl Observer) {
        let WasPlayable {
            card,
            awaiting_targeting,
        } = playable;
        let player_id = card.player_id();
        let cards = CardsRef::new(self);
        let mana_cost = cost(cards, cards.zones.still_in_hand(&card));
        self.get_player_mut(player_id).mana.pay(mana_cost);
        let targeting = awaiting_targeting
            .map(|awaiting_targeting| self.player_input.commit_targeting(awaiting_targeting));
        let object_id = self.zones.take_from_hand(card);

        let play_effects = CardsRef::new(self)
            .card_def(object_id)
            .bind_play_effects(|target_id| {
                targeting
                    .as_ref()
                    .expect("a card that declares targets should be played from its targeting")
                    .targeted_for(target_id)
                    .collect()
            });
        if !play_effects.is_empty() {
            self.effect_stack.push(object_id, play_effects);
        }

        let report = Report::Played(Play {
            player_id,
            object_id,
        });
        self.report(report, obs);

        obs.checkpoint(Views::new(self));
        if CardsRef::new(self).card_def(object_id).has_board_presence() {
            self.summon(player_id, object_id, obs);
            obs.checkpoint(Views::new(self));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::binder::sets::base::static_card_definition::SPARK;
    use crate::cards::object::ObjectBag;
    use crate::game::player_input::{Awaiting, PlayerInput, WasAwaitingNothing};
    use crate::zones::Zones;

    fn started_targeting(
        input: &mut PlayerInput,
        player: PlayerId,
        card: ObjectId,
    ) -> WasAwaitingTargeting {
        let Awaiting::Nothing(awaits_nothing) = input.awaiting(player) else {
            panic!("the game must await nothing from the player");
        };
        input.start_targeting(WasAwaitingNothing::from(awaits_nothing), card);
        let Awaiting::Targeting(awaits_targeting) = input.awaiting(player) else {
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
        let awaiting_targeting = started_targeting(&mut input, player, other);

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
        let awaiting_targeting = started_targeting(&mut input, other, card);

        playable_without_mana_check(&zones, player, card).with_targeting(awaiting_targeting);
    }
}
