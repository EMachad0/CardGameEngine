//! Attack, health and cost, computed on read from history and printed data (node C).

use crate::{
    DefId, Game, ObjectId, PlayerId,
    cards::{
        CardDefNotFound,
        definition::Effect,
        modifier::{Modifier, ModifierEffect, Modifiers},
        object::Object,
    },
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
            let def = self.binder.mana_cost(obj.def_id) as i32;
            let modifiers = self
                .modifiers(object_id)?
                .as_slice()
                .into_iter()
                .filter_map(|m| match m.effect {
                    ModifierEffect::ReduceManaCost { amount } => Some(-self.effect_amount(amount, obj.player_id)),
                    _ => None,
                })
                .sum::<i32>();
            Ok(Some((def + modifiers).max(0) as u8))
        }
    }

    pub fn health(&self, object_id: ObjectId) -> LookupResult<Option<i32>> {
        let obj = self.object(object_id)?;
        if !self
            .get_player(obj.player_id)
            .zones
            .board
            .contains(&object_id)
        {
            Ok(None)
        } else {
            let def = self.binder.health(obj.def_id).unwrap_or_default();
            let modifiers = self
                .modifiers(object_id)?
                .as_slice()
                .into_iter()
                .filter_map(|m| match m.effect {
                    ModifierEffect::BuffHealth { amount } => Some(amount),
                    _ => None,
                })
                .map(|a| self.effect_amount(a, obj.player_id))
                .sum::<i32>();
            let damage = self
                .object(object_id)
                .expect("unexpected lookup error")
                .damage as i32;
            Ok(Some(def + modifiers - damage))
        }
    }

    pub fn attack(&self, object_id: ObjectId) -> LookupResult<Option<i32>> {
        let obj = self.object(object_id)?;
        if !self
            .get_player(obj.player_id)
            .zones
            .board
            .contains(&object_id)
        {
            Ok(None)
        } else {
            let def = self.binder.attack(obj.def_id).unwrap_or_default();
            let modifiers = self
                .modifiers(object_id)?
                .as_slice()
                .iter()
                .filter_map(|m| match m.effect {
                    ModifierEffect::BuffAtk { amount } => Some(amount),
                    _ => None,
                })
                .map(|a| self.effect_amount(a, obj.player_id))
                .sum::<i32>();
            Ok(Some(def + modifiers))
        }
    }

    fn modifiers(&self, object_id: ObjectId) -> LookupResult<Modifiers> {
        let obj = self.object(object_id)?;
        let modifiers = self
            .binder
            .modifier_effects(obj.def_id)
            .into_iter()
            .map(|e| Modifier {
                source: object_id,
                effect: *e,
            })
            .collect::<Vec<_>>();
        let mut modifiers = Modifiers::new(modifiers);
        modifiers.extend(obj.modifiers.clone());

        for other_player_id in self.players().into_iter() {
            for other_object_id in self.board(other_player_id).into_iter().copied() {
                if object_id == other_object_id {
                    continue;
                }

                let other_modifiers = if obj.player_id == other_player_id {
                    self.friendly_aura(other_object_id)
                } else {
                    self.hostile_aura(other_object_id)
                }?;
                modifiers.extend(other_modifiers);
            }
        }

        Ok(modifiers)
    }

    pub(crate) fn on_play(&self, object_id: ObjectId) -> LookupResult<Vec<Effect>> {
        let obj = self.object(object_id)?;
        Ok(self.binder.on_play_effect(obj.def_id).to_vec())
    }

    pub(crate) fn on_board_enter(&self, object_id: ObjectId) -> LookupResult<Vec<Effect>> {
        let obj = self.object(object_id)?;
        Ok(self.binder.on_board_enter(obj.def_id).to_vec())
    }

    pub(crate) fn on_board_leave(&self, object_id: ObjectId) -> LookupResult<Vec<Effect>> {
        let obj = self.object(object_id)?;
        Ok(self.binder.on_board_leave(obj.def_id).to_vec())
    }

    pub(crate) fn on_death(&self, object_id: ObjectId) -> LookupResult<Vec<Effect>> {
        let obj = self.object(object_id)?;
        Ok(self.binder.on_death(obj.def_id).to_vec())
    }

    pub(crate) fn friendly_aura(&self, object_id: ObjectId) -> LookupResult<Modifiers> {
        let obj = self.object(object_id)?;
        let modifiers = self
            .binder
            .friendly_aura_effects(obj.def_id)
            .into_iter()
            .map(|e| Modifier {
                source: object_id,
                effect: *e,
            })
            .collect::<Vec<_>>();
        let mut modifiers = Modifiers::new(modifiers);
        modifiers.extend(obj.modifiers.clone());
        Ok(modifiers)
    }

    pub(crate) fn hostile_aura(&self, object_id: ObjectId) -> LookupResult<Modifiers> {
        let obj = self.object(object_id)?;
        let modifiers = self
            .binder
            .hostile_aura_effects(obj.def_id)
            .into_iter()
            .map(|e| Modifier {
                source: object_id,
                effect: *e,
            })
            .collect::<Vec<_>>();
        Ok(Modifiers::new(modifiers))
    }
}
