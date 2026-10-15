use crate::{
    cards::{definition::TriggerDef, object::ObjectId},
    effect::PendingEffect,
    event::Event,
    game::Game,
    history::{HistoryEntry, HistoryKind},
    ids::PlayerId,
    observer::Observer,
    trigger::Trigger,
    trigger::{TriggerPattern, TriggerSubject},
};

impl Game {
    pub(crate) fn emit(&mut self, event: Event, obs: &mut impl Observer) {
        self.history_from_event(&event);
        #[expect(
            clippy::disallowed_methods,
            reason = "the one place that reports an event"
        )]
        obs.event(&event);
        for object_id in self.characters_in_order_of_play().into_iter() {
            let obj = self.objects.get(object_id);
            self.enqueue_triggers(obj.player_id, object_id, &event);
        }
    }

    pub(super) fn enqueue_triggers(
        &mut self,
        player_id: PlayerId,
        object_id: ObjectId,
        event: &Event,
    ) {
        for Trigger {
            def: TriggerDef { pattern, effects },
            ..
        } in self.triggers(object_id)
        {
            if self.fulfill_trigger_pattern(player_id, object_id, pattern, event) {
                self.effect_stack.push(PendingEffect {
                    source: object_id,
                    effects,
                });
            }
        }
    }

    pub(super) fn fulfill_trigger_pattern(
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

    fn history_from_event(&mut self, event: &Event) {
        let turn = self.turn_order.turn_count();
        if let Some(entry) = match event {
            Event::Died { object_id } => Some(HistoryEntry {
                player_id: self.objects.get(*object_id).player_id,
                kind: HistoryKind::CharacterDied {
                    object: self.objects.get(*object_id).clone(),
                },
                turn,
            }),
            Event::Played {
                player_id,
                object_id,
            } => Some(HistoryEntry {
                player_id: *player_id,
                kind: HistoryKind::CardPlayed {
                    object: self.objects.get(*object_id).clone(),
                },
                turn,
            }),
            Event::BoardEntered {
                player_id,
                object_id,
            } => Some(HistoryEntry {
                player_id: *player_id,
                kind: HistoryKind::CharacterEntered {
                    object: self.objects.get(*object_id).clone(),
                },
                turn,
            }),
            _ => None,
        } {
            self.history.entries.push(entry);
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
