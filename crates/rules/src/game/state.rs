//! `Game`, the whole state of one game: what callers ask it, and what each step is lent. A step
//! writes only the parts of the game it is lent and returns what it did, which is reported once the
//! step ends, before a checkpoint. Code outside this file never holds a `Game` it can write. It is
//! lent a type that takes only its own steps.

use crate::action::{Action, IllegalAction};
use crate::cards::binder::def_id::DefId;
use crate::cards::binder::store::Binder;
use crate::cards::object::{ObjectBag, ObjectId};
use crate::game::cards_ref::CardsRef;
use crate::game::effect_stack::{EffectStack, NextEffect};
use crate::game::legal_actions::LegalAction;
use crate::game::play::WasPlayable;
use crate::game::player::{Player, Players};
use crate::game::player_input::awaiting::{
    PlayerInput, WasAwaitingNothing, WasAwaitingPicking, WasAwaitingTargeting,
};
use crate::game::reporting::Report;
use crate::game::resolve::aftermath::death;
use crate::game::resolve::aftermath::standings::Standings;
use crate::game::resolve::{effect, resolution_loop};
use crate::game::view::{self, View};
use crate::game::{action_steps, legal_actions, play, setup, stats, turn_change, zone_move};
use crate::history::History;
use crate::ids::PlayerId;
use crate::observer::{Observer, Views};
use crate::outcome::Outcome;
use crate::rng::Rng;
use crate::target::TargetId;
use crate::turn::TurnOrder;
use crate::zones::Zones;

#[derive(Debug, Clone, PartialEq)]
pub struct Game {
    rng: Rng,
    turn_order: TurnOrder,
    players: Players,
    zones: Zones,
    player_input: PlayerInput,
    objects: ObjectBag,
    binder: Binder,
    history: History,
    standings: Standings,
    effect_stack: EffectStack,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ApplyError {
    #[error("Illegal Action")]
    IllegalAction(IllegalAction),
}

/// The writes one step makes, between two checkpoints.
enum Step {
    Draw { player_id: PlayerId, count: u8 },
    StartTurn,
    Action(ActionStep),
    Effect(NextEffect),
    DeathPass,
    DecideOutcome,
}

/// The steps an action takes.
pub(super) enum ActionStep {
    EndTurn,
    Play(WasPlayable),
    Summon {
        player_id: PlayerId,
        object_id: ObjectId,
    },
    StartTargeting {
        object_id: ObjectId,
        awaiting_nothing: WasAwaitingNothing,
    },
    Target {
        awaiting_targeting: WasAwaitingTargeting,
        target_id: TargetId,
        object_id: ObjectId,
    },
    Cancel(WasAwaitingTargeting),
    Pick(WasAwaitingPicking),
}

impl Game {
    /// Seeds the `Rng`, shuffles both decks, deals 3 cards each and starts player 0's turn.
    pub fn new(seed: u64, decks: [Vec<DefId>; 2]) -> Self {
        setup::set_up(seed, decks, true)
    }

    /// Like `new`, but keeps each deck in the given order, index 0 on top.
    pub fn with_deck_order(seed: u64, decks: [Vec<DefId>; 2]) -> Self {
        setup::set_up(seed, decks, false)
    }

    /// `Ok` if and only if `action` is in `legal_actions(player_id)`.
    /// An `Err` leaves the game unchanged.
    pub fn apply(
        &mut self,
        player_id: PlayerId,
        action: Action,
        obs: &mut impl Observer,
    ) -> Result<(), ApplyError> {
        let Some(legal_action) = self.legal_action(player_id, action) else {
            return Err(ApplyError::IllegalAction(IllegalAction::new(
                player_id, action,
            )));
        };
        action_steps::take_action(Acting { game: self, obs }, legal_action);
        resolution_loop::resolve_effect_stack(Resolving { game: self, obs });
        Ok(())
    }

    pub fn applied(&self, player_id: PlayerId, action: Action) -> Result<Self, ApplyError> {
        let mut game = self.clone();
        game.apply(player_id, action, &mut ())?;
        Ok(game)
    }

    pub fn legal_actions(&self, player_id: PlayerId) -> Vec<Action> {
        self.list_legal_actions(player_id)
            .iter()
            .map(Action::from)
            .collect()
    }

    pub fn view(&self, viewer: PlayerId) -> View {
        view::view(
            self.cards(),
            &self.players,
            &self.player_input,
            &self.standings,
            viewer,
        )
    }

