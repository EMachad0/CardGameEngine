//! `Game`: the whole state, and the interface a shell drives it through.

mod actions;
mod derived;
mod draft;
mod filter;
mod history;
mod pick;
mod player;
mod resolve;
mod state_check;
mod trigger;
mod verb;
pub(crate) mod view;

use crate::cards::{
    binder::Binder,
    def_ids::static_card_definition::HERO,
    definition::{CardDef, DefId},
    effect::Effect,
    loader::CardDefLoader,
};
use crate::event::Event;
use crate::game::history::History;
use crate::game::player::{Player, PlayerInteractionState};
use crate::ids::PlayerId;
use crate::object::{Object, ObjectBag, ObjectId};
use crate::observer::Observer;
use crate::outcome::Outcome;
use crate::rng::Rng;
use crate::stack::EffectStack;
use crate::turn::TurnOrder;
use crate::zones::Deck;

#[derive(Debug, Clone, PartialEq)]
pub struct Game {
    rng: Rng,
    turn_order: TurnOrder,
    players: Vec<Player>,
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
            outcome: Default::default(),
            effect_stack: Default::default(),
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

    pub fn hand(&self, player_id: PlayerId) -> &[ObjectId] {
        self.get_player(player_id).zones.hand.as_slice()
    }

    /// A copy, top first.
    pub fn deck(&self, player_id: PlayerId) -> Vec<ObjectId> {
        self.get_player(player_id).zones.deck.to_vec()
    }

    pub fn player_board(&self, player_id: PlayerId) -> &[ObjectId] {
        self.get_player(player_id).zones.board.as_slice()
    }

    fn board_characters(&self) -> Vec<ObjectId> {
        let heroes = self.players.iter().map(|p| p.zones.hero);
        let minions = self
            .players()
            .into_iter()
            .flat_map(|p| self.player_board(p))
            .cloned();

        let mut characters = heroes.chain(minions).collect::<Vec<_>>();
        characters.sort_by_key(|o| self.objects.get(*o).order);
        characters
    }

    pub fn mana(&self, player_id: PlayerId) -> u8 {
        self.get_player(player_id).mana
    }

    /// The cards a pending Forage revealed. Empty if nothing is pending.
    pub fn revealed(&self, player_id: PlayerId) -> &[ObjectId] {
        match &self.get_player(player_id).interaction_state {
            PlayerInteractionState::PendingPick { options, .. } => options,
            _ => &[],
        }
    }

    pub fn outcome(&self) -> Option<Outcome> {
        self.outcome
    }

    fn object(&self, object_id: ObjectId) -> &Object {
        self.objects.get(object_id)
    }

    pub fn def_id(&self, object_id: ObjectId) -> DefId {
        self.object(object_id).def_id
    }

    fn card_def(&self, object_id: ObjectId) -> &CardDef {
        self.binder.get(self.def_id(object_id))
    }

    pub fn hero_id(&self, player_id: PlayerId) -> ObjectId {
        self.get_player(player_id).zones.hero
    }

    fn play_effects(&self, object_id: ObjectId) -> &[Effect] {
        self.card_def(object_id).play_effects.as_slice()
    }

    fn playing(&self, player_id: PlayerId) -> bool {
        self.get_player(player_id).playing
    }

    fn end_turn(&mut self, obs: &mut impl Observer) {
        let active_player_id = self.turn_order.get_active_player_id();
        let event = Event::TurnEnded {
            player_id: active_player_id,
        };
        self.emit(event, obs);
        self.turn_order.end_turn();
    }

    fn start_turn(&mut self, obs: &mut impl Observer) {
        let active_player_id = self.turn_order.get_active_player_id();
        let event = Event::TurnStarted {
            player_id: active_player_id,
        };
        self.emit(event, obs);
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
