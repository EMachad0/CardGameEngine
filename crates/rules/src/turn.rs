//! Seat order: whose turn it is, and who comes next.

use crate::ids::PlayerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TurnOrder {
    current_idx: usize,
    players: Vec<PlayerId>,
}

impl TurnOrder {
    pub(crate) fn new(player_count: usize) -> Self {
        let players = (0..player_count).map(PlayerId::new).collect();
        Self {
            current_idx: 0,
            players,
        }
    }

    pub(crate) fn get_current_player_id(&self) -> PlayerId {
        self.players[self.current_idx]
    }

    pub(crate) fn end_turn(&mut self) {
        self.current_idx = self.get_next_player_idx();
    }

    fn get_next_player_idx(&self) -> usize {
        (self.current_idx + 1) % self.players.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seat_zero_goes_first() {
        assert_eq!(TurnOrder::new(2).get_current_player_id(), PlayerId::new(0));
    }

    #[test]
    fn end_turn_cycles_through_every_seat_and_wraps() {
        let mut order = TurnOrder::new(3);
        let mut seen = Vec::new();
        for _ in 0..4 {
            seen.push(order.get_current_player_id().idx());
            order.end_turn();
        }
        assert_eq!(seen, [0, 1, 2, 0]);
    }
}
