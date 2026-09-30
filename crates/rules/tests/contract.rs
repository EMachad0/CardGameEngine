//! Contract tests for the session-01 toy game. Spec: ../SPEC.md.
//!
//! Scenario tests build exact positions with `with_deck_order` (node S: a test
//! can construct a state directly). Property tests run seeded random playouts
//! and check the L/P contract and R1 determinism at every step.

use rules::{Action, Card, Game, Illegal, PlayerId, Rng};

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const PLAYERS: [PlayerId; 2] = [P0, P1];

fn bolt(damage: u8) -> Card {
    Card::Bolt { damage }
}

fn play(hand_index: usize) -> Action {
    Action::Play { hand_index }
}

fn pick(index: usize) -> Action {
    Action::Pick { index }
}

fn other(p: PlayerId) -> PlayerId {
    if p == P0 { P1 } else { P0 }
}

/// Order-insensitive comparison of an action list with the expected set.
fn assert_actions(game: &Game, p: PlayerId, expected: &[Action]) {
    let actual = game.legal_actions(p);
    let missing: Vec<&Action> = expected.iter().filter(|a| !actual.contains(a)).collect();
    let extra: Vec<&Action> = actual.iter().filter(|a| !expected.contains(a)).collect();
    assert!(
        missing.is_empty() && extra.is_empty() && actual.len() == expected.len(),
        "legal_actions({p:?}) = {actual:?}\n  expected {expected:?}\n  missing {missing:?}\n  extra {extra:?}"
    );
}

/// Every action a shell could send in this position, legal or not.
fn candidate_actions(game: &Game) -> Vec<Action> {
    let widest_hand = PLAYERS.iter().map(|&p| game.hand(p).len()).max().unwrap();
    let mut all = vec![Action::EndTurn];
    all.extend((0..widest_hand + 2).map(play));
    all.extend((0..4).map(pick));
    all
}

/// Card costs per SPEC.md. Deliberately written out here, not read from the core.
fn cost(card: &Card) -> u8 {
    match card {
        Card::Bolt { damage } => *damage,
        Card::WildBolt | Card::Forage => 1,
    }
}

fn card_count(game: &Game, p: PlayerId) -> usize {
    game.hand(p).len() + game.deck(p).len() + game.revealed(p).len()
}

// ---------------------------------------------------------------------------
// Invariants checked at every step of every playout
// ---------------------------------------------------------------------------

