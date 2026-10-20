//! The mutation half of `Game::apply`. It runs an action and resolves any card
//! the action plays.
//!
//! `apply` checks legality once, and nothing after its check re-checks it.

use crate::action::{Action, IllegalAction};
use crate::cards::object::ObjectId;
use crate::game::effect_stack::NextEffect;
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
    pub fn applied(&self, player_id: PlayerId, action: Action) -> Result<Self, ApplyError> {
        let mut game = self.clone();
        game.apply(player_id, action, &mut ())?;
        Ok(game)
    }

    /// `Ok` if and only if `action` is in `legal_actions(player_id)`.
    /// An `Err` leaves the game unchanged.
    pub fn apply(
        &mut self,
        player_id: PlayerId,
        action: Action,
        obs: &mut impl Observer,
    ) -> Result<(), ApplyError> {
        if !self.legal_actions(player_id).contains(&action) {
            Err(ApplyError::IllegalAction(IllegalAction::new(
                player_id, action,
            )))
        } else {
            self.apply_action(player_id, action, obs);
            self.drain_effect_queue(obs);
            Ok(())
        }
    }

    fn apply_action(&mut self, player_id: PlayerId, action: Action, obs: &mut impl Observer) {
        match action {
            Action::Play { object_id } => {
                if let Some(targeting) = self.player_input.targeting(player_id) {
                    let target_ids = self
                        .cards()
                        .card_def(targeting.object_id())
                        .targets
                        .iter()
                        .map(|t| t.id)
                        .collect::<Vec<_>>();
                    self.player_input.commit_targeting(player_id, &target_ids);
                }

                self.play(player_id, object_id, obs);
            }
            Action::Pick { object_id } => {
                self.pick(player_id, object_id, obs);
                obs.checkpoint(Views::new(self));
            }
            Action::EndTurn => {
                self.end_turn(obs);
                obs.checkpoint(Views::new(self));
            }
            Action::StartTargeting { object_id } => {
                self.player_input.start_targeting(player_id, object_id);
                obs.checkpoint(Views::new(self));
            }
            Action::Target {
                target_id,
                object_id,
            } => {
                self.player_input.target(player_id, object_id, target_id);
                obs.checkpoint(Views::new(self));
            }
            Action::Cancel { .. } => {
                self.player_input.cancel(player_id);
                obs.checkpoint(Views::new(self));
            }
        };
    }

    fn pick(&mut self, player_id: PlayerId, object_id: ObjectId, obs: &mut impl Observer) {
        self.player_input.pick(player_id, object_id);

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
            && let Some(NextEffect { source, effect }) = self.effect_stack.next_effect()
        {
            let owner = self.objects.get(source).player_id;
            self.apply_effect(owner, source, &effect, obs);
            obs.checkpoint(Views::new(self));
        }
        self.effect_stack.finish_if_done();
    }
}
