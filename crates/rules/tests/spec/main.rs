//! Tests of what SPEC.md promises, through the public interface only.
//! Spec: ../../SPEC.md.

// SPEC.md promises `Action: Clone`, not `Copy`.
#![allow(clippy::clone_on_copy)]

mod cards;
mod derived;
mod legality;
mod model;
mod playout;
mod properties;
mod setup;
mod state_check;
mod support;
mod turns;
