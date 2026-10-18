use crate::{
    cards::{
        definition::{CharacterSelector, PlayerSelector, PlayerSelectorFilter},
        filter::fulfill_side_filter,
        modifier::Modifier,
        object::ObjectId,
    },
    effect::{Effect, PickId},
    game::{Game, reporting::Report},
    ids::PlayerId,
    observer::Observer,
};

impl Game {
    pub(super) fn apply_effect(
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
                    let modifier = Modifier {
                        granted_by: source,
                        effect: effect.clone(),
                    };
                    self.objects.grant(target, modifier);
                }
            }
            Effect::AddFriendlyAura { selector, effect } => {
                let targets = self.resolve_character_selector(owner, source, selector);
                for target in targets.into_iter() {
                    let modifier = Modifier {
                        granted_by: source,
                        effect: effect.clone(),
                    };
                    self.objects.grant_friendly_aura(target, modifier);
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

    fn reveal(
        &mut self,
        player_id: PlayerId,
        source: ObjectId,
        pick_id: PickId,
        count: u8,
        obs: &mut impl Observer,
    ) {
        #[expect(
            clippy::disallowed_methods,
            reason = "the one reveal, which posts the revealed cards as a pending pick"
        )]
        let options = self.zones.reveal(player_id, count);
        let report = Report::Revealed {
            player_id,
            object_ids: options.clone(),
        };
        if self
            .player_input
            .ask_pick(player_id, source, pick_id, options)
        {
            self.report(report, obs);
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
