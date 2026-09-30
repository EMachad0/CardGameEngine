//! Shared helpers: action builders, an independent cost oracle, the invariants
//! checked at every playout step, and the random playout driver.

use rules::{Action, Card, Game, Illegal, PlayerId, Rng};

pub(crate) const P0: PlayerId = PlayerId::new(0);
pub(crate) const P1: PlayerId = PlayerId::new(1);
pub(crate) const PLAYERS: [PlayerId; 2] = [P0, P1];

pub(crate) fn bolt(damage: u8) -> Card {
    Card::Bolt { damage }
}

pub(crate) fn play(hand_index: usize) -> Action {
    Action::Play { hand_index }
}

pub(crate) fn pick(index: usize) -> Action {
    Action::Pick { index }
}

fn other(p: PlayerId) -> PlayerId {
    if p == P0 { P1 } else { P0 }
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

/// Actions to try in this position, legal or not. The indices run past the
/// widest hand and the largest reveal.
fn candidate_actions(game: &Game) -> Vec<Action> {
    let widest_hand = PLAYERS.iter().map(|&p| game.hand(p).len()).max().unwrap();
    let mut all = vec![Action::EndTurn];
    all.extend((0..widest_hand + 2).map(play));
    all.extend((0..4).map(pick));
    all
}

/// Card costs from SPEC.md. Reading them from the core would let a wrong cost check itself.
fn cost(card: &Card) -> u8 {
    match card {
        Card::Bolt { damage } => *damage,
        Card::WildBolt | Card::Forage => 1,
    }
}

fn card_count(game: &Game, p: PlayerId) -> usize {
    game.hand(p).len() + game.deck(p).len() + game.revealed(p).len()
}

/// P for unlisted actions. Each unlisted candidate, for either player, returns
/// the exact `Illegal` and changes nothing.
pub(crate) fn assert_unlisted_rejected(game: &Game) {
    for p in PLAYERS {
        let legal = game.legal_actions(p);
        for a in candidate_actions(game) {
            if legal.contains(&a) {
                continue;
            }
            let mut g = game.clone();
            let result = g.apply(p, a.clone());
            assert_eq!(
                result,
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

fn assert_invariants(game: &Game) {
    for p in PLAYERS {
        let legal = game.legal_actions(p);
        for (i, a) in legal.iter().enumerate() {
            assert!(
                !legal[i + 1..].contains(a),
                "duplicate {a:?} in legal_actions({p:?})"
            );
        }
        assert!(game.mana(p) <= 10, "mana above cap for {p:?}");
    }

    let deciders = PLAYERS
        .iter()
        .filter(|&&p| !game.legal_actions(p).is_empty())
        .count();
    match game.winner() {
        Some(w) => {
            assert_eq!(deciders, 0, "game is over but someone still has actions");
            assert!(
                game.health(other(w)) <= 0,
                "winner declared but loser has health left"
            );
            assert!(game.health(w) > 0, "winner is dead");
        }
        None => {
            assert_eq!(deciders, 1, "exactly one player decides in this game");
            for p in PLAYERS {
                assert!(game.health(p) > 0, "{p:?} is dead but the game isn't over");
            }
        }
    }

    assert_unlisted_rejected(game);
    assert_listed_accepted_and_deterministic(game);
}

const MAX_STEPS: usize = 5_000;

pub(crate) fn sample_deck() -> Vec<Card> {
    vec![
        bolt(1),
        bolt(2),
        Card::Forage,
        bolt(3),
        Card::WildBolt,
        bolt(1),
        Card::Forage,
        bolt(2),
        bolt(4),
        Card::WildBolt,
    ]
}

/// Plays a random game and returns it with its decision log. A separate `Rng`
/// picks each step's action from both players' lists. R1 allows an RNG outside
/// the core here, because its picks are the decisions.
pub(crate) fn random_playout(seed: u64, picker_seed: u64) -> (Game, Vec<(PlayerId, Action)>) {
    let mut game = Game::new(seed, [sample_deck(), sample_deck()]);
    let mut picker = Rng::new(picker_seed);
    let mut log = Vec::new();

    for _ in 0..MAX_STEPS {
        assert_invariants(&game);

        let options: Vec<(PlayerId, Action)> = PLAYERS
            .iter()
            .flat_map(|&p| game.legal_actions(p).into_iter().map(move |a| (p, a)))
            .collect();
        if options.is_empty() {
            return (game, log);
        }

        let (p, a) = options[picker.below(options.len())].clone();
        let before = [card_count(&game, P0), card_count(&game, P1)];
        let mana_before = game.mana(p);
        let played = match a {
            Action::Play { hand_index } => Some(game.hand(p)[hand_index].clone()),
            _ => None,
        };
        game.apply(p, a.clone())
            .unwrap_or_else(|e| panic!("seed {seed}: listed action rejected: {e:?}"));
        let after = [card_count(&game, P0), card_count(&game, P1)];

        // No card appears or vanishes, except the one being played.
        let mut expected = before;
        if matches!(a, Action::Play { .. }) {
            expected[p.idx()] -= 1;
        }
        assert_eq!(
            after, expected,
            "seed {seed}: card count changed wrongly after {p:?} {a:?}"
        );

        if let Some(card) = played {
            assert_eq!(
                game.mana(p),
                mana_before - cost(&card),
                "seed {seed}: playing {card:?} didn't pay its cost"
            );
        }

        log.push((p, a));
    }
    panic!("seed {seed}: game did not end within {MAX_STEPS} steps");
}
