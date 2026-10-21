//! Playing a card from its owner's hand.

use crate::cards::object::ObjectId;
use crate::event::Event;
use crate::game::cards_ref::CardsRef;
use crate::game::reporting::Report;
use crate::game::state::Game;
use crate::ids::PlayerId;
use crate::observer::{Observer, Views};

pub(super) struct Play {
    player_id: PlayerId,
    object_id: ObjectId,
}

/// A card its player can play now, with its cost.
pub(super) struct Playable {
    object_id: ObjectId,
    cost: u8,
}

impl Playable {
    pub(super) fn new(object_id: ObjectId, cost: u8) -> Self {
        Self { object_id, cost }
    }

    pub(super) fn object_id(&self) -> ObjectId {
        self.object_id
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
    pub(super) fn play(&mut self, player_id: PlayerId, card: Playable, obs: &mut impl Observer) {
        let Playable { object_id, cost } = card;
        self.get_player_mut(player_id).mana.pay(cost);
        self.zones
            .take_from_hand(player_id, object_id)
            .expect("legal actions guarantee playable cards are in hand");

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
