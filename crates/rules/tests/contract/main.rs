//! Contract tests for what SPEC.md promises about `Game`'s public API.
//! Spec: ../../SPEC.md.
//!
//! - `legality`: exact positions built with `with_deck_order` (node S). Each
//!   unlisted action returns the exact `Illegal` and changes nothing (P).
//! - `properties`: seeded random playouts that check L, P and R1 at every step.

// SPEC.md promises `Card: Clone`, not `Copy`, since cards may later carry names or text.
#![allow(clippy::clone_on_copy)]

mod legality;
mod properties;
mod support;
