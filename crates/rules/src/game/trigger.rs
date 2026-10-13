use crate::{
    cards::trigger::{TriggerDef, TriggerPattern, TriggerSubject},
    event::Event,
    game::Game,
    ids::PlayerId,
    object::ObjectId,
    observer::Observer,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Trigger {
    granted_by: ObjectId,
    pub(super) def: TriggerDef,
}

impl Game {
    pub(super) fn emit(&mut self, event: Event, obs: &mut impl Observer) {
        self.history_from_event(&event);
        #[expect(
            clippy::disallowed_methods,
            reason = "the one place that reports an event"
        )]
        obs.event(&event);
        for object_id in self.board_characters().into_iter() {
            let obj = self.objects.get(object_id);
            self.push_matched_triggers(obj.player_id, object_id, &event);
        }
    }

    fn push_matched_triggers(&mut self, player_id: PlayerId, object_id: ObjectId, event: &Event) {
        for Trigger {
            def: TriggerDef { pattern, effects },
            ..
        } in self.triggers(object_id)
        {
            if self.fulfill_trigger_pattern(player_id, object_id, pattern, event) {
                self.effect_stack.push(object_id, effects);
            }
        }
    }

    fn fulfill_trigger_pattern(
        &self,
        trigger_owner: PlayerId,
        trigger_object: ObjectId,
        pattern: TriggerPattern,
        event: &Event,
    ) -> bool {
        match (pattern, event) {
            (TriggerPattern::Died(subject), Event::Died { object_id }) => {
                self.fulfill_trigger_subject(subject, trigger_owner, trigger_object, *object_id)
            }
            (
                TriggerPattern::Played(card_filter),
                Event::Played {
                    player_id,
                    object_id,
                },
            ) => self.fulfill_card_filter(trigger_owner, *player_id, *object_id, &card_filter),
            (TriggerPattern::Damaged(subject), Event::Damaged { target, .. }) => {
                self.fulfill_trigger_subject(subject, trigger_owner, trigger_object, *target)
            }
            (_, _) => false,
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

    pub(super) fn death_rattles(&self, object_id: ObjectId) -> Vec<Trigger> {
        self.triggers(object_id)
            .into_iter()
            .filter(|t| matches!(t.def.pattern, TriggerPattern::Died(TriggerSubject::Itself)))
            .collect()
    }

    fn triggers(&self, object_id: ObjectId) -> Vec<Trigger> {
        let printed = self.card_def(object_id).triggers.iter().map(|td| Trigger {
            granted_by: object_id,
            def: td.clone(),
        });
        let modifiers = self.modifiers(object_id);
        let from_modifiers = modifiers
            .granted_triggers()
            .map(|(granted_by, def)| Trigger {
                granted_by,
                def: def.clone(),
            });
        let mut triggers = from_modifiers.chain(printed).collect::<Vec<_>>();
        triggers.sort_by_key(|t| self.objects.get(t.granted_by).order);
        triggers
    }
}
