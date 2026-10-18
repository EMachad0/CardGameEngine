//! Reporting an event: it is recorded in history and sent to the observer, and the triggers it
//! matched wait on the effect stack. A guarded event's variant holds a value with a private
//! field, so only the module that defines that value can report the event.

use crate::{
    cards::object::{ObjectBag, ObjectId},
    event::Event,
    game::{
        Game,
        effect_stack::EffectStack,
        state_check::{Death, Ending},
        triggering::MatchedTrigger,
        zone_move::Play,
    },
    history::History,
    ids::PlayerId,
    observer::Observer,
};

pub(super) enum Report {
    TurnStarted {
        player_id: PlayerId,
    },
    TurnEnded {
        player_id: PlayerId,
    },
    Drew {
        player_id: PlayerId,
        object_id: ObjectId,
    },
    Played(Play),
    Damaged {
        target: ObjectId,
        amount: u8,
        source: ObjectId,
    },
    Revealed {
        player_id: PlayerId,
        object_ids: Vec<ObjectId>,
    },
    Picked {
        player_id: PlayerId,
        object_id: ObjectId,
    },
    Buried {
        player_id: PlayerId,
        object_id: ObjectId,
    },
    AddedToHand {
        player_id: PlayerId,
        object_id: ObjectId,
    },
    BoardEntered {
        player_id: PlayerId,
        object_id: ObjectId,
    },
    FatigueDamaged {
        amount: u8,
        player_id: PlayerId,
        object_id: ObjectId,
    },
    Died(Death),
    GameEnded(Ending),
}

impl Report {
    fn into_event(self) -> Event {
        match self {
            Report::TurnStarted { player_id } => Event::TurnStarted { player_id },
            Report::TurnEnded { player_id } => Event::TurnEnded { player_id },
            Report::Drew {
                player_id,
                object_id,
            } => Event::Drew {
                player_id,
                object_id,
            },
            Report::Played(play) => Event::Played {
                player_id: play.player_id(),
                object_id: play.object_id(),
            },
            Report::Damaged {
                target,
                amount,
                source,
            } => Event::Damaged {
                target,
                amount,
                source,
            },
            Report::Revealed {
                player_id,
                object_ids,
            } => Event::Revealed {
                player_id,
                object_ids,
            },
            Report::Picked {
                player_id,
                object_id,
            } => Event::Picked {
                player_id,
                object_id,
            },
            Report::Buried {
                player_id,
                object_id,
            } => Event::Buried {
                player_id,
                object_id,
            },
            Report::AddedToHand {
                player_id,
                object_id,
            } => Event::AddedToHand {
                player_id,
                object_id,
            },
            Report::BoardEntered {
                player_id,
                object_id,
            } => Event::BoardEntered {
                player_id,
                object_id,
            },
            Report::FatigueDamaged {
                amount,
                player_id,
                object_id,
            } => Event::FatigueDamaged {
                amount,
                player_id,
                object_id,
            },
            Report::Died(death) => Event::Died {
                object_id: death.object_id(),
            },
            Report::GameEnded(ending) => Event::GameEnded {
                outcome: ending.outcome(),
            },
        }
    }
}

impl Game {
    pub(super) fn report(&mut self, report: Report, obs: &mut impl Observer) {
        let event = report.into_event();
        let matched = self.matched_triggers(&event);
        report_event(
            &event,
            matched,
            &mut self.history,
            self.turn_order.turn_count(),
            &self.objects,
            &mut self.effect_stack,
            obs,
        );
    }
}

fn report_event(
    event: &Event,
    matched: Vec<MatchedTrigger>,
    history: &mut History,
    turn: u32,
    objects: &ObjectBag,
    effect_stack: &mut EffectStack,
    obs: &mut impl Observer,
) {
    history.record(event, turn, objects);
    #[expect(
        clippy::disallowed_methods,
        reason = "the one place that reports an event"
    )]
    obs.event(event);
    for MatchedTrigger { source, effects } in matched {
        effect_stack.push(source, effects);
    }
}
