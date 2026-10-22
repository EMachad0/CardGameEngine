use std::collections::BTreeMap;
use std::sync::Arc;

use crate::cards::binder::sets::base;
use crate::cards::binder::validity::{self, CardDefError};
use crate::cards::definition::{CardDef, DefId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Binder {
    /// Arc so clone and eq are cheap
    defs: Arc<BTreeMap<DefId, CardDef>>,
}

impl Binder {
    pub(crate) fn built_in() -> Result<Self, CardDefError> {
        let defs = base::definitions();
        validity::validate(&defs)?;

        let mut map = BTreeMap::new();
        defs.into_iter().for_each(|def| {
            map.insert(def.id, def);
        });

        Ok(Self {
            defs: Arc::new(map),
        })
    }

    pub(crate) fn get(&self, def_id: DefId) -> &CardDef {
        self.defs.get(&def_id).expect("card def not found")
    }
}
