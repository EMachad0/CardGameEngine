//! Who is still playing, and the outcome once the game is over. The game ends once, when at most
//! one player is left.

use crate::{ids::PlayerId, outcome::Outcome};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::game) struct Standings {
    playing: Vec<PlayerId>,
    outcome: Option<Outcome>,
}

impl Standings {
    /// Takes the players in seat order, all playing.
    pub(in crate::game) fn new(players: Vec<PlayerId>) -> Self {
        Self {
            playing: players,
            outcome: None,
        }
    }

    pub(in crate::game) fn outcome(&self) -> Option<Outcome> {
        self.outcome
    }

    /// In seat order.
    pub(super) fn playing(&self) -> impl Iterator<Item = PlayerId> {
        self.playing.iter().copied()
    }

    pub(super) fn eliminate(&mut self, players: &[PlayerId]) {
        self.playing
            .retain(|player_id| !players.contains(player_id));
    }

    /// Ends the game if at most one player is left and it hasn't ended yet. Returns the outcome
    /// only when this call decided it.
    pub(super) fn decide(&mut self) -> Option<Outcome> {
        if self.outcome.is_some() {
            return None;
        }
        self.outcome = match self.playing.as_slice() {
            [] => Some(Outcome::Draw),
            [player_id] => Some(Outcome::Won(*player_id)),
            _ => None,
        };
        self.outcome
    }
}

#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    reason = "each test starts the standings it tests"
)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    fn players<const N: usize>() -> [PlayerId; N] {
        std::array::from_fn(PlayerId::new)
    }

    #[test]
    fn eliminated_players_stop_playing_and_the_rest_keep_seat_order() {
        let [a, b, c] = players();
        let mut standings = Standings::new(vec![a, b, c]);

        standings.eliminate(&[b]);

        assert_eq!(standings.playing().collect::<Vec<_>>(), [a, c]);
    }

    #[test]
    fn the_last_player_left_wins() {
        let [a, b] = players();
        let mut standings = Standings::new(vec![a, b]);
        standings.eliminate(&[a]);

        assert_eq!(standings.decide(), Some(Outcome::Won(b)));
        assert_eq!(standings.outcome(), Some(Outcome::Won(b)));
    }

    #[test]
    fn no_player_left_is_a_draw() {
        let [a, b] = players();
        let mut standings = Standings::new(vec![a, b]);
        standings.eliminate(&[a, b]);

        assert_eq!(standings.decide(), Some(Outcome::Draw));
    }

    #[test]
    fn two_players_left_decide_nothing() {
        let [a, b, c] = players();
        let mut standings = Standings::new(vec![a, b, c]);
        standings.eliminate(&[a]);

        assert_eq!(standings.decide(), None);
        assert_eq!(standings.outcome(), None);
    }

    #[test]
    fn a_decided_outcome_is_not_decided_again() {
        let [a, b] = players();
        let mut standings = Standings::new(vec![a, b]);
        standings.eliminate(&[a]);
        standings.decide();
        standings.eliminate(&[b]);

        assert_eq!(standings.decide(), None, "the game ends once");
        assert_eq!(standings.outcome(), Some(Outcome::Won(b)));
    }
}
