use crate::cards::definition::TriggerDef;
use crate::cards::object::ObjectId;
use crate::effect::EffectSequence;
use crate::event::Event;
use crate::game::cards::Cards;
use crate::game::modifiers::modifiers;
use crate::ids::PlayerId;
use crate::trigger::{Trigger, TriggerPattern, TriggerSubject};
use crate::zones::Zones;

pub(super) struct MatchedTrigger {
    pub(super) source: ObjectId,
    pub(super) effects: EffectSequence,
}

pub(super) fn matched_triggers(cards: Cards<'_>, event: &Event) -> Vec<MatchedTrigger> {
    let mut matched = Vec::new();
    for object_id in characters_in_order_of_play(cards.zones).into_iter() {
        let player_id = cards.objects.get(object_id).player_id;
        for Trigger {
            def: TriggerDef { pattern, effects },
            ..
        } in triggers(cards, object_id)
        {
            if fulfill_trigger_pattern(cards, player_id, object_id, pattern, event) {
                matched.push(MatchedTrigger {
                    source: object_id,
                    effects,
                });
            }
        }
    }
    matched
}

pub(super) fn death_rattles(cards: Cards<'_>, object_id: ObjectId) -> Vec<MatchedTrigger> {
    triggers(cards, object_id)
        .into_iter()
        .filter(|t| matches!(t.def.pattern, TriggerPattern::Died(TriggerSubject::Itself)))
        .map(|t| MatchedTrigger {
            source: object_id,
            effects: t.def.effects,
        })
        .collect()
}

fn characters_in_order_of_play(zones: &Zones) -> Vec<ObjectId> {
    let mut characters = zones.characters_in_play().collect::<Vec<_>>();
    characters.sort_by_key(|&o| zones.match_order(o));
    characters
}

fn triggers(cards: Cards<'_>, object_id: ObjectId) -> Vec<Trigger> {
    let modifiers = modifiers(cards, object_id);
    let granted = modifiers
        .granted_triggers()
        .map(|(granted_by, def)| Trigger {
            granted_by,
            def: def.clone(),
        });
    let printed = cards
        .card_def(object_id)
        .triggers
        .iter()
        .map(|def| Trigger {
            granted_by: object_id,
            def: def.clone(),
        });
    let mut triggers = granted.chain(printed).collect::<Vec<_>>();
    triggers.sort_by_key(|t| cards.zones.grant_order(t.granted_by));
    triggers
}

fn fulfill_trigger_pattern(
    cards: Cards<'_>,
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
            fulfill_trigger_subject(cards, subject, trigger_owner, trigger_object, *object_id)
        }
        TriggerPattern::Played(card_filter) => {
            let Event::Played {
                player_id,
                object_id,
            } = event
            else {
                return false;
            };
            let kind = cards.card_def(*object_id).card_kind();
            card_filter.passes(trigger_owner, *player_id, kind)
        }
        TriggerPattern::Damaged(subject) => {
            let Event::Damaged { target, .. } = event else {
                return false;
            };
            fulfill_trigger_subject(cards, subject, trigger_owner, trigger_object, *target)
        }
    }
}

fn fulfill_trigger_subject(
    cards: Cards<'_>,
    subject: TriggerSubject,
    trigger_owner: PlayerId,
    trigger_object: ObjectId,
    event_object: ObjectId,
) -> bool {
    match subject {
        TriggerSubject::Itself => trigger_object == event_object,
        TriggerSubject::Character(character_filter) => {
            cards.object_passes(event_object, trigger_owner, &character_filter)
        }
    }
}
