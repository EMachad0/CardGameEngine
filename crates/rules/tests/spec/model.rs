//! An independent model of the rules: printed data, costs in hand, and both
//! boards. The playout checks compare the core against it, so a wrong value in
//! the core can't check itself.

use rules::DefId;
use rules::static_card_definition::{
    BARRACKS, BLAST, BOLT, CAPTAIN, FORAGE, GIANT, RECRUIT, SPARK, SQUIRE, WILD_BOLT,
};

/// Attack and health.
pub(crate) type Stats = (i32, i32);

const PRINTED: [(DefId, u8, Option<Stats>); 10] = [
    (SPARK, 1, None),
    (BOLT, 2, None),
    (WILD_BOLT, 1, None),
    (FORAGE, 1, None),
    (BLAST, 3, None),
    (RECRUIT, 2, Some((2, 2))),
    (CAPTAIN, 3, Some((1, 1))),
    (GIANT, 8, Some((5, 5))),
    (BARRACKS, 2, None),
    (SQUIRE, 1, Some((1, 1))),
];

fn printed(def: DefId) -> (u8, Option<Stats>) {
    let (_, cost, stats) = PRINTED
        .iter()
        .find(|(d, _, _)| *d == def)
        .unwrap_or_else(|| panic!("{def:?} is not in the model"));
    (*cost, *stats)
}

pub(crate) fn is_spell(def: DefId) -> bool {
    printed(def).1.is_none()
}

/// A card's cost in hand, given how many spells its holder has cast this game.
pub(crate) fn expected_cost(def: DefId, spells_cast: u8) -> u8 {
    let cost = printed(def).0;
    if def == GIANT {
        cost.saturating_sub(spells_cast)
    } else {
        cost
    }
}

/// Each board's minions, left to right, with the damage marked on each. A board is
/// named by its owner's index in `Game::players`.
#[derive(Default)]
pub(crate) struct BoardModel {
    boards: [Vec<(DefId, i32)>; 2],
}

impl BoardModel {
    /// `owner`'s minions, left to right, with their current attack and health.
    pub(crate) fn minions(&self, owner: usize) -> Vec<(DefId, Stats)> {
        let board = &self.boards[owner];
        (0..board.len())
            .map(|i| (board[i].0, Self::stats(board, i)))
            .collect()
    }

    fn stats(board: &[(DefId, i32)], i: usize) -> Stats {
        let (def, damage) = board[i];
        let (attack, health) = printed(def).1.expect("only minions are on a board");
        let captains = board
            .iter()
            .enumerate()
            .filter(|&(j, &(d, _))| j != i && d == CAPTAIN)
            .count() as i32;
        (attack + captains, health + captains - damage)
    }

    pub(crate) fn played(&mut self, owner: usize, def: DefId) {
        if !is_spell(def) {
            self.boards[owner].push((def, 0));
        }
        if def == BARRACKS {
            self.boards[owner].push((SQUIRE, 0));
        }
        if def == BLAST {
            for minion in self.boards.iter_mut().flatten() {
                minion.1 += 2;
            }
        }
    }

    /// Removes every minion at 0 health or less, all at once, until none is left.
    pub(crate) fn check(&mut self) {
        loop {
            let dead: Vec<Vec<bool>> = self
                .boards
                .iter()
                .map(|board| {
                    (0..board.len())
                        .map(|i| Self::stats(board, i).1 <= 0)
                        .collect()
                })
                .collect();
            if dead.iter().flatten().all(|d| !d) {
                return;
            }
            for (board, dead) in self.boards.iter_mut().zip(&dead) {
                let mut flags = dead.iter();
                board.retain(|_| !flags.next().unwrap());
            }
        }
    }
}
