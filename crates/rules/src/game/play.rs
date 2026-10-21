//! Playing a card from its owner's hand.

use crate::cards::object::ObjectId;
use crate::event::Event;
use crate::game::cards::Cards;
use crate::game::reporting::Report;
use crate::game::state::Game;
use crate::ids::PlayerId;
use crate::observer::{Observer, Views};

pub(super) struct Play {
    player_id: PlayerId,
    object_id: ObjectId,
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
    pub(super) fn play(
        &mut self,
        player_id: PlayerId,
        object_id: ObjectId,
        obs: &mut impl Observer,
    ) {
        let mana_cost = self
            .mana_cost(object_id)
            .expect("legal actions guarantee playable cards are in hand");
        self.get_player_mut(player_id).mana.pay(mana_cost);
        self.zones
            .take_from_hand(player_id, object_id)
            .expect("legal actions guarantee playable cards are in hand");

        let play_effects = Cards::new(self).play_effects(object_id);
        if !play_effects.is_empty() {
            self.effect_stack.push(object_id, play_effects.to_vec());
        }

        let report = Report::Played(Play {
            player_id,
            object_id,
        });
        self.report(report, obs);

        obs.checkpoint(Views::new(self));
        if Cards::new(self).card_def(object_id).has_board_presence() {
            self.summon(player_id, object_id, obs);
            obs.checkpoint(Views::new(self));
        }
    }
}
