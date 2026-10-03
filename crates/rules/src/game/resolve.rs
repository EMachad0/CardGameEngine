//! The mutation half of `Game::apply`. It runs an action and resolves any card
//! the action plays.
//!
//! Every action that reaches this module is already in `legal_actions`, so
//! nothing here re-checks legality.

use super::Game;
use crate::action::Action;
use crate::cards::definition::{Effect, MinionSelector, PlayerSelector};
use crate::cards::modifier::{EffectAmount, Modifier};
use crate::history::{HistoryKind, HistoryQuery, HistoryQueryKind, PlayerFilter};
use crate::ids::PlayerId;
use crate::{Event, IllegalAction, ObjectId, Observer, Target, Views};

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
            self.update(obs);
            Ok(())
        }
    }

    pub(super) fn apply_action(
        &mut self,
        player_id: PlayerId,
        action: Action,
        obs: &mut impl Observer,
    ) {
        match action {
            Action::Play { object_id } => {
                let mana_cost = self
                    .mana_cost(object_id)
                    .expect("[legal_actions] guarantees a play action object_id has mana");
                self.get_player_mut(player_id).mana -= mana_cost;
                self.play(player_id, object_id, obs);
            }
            Action::Pick { object_id } => {
                self.pick(player_id, object_id, obs);
            }
            Action::EndTurn => {
                self.end_turn(obs);
                self.start_turn(obs);
            }
        };
        obs.checkpoint(Views::new(self));
    }

    pub(crate) fn apply_effect(
        &mut self,
        caster: PlayerId,
        object_id: ObjectId,
        effect: Effect,
        obs: &mut impl Observer,
    ) {
        match effect {
            Effect::DamagePlayer { selector, damage } => {
                let targets = self.resolve_player_selector(caster, selector);
                for target in targets.into_iter() {
                    let player = self.get_player_mut(target);
                    player.health -= damage as i32;
                    obs.event(&Event::Damaged {
                        target: Target::Hero(target),
                        amount: damage,
                        source: object_id,
                    });
                }
            }
            Effect::Draw { selector, count } => {
                let targets = self.resolve_player_selector(caster, selector);
                for target in targets.into_iter() {
                    self.draw(target, count, obs);
                }
            }
            Effect::Reveal { selector, count } => {
                let targets = self.resolve_player_selector(caster, selector);
                for target in targets.into_iter() {
                    self.reveal(target, count, obs);
                }
            }
            Effect::DamageMinion { selector, damage } => {
                let targets = self.resolve_minion_selector(caster, object_id, selector);
                for target in targets.into_iter() {
                    let object = self.objects.get_mut(target);
                    object.damage += damage;
                    obs.event(&Event::Damaged {
                        target: Target::Minion(target),
                        amount: damage,
                        source: object_id,
                    });
                }
            }
            Effect::AddFriendlyAura { selector, effect } => {
                let targets = self.resolve_minion_selector(caster, object_id, selector);
                for target in targets.into_iter() {
                    let object = self.objects.get_mut(target);
                    object.friendly_aura.add(Modifier {
                        source: object_id,
                        effect,
                    });
                }
            }
        }
    }

    pub(crate) fn apply_effects(
        &mut self,
        player_id: PlayerId,
        object_id: ObjectId,
        effects: Vec<Effect>,
        obs: &mut impl Observer,
    ) {
        effects.into_iter().for_each(|e| {
            self.apply_effect(player_id, object_id, e, obs);
        });
    }

    pub(crate) fn effect_amount(&self, amount: EffectAmount, player_id: PlayerId) -> i32 {
        match amount {
            EffectAmount::Static(value) => value,
            EffectAmount::History(query) => self.history_query(query, player_id),
        }
    }

    pub(crate) fn history_query(&self, query: HistoryQuery, player_id: PlayerId) -> i32 {
        let entries = self.history.entries.iter();
        let targets = match query.scope {
            PlayerFilter::All => self.players.iter().map(|p| p.id).collect(),
            PlayerFilter::Owner => vec![player_id],
            PlayerFilter::Current => vec![self.turn_order.get_current_player_id()],
        };
        match query.kind {
            HistoryQueryKind::SpellsPlayed => entries
                .filter(|entry| targets.contains(&entry.player_id))
                .filter(|entry| match query.turn {
                    crate::history::TurnFilter::Current => {
                        entry.turn == self.turn_order.turn_count()
                    }
                    crate::history::TurnFilter::All => true,
                })
                .filter(|entry| {
                    matches!(
                        &entry.kind,
                        HistoryKind::CardPlayed { object } if self.binder.is_spell(object.def_id)
                    )
                })
                .count() as i32,
            HistoryQueryKind::MinionDied => entries
                .filter(|entry| matches!(&entry.kind, HistoryKind::MinionDied { .. }))
                .count() as i32,
        }
    }

    fn resolve_player_selector(
        &mut self,
        caster: PlayerId,
        selector: PlayerSelector,
    ) -> Vec<PlayerId> {
        match selector {
            PlayerSelector::All => self.players.iter().map(|p| p.id).collect(),
            PlayerSelector::Caster => vec![caster],
            PlayerSelector::RandomPlayer => {
                vec![PlayerId::new(self.rng.below(self.players.len()))]
            }
            PlayerSelector::NextPlayer => vec![self.turn_order.get_player_after(caster)],
        }
    }

    fn resolve_minion_selector(
        &mut self,
        _caster: PlayerId,
        object_id: ObjectId,
        selector: MinionSelector,
    ) -> Vec<ObjectId> {
        match selector {
            MinionSelector::All => self
                .players
                .iter()
                .flat_map(|player| player.zones.board.as_slice())
                .copied()
                .collect(),
            MinionSelector::Itself => vec![object_id],
        }
    }
}
