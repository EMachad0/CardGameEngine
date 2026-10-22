use std::collections::BTreeMap;
use std::sync::Arc;

use crate::cards::binder::def_id::DefId;
use crate::cards::binder::sets::base;
use crate::cards::binder::sets::base::static_card_definition::ALL_DEF_ID;
use crate::cards::binder::validity::{self, CardDefError};
use crate::cards::definition::CardDef;
use crate::target::{DeclaredTarget, TargetId};

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
enum BinderError {
    #[error("Card definition not found {0:?}")]
    NotFound(DefId),
    #[error("Duplicate {0:?}")]
    Duplicate(DefId),
    #[error("{1} on {0:?}")]
    Invalid(DefId, CardDefError),
}

/// Holds one definition for every def id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Binder {
    /// Arc so clone and eq are cheap
    defs: Arc<BTreeMap<DefId, CardDef<DeclaredTarget>>>,
}

impl Binder {
    pub(crate) fn built_in() -> Self {
        Self::new(base::definitions()).expect("the built-in card definitions should be valid")
    }

    fn new(table: Vec<(DefId, CardDef<TargetId>)>) -> Result<Self, BinderError> {
        let mut unparsed = BTreeMap::new();
        for (def_id, def) in table {
            if unparsed.insert(def_id, def).is_some() {
                return Err(BinderError::Duplicate(def_id));
            }
        }
        if let Some(&def_id) = ALL_DEF_ID
            .iter()
            .find(|def_id| !unparsed.contains_key(def_id))
        {
            return Err(BinderError::NotFound(def_id));
        }
        let defs = unparsed
            .into_iter()
            .map(|(def_id, def)| match validity::parse(def) {
                Ok(def) => Ok((def_id, def)),
                Err(error) => Err(BinderError::Invalid(def_id, error)),
            })
            .collect::<Result<_, _>>()?;
        Ok(Self {
            defs: Arc::new(defs),
        })
    }

    pub(crate) fn get(&self, def_id: DefId) -> &CardDef<DeclaredTarget> {
        self.defs
            .get(&def_id)
            .expect("the binder should hold a definition for every def id")
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::cards::binder::sets::base::static_card_definition::{BOLT, PING, SPARK};
    use crate::target::Targets;

    fn validate(table: &[(DefId, CardDef<TargetId>)]) -> Result<(), BinderError> {
        Binder::new(table.to_vec()).map(drop)
    }

    #[test]
    fn the_built_in_table_is_valid() {
        assert_eq!(validate(&base::definitions()), Ok(()));
    }

    #[test]
    fn a_constant_missing_from_the_table_is_not_found() {
        let mut defs = base::definitions();
        defs.retain(|&(def_id, _)| def_id != SPARK);

        assert_eq!(validate(&defs), Err(BinderError::NotFound(SPARK)));
    }

    #[test]
    fn two_defs_with_one_id_are_a_duplicate() {
        let mut defs = base::definitions();
        let bolt = defs
            .iter()
            .find(|&&(def_id, _)| def_id == BOLT)
            .unwrap()
            .clone();
        defs.push(bolt);

        assert_eq!(validate(&defs), Err(BinderError::Duplicate(BOLT)));
    }

    #[test]
    fn an_invalid_definition_is_rejected_with_its_def_id() {
        let mut defs = base::definitions();
        let (_, ping) = defs.iter_mut().find(|(def_id, _)| *def_id == PING).unwrap();
        ping.targets = Targets::default();

        assert_eq!(
            validate(&defs),
            Err(BinderError::Invalid(
                PING,
                CardDefError::TargetNotFound(TargetId(0))
            )),
        );
    }
}
