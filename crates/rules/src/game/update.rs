use crate::{Game, ObjectId, Outcome};

impl Game {
    pub(crate) fn update(&mut self) {
        self.update_deaths();
        self.update_outcome();
    }

    fn find_dead(&self) -> Option<ObjectId> {
        for player_id in self.players().into_iter() {
            for object_id in self.board(player_id).iter().copied() {
                if let Ok(Some(health)) = self.health(object_id)
                    && health <= 0
                {
                    return Some(object_id);
                }
            }
        }
        None
    }

    fn update_deaths(&mut self) {
        while let Some(object_id) = self.find_dead() {
            self.kill(self.objects.get(object_id).unwrap().player_id, object_id);
        }
    }

    fn update_outcome(&mut self) {
        let mut alive = self
            .players()
            .into_iter()
            .filter(|&player_id| self.hero_health(player_id) > 0);
        match (alive.next(), alive.next()) {
            (None, None) => self.outcome = Some(Outcome::Draw),
            (Some(player_id), None) => self.outcome = Some(Outcome::Won(player_id)),
            (_, _) => {}
        }
    }
}
