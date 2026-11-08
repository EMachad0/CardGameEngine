use enumset::EnumSet;

use crate::cards::filter::Side;
use crate::choice::Choice;
use crate::effect::CardVerb;
use crate::pick::CardPickOptions;
use crate::target::TargetForm;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerSelectorFilter {
    pub side: EnumSet<Side>,
}

impl PlayerSelectorFilter {
    pub fn all() -> Self {
        Self {
            side: EnumSet::all(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerSelector {
    All(PlayerSelectorFilter),
    Owner,
    Random(PlayerSelectorFilter),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardSelector<S: TargetForm> {
    Targeted(S::CardTarget),
    /// `others` acts on the cards the pick covered and the player didn't pick, in the order
    /// shown, after the effect's verbs act on the cards picked.
    Picked {
        choice: Choice<CardPickOptions>,
        others: Vec<CardVerb>,
    },
}

impl<S: TargetForm> CardSelector<S> {
    pub(crate) fn target(&self) -> Option<&S::CardTarget> {
        match self {
            CardSelector::Targeted(target) => Some(target),
            CardSelector::Picked {
                choice: _,
                others: _,
            } => None,
        }
    }

    pub(crate) fn map_targeted<R: TargetForm>(
        self,
        f: impl FnOnce(S::CardTarget) -> R::CardTarget,
    ) -> CardSelector<R> {
        match self {
            CardSelector::Targeted(target) => CardSelector::Targeted(f(target)),
            CardSelector::Picked { choice, others } => CardSelector::Picked { choice, others },
        }
    }
}
