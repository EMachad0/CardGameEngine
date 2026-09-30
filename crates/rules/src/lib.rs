//! Rules core: a deterministic card game state machine. Spec: ../SPEC.md.
//!
//! The crate interface is the `pub use` list below and nothing else. Every
//! other item is `pub(crate)` or narrower, so internals can move freely.
//!
//! Layout:
//! - `ids`, `action`, `cards`: public data types.
//! - `rng`, `turn`, `zones`: modules that own private state behind a small interface.
//! - `game`: `Game` and its interface. Procedures that need all of `Game`'s
//!   state live in child modules under `game/`, which can read its private fields.

mod action;
mod cards;
mod game;
mod ids;
mod rng;
mod turn;
mod zones;

pub use action::{Action, Illegal};
pub use cards::Card;
pub use game::Game;
pub use ids::PlayerId;
pub use rng::Rng;
