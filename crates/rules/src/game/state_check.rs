use crate::{
    cards::object::ObjectId,
    game::{Game, reporting::Report},
    observer::{Observer, Views},
    outcome::Outcome,
};

/// A character the state check found dead.
pub(super) struct Death {
    object_id: ObjectId,
}

impl Death {
    pub(super) fn object_id(&self) -> ObjectId {
        self.object_id
    }
}

/// The outcome the state check decided.
pub(super) struct Ending {
    outcome: Outcome,
}

impl Ending {
    pub(super) fn outcome(&self) -> Outcome {
        self.outcome
    }
}

impl Game {
    pub(crate) fn check_state(&mut self, obs: &mut impl Observer) {
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
                .players()
                .into_iter()
                .filter(|&player_id| self.playing(player_id))
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

            for player_id in dead_players.into_iter() {
                let player = self.get_player_mut(player_id);
                player.playing = false;
            }

            if player_died || minion_died {
                obs.checkpoint(Views::new(self));
            } else {
                break;
            }
        }
    }

    fn decide_outcome(&mut self, obs: &mut impl Observer) {
        if self.outcome.is_some() {
            return;
        }

        let mut alive = self
            .players()
            .into_iter()
            .filter(|&player_id| self.playing(player_id))
            .filter(|&player_id| self.hero_health(player_id) > 0);
        let outcome = match (alive.next(), alive.next()) {
            (None, None) => Some(Outcome::Draw),
            (Some(player_id), None) => Some(Outcome::Won(player_id)),
            (_, _) => None,
        };

        self.outcome = outcome;
        if let Some(outcome) = outcome {
            self.report(Report::GameEnded(Ending { outcome }), obs);
            obs.checkpoint(Views::new(self));
        }
    }
}
