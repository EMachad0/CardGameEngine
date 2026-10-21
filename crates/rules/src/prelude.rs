//! The crate's public API. Import each item by name, not with a glob.

pub use crate::action::{Action, IllegalAction};
pub use crate::cards::binder::sets::base::static_card_definition;
pub use crate::cards::definition::{CardDef, DefId};
pub use crate::cards::object::ObjectId;
pub use crate::effect::PickId;
pub use crate::event::Event;
pub use crate::game::resolve::apply::ApplyError;
pub use crate::game::state::Game;
pub use crate::game::view::{
    BoardCard, Face, HandCard, HeroCard, PlayerView, RevealedCard, TargetingView, View,
};
pub use crate::ids::PlayerId;
pub use crate::observer::{Observer, Views};
pub use crate::outcome::Outcome;
pub use crate::rng::Rng;
pub use crate::target::TargetId;
