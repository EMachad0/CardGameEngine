//! Matches each reported event against the triggers in play, then records it in history, tells
//! the observer, and pushes the triggers it matched. A variant that holds a value with private
//! fields can be built only in the file that defines the value.

use crate::cards::object::{ObjectBag, ObjectId};
use crate::event::Event;
use crate::game::cards_ref::CardsRef;
use crate::game::effect_stack::EffectStack;
use crate::game::play::Play;
use crate::game::resolve::aftermath::death::Death;
use crate::game::resolve::aftermath::standings::Ending;
use crate::game::triggering::{MatchedTrigger, matched_triggers};
use crate::history::History;
use crate::ids::PlayerId;
use crate::observer::Observer;

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
    pub(super) fn matched(self, cards: CardsRef) -> MatchedEvent {
        let event = self.into_event();
        let triggers = matched_triggers(cards, &event);
        MatchedEvent { event, triggers }
    }

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
            Report::Played(play) => play.into(),
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
            Report::Died(death) => death.into(),
            Report::GameEnded(ending) => ending.into(),
        }
    }
}

/// An event, and the triggers it matched.
pub(super) struct MatchedEvent {
    event: Event,
    triggers: Vec<MatchedTrigger>,
}

impl MatchedEvent {
    /// Records the event in history, tells the observer, and pushes the triggers it matched.
    pub(super) fn report(
        self,
        history: &mut History,
        turn: u32,
        objects: &ObjectBag,
        effect_stack: &mut EffectStack,
        obs: &mut impl Observer,
    ) {
        let Self { event, triggers } = self;
        history.record(&event, turn, objects);
        #[expect(
            clippy::disallowed_methods,
            reason = "the one report to the observer, reached through Game::report"
        )]
        obs.event(&event);
        for MatchedTrigger { source, effects } in triggers {
            effect_stack.push(source, effects);
        }
    }
}
