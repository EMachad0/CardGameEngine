//! Attack, health and cost, computed on read from history and printed data (node C).

use crate::{
    DefId, Game, ObjectId, PlayerId,
    cards::{CardDefNotFound, definition::Effect, object::Object},
};

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum LookupError {
    #[error("Object not found")]
    ObjectNotFound(ObjectId),
    #[error("Card definition not found")]
    DefinitionNotFound(#[from] CardDefNotFound),
}

type LookupResult<T> = Result<T, LookupError>;

impl Game {
    pub fn hero_health(&self, player_id: PlayerId) -> i32 {
        self.get_player(player_id).health
    }

    fn object(&self, object_id: ObjectId) -> LookupResult<&Object> {
        self.objects
            .get(object_id)
            .ok_or(LookupError::ObjectNotFound(object_id))
    }

    pub fn def_id(&self, object_id: ObjectId) -> LookupResult<DefId> {
        self.object(object_id).map(|obj| obj.def_id)
    }

    pub fn mana_cost(&self, object_id: ObjectId) -> LookupResult<Option<u8>> {
        let obj = self.object(object_id)?;
        if !self
            .get_player(obj.player_id)
            .zones
            .hand
            .contains(&object_id)
        {
            Ok(None)
        } else {
            let def_mana_cost = self.binder.mana_cost(obj.def_id);
            Ok(Some(def_mana_cost))
        }
    }

    pub fn health(&self, object_id: ObjectId) -> LookupResult<Option<i32>> {
        let obj = self.object(object_id)?;
        Ok(self.binder.health(obj.def_id))
    }

    pub fn attack(&self, object_id: ObjectId) -> LookupResult<Option<i32>> {
        let obj = self.object(object_id)?;
        Ok(self.binder.attack(obj.def_id))
    }

    pub fn on_play_effect(&self, object_id: ObjectId) -> LookupResult<&[Effect]> {
        let obj = self.object(object_id)?;
        Ok(self.binder.on_play_effect(obj.def_id))
    }
}
