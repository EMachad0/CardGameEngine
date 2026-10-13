//! Rules core: a deterministic card game state machine.
//!
//! The public API is the `pub use` list below. Every other item is
//! `pub(crate)` or narrower, so changing one can't break a caller.

mod action;
mod cards;
mod choice;
mod event;
mod game;
mod ids;
mod object;
mod observer;
mod outcome;
mod rng;
mod stack;
mod turn;
mod zones;

pub use action::{Action, ApplyError, IllegalAction};
pub use cards::{
    choice::ChoiceId,
    def_ids::static_card_definition,
    definition::{CardDef, DefId},
    effect::PickId,
    loader::CardDefLoader,
    validate::CardDefError,
};
pub use event::Event;
pub use game::{
    Game,
    view::{BoardCard, DraftView, Face, HandCard, HeroCard, PlayerView, RevealedCard, View},
};
pub use ids::PlayerId;
pub use object::ObjectId;
pub use observer::{Observer, Views};
pub use outcome::Outcome;
pub use rng::Rng;
