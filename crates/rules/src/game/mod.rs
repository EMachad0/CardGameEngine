//! `Game`: the whole state, and the interface a shell drives it through.
//!
//! `legal_actions` is the one definition of legality. `apply` accepts exactly
//! what it lists, and an `Err` leaves the game unchanged. Child modules:
//! - `effect_stack`: the pending effects and the resolving effect.
//! - `player`: one player's resources.
//! - `player_input`: what the game awaits from each player, and what they targeted and picked.
//! - `reporting`: recording an event, telling the observer and pushing the triggers it matched.
//! - `resolve`: the mutation half of `apply`.
//! - `lookup`: attack, health and cost, computed on read.
#![expect(
    clippy::definition_in_module_root,
    reason = "the child modules read Game's private fields, which a leaf would hide from them"
)]

mod condition;
mod effect_stack;
mod lookup;
mod player;
mod player_input;
mod reporting;
pub(crate) mod resolve;
mod triggering;
pub(crate) mod view;
mod zone_move;

use crate::action::Action;
use crate::cards::binder::sets::base::static_card_definition::HERO;
use crate::cards::binder::store::Binder;
use crate::cards::definition::DefId;
use crate::cards::object::{ObjectBag, ObjectId};
use crate::game::effect_stack::EffectStack;
use crate::game::player::Player;
use crate::game::player_input::{Awaiting, PlayerInput};
use crate::game::reporting::Report;
use crate::history::History;
use crate::ids::PlayerId;
use crate::observer::Observer;
use crate::outcome::Outcome;
use crate::rng::Rng;
use crate::turn::TurnOrder;
use crate::zones::Zones;

#[derive(Debug, Clone, PartialEq)]
pub struct Game {
    rng: Rng,
    turn_order: TurnOrder,
    players: Vec<Player>,
    zones: Zones,
    player_input: PlayerInput,
    objects: ObjectBag,
    binder: Binder,
    history: History,
    outcome: Option<Outcome>,
    effect_stack: EffectStack,
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
        let binder = Binder::built_in().unwrap();

        let mut objects = ObjectBag::default();

        let player_count = decks.len();
        let player_ids = (0..player_count).map(PlayerId::new).collect::<Vec<_>>();
        let players = player_ids
            .iter()
            .map(|&player_id| Player::new(player_id))
            .collect::<Vec<_>>();
        let heroes = player_ids
            .iter()
            .map(|&player_id| objects.insert(HERO, player_id))
            .collect::<Vec<_>>();
        let decks = player_ids
            .iter()
            .zip(decks)
            .map(|(&player_id, deck_defs)| {
                deck_defs
                    .into_iter()
                    .map(|def_id| objects.insert(def_id, player_id))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let zones = Zones::new(heroes.into_iter().zip(decks));
        let turn_order = TurnOrder::new(player_ids.clone());
        let history = History::default();

        let mut game = Self {
            rng,
            turn_order,
            players,
            zones,
            player_input: Default::default(),
            objects,
            binder,
            history,
            outcome: Default::default(),
            effect_stack: Default::default(),
        };

        if shuffle {
            for &player_id in &player_ids {
                game.zones.shuffle_deck(player_id, &mut game.rng);
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
        match self.player_input.awaiting(player_id) {
            None => {
                if self.turn_order.get_active_player_id() != player_id
                    || self.player_input.any_awaiting()
                {
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

                    match self.card_def(object_id).targets.is_empty() {
                        true => {
                            actions.push(Action::Play { object_id });
                        }
                        false => {
                            if self.can_finish_targeting(object_id, &mut Vec::new()) {
                                actions.push(Action::StartTargeting { object_id });
                            }
                        }
                    }
                }

                actions.push(Action::EndTurn);
            }
            Some(Awaiting::Pick(pick)) => {
                actions.extend(
                    pick.options()
                        .iter()
                        .cloned()
                        .map(|object_id| Action::Pick { object_id }),
                );
            }
            Some(Awaiting::Targets(targeting)) => {
                let object_id = targeting.object_id();
                let targeted = targeting.targeted();
                let mut all_targets_fulfilled = true;
                for target in &self.card_def(object_id).targets {
                    let targeted_count = targeted
                        .iter()
                        .filter(|(_, target_id)| target.id == *target_id)
                        .count();
                    if self.check_target_count_fulfilled(object_id, target.id, targeted_count as u8)
                    {
                        continue;
                    }

                    let candidates = self.scan_for_valid_targets(object_id, target, targeted);
                    for candidate in candidates.into_iter() {
                        let mut targeted = targeted.to_vec();
                        targeted.push((candidate, target.id));
                        if self.can_finish_targeting(object_id, &mut targeted) {
                            actions.push(Action::Target {
                                target_id: target.id,
                                object_id: candidate,
                            })
                        }
                    }
                    all_targets_fulfilled = false;
                    break;
                }
                if all_targets_fulfilled {
                    actions.push(Action::Play { object_id });
                }
                actions.push(Action::Cancel { object_id });
            }
        }
        actions
    }

    pub fn hand(&self, player_id: PlayerId) -> &[ObjectId] {
        self.zones.hand(player_id)
    }

    /// A copy, top first.
    pub fn deck(&self, player_id: PlayerId) -> Vec<ObjectId> {
        self.zones.deck(player_id)
    }

    pub fn player_board(&self, player_id: PlayerId) -> &[ObjectId] {
        self.zones.board(player_id)
    }

    fn characters_in_order_of_play(&self) -> Vec<ObjectId> {
        let mut characters = self.zones.characters_in_play().collect::<Vec<_>>();
        characters.sort_by_key(|&o| self.zones.match_order(o));
        characters
    }

    pub fn mana(&self, player_id: PlayerId) -> u8 {
        self.get_player(player_id).mana
    }

    /// The cards a pending Forage revealed. Empty if nothing is pending.
    pub fn revealed(&self, player_id: PlayerId) -> &[ObjectId] {
        self.player_input.revealed(player_id)
    }

    pub fn outcome(&self) -> Option<Outcome> {
        self.outcome
    }

    fn end_turn(&mut self, obs: &mut impl Observer) {
        let active_player_id = self.turn_order.get_active_player_id();
        let report = Report::TurnEnded {
            player_id: active_player_id,
        };
        self.report(report, obs);
        self.turn_order.end_turn();
    }

    fn start_turn(&mut self, obs: &mut impl Observer) {
        let active_player_id = self.turn_order.get_active_player_id();
        let report = Report::TurnStarted {
            player_id: active_player_id,
        };
        self.report(report, obs);
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
