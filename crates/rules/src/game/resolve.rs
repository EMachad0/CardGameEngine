//! The mutation half of `Game::apply`. It runs an action and resolves any card
//! the action plays.
//!
//! Every action that reaches this module is already in `legal_actions`, so
//! nothing here re-checks legality.

use super::Game;
use crate::action::Action;
use crate::cards::definition::{
    CardDefKind, Effect, MonsterCardDef, MonsterTargeteer, PlayerTargeteer,
};
use crate::game::PlayerInteractionState;
use crate::game::lookup::LookupError;
use crate::ids::PlayerId;
use crate::{IllegalAction, ObjectId};

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ApplyError {
    #[error("Lookup: {0}")]
    Lookup(#[from] LookupError),
    #[error("Illegal Action")]
    IllegalAction(IllegalAction),
}

impl Game {
    pub fn applied(&self, player_id: PlayerId, action: Action) -> Result<Self, ApplyError> {
        let mut game = self.clone();
        game.apply(player_id, action)?;
        Ok(game)
    }

    /// `Ok` if and only if `action` is in `legal_actions(player_id)`.
    /// An `Err` leaves the game unchanged.
    pub fn apply(&mut self, player_id: PlayerId, action: Action) -> Result<(), ApplyError> {
        if !self.legal_actions(player_id).contains(&action) {
            Err(ApplyError::IllegalAction(IllegalAction::new(
                player_id, action,
            )))
        } else {
            self.apply_action(player_id, action)?;
            Ok(())
        }
    }

    pub(super) fn apply_action(
        &mut self,
        player_id: PlayerId,
        action: Action,
    ) -> Result<(), ApplyError> {
        match action {
            Action::Play { object_id } => {
                let mana_cost = self.mana_cost(object_id).map_err(ApplyError::from)?.ok_or(
                    ApplyError::IllegalAction(IllegalAction::new(player_id, action)),
                )?;
                self.get_player_mut(player_id)
                    .zones
                    .hand
                    .remove(object_id)
                    .expect("object not in hand");
                self.get_player_mut(player_id).mana -= mana_cost;

                // TODO remove this clone
                let effects = self
                    .on_play_effect(object_id)
                    .expect("unexpected lookup error")
                    .to_vec();

                effects.into_iter().for_each(|e| {
                    self.apply_effect(player_id, e);
                });

                let def_id = self.def_id(object_id).expect("unexpected lookup error");
                let def = self.binder.get(def_id);
                if def.kind.board_presence() {
                    self.get_player_mut(player_id).zones.board.add(object_id);
                }
            }
            Action::Pick { object_id } => {
                let player = self.get_player_mut(player_id);
                let PlayerInteractionState::Picker { mut options } =
                    std::mem::take(&mut player.interaction_state)
                else {
                    unreachable!();
                };
                if let Some(idx) = options.iter().position(|id| *id == object_id) {
                    let picked = options.remove(idx);
                    player.zones.hand.add(picked);
                    options
                        .into_iter()
                        .for_each(|c| player.zones.deck.push_back(c));
                }
            }
            Action::EndTurn => {
                self.turn_order.end_turn();
                self.start_turn();
            }
        };
        Ok(())
    }

    fn apply_effect(&mut self, caster: PlayerId, effect: Effect) {
        match effect {
            Effect::DamagePlayer { targeteer, damage } => {
                let target = self.resolve_player_targeteer(caster, targeteer);
                let player = self.get_player_mut(target);
                player.health -= damage as i32;
            }
            Effect::Draw { targeteer, count } => {
                let target = self.resolve_player_targeteer(caster, targeteer);
                self.draw(target, count);
            }
            Effect::RevealToPicker { targeteer, count } => {
                let target = self.resolve_player_targeteer(caster, targeteer);
                let player = self.get_player_mut(target);

                let mut options = Vec::new();
                for _ in 0..count {
                    let Some(card) = player.zones.deck.pop_front() else {
                        break;
                    };

                    options.push(card);
                }
                if !options.is_empty() {
                    player.interaction_state = PlayerInteractionState::Picker { options }
                }
            }
            Effect::DamageMonster { targeteer, damage } => {
                let targets = self
                    .resolve_monster_targeteer(caster, targeteer)
                    .into_iter()
                    .copied()
                    .collect::<Vec<_>>();
                for target in targets.into_iter() {
                    if let Some(object) = self.objects.get_mut(target) {
                        object.damage += damage;
                    }
                }
            }
        }
    }

    fn resolve_player_targeteer(
        &mut self,
        caster: PlayerId,
        targeteer: PlayerTargeteer,
    ) -> PlayerId {
        match targeteer {
            PlayerTargeteer::Caster => caster,
            PlayerTargeteer::RandomPlayer => PlayerId::new(self.rng.below(self.players.len())),
            PlayerTargeteer::NextPlayer => self.turn_order.get_player_after(caster),
        }
    }

    fn resolve_monster_targeteer(
        &mut self,
        _caster: PlayerId,
        targeteer: MonsterTargeteer,
    ) -> Vec<&ObjectId> {
        match targeteer {
            MonsterTargeteer::All => self
                .players
                .iter()
                .flat_map(|player| player.zones.board.as_slice())
                .collect(),
        }
    }

    /// Moves the top card to the end of the hand, `count` times.
    /// Each draw from an empty deck costs 1 health instead.
    pub(crate) fn draw(&mut self, player_id: PlayerId, count: usize) {
        let player = self.get_player_mut(player_id);
        for _ in 0..count {
            if let Some(card) = player.zones.deck.pop_front() {
                player.zones.hand.add(card);
            } else {
                player.health -= 1;
            };
        }
    }
}
