use crate::{Event, Game, ObjectId, Observer, Outcome, Views};

impl Game {
    pub(crate) fn check_state(&mut self, obs: &mut impl Observer) {
        self.remove_dead(obs);
        self.decide_outcome(obs);
    }

    fn find_dead(&self) -> Option<ObjectId> {
        for player_id in self.players().into_iter() {
            for object_id in self.board(player_id).iter().copied() {
                let health = self
                    .health(object_id)
                    .expect("board objects always have health");
                if health <= 0 {
                    return Some(object_id);
                }
            }
        }
        None
    }

    fn remove_dead(&mut self, obs: &mut impl Observer) {
        while let Some(object_id) = self.find_dead() {
            self.kill(self.objects.get(object_id).player_id, object_id, obs);
            obs.checkpoint(Views::new(self));
        }

        let dead_players = self
            .players()
            .into_iter()
            .filter(|&player_id| self.playing(player_id))
            .filter(|&player_id| self.hero_health(player_id) <= 0)
            .collect::<Vec<_>>();
        let player_died = !dead_players.is_empty();
        for player_id in dead_players.into_iter() {
            self.kill(player_id, self.hero_id(player_id), obs);
            // Hero deathrattle may update hero
            if self.hero_health(player_id) <= 0 {
                let player = self.get_player_mut(player_id);
                player.playing = false;
            }
        }
        if player_died {
            obs.checkpoint(Views::new(self));
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
            obs.event(&Event::GameEnded { outcome });
            obs.checkpoint(Views::new(self));
        }
    }
}
