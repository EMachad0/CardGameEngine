//! The mutation half of `Game::apply`. It runs an action and resolves any card
//! the action plays.
//!
//! Every action that reaches this module is already in `legal_actions`, so
//! nothing here re-checks legality.

use super::Game;
use crate::action::Action;
use crate::cards::definition::{Effect, MonsterTargeteer, PlayerTargeteer};
use crate::cards::object::Modifier;
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
            self.update();
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
                self.get_player_mut(player_id).mana -= mana_cost;
                self.play(player_id, object_id);
            }
            Action::Pick { object_id } => {
                self.pick(player_id, object_id);
            }
            Action::EndTurn => {
                self.turn_order.end_turn();
                self.start_turn();
            }
        };
        Ok(())
    }

    pub(crate) fn apply_effect(&mut self, caster: PlayerId, object_id: ObjectId, effect: Effect) {
        match effect {
            Effect::DamagePlayer { targeteer, damage } => {
                let targets = self.resolve_player_targeteer(caster, targeteer);
                for target in targets.into_iter() {
                    let player = self.get_player_mut(target);
                    player.health -= damage as i32;
                }
            }
            Effect::Draw { targeteer, count } => {
                let targets = self.resolve_player_targeteer(caster, targeteer);
                for target in targets.into_iter() {
                    self.draw(target, count);
                }
            }
            Effect::Reveal { targeteer, count } => {
                let targets = self.resolve_player_targeteer(caster, targeteer);
                for target in targets.into_iter() {
                    self.reveal(target, count);
                }
            }
            Effect::DamageMonster { targeteer, damage } => {
                let targets = self.resolve_monster_targeteer(caster, object_id, targeteer);
                for target in targets.into_iter() {
                    if let Some(object) = self.objects.get_mut(target) {
                        object.damage += damage;
                    }
                }
            }
            Effect::AddFriendlyAura { targeteer, effect } => {
                let targets = self.resolve_monster_targeteer(caster, object_id, targeteer);
                for target in targets.into_iter() {
                    if let Some(object) = self.objects.get_mut(target) {
                        object.friendly_aura.add(Modifier {
                            source: object_id,
                            effect,
                        });
                    }
                }
            }
        }
    }

    pub(crate) fn apply_effects(
        &mut self,
        player_id: PlayerId,
        object_id: ObjectId,
        effects: Vec<Effect>,
    ) {
        effects.into_iter().for_each(|e| {
            self.apply_effect(player_id, object_id, e);
        });
    }

    fn resolve_player_targeteer(
        &mut self,
        caster: PlayerId,
        targeteer: PlayerTargeteer,
    ) -> Vec<PlayerId> {
        match targeteer {
            PlayerTargeteer::All => self.players.iter().map(|p| p.id).collect(),
            PlayerTargeteer::Caster => vec![caster],
            PlayerTargeteer::RandomPlayer => {
                vec![PlayerId::new(self.rng.below(self.players.len()))]
            }
            PlayerTargeteer::NextPlayer => vec![self.turn_order.get_player_after(caster)],
        }
    }

    fn resolve_monster_targeteer(
        &mut self,
        _caster: PlayerId,
        object_id: ObjectId,
        targeteer: MonsterTargeteer,
    ) -> Vec<ObjectId> {
        match targeteer {
            MonsterTargeteer::All => self
                .players
                .iter()
                .flat_map(|player| player.zones.board.as_slice())
                .copied()
                .collect(),
            MonsterTargeteer::Itself => vec![object_id],
        }
    }
}