    pub fn hand(&self, player_id: PlayerId) -> &[ObjectId] {
        self.zones.hand(player_id)
    }

    /// A copy, top first.
    pub fn deck(&self, player_id: PlayerId) -> Vec<ObjectId> {
        self.zones.deck(player_id)
    }

    pub fn player_board(&self, player_id: PlayerId) -> &[ObjectId] {
        self.zones.board(player_id)
    }

    pub fn hero_id(&self, player_id: PlayerId) -> ObjectId {
        self.zones.hero(player_id)
    }

    pub fn def_id(&self, object_id: ObjectId) -> DefId {
        self.cards().def_id(object_id)
    }

    /// `None` unless the card is in its owner's hand. Panics on an id the game never made.
    pub fn mana_cost(&self, object_id: ObjectId) -> Option<u8> {
        stats::mana_cost(self.cards(), object_id)
    }

    /// `None` unless the card is on its owner's board. Panics on an id the game never made.
    pub fn attack(&self, object_id: ObjectId) -> Option<i32> {
        stats::attack(self.cards(), object_id)
    }

    /// `None` unless the object is a character in play. Panics on an id the game never made.
    pub fn health(&self, object_id: ObjectId) -> Option<i32> {
        stats::health(self.cards(), object_id)
    }

    pub fn hero_health(&self, player_id: PlayerId) -> i32 {
        stats::hero_health(self.cards(), player_id)
    }

    pub fn mana(&self, player_id: PlayerId) -> u8 {
        self.players.get(player_id).mana.current()
    }

    /// The options of the player's picking. Empty unless they are picking.
    pub fn revealed(&self, player_id: PlayerId) -> &[ObjectId] {
        self.player_input.revealed(player_id)
    }

    pub fn outcome(&self) -> Option<Outcome> {
        self.standings.outcome()
    }

    pub fn players(&self) -> Vec<PlayerId> {
        self.players.iter().map(Player::id).collect()
    }

    /// `None` unless `legal_actions(player_id)` lists `action`.
    fn legal_action(&self, player_id: PlayerId, action: Action) -> Option<LegalAction> {
        self.list_legal_actions(player_id)
            .into_iter()
            .find(|legal| Action::from(legal) == action)
    }

    fn list_legal_actions(&self, player_id: PlayerId) -> Vec<LegalAction> {
        legal_actions::legal_actions(
            self.cards(),
            &self.standings,
            &self.player_input,
            self.players.get(player_id),
        )
    }

    /// Makes the step's writes, reports what it did, then lets the observer read the views.
    /// `false`, with no report and no checkpoint, if `write` found no step to take.
    fn take_step(&mut self, step: Step, obs: &mut impl Observer) -> bool {
        let Some(reports) = self.write(step) else {
            return false;
        };
        for report in reports {
            self.report(report, obs);
        }
        obs.checkpoint(Views::new(self));
        true
    }

    /// Makes the step's writes, lending it what it reads and writes. Returns the events it caused,
    /// in order. `None` for a death pass that finds no dead, or a decision that doesn't end the
    /// game.
    fn write(&mut self, step: Step) -> Option<Vec<Report>> {
        let reports = match step {
            Step::Draw { player_id, count } => {
                zone_move::draw(player_id, count, &mut self.zones, &mut self.objects)
            }
            Step::StartTurn => turn_change::start_turn(
                &self.turn_order,
                &mut self.players,
                &mut self.zones,
                &mut self.objects,
            ),
            Step::Action(ActionStep::EndTurn) => turn_change::end_turn(
                &mut self.turn_order,
                &mut self.players,
                &mut self.zones,
                &mut self.objects,
            ),
            Step::Action(ActionStep::Play(playable)) => {
                let prepared = play::prepare(self.cards(), &self.player_input, playable);
                vec![play::play(
                    prepared,
                    &mut self.players,
                    &mut self.player_input,
                    &mut self.zones,
                    &mut self.effect_stack,
                )]
            }
            Step::Action(ActionStep::Summon {
                player_id,
                object_id,
            }) => vec![zone_move::summon(player_id, object_id, &mut self.zones)],
            Step::Action(ActionStep::StartTargeting {
                object_id,
                awaiting_nothing,
            }) => {
                self.player_input
                    .start_targeting(awaiting_nothing, object_id);
                Vec::new()
            }
            Step::Action(ActionStep::Target {
                awaiting_targeting,
                target_id,
                object_id,
            }) => {
                self.player_input
                    .target(awaiting_targeting, object_id, target_id);
                Vec::new()
            }
            Step::Action(ActionStep::Cancel(awaiting_targeting)) => {
                self.player_input.cancel(awaiting_targeting);
                Vec::new()
            }
            Step::Action(ActionStep::Pick(awaiting_picking)) => {
                let (player_id, object_id) = self.player_input.pick(awaiting_picking);
                vec![Report::Picked {
                    player_id,
                    object_id,
                }]
            }
            Step::Effect(next) => {
                let prepared =
                    effect::prepare(self.cards(), &self.players(), &self.player_input, next);
                effect::resolve(
                    prepared,
                    &mut self.rng,
                    &mut self.zones,
                    &mut self.objects,
                    &mut self.player_input,
                )
            }
            Step::DeathPass => {
                let death_pass =
                    death::find_deaths(self.cards(), &self.players(), &self.standings)?;
                death::kill(
                    death_pass,
                    &mut self.zones,
                    &mut self.effect_stack,
                    &mut self.standings,
                )
            }
            Step::DecideOutcome => vec![Report::GameEnded(self.standings.decide()?)],
        };
        Some(reports)
    }

