//! Helpers shared by the spec tests: action builders, card lookups, turn helpers and the
//! legality assertions.

use std::collections::BTreeSet;

use rules::cards::BOLT;
use rules::{Action, ApplyError, DefId, Game, IllegalAction, ObjectId, PlayerId};

pub(crate) const P0: PlayerId = PlayerId::new(0);
pub(crate) const P1: PlayerId = PlayerId::new(1);
pub(crate) const PLAYERS: [PlayerId; 2] = [P0, P1];

pub(crate) fn play(object_id: ObjectId) -> Action {
    Action::Play { object_id }
}

pub(crate) fn pick(object_id: ObjectId) -> Action {
    Action::Pick { object_id }
}

pub(crate) fn other(p: PlayerId) -> PlayerId {
    if p == P0 { P1 } else { P0 }
}

/// `top`, then 20 Bolts as filler.
pub(crate) fn deck_with_top(top: &[DefId]) -> Vec<DefId> {
    let mut deck = top.to_vec();
    deck.extend(vec![BOLT; 20]);
    deck
}

pub(crate) fn defs(game: &Game, ids: &[ObjectId]) -> Vec<DefId> {
    ids.iter()
        .map(|&id| {
            game.def_id(id)
                .unwrap_or_else(|e| panic!("def_id({id:?}): {e:?}"))
        })
        .collect()
}

pub(crate) fn hand_defs(game: &Game, p: PlayerId) -> Vec<DefId> {
    defs(game, game.hand(p))
}

pub(crate) fn deck_defs(game: &Game, p: PlayerId) -> Vec<DefId> {
    defs(game, &game.deck(p))
}

pub(crate) fn revealed_defs(game: &Game, p: PlayerId) -> Vec<DefId> {
    defs(game, game.revealed(p))
}

pub(crate) fn board_defs(game: &Game, p: PlayerId) -> Vec<DefId> {
    defs(game, game.board(p))
}

pub(crate) fn has_in_hand(game: &Game, p: PlayerId, def: DefId) -> bool {
    game.hand(p).iter().any(|&id| game.def_id(id) == Ok(def))
}

/// The first card in `p`'s hand with definition `def`.
pub(crate) fn in_hand(game: &Game, p: PlayerId, def: DefId) -> ObjectId {
    game.hand(p)
        .iter()
        .copied()
        .find(|&id| game.def_id(id) == Ok(def))
        .unwrap_or_else(|| panic!("no {def:?} in {p:?}'s hand"))
}

/// Plays the first card in `p`'s hand with definition `def`.
pub(crate) fn play_def(game: &mut Game, p: PlayerId, def: DefId) {
    let action = play(in_hand(game, p, def));
    game.apply(p, action)
        .unwrap_or_else(|e| panic!("playing {def:?}: {e:?}"));
}

/// Plays minion `def` from `p`'s hand and returns its board id.
pub(crate) fn summon(game: &mut Game, p: PlayerId, def: DefId) -> ObjectId {
    play_def(game, p, def);
    *game.board(p).last().expect("the minion entered the board")
}

/// Every id in a zone: hand, deck, revealed and board, for each player.
pub(crate) fn zone_ids(game: &Game) -> Vec<ObjectId> {
    PLAYERS
        .iter()
        .flat_map(|&p| [game.hand(p), &game.deck(p), game.revealed(p), game.board(p)].concat())
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

/// Ends turns until it's `p`'s turn and `p` has at least `mana` mana.
pub(crate) fn turn_with_mana(game: &mut Game, p: PlayerId, mana: u8) {
    for _ in 0..40 {
        if game.legal_actions(p).contains(&Action::EndTurn) && game.mana(p) >= mana {
            return;
        }
        end_turn(game);
    }
    panic!("{p:?} never had {mana} mana on their turn");
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
/// the exact `IllegalAction` and changes nothing.
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
                Err(ApplyError::IllegalAction(IllegalAction {
                    player_id: p,
                    action: a.clone()
                })),
                "unlisted {a:?} for {p:?} was not rejected"
            );
            assert_eq!(&g, game, "rejected {a:?} for {p:?} changed the game");
        }
    }
}
