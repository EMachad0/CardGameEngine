use crate::{Event, Game, ObjectId, Observer, Outcome, Views};

impl Game {
    pub(crate) fn update(&mut self, obs: &mut impl Observer) {
        self.update_deaths(obs);
        self.update_outcome(obs);
        obs.checkpoint(Views::new(self));
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

    fn update_deaths(&mut self, obs: &mut impl Observer) {
        while let Some(object_id) = self.find_dead() {
            self.kill(self.objects.get(object_id).player_id, object_id, obs);
            obs.checkpoint(Views::new(self));
        }
    }

    fn update_outcome(&mut self, obs: &mut impl Observer) {
        if self.outcome.is_some() {
            return;
        }

        let mut alive = self
            .players()
            .into_iter()
            .filter(|&player_id| self.hero_health(player_id) > 0);
        let outcome = match (alive.next(), alive.next()) {
            (None, None) => Some(Outcome::Draw),
            (Some(player_id), None) => Some(Outcome::Won(player_id)),
            (_, _) => None,
        };
        if let Some(outcome) = outcome {
            obs.event(&Event::GameEnded { outcome });
        }
        self.outcome = outcome;
    }
}
