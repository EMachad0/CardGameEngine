//! Helpers shared by the spec tests: action builders, card lookups, turn helpers, the
//! legality assertions and an observer that records what `apply` reports.

use std::collections::BTreeSet;

use rules::static_card_definition::{BOLT, RECRUIT};
use rules::{
    Action, ApplyError, ChoiceId, DefId, Event, Game, IllegalAction, ObjectId, Observer, PlayerId,
    PlayerView, View, Views,
};

/// The game's players, in `Game::players` order. The tests assume the player at index `i`
/// got `decks[i]`.
pub(crate) fn players(game: &Game) -> [PlayerId; 2] {
    game.players().try_into().expect("a game has two players")
}

/// `p`'s entry in `view.players`, found by id.
pub(crate) fn player_view(view: &View, p: PlayerId) -> &PlayerView {
    view.players
        .iter()
        .find(|player| player.player_id == p)
        .unwrap_or_else(|| panic!("{p:?} is not in the view"))
}

pub(crate) fn play(object_id: ObjectId) -> Action {
    Action::Play { object_id }
}

pub(crate) fn pick(object_id: ObjectId) -> Action {
    Action::Pick { object_id }
}

pub(crate) fn draft(object_id: ObjectId) -> Action {
    Action::Draft { object_id }
}

/// Chooses `object_id` for the card's choice with id `choice`.
pub(crate) fn choose(choice: usize, object_id: ObjectId) -> Action {
    Action::Choose {
        choice_id: ChoiceId(choice),
        object_id,
    }
}

pub(crate) fn cancel(object_id: ObjectId) -> Action {
    Action::Cancel { object_id }
}

pub(crate) fn other(game: &Game, p: PlayerId) -> PlayerId {
    let [p0, p1] = players(game);
    if p == p0 { p1 } else { p0 }
}

/// `top`, then 20 Bolts as filler.
pub(crate) fn deck_with_top(top: &[DefId]) -> Vec<DefId> {
    let mut deck = top.to_vec();
    deck.extend(vec![BOLT; 20]);
    deck
}

pub(crate) fn defs(game: &Game, ids: &[ObjectId]) -> Vec<DefId> {
    ids.iter().map(|&id| game.def_id(id)).collect()
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
    game.hand(p).iter().any(|&id| game.def_id(id) == def)
}

/// The first card in `p`'s hand with definition `def`.
pub(crate) fn in_hand(game: &Game, p: PlayerId, def: DefId) -> ObjectId {
    game.hand(p)
        .iter()
        .copied()
        .find(|&id| game.def_id(id) == def)
        .unwrap_or_else(|| panic!("no {def:?} in {p:?}'s hand"))
}

/// Plays the first card in `p`'s hand with definition `def`.
pub(crate) fn play_def(game: &mut Game, p: PlayerId, def: DefId) {
    let action = play(in_hand(game, p, def));
    game.apply(p, action, &mut ())
        .unwrap_or_else(|e| panic!("playing {def:?}: {e:?}"));
}

/// Applies each of `actions` for `p`, in order.
pub(crate) fn apply_all(game: &mut Game, p: PlayerId, actions: &[Action]) {
    for &a in actions {
        game.apply(p, a, &mut ())
            .unwrap_or_else(|e| panic!("{a:?} for {p:?}: {e:?}"));
    }
}

/// P0's turn with at least 2 mana, built by `table`.
pub(crate) struct Table {
    pub(crate) game: Game,
    pub(crate) p0: PlayerId,
    pub(crate) p1: PlayerId,
    /// P0's Recruits, left to right.
    pub(crate) friendly: Vec<ObjectId>,
    /// P1's Recruits, left to right.
    pub(crate) enemy: Vec<ObjectId>,
}

/// P0 holds `hand` and has `friendly` Recruits on the board, and P1 has `enemy` Recruits.
/// `friendly` and `hand` together are at most 3 cards, so they fit in P0's opening hand.
/// `enemy` is at most 3.
pub(crate) fn table(hand: &[DefId], friendly: usize, enemy: usize) -> Table {
    let mut top0 = vec![RECRUIT; friendly];
    top0.extend_from_slice(hand);
    let mut game = Game::with_deck_order(
        0,
        [deck_with_top(&top0), deck_with_top(&vec![RECRUIT; enemy])],
    );
    let [p0, p1] = players(&game);

    let mut friendly_ids = Vec::new();
    if friendly > 0 {
        turn_with_mana(&mut game, p0, 2 * friendly as u8);
        for _ in 0..friendly {
            friendly_ids.push(summon(&mut game, p0, RECRUIT));
        }
    }
    let mut enemy_ids = Vec::new();
    if enemy > 0 {
        turn_with_mana(&mut game, p1, 2 * enemy as u8);
        for _ in 0..enemy {
            enemy_ids.push(summon(&mut game, p1, RECRUIT));
        }
    }
    turn_with_mana(&mut game, p0, 2);

    Table {
        game,
        p0,
        p1,
        friendly: friendly_ids,
        enemy: enemy_ids,
    }
}

