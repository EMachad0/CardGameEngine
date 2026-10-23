//! The players inside `Game`, each found by their ID, and each one's resources.

use crate::game::mana::Mana;
use crate::ids::PlayerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Players {
    /// Each player at the index of their ID.
    players: Vec<Player>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Player {
    id: PlayerId,
    pub(super) mana: Mana,
}

impl Players {
    /// `count` players, with IDs from 0.
    pub(super) fn new(count: usize) -> Self {
        Self {
            players: (0..count)
                .map(|idx| Player::new(PlayerId::new(idx)))
                .collect(),
        }
    }

    pub(super) fn get(&self, player_id: PlayerId) -> &Player {
        &self.players[player_id.idx()]
    }

    pub(super) fn get_mut(&mut self, player_id: PlayerId) -> &mut Player {
        &mut self.players[player_id.idx()]
    }

    /// By ID.
    pub(super) fn iter(&self) -> impl Iterator<Item = &Player> {
        self.players.iter()
    }
}

impl Player {
    fn new(id: PlayerId) -> Self {
        Self {
            id,
            mana: Mana::default(),
        }
    }

    pub(super) fn id(&self) -> PlayerId {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn new_players_come_by_id_from_zero() {
        let players = Players::new(3);

        assert_eq!(
            players.iter().map(Player::id).collect::<Vec<_>>(),
            [PlayerId::new(0), PlayerId::new(1), PlayerId::new(2)],
        );
    }

    #[test]
    fn a_player_is_found_by_their_id() {
        let players = Players::new(3);

        assert_eq!(players.get(PlayerId::new(2)).id(), PlayerId::new(2));
    }

    #[test]
    fn a_write_through_one_player_leaves_the_others_alone() {
        let mut players = Players::new(2);

        players.get_mut(PlayerId::new(1)).mana.refill();

        assert_eq!(
            players
                .iter()
                .map(|player| player.mana.max())
                .collect::<Vec<_>>(),
            [0, 1],
        );
    }
}
