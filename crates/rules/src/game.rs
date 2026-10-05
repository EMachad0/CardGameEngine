//! `Game`: the whole state, and the interface a shell drives it through.
//!
//! `legal_actions` is the one definition of legality. `apply` accepts exactly
//! what it lists, and an `Err` leaves the game unchanged. Child modules:
//! - `player`: one player's resources, zones and pending choice.
//! - `resolve`: the mutation half of `apply`.
//! - `lookup`: attack, health and cost, computed on read.

mod lookup;
mod player;
mod resolve;
mod state_check;
pub(crate) mod view;
mod zone_move;

use crate::action::Action;
use crate::cards::definition::{CharacterSelectorFilter, Precondition};
use crate::cards::{binder::Binder, loader::CardDefLoader, object::ObjectBag};
use crate::choice::ChoiceId;
use crate::game::player::{Player, PlayerInteractionState};
use crate::game::resolve::fulfill_chacter_side_filter;
use crate::history::History;
use crate::ids::PlayerId;
use crate::rng::Rng;
use crate::static_card_definition::HERO;
use crate::turn::TurnOrder;
use crate::zones::Deck;
use crate::{DefId, Event, ObjectId, Observer, Outcome};

pub use resolve::ApplyError;

#[derive(Debug, Clone, PartialEq)]
pub struct Game {
    rng: Rng,
    turn_order: TurnOrder,
    players: Vec<Player>,
    objects: ObjectBag,
    binder: Binder,
    history: History,
    outcome: Option<Outcome>,
}

impl Game {
    /// Seeds the `Rng`, shuffles both decks, deals 3 cards each and starts player 0's turn.
    pub fn new(seed: u64, decks: [Vec<DefId>; 2]) -> Self {
        Self::setup(seed, decks, true)
    }

    /// Like `new`, but keeps each deck in the given order, index 0 on top.
    pub fn with_deck_order(seed: u64, decks: [Vec<DefId>; 2]) -> Self {
        Self::setup(seed, decks, false)
    }

    fn setup(seed: u64, decks: [Vec<DefId>; 2], shuffle: bool) -> Self {
        let rng = Rng::new(seed);

        // may explode not sure how to handle yet
        let defs = CardDefLoader.load_and_validate().unwrap();
        let binder = Binder::new(defs);

        let mut objects = ObjectBag::default();

        let player_count = decks.len();
        let player_ids = (0..player_count).map(PlayerId::new).collect::<Vec<_>>();
        let mut players = player_ids
            .iter()
            .map(|&player_id| {
                let hero_id = objects.insert(HERO, player_id);
                Player::new(player_id, hero_id)
            })
            .collect::<Vec<_>>();
        for (player, deck_defs) in players.iter_mut().zip(decks) {
            let deck = deck_defs
                .into_iter()
                .map(|def_id| objects.insert(def_id, player.id))
                .collect::<Vec<_>>();
            player.zones.deck = Deck::new(deck);
        }
        let turn_order = TurnOrder::new(player_ids.clone());
        let history = History::default();

        let mut game = Self {
            rng,
            turn_order,
            players,
            objects,
            binder,
            history,
            outcome: None,
        };

        for player in game.players.iter_mut() {
            if shuffle {
                player.zones.deck.shuffle(&mut game.rng);
            }
        }

        for player_id in player_ids {
            game.draw(player_id, 3, &mut ());
        }

        game.start_turn(&mut ());
        game
    }

    pub fn legal_actions(&self, player_id: PlayerId) -> Vec<Action> {
        if self.outcome().is_some() {
            return Vec::new();
        }

        let mut actions = Vec::new();
        match &self.get_player(player_id).interaction_state {
            PlayerInteractionState::Idle => {
                if self.turn_order.get_active_player_id() != player_id {
                    return Vec::new();
                }

                let mana = self.mana(player_id);
                let hand = self.hand(player_id);
                for object_id in hand.iter().copied() {
                    let mana_cost = self
                        .mana_cost(object_id)
                        .expect("objects in hand always have mana cost");
                    if mana_cost > mana {
                        continue;
                    }

                    let def_id = self.def_id(object_id);
                    match self.binder.preconditions(def_id).is_empty() {
                        true => {
                            actions.push(Action::Play { object_id });
                        }
                        false => {
                            if self.can_fill_preconditions(object_id) {
                                actions.push(Action::Draft { object_id });
                            }
                        }
                    }
                }

                actions.push(Action::EndTurn);
            }
            PlayerInteractionState::PendingPick { options } => {
                actions.extend(
                    options
                        .iter()
                        .cloned()
                        .map(|object_id| Action::Pick { object_id }),
                );
            }
            PlayerInteractionState::Draft { of_object_id } => {
                let def_id = self.def_id(*of_object_id);
                for precondition in self.binder.preconditions(def_id) {
                    if self.precondition_fulfilled(*of_object_id, precondition) {
                        continue;
                    }

                    match precondition {
                        Precondition::Chosen(choice_id) => {
                            self.scan_for_valid_choices(*of_object_id, *choice_id)
                                .iter()
                                .for_each(|&object_id| {
                                    // TODO: Check if by choosing this one we can still fill
                                    // preconditions
                                    actions.push(Action::Choose {
                                        choice_id: *choice_id,
                                        object_id,
                                    })
                                });
                        }
                    }
                }
                if actions.is_empty() {
                    actions.push(Action::Play {
                        object_id: *of_object_id,
                    });
                }
                actions.push(Action::Cancel {
                    object_id: *of_object_id,
                });
            }
            PlayerInteractionState::Choosing {
                object_id,
                choice_id,
            } => {
                self.scan_for_valid_choices(*object_id, *choice_id)
                    .iter()
                    .for_each(|&object_id| {
                        actions.push(Action::Choose {
                            choice_id: *choice_id,
                            object_id,
                        })
                    });
            }
        }
        actions
    }