/// P, "Err changes nothing" direction: every unlisted (player, action) pair is
/// rejected with the exact `Illegal`, and the game is left untouched.
fn assert_unlisted_rejected(game: &Game) {
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

/// P, "listed means accepted" direction, plus R1: applying the same listed
/// action to two clones gives equal games (no hidden input like OS randomness).
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

// ---------------------------------------------------------------------------
// Playout driver
// ---------------------------------------------------------------------------

const MAX_STEPS: usize = 5_000;

fn sample_deck() -> Vec<Card> {
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

/// Plays a random game: at each step, gathers every (player, action) pair from
/// both lists and picks one with the test's own RNG. The picker's RNG is outside
/// the core, which is fine under R1: the picks it makes are the decisions.
fn random_playout(seed: u64, picker_seed: u64) -> (Game, Vec<(PlayerId, Action)>) {
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

        // Cards never appear from nowhere or vanish, except the one being played.
        let mut expected = before;
        if matches!(a, Action::Play { .. }) {
            expected[p.0] -= 1;
        }
        assert_eq!(
            after, expected,
            "seed {seed}: card count changed wrongly after {p:?} {a:?}"
        );

        // Playing any card pays exactly its cost.
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

// ---------------------------------------------------------------------------
// Scenario tests
// ---------------------------------------------------------------------------

#[test]
fn setup_draws_three_each_then_starts_player_0s_turn() {
    let deck0 = vec![bolt(1), bolt(2), bolt(3), bolt(4), bolt(5)];
    let deck1 = vec![bolt(1), bolt(1), bolt(1), bolt(2), bolt(2)];
    let game = Game::with_deck_order(0, [deck0, deck1]);

    assert_eq!(game.hand(P0), &[bolt(1), bolt(2), bolt(3), bolt(4)]);
    assert_eq!(game.deck(P0), &[bolt(5)]);
    assert_eq!(game.hand(P1), &[bolt(1), bolt(1), bolt(1)]);
    assert_eq!(game.deck(P1), &[bolt(2), bolt(2)]);
    assert_eq!(game.mana(P0), 1);
    assert_eq!(game.mana(P1), 0);
    assert_eq!(game.health(P0), 10);
    assert_eq!(game.health(P1), 10);
    assert_eq!(game.winner(), None);
    assert!(game.revealed(P0).is_empty());

    assert_actions(&game, P0, &[play(0), Action::EndTurn]);
    assert_actions(&game, P1, &[]);
}

#[test]
fn playing_a_bolt_pays_mana_and_hits_the_enemy() {
    let deck = vec![bolt(1), bolt(2), bolt(3), bolt(4), bolt(5)];
    let mut game = Game::with_deck_order(0, [deck.clone(), deck]);

    game.apply(P0, play(0)).unwrap();

    assert_eq!(game.health(P1), 9);
    assert_eq!(game.mana(P0), 0);
    assert_eq!(game.hand(P0), &[bolt(2), bolt(3), bolt(4)]);
    assert_actions(&game, P0, &[Action::EndTurn]);
}

#[test]
fn end_turn_starts_the_opponents_turn() {
    let deck = vec![bolt(1), bolt(2), bolt(3), bolt(4), bolt(5), bolt(6)];
    let mut game = Game::with_deck_order(0, [deck.clone(), deck]);

    game.apply(P0, Action::EndTurn).unwrap();
    assert_eq!(game.mana(P1), 1);
    assert_eq!(game.hand(P1), &[bolt(1), bolt(2), bolt(3), bolt(4)]);
    assert_actions(&game, P0, &[]);
    assert_actions(&game, P1, &[play(0), Action::EndTurn]);

    game.apply(P1, Action::EndTurn).unwrap();
    assert_eq!(game.mana(P0), 2, "max mana grows by one per turn");
    assert_eq!(
        game.hand(P0),
        &[bolt(1), bolt(2), bolt(3), bolt(4), bolt(5)]
    );
    assert_actions(&game, P0, &[play(0), play(1), Action::EndTurn]);
}

#[test]
fn mana_caps_at_ten() {
    let deck = vec![bolt(9); 20];
    let mut game = Game::with_deck_order(0, [deck.clone(), deck]);
    for _ in 0..12 {
        game.apply(P0, Action::EndTurn).unwrap();
        game.apply(P1, Action::EndTurn).unwrap();
    }
    assert_eq!(game.mana(P0), 10);
}

#[test]
fn acting_on_the_opponents_turn_is_rejected_and_changes_nothing() {
    let deck = vec![bolt(1); 6];
    let game = Game::with_deck_order(0, [deck.clone(), deck]);

    for a in [Action::EndTurn, play(0)] {
        let mut g = game.clone();
        assert_eq!(
            g.apply(P1, a.clone()),
            Err(Illegal {
                player: P1,
                action: a
            })
        );
        assert_eq!(g, game);
    }
}

#[test]
fn unaffordable_and_out_of_range_plays_are_rejected() {
    let deck = vec![bolt(1), bolt(5), bolt(5), bolt(5), bolt(5)];
    let game = Game::with_deck_order(0, [deck.clone(), deck]);

    assert_actions(&game, P0, &[play(0), Action::EndTurn]);
    assert_unlisted_rejected(&game);
}

#[test]
fn forage_offers_only_picks_until_one_is_made() {
    let deck0 = vec![
        Card::Forage,
        bolt(1),
        bolt(1),
        bolt(1),
        bolt(5),
        bolt(6),
        bolt(2),
    ];
    let deck1 = vec![bolt(1); 6];
    let mut game = Game::with_deck_order(0, [deck0, deck1]);

    game.apply(P0, play(0)).unwrap();

    assert_eq!(game.revealed(P0), &[bolt(5), bolt(6)]);
    assert_eq!(game.deck(P0), &[bolt(2)]);
    assert_actions(&game, P0, &[pick(0), pick(1)]);
    assert_actions(&game, P1, &[]);
    assert_unlisted_rejected(&game);

    game.apply(P0, pick(1)).unwrap();

    assert!(game.revealed(P0).is_empty());
    assert_eq!(game.hand(P0), &[bolt(1), bolt(1), bolt(1), bolt(6)]);
    assert_eq!(
        game.deck(P0),
        &[bolt(2), bolt(5)],
        "unpicked card goes to the bottom"
    );
    assert_actions(&game, P0, &[Action::EndTurn]);
}

#[test]
fn forage_with_one_card_left_still_asks_for_the_pick() {
    let deck0 = vec![Card::Forage, bolt(1), bolt(1), bolt(1), bolt(5)];
    let mut game = Game::with_deck_order(0, [deck0, vec![bolt(1); 6]]);

    game.apply(P0, play(0)).unwrap();
    assert_eq!(game.revealed(P0), &[bolt(5)]);
    assert_actions(&game, P0, &[pick(0)]);

    game.apply(P0, pick(0)).unwrap();
    assert_eq!(game.hand(P0), &[bolt(1), bolt(1), bolt(1), bolt(5)]);
    assert!(game.deck(P0).is_empty());
}

#[test]
fn forage_on_an_empty_deck_does_nothing() {
    let deck0 = vec![Card::Forage, bolt(1), bolt(1), bolt(1)];
    let mut game = Game::with_deck_order(0, [deck0, vec![bolt(1); 6]]);

    game.apply(P0, play(0)).unwrap();
    assert!(game.revealed(P0).is_empty());
    assert_eq!(game.hand(P0), &[bolt(1), bolt(1), bolt(1)]);
    assert_actions(&game, P0, &[Action::EndTurn]);
}

#[test]
fn drawing_from_an_empty_deck_deals_one_damage() {
    let deck1 = vec![bolt(1), bolt(1), bolt(1)];
    let mut game = Game::with_deck_order(0, [vec![bolt(1); 6], deck1]);

    game.apply(P0, Action::EndTurn).unwrap();
    assert_eq!(game.health(P1), 9);
    assert_eq!(game.hand(P1).len(), 3);
}

#[test]
fn lethal_ends_the_game_mid_turn() {
    // Player 1 has an empty deck and takes 1 fatigue per turn.
    // Player 0 plays Bolt 9 once mana allows.
    let deck0 = vec![bolt(9); 10];
    let deck1 = vec![bolt(9); 3];
    let mut game = Game::with_deck_order(0, [deck0, deck1]);

    for _ in 0..8 {
        game.apply(P0, Action::EndTurn).unwrap();
        game.apply(P1, Action::EndTurn).unwrap();
    }
    assert_eq!(game.health(P1), 2);
    assert_eq!(game.mana(P0), 9);

    game.apply(P0, play(0)).unwrap();

    assert_eq!(game.winner(), Some(P0));
    assert_actions(&game, P0, &[]);
    assert_actions(&game, P1, &[]);
    assert_unlisted_rejected(&game);
}

#[test]
fn fatigue_at_turn_start_can_end_the_game() {
    let mut game = Game::with_deck_order(0, [vec![bolt(9); 30], vec![bolt(9); 3]]);
    while game.winner().is_none() {
        game.apply(P0, Action::EndTurn).unwrap();
        if game.winner().is_none() {
            game.apply(P1, Action::EndTurn).unwrap();
        }
    }
    assert_eq!(game.winner(), Some(P0));
    assert!(game.health(P1) <= 0);
}

#[test]
fn applied_returns_a_new_game_and_leaves_the_original_alone() {
    let deck = vec![bolt(1); 6];
    let game = Game::with_deck_order(0, [deck.clone(), deck]);
    let snapshot = game.clone();

    let next = game.applied(P0, play(0)).unwrap();
    assert_eq!(game, snapshot);
    assert_eq!(next.health(P1), 9);

    assert_eq!(
        game.applied(P1, Action::EndTurn),
        Err(Illegal {
            player: P1,
            action: Action::EndTurn
        })
    );
}

// ---------------------------------------------------------------------------
// Determinism (R1) and property tests
// ---------------------------------------------------------------------------

#[test]
fn new_is_a_function_of_seed_and_decks() {
    for seed in 0..20 {
        let a = Game::new(seed, [sample_deck(), sample_deck()]);
        let b = Game::new(seed, [sample_deck(), sample_deck()]);
        assert_eq!(a, b, "seed {seed}");
    }
}

#[test]
fn new_actually_shuffles() {
    let unshuffled = Game::with_deck_order(0, [sample_deck(), sample_deck()]);
    let differs = (0..20).any(|seed| {
        let g = Game::new(seed, [sample_deck(), sample_deck()]);
        g.hand(P0) != unshuffled.hand(P0) || g.deck(P0) != unshuffled.deck(P0)
    });
    assert!(differs, "20 seeds and never a different deck order");
}

#[test]
fn random_playouts_keep_every_invariant_and_terminate() {
    for seed in 0..200 {
        let (game, log) = random_playout(seed, seed.wrapping_mul(31).wrapping_add(7));
        assert!(
            game.winner().is_some(),
            "seed {seed}: ended without a winner"
        );
        assert!(!log.is_empty());
    }
}

#[test]
fn replaying_seed_plus_decisions_reproduces_the_game() {
    for seed in 0..50 {
        let (original, log) = random_playout(seed, seed ^ 0xC0FFEE);

        let mut replay = Game::new(seed, [sample_deck(), sample_deck()]);
        for (p, a) in log {
            replay
                .apply(p, a.clone())
                .unwrap_or_else(|e| panic!("seed {seed}: replayed action rejected: {e:?}"));
        }
        assert_eq!(replay, original, "seed {seed}: replay diverged");
    }
}
