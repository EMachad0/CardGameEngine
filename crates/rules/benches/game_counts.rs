//! Instruction counts for the public `Game` interface, measured once under Callgrind.
//!
//! Each setup makes the measured call once before the count starts, so the allocator is warm.
//! A generic call instantiated with a closure defined inside a benchmark function matches
//! gungraun's collection toggle and stops the count, so benchmark functions define no closures.

mod support;

use std::hint::black_box;

use gungraun::prelude::{library_benchmark, library_benchmark_group, main};
use rules::prelude::{Game, PlayerId};
use support::{
    DeckFn, PLAYOUT_SEEDS, STAGE_SEED, decider, drafting_deck, drafting_stages, new_game,
    play_to_end, sample_deck, stages,
};

/// The state `name` from `stages` or `drafting_stages`.
fn stage(name: &str) -> Game {
    stages()
        .into_iter()
        .chain(drafting_stages())
        .find(|(stage, _)| *stage == name)
        .expect("a stage name")
        .1
}

fn warm_new(deck: DeckFn) -> DeckFn {
    drop(black_box(new_game(deck, STAGE_SEED)));
    deck
}

fn warm_clone(name: &str) -> Game {
    let game = stage(name);
    drop(black_box(game.clone()));
    game
}

fn warm_eq(name: &str) -> (Game, Game) {
    let game = stage(name);
    let copy = game.clone();
    black_box(game == copy);
    (game, copy)
}

fn warm_legal_actions(name: &str) -> (Game, PlayerId) {
    let game = stage(name);
    let p = decider(&game);
    drop(black_box(game.legal_actions(p)));
    (game, p)
}

fn warm_playout(deck: DeckFn) -> DeckFn {
    drop(black_box(play_to_end(deck, PLAYOUT_SEEDS.start)));
    deck
}

#[library_benchmark]
#[bench::sample(args = (sample_deck), setup = warm_new)]
fn new(deck: DeckFn) {
    drop(black_box(new_game(deck, black_box(STAGE_SEED))));
}

#[library_benchmark]
#[bench::early(args = ("early"), setup = warm_clone)]
#[bench::mid(args = ("mid"), setup = warm_clone)]
#[bench::late(args = ("late"), setup = warm_clone)]
fn clone(game: Game) -> Game {
    drop(black_box(black_box(&game).clone()));
    game
}

#[library_benchmark]
#[bench::early(args = ("early"), setup = warm_eq)]
#[bench::mid(args = ("mid"), setup = warm_eq)]
#[bench::late(args = ("late"), setup = warm_eq)]
fn eq(games: (Game, Game)) -> (Game, Game) {
    black_box(black_box(&games.0) == black_box(&games.1));
    games
}

#[library_benchmark]
#[bench::early(args = ("early"), setup = warm_legal_actions)]
#[bench::mid(args = ("mid"), setup = warm_legal_actions)]
#[bench::late(args = ("late"), setup = warm_legal_actions)]
fn legal_actions(input: (Game, PlayerId)) -> Game {
    let (game, p) = input;
    drop(black_box(black_box(&game).legal_actions(black_box(p))));
    game
}

#[library_benchmark]
#[bench::idle(args = ("idle"), setup = warm_legal_actions)]
#[bench::open(args = ("open"), setup = warm_legal_actions)]
fn legal_actions_drafting(input: (Game, PlayerId)) -> Game {
    let (game, p) = input;
    drop(black_box(black_box(&game).legal_actions(black_box(p))));
    game
}

#[library_benchmark]
#[bench::sample(args = (sample_deck), setup = warm_playout)]
#[bench::drafting(args = (drafting_deck), setup = warm_playout)]
fn playout(deck: DeckFn) {
    for seed in PLAYOUT_SEEDS {
        drop(black_box(play_to_end(deck, black_box(seed))));
    }
}

library_benchmark_group!(
    name = game,
    benchmarks = [
        new,
        clone,
        eq,
        legal_actions,
        legal_actions_drafting,
        playout
    ]
);

main!(library_benchmark_groups = game);
