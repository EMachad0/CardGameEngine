//! The random playout driver and the invariants it checks before every step.

use std::collections::{BTreeMap, BTreeSet};

use rules::cards::{BLAST, BOLT, CAPTAIN, FORAGE, GIANT, RECRUIT, SPARK, WILD_BOLT};
use rules::{Action, DefId, Game, LookupError, ObjectId, Outcome, PlayerId, Rng};

use crate::model::{BoardModel, expected_cost, is_spell};
use crate::support::*;

const MAX_STEPS: usize = 5_000;

pub(crate) fn sample_deck() -> Vec<DefId> {
    vec![
        SPARK, BOLT, WILD_BOLT, FORAGE, BLAST, RECRUIT, CAPTAIN, GIANT, RECRUIT, CAPTAIN, SPARK,
        FORAGE, BLAST, RECRUIT, CAPTAIN, BOLT,
    ]
}

/// What the driver remembers between steps.
#[derive(Default)]
struct History {
    /// The definition each id had when first seen.
    defs: BTreeMap<ObjectId, DefId>,
    /// Ids that were in a zone once and are in none now.
    gone: BTreeSet<ObjectId>,
    spells_cast: [u8; 2],
    boards: BoardModel,
}

/// Plays a random game and returns it with its decision log. A separate `Rng`
/// picks each step's action from both players' lists. R1 allows an RNG outside
/// the core here, because its picks are the decisions.
pub(crate) fn random_playout(seed: u64, picker_seed: u64) -> (Game, Vec<(PlayerId, Action)>) {
    let mut game = Game::new(seed, [sample_deck(), sample_deck()]);
    let mut picker = Rng::new(picker_seed);
    let mut history = History::default();
    let mut log = Vec::new();

    for step in 0..MAX_STEPS {
        let context = format!("seed {seed}, step {step}");
        assert_invariants(&game, &mut history, &context);

        let options: Vec<(PlayerId, Action)> = PLAYERS
            .iter()
            .flat_map(|&p| game.legal_actions(p).into_iter().map(move |a| (p, a)))
            .collect();
        if options.is_empty() {
            return (game, log);
        }

        let (p, a) = options[picker.below(options.len())].clone();
        let off_board_before = PLAYERS.map(|q| off_board_count(&game, q));
        let mana_before = game.mana(p);
        let played = match a {
            Action::Play { object_id } => Some(game.def_id(object_id).unwrap()),
            _ => None,
        };

        game.apply(p, a.clone())
            .unwrap_or_else(|e| panic!("{context}: listed action rejected: {e:?}"));

        let mut expected = off_board_before;
        if let Some(def) = played {
            expected[p.idx()] -= 1;
            let cost = expected_cost(def, history.spells_cast[p.idx()]);
            assert_eq!(
                game.mana(p),
                mana_before - cost,
                "{context}: playing {def:?} didn't pay {cost}"
            );
            if is_spell(def) {
                history.spells_cast[p.idx()] += 1;
            }
            history.boards.played(p, def);
        }
        history.boards.check();
        assert_eq!(
            PLAYERS.map(|q| off_board_count(&game, q)),
            expected,
            "{context}: a card appeared or vanished off the board after {p:?} {a:?}"
        );

        log.push((p, a));
    }
    panic!("seed {seed}: game did not end within {MAX_STEPS} steps");
}

fn off_board_count(game: &Game, p: PlayerId) -> usize {
    game.hand(p).len() + game.deck(p).len() + game.revealed(p).len()
}

fn assert_invariants(game: &Game, history: &mut History, context: &str) {
    for p in PLAYERS {
        let legal = game.legal_actions(p);
        for (i, a) in legal.iter().enumerate() {
            assert!(
                !legal[i + 1..].contains(a),
                "{context}: duplicate {a:?} in legal_actions({p:?})"
            );
        }
        assert!(game.mana(p) <= 10, "{context}: mana above cap for {p:?}");
    }
    assert_outcome_consistent(game, context);
    assert_ids(game, history, context);
    assert_accessors(game, history, context);
    assert_boards_match(game, &history.boards, context);
    assert_unlisted_rejected_with(game, &history.gone);
    assert_listed_accepted_and_deterministic(game);
}

