//! Playing a card from its owner's hand.

use crate::cards::object::ObjectId;
use crate::event::Event;
use crate::game::cards_ref::CardsRef;
use crate::game::mana::Payment;
use crate::game::player_input::TargetingSlot;
use crate::game::reporting::Report;
use crate::game::state::Game;
use crate::ids::PlayerId;
use crate::observer::{Observer, Views};
use crate::zones::HandSlot;

pub(super) struct Play {
    player_id: PlayerId,
    object_id: ObjectId,
}

/// A card its player can play now: where it is in their hand, the payment of its cost, and their
/// targeting of it if they play it from there.
pub(super) struct Playable {
    card: HandSlot,
    payment: Payment,
    targeting: Option<TargetingSlot>,
}

impl Playable {
    pub(super) fn new(card: HandSlot, payment: Payment) -> Self {
        Self {
            card,
            payment,
            targeting: None,
        }
    }

    pub(super) fn with_targeting(self, targeting: TargetingSlot) -> Self {
        assert!(
            targeting.object_id() == self.card.object_id(),
            "a card is played from its own targeting",
        );
        Self {
            targeting: Some(targeting),
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
    /// Closes the player's targeting first, if they play the card from there.
    pub(super) fn play(&mut self, playable: Playable, obs: &mut impl Observer) {
        let Playable {
            card,
            payment,
            targeting,
        } = playable;
        let player_id = card.player_id();
        if let Some(targeting) = targeting {
            let target_ids = CardsRef::new(self)
                .card_def(card.object_id())
                .targets
                .iter()
                .map(|t| t.id)
                .collect::<Vec<_>>();
            self.player_input.commit_targeting(targeting, &target_ids);
        }
        self.get_player_mut(player_id).mana.pay(payment);
        let object_id = self.zones.take_from_hand(card);

        let play_effects = CardsRef::new(self).play_effects(object_id);
        if !play_effects.is_empty() {
            self.effect_stack.push(object_id, play_effects.to_vec());
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
    use crate::game::mana::Mana;
    use crate::game::player_input::{Awaiting, PlayerInput};
    use crate::zones::Zones;

    #[test]
    #[should_panic(expected = "a card is played from its own targeting")]
    fn a_card_cannot_be_played_from_another_cards_targeting() {
        let player = PlayerId::new(0);
        let mut bag = ObjectBag::default();
        let [hero, card, other] = std::array::from_fn(|_| bag.insert(SPARK, player));
        let mut zones = Zones::new([(hero, vec![card, other])]);
        zones.draw(player);
        let mut input = PlayerInput::default();
        input.start_targeting(input.nothing_awaited().unwrap(), player, other);
        let Some(Awaiting::Targets(targeting)) = input.awaiting(player) else {
            panic!("the player must be targeting");
        };
        let found = zones.find_in_hand(player, card).unwrap().slot();
        let playable = Playable::new(found, Mana::default().payment(0).unwrap());

        playable.with_targeting(targeting.slot());
    }
}
