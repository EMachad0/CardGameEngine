use crate::{
    cards::object::ObjectId,
    event::Event,
    game::{Game, reporting::Report, triggering::MatchedTrigger},
    ids::PlayerId,
    observer::{Observer, Views},
    outcome::Outcome,
};

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

pub(in crate::game) struct Ending {
    outcome: Outcome,
}

impl From<Ending> for Event {
    fn from(ending: Ending) -> Self {
        Event::GameEnded {
            outcome: ending.outcome,
        }
    }
}

impl Game {
    pub(in crate::game::resolve) fn check_state(&mut self, obs: &mut impl Observer) {
        self.remove_dead(obs);
        self.decide_outcome(obs);
    }

    fn find_dead(&self) -> Vec<ObjectId> {
        let mut death_pass = Vec::new();
        for player_id in self.players().into_iter() {
            for object_id in self.player_board(player_id).iter().copied() {
                let health = self
                    .health(object_id)
                    .expect("board objects always have health");
                if health <= 0 {
                    death_pass.push(object_id);
                }
            }
        }
        death_pass
    }

    fn remove_dead(&mut self, obs: &mut impl Observer) {
        loop {
            let dead_minions = self.find_dead();
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
            for MatchedTrigger { source, effects } in self.death_rattles(death.object_id) {
                self.effect_stack.push(source, effects);
            }
        }

        for death in deaths.into_iter() {
            let object_id = death.object_id;
            let object = self.objects.get(object_id);
            if self.card_def(object_id).leaves_zone_on_death() {
                self.destroy(object.player_id, object_id);
            }
            self.report(Report::Died(death), obs);
        }
    }

    fn destroy(&mut self, player_id: PlayerId, object_id: ObjectId) {
        #[expect(
            clippy::disallowed_methods,
            reason = "the one removal from the board, made as the state check removes the dead"
        )]
        self.zones
            .leave_board(player_id, object_id)
            .expect("object not in board");

        if self.card_def(object_id).goes_to_graveyard() {
            self.entomb(player_id, object_id);
        }
    }

    fn entomb(&mut self, player_id: PlayerId, object_id: ObjectId) {
        #[expect(
            clippy::disallowed_methods,
            reason = "the one entry into a graveyard, made as the state check removes the dead"
        )]
        self.zones.entomb(player_id, object_id);
    }

    fn decide_outcome(&mut self, obs: &mut impl Observer) {
        if let Some(outcome) = self.standings.decide() {
            self.report(Report::GameEnded(Ending { outcome }), obs);
            obs.checkpoint(Views::new(self));
        }
    }
}
