//! Rules core: a deterministic card game state machine.
//!
//! The public API is the `pub use` list below. Every other item is
//! `pub(crate)` or narrower, so changing one can't break a caller.
//!
//! Layout:
//! - `ids`, `action`, `cards`, `event`: public data types.
//! - `observer`: the `Observer` trait and the `Views` handle it receives.
//! - `rng`, `turn`, `zones`: types whose fields are private to their module.
//! - `game`: `Game` and its methods, and `View`. Its child modules can read `Game`'s private fields.

mod action;
mod cards;
mod choice;
mod effect;
mod event;
mod game;
mod history;
mod ids;
mod observer;
mod outcome;
mod rng;
mod trigger;
mod turn;
mod zones;

pub use action::{Action, IllegalAction};
pub use cards::{
    def_ids::static_card_definition,
    definition::{CardDef, DefId},
    loader::CardDefLoader,
    object::ObjectId,
    validate::CardDefError,
};
pub use choice::{ChoiceId, PickId};
pub use event::Event;
pub use game::{
    ApplyError, Game,
    view::{BoardCard, DraftView, Face, HandCard, HeroCard, PlayerView, RevealedCard, View},
};
pub use ids::PlayerId;
pub use observer::{Observer, Views};
pub use outcome::Outcome;
pub use rng::Rng;
