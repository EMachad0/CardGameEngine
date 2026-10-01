//! Helpers shared by the in-file tests. Built only under `cfg(test)`.
//! They use only `Game`'s public API.

use crate::cards::BOLT;
use crate::{Action, DefId, Game, ObjectId, PlayerId};

pub(crate) const P0: PlayerId = PlayerId::new(0);
pub(crate) const P1: PlayerId = PlayerId::new(1);

pub(crate) fn play(card: ObjectId) -> Action {
    Action::Play { card }
}

pub(crate) fn pick(card: ObjectId) -> Action {
    Action::Pick { card }
}

/// `top`, then 20 Bolts as filler.
pub(crate) fn deck_with_top(top: &[DefId]) -> Vec<DefId> {
    let mut deck = top.to_vec();
    deck.extend(vec![BOLT; 20]);
    deck
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

pub(crate) fn defs(game: &Game, ids: &[ObjectId]) -> Vec<DefId> {
    ids.iter()
        .map(|&id| {
            game.def(id)
                .unwrap_or_else(|| panic!("def({id:?}) is None"))
        })
        .collect()
}

pub(crate) fn hand_defs(game: &Game, p: PlayerId) -> Vec<DefId> {
    defs(game, &game.hand(p))
}

pub(crate) fn deck_defs(game: &Game, p: PlayerId) -> Vec<DefId> {
    defs(game, &game.deck(p))
}

pub(crate) fn revealed_defs(game: &Game, p: PlayerId) -> Vec<DefId> {
    defs(game, &game.revealed(p))
}

pub(crate) fn board_defs(game: &Game, p: PlayerId) -> Vec<DefId> {
    defs(game, &game.board(p))
}

pub(crate) fn has_in_hand(game: &Game, p: PlayerId, def: DefId) -> bool {
    game.hand(p).into_iter().any(|id| game.def(id) == Some(def))
}

/// The first card in `p`'s hand with definition `def`.
pub(crate) fn in_hand(game: &Game, p: PlayerId, def: DefId) -> ObjectId {
    game.hand(p)
        .into_iter()
        .find(|&id| game.def(id) == Some(def))
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

/// Ends the current turn.
pub(crate) fn end_turn(game: &mut Game) {
    let p = [P0, P1]
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
