//! Legality and dispatch. `legal_actions` is the one definition of legality, and
//! `apply_action` re-checks nothing.

use crate::{
    action::{Action, ApplyError, IllegalAction},
    choice::ChoiceTarget,
    game::{Game, PlayerInteractionState},
    ids::PlayerId,
    observer::{Observer, Views},
};

impl Game {
    pub fn legal_actions(&self, player_id: PlayerId) -> Vec<Action> {
        if self.outcome().is_some() {
            return Vec::new();
        }

        let mut actions = Vec::new();
        match &self.get_player(player_id).interaction_state {
            PlayerInteractionState::Idle => {
                if self.turn_order.get_active_player_id() != player_id
                    || self.is_pending_interaction()
                {
                    return Vec::new();
                }

                let mana = self.mana(player_id);
                let hand = self.hand(player_id);
                for object_id in hand.iter().copied() {
                    let mana_cost = self
                        .mana_cost(object_id)
                        .expect("objects in hand always have mana cost");
                    if mana_cost > mana {
                        continue;
                    }

                    match self.card_def(object_id).choices.is_empty() {
                        true => {
                            actions.push(Action::Play { object_id });
                        }
                        false => {
                            if self.can_finish_choices(object_id, &mut Vec::new()) {
                                actions.push(Action::Draft { object_id });
                            }
                        }
                    }
                }

                actions.push(Action::EndTurn);
            }
            PlayerInteractionState::PendingPick { options, .. } => {
                actions.extend(
                    options
                        .iter()
                        .cloned()
                        .map(|object_id| Action::Pick { object_id }),
                );
            }
            PlayerInteractionState::Draft {
                of_object_id,
                chosen,
            } => {
                let mut all_choices_fulfilled = true;
                for choice in &self.card_def(*of_object_id).choices {
                    let chosen_for_choice_count = chosen
                        .iter()
                        .filter(|(_, choice_id)| choice.id == *choice_id)
                        .count();
                    if self.check_choice_count_fulfilled(
                        *of_object_id,
                        choice.id,
                        chosen_for_choice_count as u8,
                    ) {
                        continue;
                    }

                    let candidates = self.scan_for_valid_choices(*of_object_id, choice, chosen);
                    for candidate in candidates.into_iter() {
                        let mut chosen = chosen.clone();
                        chosen.push((candidate, choice.id));
                        if self.can_finish_choices(*of_object_id, &mut chosen) {
                            actions.push(Action::Choose {
                                choice_id: choice.id,
                                object_id: candidate,
                            })
                        }
                    }
                    all_choices_fulfilled = false;
                    break;
                }
                if all_choices_fulfilled {
                    actions.push(Action::Play {
                        object_id: *of_object_id,
                    });
                }
                actions.push(Action::Cancel {
                    object_id: *of_object_id,
                });
            }
        }
        actions
    }

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
            self.resolve_effect_stack(obs);
            Ok(())
        }
    }

    fn apply_action(&mut self, player_id: PlayerId, action: Action, obs: &mut impl Observer) {
        match action {
            Action::Play { object_id } => {
                let from_draft = matches!(
                    self.get_player(player_id).interaction_state,
                    PlayerInteractionState::Draft { .. }
                );
                if from_draft {
                    let PlayerInteractionState::Draft {
                        of_object_id,
                        chosen,
                    } = std::mem::take(&mut self.get_player_mut(player_id).interaction_state)
                    else {
                        unreachable!();
                    };
                    let choice_ids = self
                        .card_def(of_object_id)
                        .choices
                        .iter()
                        .map(|c| c.id)
                        .collect::<Vec<_>>();

                    let obj = self.objects.get_mut(of_object_id);
                    obj.choice_targets.clear();
                    for choice_id in choice_ids {
                        obj.choice_targets.push(ChoiceTarget {
                            choice_id,
                            targets: chosen
                                .iter()
                                .filter(|(_, c)| *c == choice_id)
                                .map(|(o, _)| *o)
                                .collect(),
                        });
                    }
                }

                let mana_cost = self
                    .mana_cost(object_id)
                    .expect("legal_actions lists Play only for a card in hand, which has a cost");
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
            Action::Draft { object_id } => {
                self.get_player_mut(player_id).interaction_state = PlayerInteractionState::Draft {
                    of_object_id: object_id,
                    chosen: Vec::new(),
                };
                obs.checkpoint(Views::new(self));
            }
            Action::Choose {
                choice_id,
                object_id,
            } => {
                let player = self.get_player_mut(player_id);
                let PlayerInteractionState::Draft { chosen, .. } = &mut player.interaction_state
                else {
                    unreachable!();
                };
                chosen.push((object_id, choice_id));
                obs.checkpoint(Views::new(self));
            }
            Action::Cancel { .. } => {
                let player = self.get_player_mut(player_id);
                player.interaction_state = PlayerInteractionState::default();
                obs.checkpoint(Views::new(self));
            }
        };
    }
}