    fn report(&mut self, report: Report, obs: &mut impl Observer) {
        report.matched(self.cards()).report(
            &mut self.history,
            self.turn_order.turn_count(),
            &self.objects,
            &mut self.effect_stack,
            obs,
        );
    }

    fn cards(&self) -> CardsRef<'_> {
        CardsRef::new(
            &self.binder,
            &self.objects,
            &self.zones,
            &self.history,
            &self.turn_order,
        )
    }
}

/// A game being set up, until its first turn starts.
pub(super) struct Opening {
    game: Game,
}

impl Opening {
    pub(super) fn new(
        rng: Rng,
        turn_order: TurnOrder,
        players: Players,
        zones: Zones,
        objects: ObjectBag,
        binder: Binder,
        standings: Standings,
    ) -> Self {
        Self {
            game: Game {
                rng,
                turn_order,
                players,
                zones,
                player_input: Default::default(),
                objects,
                binder,
                history: History::default(),
                standings,
                effect_stack: Default::default(),
            },
        }
    }

    pub(super) fn deal(&mut self, player_id: PlayerId, count: u8) {
        self.game
            .take_step(Step::Draw { player_id, count }, &mut ());
    }

    pub(super) fn start_first_turn(mut self) -> Game {
        self.game.take_step(Step::StartTurn, &mut ());
        self.game
    }
}

/// A game lent to the code that runs one action.
pub(super) struct Acting<'g, O> {
    game: &'g mut Game,
    obs: &'g mut O,
}

impl<O: Observer> Acting<'_, O> {
    pub(super) fn cards(&self) -> CardsRef<'_> {
        self.game.cards()
    }

    pub(super) fn take_step(&mut self, step: ActionStep) {
        self.game.take_step(Step::Action(step), self.obs);
    }
}

/// A game lent to the resolution loop.
pub(super) struct Resolving<'g, O> {
    game: &'g mut Game,
    obs: &'g mut O,
}

impl<O: Observer> Resolving<'_, O> {
    pub(super) fn any_awaiting(&self) -> bool {
        self.game.player_input.any_awaiting()
    }

    pub(super) fn is_over(&self) -> bool {
        self.game.outcome().is_some()
    }

    pub(super) fn is_resolving(&self) -> bool {
        self.game.effect_stack.is_resolving()
    }

    pub(super) fn is_idle(&self) -> bool {
        self.game.effect_stack.is_idle()
    }

    pub(super) fn start_next(&mut self) {
        self.game.effect_stack.start_next();
    }

    /// Takes the resolving effect's next effect as a step. `false`, with no step taken, if it has
    /// none left.
    pub(super) fn resolve_next_effect(&mut self) -> bool {
        let Some(next) = self.game.effect_stack.next_effect() else {
            return false;
        };
        self.game.take_step(Step::Effect(next), self.obs);
        true
    }

    pub(super) fn finish_if_done(&mut self) {
        self.game.effect_stack.finish_if_done();
    }

    pub(super) fn check_state(&mut self) {
        while self.game.take_step(Step::DeathPass, self.obs) {}
        self.game.take_step(Step::DecideOutcome, self.obs);
    }
}
