use crate::cards::modifier::Modifier;
use crate::cards::object::ObjectId;
use crate::effect::{Effect, PickId};
use crate::game::Game;
use crate::game::reporting::Report;
use crate::ids::PlayerId;
use crate::observer::Observer;

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
                let amount = self.cards().effect_amount(amount, owner);
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
                let amount = self.cards().effect_amount(amount, owner);
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
                let amount = self.cards().effect_amount(amount, owner);
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

    fn damage(&mut self, target: ObjectId, amount: u8, source: ObjectId, obs: &mut impl Observer) {
        self.objects.add_damage(target, amount);
        let report = Report::Damaged {
            target,
            amount,
            source,
        };
        self.report(report, obs);
    }
}
