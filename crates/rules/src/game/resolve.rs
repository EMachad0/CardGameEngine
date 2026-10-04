//! The mutation half of `Game::apply`. It runs an action and resolves any card
//! the action plays.
//!
//! Every action that reaches this module is already in `legal_actions`, so
//! nothing here re-checks legality.

use super::Game;
use crate::action::Action;
use crate::cards::definition::{
    CharacterKindFilter, CharacterSelector, CharacterSelectorFilter, CharacterSideFilter, Effect,
    PlayerSelector,
};
use crate::cards::modifier::{EffectAmount, Modifier};
use crate::history::{HistoryKind, HistoryQuery, HistoryQueryKind, PlayerFilter};
use crate::ids::PlayerId;
use crate::{Event, IllegalAction, ObjectId, Observer, Views};

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ApplyError {
    #[error("Illegal Action")]
    IllegalAction(IllegalAction),
}

impl Game {
    pub fn applied(&self, player_id: PlayerId, action: Action) -> Result<Self, ApplyError> {
        let mut game = self.clone();
        game.apply(player_id, action, &mut ())?;
        Ok(game)
    }

    /// `Ok` if and only if `action` is in `legal_actions(player_id)`.
    /// An `Err` leaves the game unchanged.
    pub fn apply(
        &mut self,
        player_id: PlayerId,
        action: Action,
        obs: &mut impl Observer,
    ) -> Result<(), ApplyError> {
        if !self.legal_actions(player_id).contains(&action) {
            Err(ApplyError::IllegalAction(IllegalAction::new(
                player_id, action,
            )))
        } else {
            self.apply_action(player_id, action, obs);
            self.check_state(obs);
            Ok(())
        }
    }

    pub(super) fn apply_action(
        &mut self,
        player_id: PlayerId,
        action: Action,
        obs: &mut impl Observer,
    ) {
        match action {
            Action::Play { object_id } => {
                let mana_cost = self
                    .mana_cost(object_id)
                    .expect("[legal_actions] guarantees a play action object_id has mana");
                self.get_player_mut(player_id).mana -= mana_cost;
                self.play(player_id, object_id, obs);
            }
            Action::Pick { object_id } => {
                self.pick(player_id, object_id, obs);
            }
            Action::EndTurn => {
                self.end_turn(obs);
                self.start_turn(obs);
            }
        };
        obs.checkpoint(Views::new(self));
    }

    pub(crate) fn apply_effect(
        &mut self,
        owner: PlayerId,
        source: ObjectId,
        effect: Effect,
        obs: &mut impl Observer,
    ) {
        match effect {
            Effect::Draw { selector, amount } => {
                let amount = self.effect_amount(amount, owner);
                let targets = self.resolve_player_selector(owner, selector);
                for target in targets.into_iter() {
                    self.draw(target, amount, obs);
                }
            }
            Effect::Reveal { selector, amount } => {
                let amount = self.effect_amount(amount, owner);
                let targets = self.resolve_player_selector(owner, selector);
                for target in targets.into_iter() {
                    self.reveal(target, amount, obs);
                }
            }
            Effect::Damage { selector, amount } => {
                let amount = self.effect_amount(amount, owner);
                let targets = self.resolve_character_selector(owner, source, selector);
                for target in targets.into_iter() {
                    let object = self.objects.get_mut(target);
                    object.damage += amount;
                    obs.event(&Event::Damaged {
                        target,
                        amount,
                        source,
                    });
                }
            }
            Effect::AddFriendlyAura { selector, effect } => {
                let targets = self.resolve_character_selector(owner, source, selector);
                for target in targets.into_iter() {
                    let object = self.objects.get_mut(target);
                    object.friendly_aura.add(Modifier { source, effect });
                }
            }
            Effect::Summon { selector, def_id } => {
                let targets = self.resolve_player_selector(owner, selector);
                for player_id in targets.into_iter() {
                    let object_id = self.objects.insert(def_id, player_id);
                    self.summon(player_id, object_id, obs);
                }
            }
        }
    }

