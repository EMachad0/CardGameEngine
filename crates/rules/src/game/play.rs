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

/// A card its player can play now: where it is in their hand, and the payment of its cost.
pub(super) struct Playable {
    card: HandSlot,
    payment: Payment,
}

impl Playable {
    pub(super) fn new(card: HandSlot, payment: Payment) -> Self {
        Self { card, payment }
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
    /// Closes the player's targeting first, if they played the card from there.
    pub(super) fn play(
        &mut self,
        player_id: PlayerId,
        card: Playable,
        targeting: Option<TargetingSlot>,
        obs: &mut impl Observer,
    ) {
        let Playable { card, payment } = card;
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
