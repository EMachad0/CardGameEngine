use std::collections::BTreeMap;
use std::sync::Arc;

use crate::cards::binder::def_code::DefCode;
use crate::cards::binder::def_id::DefId;
use crate::cards::binder::parse;
use crate::cards::binder::set::{Set, SetCard};
use crate::cards::parsed::CardDef;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum BinderError {
    #[error("no card has the code {0}")]
    NotFound(DefCode),
    #[error("two cards have the code {0}")]
    Duplicate(DefCode),
    #[error("a summon names {0}, a card that is not a minion")]
    NotAMinion(DefCode),
}

/// The card definitions a game can use, each found by its code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binder {
    /// Arc so clone and eq are cheap
    cards: Arc<Cards>,
}

#[derive(Debug, PartialEq, Eq)]
struct Cards {
    /// Each card at its `DefId`'s place.
    by_id: Vec<(DefCode, CardDef)>,
    by_code: BTreeMap<DefCode, DefId>,
}

impl Binder {
    /// Gives each card the code its set's prefix, its name and its version compose, and parses
    /// every card reference against the cards of every set.
    pub fn new(sets: impl IntoIterator<Item = Set>) -> Result<Self, BinderError> {
        let written = sets
            .into_iter()
            .flat_map(|Set { prefix, cards }| {
                cards.into_iter().map(move |card| {
                    let SetCard { name, version, def } = card;
                    let code = DefCode {
                        set: prefix,
                        name,
                        version,
                    };
                    (code, def)
                })
            })
            .collect::<Vec<_>>();

        let mut codes = BTreeMap::new();
        for (idx, (code, def)) in written.iter().enumerate() {
            let named = parse::Named {
                def_id: DefId::new(idx),
                kind: parse::kind(&def.kind),
            };
            if codes.insert(*code, named).is_some() {
                return Err(BinderError::Duplicate(*code));
            }
        }
        let by_id = written
            .into_iter()
            .map(|(code, def)| Ok((code, parse::card(def, &codes)?)))
            .collect::<Result<_, _>>()?;
        let by_code = codes
            .into_iter()
            .map(|(code, named)| (code, named.def_id))
            .collect();

        Ok(Self {
            cards: Arc::new(Cards { by_id, by_code }),
        })
    }

    /// `None` if no card has `code`.
    pub(crate) fn def_id(&self, code: DefCode) -> Option<DefId> {
        self.cards.by_code.get(&code).copied()
    }

    pub(crate) fn code(&self, def_id: DefId) -> DefCode {
        self.entry(def_id).0
    }

    pub(crate) fn get(&self, def_id: DefId) -> &CardDef {
        &self.entry(def_id).1
    }

    fn entry(&self, def_id: DefId) -> &(DefCode, CardDef) {
        self.cards
            .by_id
            .get(def_id.idx())
            .expect("a def id should come from the binder it is used with")
    }
}
