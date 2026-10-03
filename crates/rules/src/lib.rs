//! Rules core: a deterministic card game state machine. Spec: ../SPEC.md.
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
pub mod cards;
mod event;
mod game;
mod history;
mod ids;
mod observer;
mod outcome;
mod rng;
mod turn;
mod zones;

pub use action::{Action, IllegalAction};
pub use cards::{definition::DefId, object::ObjectId};
pub use event::{Event, Target};
pub use game::view::{BoardCard, Face, HandCard, PlayerView, RevealedCard, View};
pub use game::{ApplyError, Game};
pub use ids::PlayerId;
pub use observer::{Observer, Views};
pub use outcome::Outcome;
pub use rng::Rng;
