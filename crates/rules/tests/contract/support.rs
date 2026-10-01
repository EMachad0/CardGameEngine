//! Helpers shared by the contract tests: action builders, turn helpers and the
//! legality assertions.

use std::collections::BTreeSet;

use rules::{Action, DefId, Game, Illegal, ObjectId, PlayerId};

pub(crate) const P0: PlayerId = PlayerId::new(0);
pub(crate) const P1: PlayerId = PlayerId::new(1);
pub(crate) const PLAYERS: [PlayerId; 2] = [P0, P1];

pub(crate) fn play(card: ObjectId) -> Action {
    Action::Play { card }
}

pub(crate) fn pick(card: ObjectId) -> Action {
    Action::Pick { card }
}

pub(crate) fn other(p: PlayerId) -> PlayerId {
    if p == P0 { P1 } else { P0 }
}

/// The first card in `p`'s hand with definition `def`.
pub(crate) fn in_hand(game: &Game, p: PlayerId, def: DefId) -> ObjectId {
    game.hand(p)
        .into_iter()
        .find(|&id| game.def(id) == Some(def))
        .unwrap_or_else(|| panic!("no {def:?} in {p:?}'s hand"))
}

/// Every id in a zone: hand, deck, revealed and board, for each player.
pub(crate) fn zone_ids(game: &Game) -> Vec<ObjectId> {
    PLAYERS
        .iter()
        .flat_map(|&p| [game.hand(p), game.deck(p), game.revealed(p), game.board(p)].concat())
        .collect()
}

/// Ends the current turn.
pub(crate) fn end_turn(game: &mut Game) {
    let p = PLAYERS
        .into_iter()
        .find(|&p| game.legal_actions(p).contains(&Action::EndTurn))
        .expect("someone can end their turn");
    game.apply(p, Action::EndTurn).unwrap();
}

/// Ends turns until the game is over. Panics after 60 turns instead of looping forever.
pub(crate) fn end_turns_until_over(game: &mut Game) {
    for _ in 0..60 {
        if game.outcome().is_some() {
            return;
        }
        end_turn(game);
    }
    panic!("no outcome after 60 turns");
}

/// Asserts that `legal_actions(p)` holds the same actions as `expected`, in any order.
pub(crate) fn assert_actions(game: &Game, p: PlayerId, expected: &[Action]) {
    let actual = game.legal_actions(p);
    let missing: Vec<&Action> = expected.iter().filter(|a| !actual.contains(a)).collect();
    let extra: Vec<&Action> = actual.iter().filter(|a| !expected.contains(a)).collect();
    assert!(
        missing.is_empty() && extra.is_empty() && actual.len() == expected.len(),
        "legal_actions({p:?}) = {actual:?}\n  expected {expected:?}\n  missing {missing:?}\n  extra {extra:?}"
    );
}

/// P for unlisted actions. Each unlisted candidate, for either player, returns
/// the exact `Illegal` and changes nothing.
pub(crate) fn assert_unlisted_rejected(game: &Game) {
    assert_unlisted_rejected_with(game, &BTreeSet::new());
}

/// Like `assert_unlisted_rejected`, and also tries `Play` and `Pick` on every id in `extra`.
pub(crate) fn assert_unlisted_rejected_with(game: &Game, extra: &BTreeSet<ObjectId>) {
    let ids: BTreeSet<ObjectId> = zone_ids(game)
        .into_iter()
        .chain(extra.iter().copied())
        .collect();
    let mut candidates = vec![Action::EndTurn];
    for id in ids {
        candidates.push(play(id));
        candidates.push(pick(id));
    }

    for p in PLAYERS {
        let legal = game.legal_actions(p);
        for a in &candidates {
            if legal.contains(a) {
                continue;
            }
            let mut g = game.clone();
            assert_eq!(
                g.apply(p, a.clone()),
                Err(Illegal {
                    player: p,
                    action: a.clone()
                }),
                "unlisted {a:?} for {p:?} was not rejected"
            );
            assert_eq!(&g, game, "rejected {a:?} for {p:?} changed the game");
        }
    }
}
