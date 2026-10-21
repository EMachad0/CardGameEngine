//! What happens as a turn ends and the next one starts.

use crate::game::reporting::Report;
use crate::game::state::Game;
use crate::observer::Observer;

impl Game {
    /// Ends the active player's turn and starts the next player's.
    pub(super) fn end_turn(&mut self, obs: &mut impl Observer) {
        let active_player_id = self.turn_order.get_active_player_id();
        let report = Report::TurnEnded {
            player_id: active_player_id,
        };
        self.report(report, obs);
        self.turn_order.end_turn();
        self.start_turn(obs);
    }

    pub(super) fn start_turn(&mut self, obs: &mut impl Observer) {
        let active_player_id = self.turn_order.get_active_player_id();
        let report = Report::TurnStarted {
            player_id: active_player_id,
        };
        self.report(report, obs);
        self.get_player_mut(active_player_id).mana.refill();
        self.draw(active_player_id, 1, obs);
    }
}