    pub(crate) fn apply_effects(
        &mut self,
        source: ObjectId,
        effects: Vec<Effect>,
        obs: &mut impl Observer,
    ) {
        let owner = self.objects.get(source).player_id;
        effects.into_iter().for_each(|e| {
            self.apply_effect(owner, source, e, obs);
        });
    }

    pub(crate) fn effect_amount(&self, amount: EffectAmount, player_id: PlayerId) -> u8 {
        match amount {
            EffectAmount::Static(value) => value,
            EffectAmount::History(query) => self.history_query(query, player_id),
        }
    }

    pub fn history_query(&self, query: HistoryQuery, player_id: PlayerId) -> u8 {
        let entries = self.history.entries.iter();
        let targets = match query.scope {
            PlayerFilter::All => self.players.iter().map(|p| p.id).collect(),
            PlayerFilter::Owner => vec![player_id],
            PlayerFilter::Active => vec![self.turn_order.get_active_player_id()],
        };
        match query.kind {
            HistoryQueryKind::SpellsPlayed => entries
                .filter(|entry| targets.contains(&entry.player_id))
                .filter(|entry| match query.turn {
                    crate::history::TurnFilter::Current => {
                        entry.turn == self.turn_order.turn_count()
                    }
                    crate::history::TurnFilter::All => true,
                })
                .filter(|entry| {
                    matches!(
                        &entry.kind,
                        HistoryKind::CardPlayed { object } if self.binder.is_spell(object.def_id)
                    )
                })
                .count() as u8,
            HistoryQueryKind::MinionDied => entries
                .filter(|entry| targets.contains(&entry.player_id))
                .filter(|entry| match query.turn {
                    crate::history::TurnFilter::Current => {
                        entry.turn == self.turn_order.turn_count()
                    }
                    crate::history::TurnFilter::All => true,
                })
                .filter(|entry| matches!(&entry.kind, HistoryKind::CharacterDied { object } if self.binder.is_minion(object.def_id)))
                .count() as u8,
        }
    }

    fn resolve_player_selector(
        &mut self,
        owner: PlayerId,
        selector: PlayerSelector,
    ) -> Vec<PlayerId> {
        match selector {
            PlayerSelector::All => self.players.iter().map(|p| p.id).collect(),
            PlayerSelector::Owner => vec![owner],
            PlayerSelector::Random => {
                vec![PlayerId::new(self.rng.below(self.players.len()))]
            }
            PlayerSelector::Enemy => self
                .players
                .iter()
                .map(|p| p.id)
                .filter(|&id| id != owner)
                .collect(),
        }
    }

    fn resolve_character_selector(
        &mut self,
        owner: PlayerId,
        object_id: ObjectId,
        selector: CharacterSelector,
    ) -> Vec<ObjectId> {
        let all = |CharacterSelectorFilter { kind, side }| {
            let kind_filter = |id: PlayerId| {
                (side.contains(CharacterSideFilter::Enemy) && id != owner)
                    || (side.contains(CharacterSideFilter::Friendly) && id == owner)
            };

            let mut characters = Vec::new();
            if kind.contains(CharacterKindFilter::Minions) {
                characters.extend(
                    self.players
                        .iter()
                        .filter(|player| kind_filter(player.id))
                        .flat_map(|player| player.zones.board.as_slice()),
                );
            }
            if kind.contains(CharacterKindFilter::Heroes) {
                characters.extend(
                    self.players
                        .iter()
                        .filter(|player| kind_filter(player.id))
                        .map(|player| player.zones.hero),
                );
            }
            characters
        };
        match selector {
            CharacterSelector::All(filter) => all(filter),
            CharacterSelector::Itself => vec![object_id],
            CharacterSelector::OwnerHero => vec![self.hero_id(owner)],
            CharacterSelector::Random(filter) => {
                let possible = all(filter);
                if !possible.is_empty() {
                    let idx = self.rng.below(possible.len());
                    vec![possible[idx]]
                } else {
                    Vec::new()
                }
            }
        }
    }
}