/// One checkpoint: the events since the previous one, then each player's view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Step {
    pub(crate) events: Vec<Event>,
    /// In `players` order.
    pub(crate) views: [View; 2],
}

impl Step {
    pub(crate) fn view(&self, viewer: PlayerId) -> &View {
        self.views
            .iter()
            .find(|view| view.viewer == viewer)
            .unwrap_or_else(|| panic!("no view for {viewer:?}"))
    }
}

/// Records what `apply` reports, one `Step` per checkpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Recorder {
    players: [PlayerId; 2],
    pub(crate) steps: Vec<Step>,
    /// Events reported after the last checkpoint.
    pub(crate) trailing: Vec<Event>,
}

impl Observer for Recorder {
    fn event(&mut self, e: &Event) {
        self.trailing.push(e.clone());
    }

    fn checkpoint(&mut self, views: Views<'_>) {
        self.steps.push(Step {
            events: std::mem::take(&mut self.trailing),
            views: self.players.map(|p| views.of(p)),
        });
    }
}

impl Recorder {
    /// Records a view per player of `game` at each checkpoint.
    pub(crate) fn new(game: &Game) -> Self {
        Self {
            players: players(game),
            steps: Vec::new(),
            trailing: Vec::new(),
        }
    }

    /// Every event, in order, with the checkpoints left out.
    pub(crate) fn events(&self) -> Vec<Event> {
        self.steps
            .iter()
            .flat_map(|step| step.events.iter())
            .chain(&self.trailing)
            .cloned()
            .collect()
    }
}

/// Applies `a` for `p` with a `Recorder` and returns what it recorded.
pub(crate) fn observe(game: &mut Game, p: PlayerId, a: Action) -> Recorder {
    let mut recorder = Recorder::new(game);
    game.apply(p, a, &mut recorder)
        .unwrap_or_else(|e| panic!("{a:?} for {p:?}: {e:?}"));
    recorder
}

/// Plays minion `def` from `p`'s hand and returns its board id.
pub(crate) fn summon(game: &mut Game, p: PlayerId, def: DefId) -> ObjectId {
    play_def(game, p, def);
    *game.board(p).last().expect("the minion entered the board")
}

/// Every id in a zone: hand, deck, revealed, board and hero, for each player.
pub(crate) fn zone_ids(game: &Game) -> Vec<ObjectId> {
    players(game)
        .iter()
        .flat_map(|&p| {
            [
                game.hand(p),
                &game.deck(p),
                game.revealed(p),
                game.board(p),
                &[game.hero_id(p)],
            ]
            .concat()
        })
        .collect()
}

/// Ends the current turn.
pub(crate) fn end_turn(game: &mut Game) {
    let p = players(game)
        .into_iter()
        .find(|&p| game.legal_actions(p).contains(&Action::EndTurn))
        .expect("someone can end their turn");
    game.apply(p, Action::EndTurn, &mut ()).unwrap();
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

/// Like `assert_unlisted_rejected`, and also tries the id-carrying actions on every id in `extra`.
pub(crate) fn assert_unlisted_rejected_with(game: &Game, extra: &BTreeSet<ObjectId>) {
    let ids: BTreeSet<ObjectId> = zone_ids(game)
        .into_iter()
        .chain(extra.iter().copied())
        .collect();
    let mut candidates = vec![Action::EndTurn];
    for id in ids {
        candidates.push(play(id));
        candidates.push(pick(id));
        candidates.push(draft(id));
        candidates.push(cancel(id));
        candidates.push(choose(0, id));
        candidates.push(choose(1, id));
    }
    let unlisted: Vec<(PlayerId, Action)> = players(game)
        .into_iter()
        .flat_map(|p| {
            let legal = game.legal_actions(p);
            candidates
                .iter()
                .filter(move |a| !legal.contains(a))
                .map(move |&a| (p, a))
        })
        .collect();
    let rejection = |p, a| {
        Err(ApplyError::IllegalAction(IllegalAction {
            player_id: p,
            action: a,
        }))
    };

    // Every rejection should leave `g` equal to `game`, so one clone and one comparison serve
    // every candidate.
    let mut g = game.clone();
    let all_rejected = unlisted
        .iter()
        .all(|&(p, a)| g.apply(p, a, &mut ()) == rejection(p, a));
    if all_rejected && &g == game {
        return;
    }

    for (p, a) in unlisted {
        let mut g = game.clone();
        assert_eq!(
            g.apply(p, a, &mut ()),
            rejection(p, a),
            "unlisted {a:?} for {p:?} was not rejected"
        );
        assert_eq!(&g, game, "rejected {a:?} for {p:?} changed the game");
    }
    panic!("the unlisted actions, applied in sequence, changed the game");
}
