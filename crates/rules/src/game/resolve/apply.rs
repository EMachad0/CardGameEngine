//! The mutation half of `Game::apply`. It runs an action and resolves any card
//! the action plays.
//!
//! Every action that reaches this module is already in `legal_actions`, so
//! nothing here re-checks legality.

use crate::{
    action::{Action, IllegalAction},
    game::{Game, effect_stack::NextEffect},
    ids::PlayerId,
    observer::{Observer, Views},
};

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
                        .card_def(targeting.object_id())
                        .targets
                        .iter()
                        .map(|t| t.id)
                        .collect::<Vec<_>>();
                    self.player_input.commit_targeting(player_id, &target_ids);
                }

                let mana_cost = self
                    .mana_cost(object_id)
                    .expect("[legal_actions] guarantees a play action object_id has mana");
                self.get_player_mut(player_id).mana -= mana_cost;
                self.play(player_id, object_id, obs);
            }
            Action::Pick { object_id } => {
                self.pick(player_id, object_id, obs);
                obs.checkpoint(Views::new(self));
            }
            Action::EndTurn => {
                self.end_turn(obs);
                self.start_turn(obs);
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
        if self.effect_stack.finish_if_done() {
            self.check_state(obs);
        }
    }
}
