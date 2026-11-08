use crate::cards::binder::def_code::DefCode;
use crate::cards::binder::def_id::DefId;
use crate::cards::binder::store::Binder;
use crate::cards::definition::CardKind;

/// A player's hero and the cards of their deck, top first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deck {
    pub hero: DefCode,
    pub cards: Vec<DefCode>,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum DeckError {
    #[error("no card has the code {0}")]
    NotFound(DefCode),
    #[error("the hero {0} names a card that is not a hero")]
    NotAHero(DefCode),
}

/// A deck whose codes name cards in the binder it was parsed against, with a hero card as hero.
pub(super) struct ParsedDeck {
    hero: DefId,
    cards: Vec<DefId>,
}

impl ParsedDeck {
    /// The hero, then the cards, top first.
    pub(super) fn into_parts(self) -> (DefId, Vec<DefId>) {
        (self.hero, self.cards)
    }
}

impl Deck {
    pub(super) fn parse(self, binder: &Binder) -> Result<ParsedDeck, DeckError> {
        let Self { hero, cards } = self;
        let def_id = |code| binder.def_id(code).ok_or(DeckError::NotFound(code));
        let hero_id = def_id(hero)?;
        if binder.get(hero_id).card_kind() != CardKind::Hero {
            return Err(DeckError::NotAHero(hero));
        }
        Ok(ParsedDeck {
            hero: hero_id,
            cards: cards.into_iter().map(def_id).collect::<Result<_, _>>()?,
        })
    }
}
