//! Fixtures shared by the `game` and `game_counts` benches, so both measure the same games.

use std::ops::Range;

use rules::prelude::static_card_definition::{
    BLAST, BOLT, CAPTAIN, CROSSFIRE, FORAGE, GIANT, PING, RECRUIT, SHOVE, SPARK, TWIN_SHOT,
    WILD_BOLT,
};
use rules::prelude::{Action, DefId, Game, PlayerId, Rng};

const MAX_STEPS: usize = 5_000;
pub(crate) const STAGE_SEED: u64 = 0;
pub(crate) const PLAYOUT_SEEDS: Range<u64> = 0..10;

pub(crate) type Deck = fn() -> Vec<DefId>;

pub(crate) fn sample_deck() -> Vec<DefId> {
    vec![
        SPARK, BOLT, WILD_BOLT, FORAGE, BLAST, RECRUIT, CAPTAIN, GIANT, RECRUIT, CAPTAIN, SPARK,
        FORAGE, BLAST, RECRUIT, CAPTAIN, BOLT,
    ]
}

/// `sample_deck` plus one of each card that is played through a draft.
pub(crate) fn drafting_deck() -> Vec<DefId> {
    let mut deck = sample_deck();
    deck.extend([PING, TWIN_SHOT, CROSSFIRE, SHOVE]);
    deck
}

pub(crate) fn new_game(deck: Deck, seed: u64) -> Game {
    Game::new(seed, [deck(), deck()])
}

/// Plays `seed`'s game with `deck` for both players to the end, picking uniformly
/// among both players' listed actions, and calls `visit` on every state that still
/// has a decision.
fn play_out(deck: Deck, seed: u64, mut visit: impl FnMut(&Game)) -> Game {
    let mut game = new_game(deck, seed);
    let mut picker = Rng::new(seed);
    for _ in 0..MAX_STEPS {
        let mut options: Vec<(PlayerId, Action)> = game
            .players()
            .into_iter()
            .flat_map(|p| game.legal_actions(p).into_iter().map(move |a| (p, a)))
            .collect();
        if options.is_empty() {
            return game;
        }
        visit(&game);
        let (p, a) = options.swap_remove(picker.below(options.len()));
        game.apply(p, a, &mut ())
            .expect("a listed action is accepted");
    }
    panic!("seed {seed}: no outcome within {MAX_STEPS} steps");
}

/// `play_out` without a visitor.
pub(crate) fn play_to_end(deck: Deck, seed: u64) -> Game {
    play_out(deck, seed, |_| {})
}

/// The first, middle and last decision states of one game.
pub(crate) fn stages() -> [(&'static str, Game); 3] {
    let mut states = Vec::new();
    play_out(sample_deck, STAGE_SEED, |game| states.push(game.clone()));
    [
        ("early", states[0].clone()),
        ("mid", states[states.len() / 2].clone()),
        ("late", states[states.len() - 1].clone()),
    ]
}

/// Across the drafting deck's playouts, the idle state that offers a `Draft` with the
/// most minions on the boards, and the open draft that offers the most `Choose`s.
pub(crate) fn drafting_stages() -> [(&'static str, Game); 2] {
    let mut idle: Option<(usize, Game)> = None;
    let mut open: Option<(usize, Game)> = None;
    for seed in PLAYOUT_SEEDS {
        play_out(drafting_deck, seed, |game| {
            let actions = game.legal_actions(decider(game));
            let minions = game
                .players()
                .iter()
                .map(|&p| game.player_board(p).len())
                .sum();
            let chooses = actions
                .iter()
                .filter(|a| matches!(a, Action::Choose { .. }))
                .count();
            if actions.iter().any(|a| matches!(a, Action::Draft { .. }))
                && idle.as_ref().is_none_or(|(best, _)| minions > *best)
            {
                idle = Some((minions, game.clone()));
            }
            if chooses > 0 && open.as_ref().is_none_or(|(best, _)| chooses > *best) {
                open = Some((chooses, game.clone()));
            }
        });
    }
    let idle = idle.expect("some playout offers a draft").1;
    let open = open.expect("some playout opens a draft").1;
    [("idle", idle), ("open", open)]
}

pub(crate) fn decider(game: &Game) -> PlayerId {
    game.players()
        .into_iter()
        .find(|&p| !game.legal_actions(p).is_empty())
        .expect("every stage has a decision")
}
