//! The mutation half of `Game::apply`. It runs an action and resolves any card
//! the action plays.
//!
//! `apply` checks legality once, then runs the legal action the check built.

use crate::action::{Action, IllegalAction};
use crate::game::effect_stack::NextEffect;
use crate::game::legal_actions::LegalAction;
use crate::game::player_input::WasAwaitingPicking;
use crate::game::reporting::Report;
use crate::game::state::Game;
use crate::ids::PlayerId;
use crate::observer::{Observer, Views};

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ApplyError {
    #[error("Illegal Action")]
    IllegalAction(IllegalAction),
}

impl Game {
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
        self.apply_action(legal_action, obs);
        self.drain_effect_queue(obs);
        Ok(())
    }

    fn apply_action(&mut self, legal_action: LegalAction, obs: &mut impl Observer) {
        match legal_action {
            LegalAction::Play(playable) => {
                self.play(playable, obs);
            }
            LegalAction::Pick(awaiting_picking) => {
                self.pick(awaiting_picking, obs);
                obs.checkpoint(Views::new(self));
            }
            LegalAction::EndTurn => {
                self.end_turn(obs);
                obs.checkpoint(Views::new(self));
            }
            LegalAction::StartTargeting {
                object_id,
                awaiting_nothing,
            } => {
                self.player_input
                    .start_targeting(awaiting_nothing, object_id);
                obs.checkpoint(Views::new(self));
            }
            LegalAction::Target {
                awaiting_targeting,
                target_id,
                object_id,
            } => {
                self.player_input
                    .target(awaiting_targeting, object_id, target_id);
                obs.checkpoint(Views::new(self));
            }
            LegalAction::Cancel(awaiting_targeting) => {
                self.player_input.cancel(awaiting_targeting);
                obs.checkpoint(Views::new(self));
            }
        };
    }

    fn pick(&mut self, awaiting_picking: WasAwaitingPicking, obs: &mut impl Observer) {
        let (player_id, object_id) = self.player_input.pick(awaiting_picking);

        let report = Report::Picked {
            player_id,
            object_id,
        };
        self.report(report, obs);
    }

    fn drain_effect_queue(&mut self, obs: &mut impl Observer) {
        while !self.player_input.any_awaiting() && self.outcome().is_none() {
            self.continue_resolving_effect(obs);

            if !self.effect_stack.is_resolving() {
                self.check_state(obs);
                self.effect_stack.start_next();
            }

            if self.effect_stack.is_idle() {
                break;
            }
        }
    }

    fn continue_resolving_effect(&mut self, obs: &mut impl Observer) {
        while !self.player_input.any_awaiting()
            && let Some(next) = self.effect_stack.next_effect()
        {
            match next {
                NextEffect::Played { source, effect } => self.apply_effect(source, &effect, obs),
                NextEffect::Triggered { source, effect } => self.apply_effect(source, &effect, obs),
            }
            obs.checkpoint(Views::new(self));
        }
        self.effect_stack.finish_if_done();
    }
}
