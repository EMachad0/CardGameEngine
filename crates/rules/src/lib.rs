//! Rules core: a deterministic card game state machine. Spec: ../SPEC.md.
//!
//! The public API is the `pub use` list below. Every other item is
//! `pub(crate)` or narrower, so changing one can't break a caller.
//!
//! Layout:
//! - `ids`, `action`, `cards`: public data types.
//! - `rng`, `turn`, `zones`: types whose fields are private to their module.
//! - `game`: `Game` and its methods. Its child modules can read `Game`'s private fields.

mod action;
pub mod cards;
mod game;
mod ids;
mod outcome;
mod rng;
mod turn;
mod zones;

// #[cfg(test)]
mod testkit;

pub use action::{Action, IllegalAction};
pub use cards::{ObjectBag, ObjectId, DefId};
pub use game::{ApplyError, Game};
pub use ids::PlayerId;
pub use outcome::Outcome;
pub use rng::Rng;
