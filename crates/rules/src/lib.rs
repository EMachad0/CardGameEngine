//! Rules core: a deterministic card game state machine.
//!
//! The public API is what the one public module lists. Every other module is
//! private, so no other crate can reach an item it leaves out.

mod action;
mod cards;
mod effect;
mod event;
mod game;
mod history;
mod ids;
mod observer;
mod outcome;
pub mod prelude;
mod rng;
mod target;
mod trigger;
mod turn;
mod zones;
