//! Contract tests for what SPEC.md promises about `Game`'s public API.
//! Spec: ../../SPEC.md.
//!
//! - `legality`: P on exact positions built with `with_deck_order`.
//! - `properties`: R1, L, P, B and C over seeded random playouts.
//! - `playout`: the random driver and the invariants it checks at every step.
//! - `model`: an independent model of SPEC.md's printed data, costs and boards.
//! - `support`: builders and assertions the other modules share.

// SPEC.md promises `Action: Clone`, not `Copy`.
#![allow(clippy::clone_on_copy)]

mod legality;
mod model;
mod playout;
mod properties;
mod support;
