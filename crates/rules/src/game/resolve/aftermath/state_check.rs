use crate::{
    game::{Game, reporting::Report},
    observer::{Observer, Views},
};

impl Game {
    pub(in crate::game::resolve) fn check_state(&mut self, obs: &mut impl Observer) {
        self.remove_dead(obs);
        if let Some(ending) = self.standings.decide() {
            self.report(Report::GameEnded(ending), obs);
            obs.checkpoint(Views::new(self));
        }
    }
}
