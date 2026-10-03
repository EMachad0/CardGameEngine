//! Seat order: whose turn it is, and who comes next.

use crate::ids::PlayerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TurnOrder {
    active_idx: usize,
    turn_count: u32,
    players: Vec<PlayerId>,
}

impl TurnOrder {
    pub(crate) fn new(players: Vec<PlayerId>) -> Self {
        Self {
            active_idx: 0,
            turn_count: 0,
            players,
        }
    }

    pub(crate) fn get_active_player_id(&self) -> PlayerId {
        self.players[self.active_idx]
    }

    pub(crate) fn end_turn(&mut self) {
        self.active_idx = self.get_next_player_idx();
        self.turn_count += 1;
    }

    pub(crate) fn get_player_after(&self, player_id: PlayerId) -> PlayerId {
        let idx = self
            .players
            .iter()
            .position(|id| *id == player_id)
            .expect("invalid player id");
        self.players[(idx + 1) % self.players.len()]
    }

    pub(crate) fn turn_count(&self) -> u32 {
        self.turn_count
    }

    fn get_next_player_idx(&self) -> usize {
        (self.active_idx + 1) % self.players.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const P0: PlayerId = PlayerId::new(0);
    const P1: PlayerId = PlayerId::new(1);
    const P2: PlayerId = PlayerId::new(2);

    #[test]
    fn seat_zero_goes_first() {
        assert_eq!(
            TurnOrder::new(vec![P1, P0]).get_active_player_id(),
            P1,
            "seat order decides who starts, not the player id"
        );
    }

    #[test]
    fn end_turn_cycles_through_every_seat_and_wraps() {
        let mut order = TurnOrder::new(vec![P0, P1, P2]);
        let mut seen = Vec::new();
        for _ in 0..4 {
            seen.push(order.get_active_player_id());
            order.end_turn();
        }
        assert_eq!(seen, [P0, P1, P2, P0]);
    }

    #[test]
    fn the_player_after_the_last_seat_is_the_first() {
        let order = TurnOrder::new(vec![P1, P0]);
        assert_eq!(order.get_player_after(P1), P0);
        assert_eq!(order.get_player_after(P0), P1);
    }
}
