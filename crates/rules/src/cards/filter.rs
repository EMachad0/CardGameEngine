use enumset::EnumSet;

use crate::cards::kind::{CardKind, CharacterKind};
use crate::cards::object::ObjectId;
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
    /// Whether the object whose text holds the filter can pass it.
    pub include_itself: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharacterFilter {
    pub kind: EnumSet<CharacterKind>,
    pub side: EnumSet<Side>,
    /// Whether the object whose text holds the filter can pass it.
    pub include_itself: bool,
}

impl CharacterFilter {
    pub fn all() -> Self {
        Self {
            kind: EnumSet::all(),
            side: EnumSet::all(),
            include_itself: true,
        }
    }

    /// `itself` is the object whose text holds the filter. `owner` and `kind` belong to
    /// `object_id`. `kind` is `None` for a card that is no character.
    pub(crate) fn passes(
        &self,
        asking: PlayerId,
        itself: ObjectId,
        object_id: ObjectId,
        owner: PlayerId,
        kind: Option<CharacterKind>,
    ) -> bool {
        (self.include_itself || object_id != itself)
            && fulfill_side_filter(self.side, asking, owner)
            && kind.is_some_and(|kind| self.kind.contains(kind))
    }
}

impl CardFilter {
    /// `itself` is the object whose text holds the filter. `owner` and `kind` belong to
    /// `object_id`.
    pub(crate) fn passes(
        &self,
        asking: PlayerId,
        itself: ObjectId,
        object_id: ObjectId,
        owner: PlayerId,
        kind: CardKind,
    ) -> bool {
        (self.include_itself || object_id != itself)
            && fulfill_side_filter(self.side, asking, owner)
            && self.kind.contains(kind)
    }
}

pub(crate) fn fulfill_side_filter(side: EnumSet<Side>, asking: PlayerId, owner: PlayerId) -> bool {
    (side.contains(Side::Enemy) && owner != asking)
        || (side.contains(Side::Friendly) && owner == asking)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::binder::def_code::DefCode;
    use crate::cards::binder::def_id::DefId;
    use crate::cards::binder::set::{Set, SetCard};
    use crate::cards::binder::store::Binder;
    use crate::cards::object::ObjectBag;
    use crate::cards::written;

    /// The `DefId` of a card that no test looks up.
    fn def_id() -> DefId {
        let card = SetCard {
            name: "card",
            version: 0,
            def: written::CardDef::default(),
        };
        let binder = Binder::new([Set {
            prefix: "unit",
            cards: vec![card],
        }])
        .unwrap();
        binder
            .def_id(DefCode {
                set: "unit",
                name: "card",
                version: 0,
            })
            .unwrap()
    }

    const ME: PlayerId = PlayerId::new(0);
    const THEM: PlayerId = PlayerId::new(1);

    fn ids<const N: usize>() -> [ObjectId; N] {
        let mut bag = ObjectBag::default();
        std::array::from_fn(|_| bag.insert(def_id(), ME))
    }

    fn enemy_minions() -> CharacterFilter {
        CharacterFilter {
            kind: CharacterKind::Minion.into(),
            side: Side::Enemy.into(),
            include_itself: true,
        }
    }

    fn other_minions() -> CharacterFilter {
        CharacterFilter {
            kind: CharacterKind::Minion.into(),
            side: EnumSet::all(),
            include_itself: false,
        }
    }

    fn friendly_spells() -> CardFilter {
        CardFilter {
            kind: CardKind::Spell.into(),
            side: Side::Friendly.into(),
            include_itself: true,
        }
    }

    fn other_cards() -> CardFilter {
        CardFilter {
            kind: EnumSet::all(),
            side: EnumSet::all(),
            include_itself: false,
        }
    }

    #[test]
    fn a_character_filter_passes_a_character_of_its_kind_and_side() {
        let [itself, other] = ids();

        assert!(enemy_minions().passes(ME, itself, other, THEM, Some(CharacterKind::Minion)));
    }

    #[test]
    fn a_character_filter_rejects_a_character_of_another_side() {
        let [itself, other] = ids();

        assert!(!enemy_minions().passes(ME, itself, other, ME, Some(CharacterKind::Minion)));
    }

    #[test]
    fn a_character_filter_rejects_a_character_of_another_kind() {
        let [itself, other] = ids();

        assert!(!enemy_minions().passes(ME, itself, other, THEM, Some(CharacterKind::Hero)));
    }

    #[test]
    fn a_character_filter_rejects_a_card_that_is_no_character() {
        let [itself, other] = ids();

        assert!(!CharacterFilter::all().passes(ME, itself, other, THEM, None));
    }

    #[test]
    fn a_character_filter_that_includes_itself_passes_the_object_whose_text_holds_it() {
        let [itself] = ids();

        assert!(CharacterFilter::all().passes(ME, itself, itself, ME, Some(CharacterKind::Minion)));
    }

    #[test]
    fn a_character_filter_that_leaves_itself_out_rejects_the_object_whose_text_holds_it() {
        let [itself] = ids();

        assert!(!other_minions().passes(ME, itself, itself, ME, Some(CharacterKind::Minion)));
    }

    #[test]
    fn a_character_filter_that_leaves_itself_out_passes_another_character() {
        let [itself, other] = ids();

        assert!(other_minions().passes(ME, itself, other, ME, Some(CharacterKind::Minion)));
    }

    #[test]
    fn a_card_filter_passes_a_card_of_its_kind_and_side() {
        let [itself, other] = ids();

        assert!(friendly_spells().passes(ME, itself, other, ME, CardKind::Spell));
    }

    #[test]
    fn a_card_filter_rejects_a_card_of_another_side() {
        let [itself, other] = ids();

        assert!(!friendly_spells().passes(ME, itself, other, THEM, CardKind::Spell));
    }

    #[test]
    fn a_card_filter_rejects_a_card_of_another_kind() {
        let [itself, other] = ids();

        assert!(!friendly_spells().passes(ME, itself, other, ME, CardKind::Minion));
    }

    #[test]
    fn a_card_filter_that_includes_itself_passes_the_object_whose_text_holds_it() {
        let [itself] = ids();

        assert!(friendly_spells().passes(ME, itself, itself, ME, CardKind::Spell));
    }

    #[test]
    fn a_card_filter_that_leaves_itself_out_rejects_the_object_whose_text_holds_it() {
        let [itself] = ids();

        assert!(!other_cards().passes(ME, itself, itself, ME, CardKind::Minion));
    }

    #[test]
    fn a_card_filter_that_leaves_itself_out_passes_another_card() {
        let [itself, other] = ids();

        assert!(other_cards().passes(ME, itself, other, ME, CardKind::Minion));
    }
}
