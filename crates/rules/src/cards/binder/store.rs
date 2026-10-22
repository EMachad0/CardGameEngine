use std::collections::BTreeMap;
use std::sync::Arc;

use crate::cards::binder::sets::base;
use crate::cards::binder::validity::{self, CardDefError, ValidCardDef};
use crate::cards::definition::DefId;

/// Holds a definition for every def id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Binder {
    /// Arc so clone and eq are cheap
    defs: Arc<BTreeMap<DefId, ValidCardDef>>,
}

impl Binder {
    pub(crate) fn built_in() -> Result<Self, CardDefError> {
        let defs = validity::parse(&base::definitions())?;
        Ok(Self {
            defs: Arc::new(defs),
        })
    }

    pub(crate) fn get(&self, def_id: DefId) -> &ValidCardDef {
        self.defs.get(&def_id).expect("card def not found")
    }
}