fn assert_outcome_consistent(game: &Game, context: &str) {
    let deciders = PLAYERS
        .iter()
        .filter(|&&p| !game.legal_actions(p).is_empty())
        .count();
    match game.outcome() {
        Some(Outcome::Won(w)) => {
            assert_eq!(deciders, 0, "{context}: over, but someone has actions");
            assert!(
                game.hero_health(other(w)) <= 0,
                "{context}: loser has health left"
            );
            assert!(game.hero_health(w) > 0, "{context}: winner is dead");
        }
        Some(Outcome::Draw) => {
            assert_eq!(deciders, 0, "{context}: over, but someone has actions");
            for p in PLAYERS {
                assert!(
                    game.hero_health(p) <= 0,
                    "{context}: draw, but {p:?} is alive"
                );
            }
        }
        None => {
            assert_eq!(deciders, 1, "{context}: exactly one player decides");
            for p in PLAYERS {
                assert!(
                    game.hero_health(p) > 0,
                    "{context}: {p:?} is dead, game goes on"
                );
            }
        }
    }
}

/// B. Each id is in one zone, keeps its definition, and never comes back once gone.
fn assert_ids(game: &Game, history: &mut History, context: &str) {
    let ids = zone_ids(game);
    let unique: BTreeSet<ObjectId> = ids.iter().copied().collect();
    assert_eq!(
        unique.len(),
        ids.len(),
        "{context}: an ObjectId is in two places"
    );

    for &id in &ids {
        assert!(!history.gone.contains(&id), "{context}: {id:?} came back");
        let def = game
            .def_id(id)
            .unwrap_or_else(|e| panic!("{context}: def_id({id:?}): {e:?}"));
        let first = *history.defs.entry(id).or_insert(def);
        assert_eq!(def, first, "{context}: {id:?} changed its definition");
    }
    for &id in history.defs.keys() {
        if !unique.contains(&id) {
            history.gone.insert(id);
        }
    }
}

/// Each accessor answers only for its own zone, and a hand card costs what the model says.
fn assert_accessors(game: &Game, history: &History, context: &str) {
    for p in PLAYERS {
        for &id in game.hand(p) {
            let def = game.def_id(id).unwrap();
            let cost = expected_cost(def, history.spells_cast[p.idx()]);
            assert_eq!(
                game.mana_cost(id),
                Ok(Some(cost)),
                "{context}: cost of {def:?}"
            );
            assert_eq!(
                (game.attack(id), game.health(id)),
                (Ok(None), Ok(None)),
                "{context}"
            );
        }
        let deck = game.deck(p);
        for &id in deck.iter().chain(game.revealed(p)) {
            assert_eq!(game.mana_cost(id), Ok(None), "{context}");
            assert_eq!(
                (game.attack(id), game.health(id)),
                (Ok(None), Ok(None)),
                "{context}"
            );
        }
        for &id in game.board(p) {
            assert_eq!(game.mana_cost(id), Ok(None), "{context}");
            let health = game.health(id);
            assert!(
                matches!(health, Ok(Some(h)) if h > 0),
                "{context}: a minion at {health:?} health is on the board"
            );
        }
    }
}

/// C. Both boards hold the model's minions, in order, with the model's attack and health.
fn assert_boards_match(game: &Game, model: &BoardModel, context: &str) {
    type Row = (
        Result<DefId, LookupError>,
        Result<Option<i32>, LookupError>,
        Result<Option<i32>, LookupError>,
    );
    for p in PLAYERS {
        let actual: Vec<Row> = game
            .board(p)
            .iter()
            .map(|&id| (game.def_id(id), game.attack(id), game.health(id)))
            .collect();
        let expected: Vec<Row> = model
            .minions(p)
            .into_iter()
            .map(|(def, (attack, health))| (Ok(def), Ok(Some(attack)), Ok(Some(health))))
            .collect();
        assert_eq!(
            actual, expected,
            "{context}: {p:?}'s board as (def, attack, health)"
        );
    }
}

/// P for listed actions, plus R1. `apply` accepts each one, and two clones given
/// the same action stay equal, which a hidden input like OS randomness would break.
fn assert_listed_accepted_and_deterministic(game: &Game) {
    for p in PLAYERS {
        for a in game.legal_actions(p) {
            let mut first = game.clone();
            let mut second = game.clone();
            assert_eq!(
                first.apply(p, a.clone()),
                Ok(()),
                "listed {a:?} for {p:?} was rejected"
            );
            second.apply(p, a.clone()).unwrap();
            assert_eq!(
                first, second,
                "same state + same action gave different games: {a:?}"
            );
        }
    }
}
