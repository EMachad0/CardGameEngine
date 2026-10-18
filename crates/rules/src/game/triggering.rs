use crate::{
    cards::{definition::TriggerDef, object::ObjectId},
    effect::EffectSequence,
    event::Event,
    game::Game,
    ids::PlayerId,
    trigger::Trigger,
    trigger::{TriggerPattern, TriggerSubject},
};

pub(super) struct MatchedTrigger {
    pub(super) source: ObjectId,
    pub(super) effects: EffectSequence,
}

impl Game {
    pub(super) fn matched_triggers(&self, event: &Event) -> Vec<MatchedTrigger> {
        let mut matched = Vec::new();
        for object_id in self.characters_in_order_of_play().into_iter() {
            let player_id = self.objects.get(object_id).player_id;
            for Trigger {
                def: TriggerDef { pattern, effects },
                ..
            } in self.triggers(object_id)
            {
                if self.fulfill_trigger_pattern(player_id, object_id, pattern, event) {
                    matched.push(MatchedTrigger {
                        source: object_id,
                        effects,
                    });
                }
            }
        }
        matched
    }

    pub(super) fn death_rattles(&self, object_id: ObjectId) -> Vec<MatchedTrigger> {
        self.triggers(object_id)
            .into_iter()
            .filter(|t| matches!(t.def.pattern, TriggerPattern::Died(TriggerSubject::Itself)))
            .map(|t| MatchedTrigger {
                source: object_id,
                effects: t.def.effects,
            })
            .collect()
    }

    fn fulfill_trigger_pattern(
        &self,
        trigger_owner: PlayerId,
        trigger_object: ObjectId,
        pattern: TriggerPattern,
        event: &Event,
    ) -> bool {
        match pattern {
            TriggerPattern::Died(subject) => {
                let Event::Died { object_id } = event else {
                    return false;
                };
                self.fulfill_trigger_subject(subject, trigger_owner, trigger_object, *object_id)
            }
            TriggerPattern::Played(card_filter) => {
                let Event::Played {
                    player_id,
                    object_id,
                } = event
                else {
                    return false;
                };
                self.fulfill_card_filter(trigger_owner, *player_id, *object_id, &card_filter)
            }
            TriggerPattern::Damaged(subject) => {
                let Event::Damaged { target, .. } = event else {
                    return false;
                };
                self.fulfill_trigger_subject(subject, trigger_owner, trigger_object, *target)
            }
        }
    }

    fn fulfill_trigger_subject(
        &self,
        subject: TriggerSubject,
        trigger_owner: PlayerId,
        trigger_object: ObjectId,
        event_object: ObjectId,
    ) -> bool {
        match subject {
            TriggerSubject::Itself => trigger_object == event_object,
            TriggerSubject::Character(character_filter) => {
                let owner = self.objects.get(event_object).player_id;
                self.fulfill_character_filter(trigger_owner, owner, event_object, &character_filter)
            }
        }
    }
}
