//! Effect resolution: the loop that drains the effect stack, and the
//! interpreter for each effect, its amounts and its selectors.

use crate::{
    cards::{
        amount::EffectAmount,
        effect::Effect,
        selector::{CharacterSelector, PlayerSelector, PlayerSelectorFilter},
    },
    choice::PickTarget,
    game::{Game, PlayerInteractionState, filter::fulfill_side_filter},
    ids::PlayerId,
    object::{Modifier, ObjectId},
    observer::{Observer, Views},
};

impl Game {
    fn apply_effect(
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
                let pick_target = self
                    .objects
                    .get(source)
                    .pick_targets
                    .iter()
                    .find(|pt| pt.pick_id == *pick_id)
                    .cloned();

                if let Some(PickTarget { picked, others, .. }) = pick_target {
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

    pub(super) fn drain_effect_queue(&mut self, obs: &mut impl Observer) {
        while !self.is_pending_interaction() && self.outcome().is_none() {
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
        while !self.is_pending_interaction()
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

    pub(super) fn is_pending_interaction(&self) -> bool {
        self.players().into_iter().any(|p| {
            !matches!(
                self.get_player(p).interaction_state,
                PlayerInteractionState::Idle
            )
        })
    }

    pub(super) fn effect_amount(&self, amount: &EffectAmount, player_id: PlayerId) -> u8 {
        match amount {
            EffectAmount::Static(value) => *value,
            EffectAmount::History(query) => self.history_query(*query, player_id),
        }
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
            CharacterSelector::Chosen(choice_id) => self
                .objects
                .get(object_id)
                .choice_targets
                .iter()
                .find(|c| c.choice_id == *choice_id)
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