    pub fn hand(&self, player_id: PlayerId) -> &[ObjectId] {
        self.get_player(player_id).zones.hand.as_slice()
    }

    /// A copy, top first.
    pub fn deck(&self, player_id: PlayerId) -> Vec<ObjectId> {
        self.get_player(player_id).zones.deck.to_vec()
    }

    pub fn board(&self, player_id: PlayerId) -> &[ObjectId] {
        self.get_player(player_id).zones.board.as_slice()
    }

    pub fn mana(&self, player_id: PlayerId) -> u8 {
        self.get_player(player_id).mana
    }

    /// The cards a pending Forage revealed. Empty if nothing is pending.
    pub fn revealed(&self, player_id: PlayerId) -> &[ObjectId] {
        match &self.get_player(player_id).interaction_state {
            PlayerInteractionState::PendingPick { options } => options,
            _ => &[],
        }
    }

    pub fn outcome(&self) -> Option<Outcome> {
        self.outcome
    }

    fn can_fill_preconditions(&self, _object_id: ObjectId) -> bool {
        true
    }

    fn scan_for_valid_choices(&self, chooser_id: ObjectId, choice_id: ChoiceId) -> Vec<ObjectId> {
        let mut valid_choices = Vec::new();
        for object_id in self
            .players
            .iter()
            .flat_map(|p| p.zones.board.as_slice().iter().copied())
        {
            if self.is_valid_choice(chooser_id, choice_id, object_id) {
                valid_choices.push(object_id);
            }
        }
        for object_id in self.players.iter().map(|p| p.zones.hero) {
            if self.is_valid_choice(chooser_id, choice_id, object_id) {
                valid_choices.push(object_id);
            }
        }
        valid_choices
    }

    fn precondition_fulfilled(&self, object_id: ObjectId, precondition: &Precondition) -> bool {
        match precondition {
            Precondition::Chosen(choice_id) => self.choice_fulfilled(object_id, *choice_id),
        }
    }

    fn choice_fulfilled(&self, object_id: ObjectId, choice_id: ChoiceId) -> bool {
        let obj = self.objects.get(object_id);
        let choice = self.binder.choice(obj.def_id, choice_id).unwrap();
        let Some(targets) = obj
            .choice_targets
            .iter()
            .find(|c| c.choice_id == choice_id)
            .map(|c| &c.targets)
        else {
            return false;
        };

        let count = self.effect_amount(choice.count, obj.player_id);
        let chosen_count = targets.len() as u8;
        let fulfill_count = choice.bound.is_satisfied(chosen_count, count);

        // TODO: Not needed
        let fulfill_unique = !choice.unique || {
            targets
                .iter()
                .enumerate()
                .all(|(i, x)| !targets[i + 1..].contains(x))
        };

        // TODO: Not needed
        let fulfill_filter = targets
            .iter()
            .copied()
            .all(|t| self.fulfill_character_selection_filter(t, obj.player_id, choice.filter));

        fulfill_count && fulfill_unique && fulfill_filter
    }

    fn is_valid_choice(
        &self,
        chooser_id: ObjectId,
        choice_id: ChoiceId,
        object_id: ObjectId,
    ) -> bool {
        let obj = self.objects.get(chooser_id);
        let choice = self.binder.choice(obj.def_id, choice_id).unwrap();
        let targets = obj
            .choice_targets
            .iter()
            .find(|c| c.choice_id == choice_id)
            .map(|c| c.targets.clone())
            .unwrap_or_default();

        let count = self.effect_amount(choice.count, obj.player_id);
        let chosen_count = targets.len() as u8;
        let fulfill_count =
            (chosen_count + 1) <= choice.bound.upper_bound(count).unwrap_or(u8::MAX);
        let fulfill_unique = !targets.contains(&object_id);
        let fulfill_filter =
            self.fulfill_character_selection_filter(object_id, obj.player_id, choice.filter);

        fulfill_count && fulfill_unique && fulfill_filter
    }

    fn fulfill_character_selection_filter(
        &self,
        object_id: ObjectId,
        player_id: PlayerId,
        CharacterSelectorFilter { kind, side }: CharacterSelectorFilter,
    ) -> bool {
        let obj = self.objects.get(object_id);
        fulfill_chacter_side_filter(side, obj.player_id, player_id)
            || self.fulfill_chacter_kind_filter(kind, obj.def_id)
    }

    fn end_turn(&mut self, obs: &mut impl Observer) {
        let active_player_id = self.turn_order.get_active_player_id();
        obs.event(&Event::TurnEnded {
            player_id: active_player_id,
        });
        self.turn_order.end_turn();
    }

    fn start_turn(&mut self, obs: &mut impl Observer) {
        let active_player_id = self.turn_order.get_active_player_id();
        obs.event(&Event::TurnStarted {
            player_id: active_player_id,
        });
        let player = self.get_player_mut(active_player_id);
        player.max_mana = (player.max_mana + 1).min(10);
        player.mana = player.max_mana;
        self.draw(active_player_id, 1, obs);
    }

    fn get_player(&self, player_id: PlayerId) -> &Player {
        &self.players[player_id.idx()]
    }

    fn get_player_mut(&mut self, player_id: PlayerId) -> &mut Player {
        &mut self.players[player_id.idx()]
    }

    pub fn players(&self) -> Vec<PlayerId> {
        self.players.iter().map(|p| p.id).collect()
    }
}
