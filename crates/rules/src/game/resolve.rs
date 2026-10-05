//! The mutation half of `Game::apply`. It runs an action and resolves any card
//! the action plays.
//!
//! Every action that reaches this module is already in `legal_actions`, so
//! nothing here re-checks legality.

use super::Game;
use crate::action::Action;
use crate::cards::definition::{CharacterSelector, Effect, PlayerSelector, PlayerSelectorFilter};
use crate::cards::modifier::Modifier;
use crate::choice::{ChoiceTarget, EffectAmount};
use crate::game::PlayerInteractionState;
use crate::game::condition::fulfill_character_side_filter;
use crate::history::{HistoryKind, HistoryQuery, HistoryQueryKind, PlayerFilter};
use crate::ids::PlayerId;
use crate::{Event, IllegalAction, ObjectId, Observer, Views};

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
            self.check_state(obs);
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
                    let def_id = self.def_id(of_object_id);
                    let choice_ids = self
                        .binder
                        .choices(def_id)
                        .iter()
                        .map(|c| c.id)
                        .collect::<Vec<_>>();

                    let obj = self.objects.get_mut(of_object_id);
                    for choice_id in choice_ids {
                        obj.choice_targets.push(ChoiceTarget {
                            choice_id,
                            targets: chosen
                                .iter()
                                .filter_map(|(o, c)| (*c == choice_id).then(|| *o))
                                .collect(),
                        });
                    }
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
            Action::Draft { object_id } => {
                self.get_player_mut(player_id).interaction_state = PlayerInteractionState::Draft {
                    of_object_id: object_id,
                    chosen: Vec::new(),
                }
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
            }
            Action::Cancel { .. } => {
                let player = self.get_player_mut(player_id);
                player.interaction_state = PlayerInteractionState::default();
            }
        };
    }

    pub(crate) fn apply_effect(
        &mut self,
        owner: PlayerId,
        source: ObjectId,
        effect: Effect,
        obs: &mut impl Observer,
    ) {
        match effect {
            Effect::Draw { selector, amount } => {
                let amount = self.effect_amount(amount, owner);
                let targets = self.resolve_player_selector(owner, selector);
                for target in targets.into_iter() {
                    self.draw(target, amount, obs);
                }
            }
            Effect::Reveal { selector, amount } => {
                let amount = self.effect_amount(amount, owner);
                let targets = self.resolve_player_selector(owner, selector);
                for target in targets.into_iter() {
                    self.reveal(target, amount, obs);
                }
            }
            Effect::Damage { selector, amount } => {
                let amount = self.effect_amount(amount, owner);
                let targets = self.resolve_character_selector(owner, source, selector);
                for target in targets.into_iter() {
                    let object = self.objects.get_mut(target);
                    object.damage += amount;
                    obs.event(&Event::Damaged {
                        target,
                        amount,
                        source,
                    });
                }
            }
            Effect::AddFriendlyAura { selector, effect } => {
                let targets = self.resolve_character_selector(owner, source, selector);
                for target in targets.into_iter() {
                    let object = self.objects.get_mut(target);
                    object.friendly_aura.add(Modifier { source, effect });
                }
            }
            Effect::Summon { selector, def_id } => {
                let targets = self.resolve_player_selector(owner, selector);
                for player_id in targets.into_iter() {
                    let object_id = self.objects.insert(def_id, player_id);
                    self.summon(player_id, object_id, obs);
                }
            }
        }
    }

    pub(crate) fn apply_effects(
        &mut self,
        source: ObjectId,
        effects: Vec<Effect>,
        obs: &mut impl Observer,
    ) {
        let owner = self.objects.get(source).player_id;
        effects.into_iter().for_each(|e| {
            self.apply_effect(owner, source, e, obs);
            obs.checkpoint(Views::new(self));
        });
    }

    pub(crate) fn effect_amount(&self, amount: EffectAmount, player_id: PlayerId) -> u8 {
        match amount {
            EffectAmount::Static(value) => value,
            EffectAmount::History(query) => self.history_query(query, player_id),
        }
    }

    pub(crate) fn history_query(&self, query: HistoryQuery, player_id: PlayerId) -> u8 {
        let entries = self.history.entries.iter();
        let targets = match query.scope {
            PlayerFilter::All => self.players.iter().map(|p| p.id).collect(),
            PlayerFilter::Owner => vec![player_id],
            PlayerFilter::Active => vec![self.turn_order.get_active_player_id()],
        };
        let count = match query.kind {
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
                .count(),
            HistoryQueryKind::MinionDied => entries
                .filter(|entry| targets.contains(&entry.player_id))
                .filter(|entry| match query.turn {
                    crate::history::TurnFilter::Current => {
                        entry.turn == self.turn_order.turn_count()
                    }
                    crate::history::TurnFilter::All => true,
                })
                .filter(|entry| matches!(&entry.kind, HistoryKind::CharacterDied { object } if self.binder.is_minion(object.def_id)))
                .count(),
        };
        u8::try_from(count).unwrap_or(u8::MAX)
    }

    fn resolve_player_selector(
        &mut self,
        owner: PlayerId,
        selector: PlayerSelector,
    ) -> Vec<PlayerId> {
        let all = |PlayerSelectorFilter { side }| {
            self.players
                .iter()
                .map(|p| p.id)
                .filter(|&player_id| fulfill_character_side_filter(side, owner, player_id))
                .collect()
        };
        match selector {
            PlayerSelector::All(filter) => all(filter),
            PlayerSelector::Owner => vec![owner],
            PlayerSelector::Random(filter) => match self.select_random(&all(filter)) {
                Some(v) => vec![v],
                None => Vec::new(),
            },
        }
    }

    fn resolve_character_selector(
        &mut self,
        owner: PlayerId,
        object_id: ObjectId,
        selector: CharacterSelector,
    ) -> Vec<ObjectId> {
        match selector {
            CharacterSelector::All(filter) => self.scan_characters_with_filter(owner, filter),
            CharacterSelector::Itself => vec![object_id],
            CharacterSelector::OwnerHero => vec![self.hero_id(owner)],
            CharacterSelector::Random(filter) => {
                match self.select_random(&self.scan_characters_with_filter(owner, filter)) {
                    Some(v) => vec![v],
                    None => Vec::new(),
                }
            }
            CharacterSelector::Chosen(choice_id) => self
                .objects
                .get(object_id)
                .choice_targets
                .iter()
                .find(|c| c.choice_id == choice_id)
                .expect("Choice not fulfilled")
                .targets
                .clone(),
        }
    }

    fn select_random<T: Clone>(&mut self, values: &[T]) -> Option<T> {
        (!values.is_empty()).then(|| {
            let idx = self.rng.below(values.len());
            values[idx].clone()
        })
    }
}
