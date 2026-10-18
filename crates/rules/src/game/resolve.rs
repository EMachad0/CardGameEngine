//! The mutation half of `Game::apply`. It runs an action and resolves any card
//! the action plays.
//!
//! Every action that reaches this module is already in `legal_actions`, so
//! nothing here re-checks legality.

use crate::{
    action::{Action, IllegalAction},
    cards::{
        definition::{
            CardKind, CharacterKind, CharacterSelector, PlayerSelector, PlayerSelectorFilter,
        },
        filter::fulfill_side_filter,
        modifier::Modifier,
        object::ObjectId,
    },
    effect::{Effect, EffectAmount},
    game::Game,
    history::{HistoryQuery, HistoryQueryKind, PlayerFilter, TurnFilter},
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

    pub(super) fn apply_action(
        &mut self,
        player_id: PlayerId,
        action: Action,
        obs: &mut impl Observer,
    ) {
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

    pub(crate) fn apply_effect(
        &mut self,
        owner: PlayerId,
        source: ObjectId,
        effect: &Effect,
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
            Effect::Reveal {
                pick_id,
                selector,
                amount,
            } => {
                let amount = self.effect_amount(amount, owner);
                let targets = self.resolve_player_selector(owner, selector);
                for target in targets.into_iter() {
                    self.reveal(target, source, *pick_id, amount, obs);
                }
            }
            Effect::ResolvePick {
                pick_id,
                picked_move,
                other_move,
            } => {
                let picks = self
                    .player_input
                    .picks(source, *pick_id)
                    .map(|picks| (picks.picked().to_vec(), picks.others().to_vec()));

                if let Some((picked, others)) = picks {
                    picked
                        .into_iter()
                        .for_each(|id| self.zone_move(owner, id, *picked_move, obs));
                    others
                        .into_iter()
                        .for_each(|id| self.zone_move(owner, id, *other_move, obs));
                }
            }
            Effect::Damage { selector, amount } => {
                let amount = self.effect_amount(amount, owner);
                let targets = self.resolve_character_selector(owner, source, selector);
                for target in targets.into_iter() {
                    self.damage(target, amount, source, obs);
                }
            }
            Effect::Buff { selector, effect } => {
                let targets = self.resolve_character_selector(owner, source, selector);
                for target in targets.into_iter() {
                    let object = self.objects.get_mut(target);
                    object.modifiers.add(Modifier {
                        granted_by: source,
                        effect: effect.clone(),
                    });
                }
            }
            Effect::AddFriendlyAura { selector, effect } => {
                let targets = self.resolve_character_selector(owner, source, selector);
                for target in targets.into_iter() {
                    let object = self.objects.get_mut(target);
                    object.friendly_aura.add(Modifier {
                        granted_by: source,
                        effect: effect.clone(),
                    });
                }
            }
            Effect::Summon { selector, def_id } => {
                let targets = self.resolve_player_selector(owner, selector);
                for player_id in targets.into_iter() {
                    let object_id = self.objects.insert(*def_id, player_id);
                    self.summon(player_id, object_id, obs);
                }
            }
        }
    }

    fn drain_effect_queue(&mut self, obs: &mut impl Observer) {
        while !self.player_input.any_awaiting() && self.outcome().is_none() {
            self.continue_resolving_effect(obs);

            // finished a unit
            if self.resolving_effect.is_none() {
                self.check_state(obs);

                if let Some(pending_effect) = self.effect_stack.pop() {
                    self.resolving_effect = Some(pending_effect.into());
                }
            }

            // no more units
            if self.resolving_effect.is_none() && self.effect_stack.is_empty() {
                break;
            }
        }
    }

    fn continue_resolving_effect(&mut self, obs: &mut impl Observer) {
        while !self.player_input.any_awaiting()
            && let Some(resolving) = &mut self.resolving_effect
            && !resolving.is_finished()
        {
            let effect = resolving
                .curr_effect()
                .expect("An unfinished ResolvingEffect have effects to apply")
                .clone();
            let source = resolving.source;
            resolving.advance();

            let owner = self.objects.get(source).player_id;
            self.apply_effect(owner, source, &effect, obs);
            obs.checkpoint(Views::new(self));
        }
        if let Some(resolving) = &mut self.resolving_effect
            && resolving.is_finished()
        {
            self.resolving_effect = None;
            self.check_state(obs);
        }
    }

    pub(crate) fn effect_amount(&self, amount: &EffectAmount, player_id: PlayerId) -> u8 {
        match amount {
            EffectAmount::Static(value) => *value,
            EffectAmount::History(query) => self.history_query(*query, player_id),
        }
    }

    fn history_query(&self, query: HistoryQuery, player_id: PlayerId) -> u8 {
        let scope = match query.scope {
            PlayerFilter::All => None,
            PlayerFilter::Owner => Some(player_id),
            PlayerFilter::Active => Some(self.turn_order.get_active_player_id()),
        };
        let turn = match query.turn {
            TurnFilter::Current => Some(self.turn_order.turn_count()),
            TurnFilter::All => None,
        };
        let count = match query.kind {
            HistoryQueryKind::SpellsPlayed => self
                .history
                .played(scope, turn)
                .filter(|&object_id| self.card_def(object_id).card_kind() == CardKind::Spell)
                .count(),
            HistoryQueryKind::MinionsDied => self
                .history
                .died(scope, turn)
                .filter(|&object_id| {
                    self.card_def(object_id).character_kind() == Some(CharacterKind::Minion)
                })
                .count(),
        };
        u8::try_from(count).unwrap_or(u8::MAX)
    }

    fn resolve_player_selector(
        &mut self,
        owner: PlayerId,
        selector: &PlayerSelector,
    ) -> Vec<PlayerId> {
        let all = |&PlayerSelectorFilter { side }| {
            self.players
                .iter()
                .map(|p| p.id)
                .filter(|&player_id| fulfill_side_filter(side, owner, player_id))
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
        selector: &CharacterSelector,
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
            CharacterSelector::Targeted(target_id) => self
                .player_input
                .targeted(object_id, *target_id)
                .expect("Target not fulfilled")
                .to_vec(),
        }
    }

    fn select_random<T: Clone>(&mut self, values: &[T]) -> Option<T> {
        (!values.is_empty()).then(|| {
            let idx = self.rng.below(values.len());
            values[idx].clone()
        })
    }
}
