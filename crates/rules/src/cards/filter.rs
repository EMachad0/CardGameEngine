use enumset::EnumSet;

use crate::cards::definition::{CardKind, CharacterKind};
use crate::ids::PlayerId;

#[derive(Debug, enumset::EnumSetType)]
pub enum Side {
    Friendly,
    Enemy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardFilter {
    pub kind: EnumSet<CardKind>,
    pub side: EnumSet<Side>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharacterFilter {
    pub kind: EnumSet<CharacterKind>,
    pub side: EnumSet<Side>,
}

impl CharacterFilter {
    pub fn all() -> Self {
        Self {
            kind: EnumSet::all(),
            side: EnumSet::all(),
        }
    }

    /// Whether a card of `kind` owned by `owner` passes, from `asking`'s side. `None` is a card
    /// that is no character.
    pub(crate) fn matches(
        &self,
        asking: PlayerId,
        owner: PlayerId,
        kind: Option<CharacterKind>,
    ) -> bool {
        fulfill_side_filter(self.side, asking, owner)
            && kind.is_some_and(|kind| self.kind.contains(kind))
    }
}

impl CardFilter {
    /// Whether a card of `kind` owned by `owner` passes, from `asking`'s side.
    pub(crate) fn matches(&self, asking: PlayerId, owner: PlayerId, kind: CardKind) -> bool {
        fulfill_side_filter(self.side, asking, owner) && self.kind.contains(kind)
    }
}

pub(crate) fn fulfill_side_filter(side: EnumSet<Side>, asking: PlayerId, owner: PlayerId) -> bool {
    (side.contains(Side::Enemy) && owner != asking)
        || (side.contains(Side::Friendly) && owner == asking)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ME: PlayerId = PlayerId::new(0);
    const THEM: PlayerId = PlayerId::new(1);

    fn enemy_minions() -> CharacterFilter {
        CharacterFilter {
            kind: CharacterKind::Minion.into(),
            side: Side::Enemy.into(),
        }
    }

    fn friendly_spells() -> CardFilter {
        CardFilter {
            kind: CardKind::Spell.into(),
            side: Side::Friendly.into(),
        }
    }

    #[test]
    fn a_character_filter_passes_a_character_of_its_kind_and_side() {
        assert!(enemy_minions().matches(ME, THEM, Some(CharacterKind::Minion)));
    }

    #[test]
    fn a_character_filter_rejects_a_character_of_another_side() {
        assert!(!enemy_minions().matches(ME, ME, Some(CharacterKind::Minion)));
    }

    #[test]
    fn a_character_filter_rejects_a_character_of_another_kind() {
        assert!(!enemy_minions().matches(ME, THEM, Some(CharacterKind::Hero)));
    }

    #[test]
    fn a_character_filter_rejects_a_card_that_is_no_character() {
        assert!(!CharacterFilter::all().matches(ME, THEM, None));
    }

    #[test]
    fn a_card_filter_passes_a_card_of_its_kind_and_side() {
        assert!(friendly_spells().matches(ME, ME, CardKind::Spell));
    }

    #[test]
    fn a_card_filter_rejects_a_card_of_another_side() {
        assert!(!friendly_spells().matches(ME, THEM, CardKind::Spell));
    }

    #[test]
    fn a_card_filter_rejects_a_card_of_another_kind() {
        assert!(!friendly_spells().matches(ME, ME, CardKind::Minion));
    }
}
