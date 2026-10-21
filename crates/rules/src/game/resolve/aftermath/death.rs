use crate::cards::object::ObjectId;
use crate::event::Event;
use crate::game::cards::Cards;
use crate::game::reporting::Report;
use crate::game::state::Game;
use crate::game::stats::character_health;
use crate::game::triggering::{MatchedTrigger, death_rattles};
use crate::ids::PlayerId;
use crate::observer::{Observer, Views};
use crate::zones::Zones;

pub(in crate::game) struct Death {
    object_id: ObjectId,
}

impl From<Death> for Event {
    fn from(death: Death) -> Self {
        Event::Died {
            object_id: death.object_id,
        }
    }
}

impl Game {
    pub(super) fn remove_dead(&mut self, obs: &mut impl Observer) {
        loop {
            let dead_minions = find_dead(Cards::new(self), &self.players());
            let minion_died = !dead_minions.is_empty();

            let dead_players = self
                .standings
                .playing()
                .filter(|&player_id| self.hero_health(player_id) <= 0)
                .collect::<Vec<_>>();
            let player_died = !dead_players.is_empty();
            let dead_heroes = dead_players.iter().map(|p| self.hero_id(*p));

            let mut death_pass = dead_minions
                .into_iter()
                .chain(dead_heroes)
                .collect::<Vec<_>>();
            death_pass.sort_by_key(|&o| self.zones.death_order(o));
            let deaths = death_pass
                .into_iter()
                .map(|object_id| Death { object_id })
                .collect();
            self.kill(deaths, obs);

            self.standings.eliminate(&dead_players);

            if player_died || minion_died {
                obs.checkpoint(Views::new(self));
            } else {
                break;
            }
        }
    }

    fn kill(&mut self, deaths: Vec<Death>, obs: &mut impl Observer) {
        for death in deaths.iter() {
            for MatchedTrigger { source, effects } in
                death_rattles(Cards::new(self), death.object_id)
            {
                self.effect_stack.push(source, effects);
            }
        }

        for death in deaths.into_iter() {
            let object_id = death.object_id;
            let player_id = self.objects.get(object_id).player_id;
            let card_def = Cards::new(self).card_def(object_id);
            if card_def.leaves_zone_on_death() {
                let to_graveyard = card_def.goes_to_graveyard();
                destroy(&mut self.zones, player_id, object_id, to_graveyard);
            }
            self.report(Report::Died(death), obs);
        }
    }
}

fn find_dead(cards: Cards<'_>, players: &[PlayerId]) -> Vec<ObjectId> {
    let mut death_pass = Vec::new();
    for &player_id in players {
        for card in cards.zones.board_cards(player_id) {
            if character_health(cards, card.into()) <= 0 {
                death_pass.push(card.object_id());
            }
        }
    }
    death_pass
}

fn destroy(zones: &mut Zones, player_id: PlayerId, object_id: ObjectId, to_graveyard: bool) {
    #[expect(
        clippy::disallowed_methods,
        reason = "the one removal from the board, made as the state check removes the dead"
    )]
    zones
        .leave_board(player_id, object_id)
        .expect("object not in board");

    if to_graveyard {
        entomb(zones, player_id, object_id);
    }
}

fn entomb(zones: &mut Zones, player_id: PlayerId, object_id: ObjectId) {
    #[expect(
        clippy::disallowed_methods,
        reason = "the one entry into a graveyard, made as the state check removes the dead"
    )]
    zones.entomb(player_id, object_id);
}
