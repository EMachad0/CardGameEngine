//! `Game`: the whole state, and the interface a shell drives it through.
//!
//! `legal_actions` is the one definition of legality. `apply` accepts exactly
//! what it lists, and an `Err` leaves the game unchanged. Child modules:
//! - `player`: one player's resources, zones and pending choice.
//! - `resolve`: the mutation half of `apply`.
//! - `derived`: attack, health and cost, computed on read.
//! - `check`: the state check at the end of every `apply`.

mod check;
mod lookup;
mod player;
mod resolve;

use crate::action::Action;
use crate::cards::{Binder, CardDefLoader, Object};
use crate::ids::PlayerId;
use crate::rng::Rng;
use crate::turn::TurnOrder;
use crate::zones::Deck;
use crate::{DefId, ObjectBag, ObjectId, Outcome};
use player::{Player, PlayerInteractionState};

pub use resolve::ApplyError;

#[derive(Debug, Clone, PartialEq)]
pub struct Game {
    rng: Rng,
    turn_order: TurnOrder,
    players: Vec<Player>,
    // todo: only pub because of testkit which is bad design
    pub(crate) objects: ObjectBag,
    binder: Binder,
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
            .map(|player_id| Player::new(*player_id))
            .collect::<Vec<_>>();
        for (player, deck_defs) in players.iter_mut().zip(decks.into_iter()) {
            let deck_objs = deck_defs
                .into_iter()
                .map(|def_id| Object {
                    def_id,
                    object_id: objects.next_id(),
                    player_id: player.id,
                })
                .collect::<Vec<_>>();
            let deck = Deck::new(objects.insert_all(deck_objs));
            player.zones.deck = deck;
        }
        let turn_order = TurnOrder::new(player_ids.clone());

        let mut game = Self {
            rng,
            turn_order,
            players,
            objects,
            binder,
            outcome: None,
        };

        for player in game.players.iter_mut() {
            if shuffle {
                player.zones.deck.shuffle(&mut game.rng);
            }
        }

        for player_id in player_ids {
            game.draw(player_id, 3);
        }

        game.start_turn();
        game
    }

    pub fn legal_actions(&self, player_id: PlayerId) -> Vec<Action> {
        if self.outcome().is_some() {
            return Vec::new();
        }

        let mut actions = Vec::new();
        match &self.get_player(player_id).interaction_state {
            PlayerInteractionState::Board => {
                if self.turn_order.get_current_player_id() != player_id {
                    return Vec::new();
                }

                let mana = self.mana(player_id);
                let hand = self.hand(player_id);
                for object_id in hand.iter().cloned() {
                    if let Ok(Some(mana_cost)) = self.mana_cost(object_id)
                        && mana_cost <= mana
                    {
                        actions.push(Action::Play { object_id });
                    }
                }

                actions.push(Action::EndTurn);
            }
            PlayerInteractionState::Picker { options } => {
                actions.extend(
                    options
                        .iter()
                        .cloned()
                        .map(|object_id| Action::Pick { object_id }),
                );
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
            PlayerInteractionState::Board => &[],
            PlayerInteractionState::Picker { options } => options,
        }
    }

    pub fn outcome(&self) -> Option<Outcome> {
        self.outcome
    }

    fn start_turn(&mut self) {
        let current_player_id = self.turn_order.get_current_player_id();
        let player = self.get_player_mut(current_player_id);
        player.max_mana = (player.max_mana + 1).min(10);
        player.mana = player.max_mana;
        self.draw(current_player_id, 1);
    }

    fn get_player(&self, player_id: PlayerId) -> &Player {
        &self.players[player_id.idx()]
    }

    fn get_player_mut(&mut self, player_id: PlayerId) -> &mut Player {
        &mut self.players[player_id.idx()]
    }
}

#[cfg(test)]
mod tests {
    use crate::cards::{BLAST, BOLT, CAPTAIN, FORAGE, GIANT, RECRUIT, SPARK, WILD_BOLT};
    use crate::game::resolve::ApplyError;
    use crate::testkit::*;
    use crate::{Action, DefId, Game, IllegalAction, ObjectId, Outcome};

    fn sample_deck() -> Vec<DefId> {
        vec![
            SPARK, BOLT, FORAGE, RECRUIT, WILD_BOLT, CAPTAIN, BLAST, GIANT, SPARK, RECRUIT,
        ]
    }

    #[test]
    fn setup_draws_three_each_then_starts_player_0s_turn() {
        let deck0 = vec![SPARK, BOLT, RECRUIT, CAPTAIN, BLAST];
        let deck1 = vec![SPARK, SPARK, SPARK, BOLT, BOLT];
        let game = Game::with_deck_order(0, [deck0, deck1]);

        assert_eq!(hand_defs(&game, P0), [SPARK, BOLT, RECRUIT, CAPTAIN]);
        assert_eq!(deck_defs(&game, P0), [BLAST]);
        assert_eq!(hand_defs(&game, P1), [SPARK, SPARK, SPARK]);
        assert_eq!(deck_defs(&game, P1), [BOLT, BOLT]);
        assert_eq!(game.mana(P0), 1);
        assert_eq!(game.mana(P1), 0);
        assert_eq!(game.hero_health(P0), 10);
        assert_eq!(game.hero_health(P1), 10);
        assert_eq!(game.outcome(), None);
        for p in [P0, P1] {
            assert!(game.revealed(p).is_empty());
            assert!(game.board(p).is_empty());
        }

        let spark = in_hand(&game, P0, SPARK);
        assert_actions(&game, P0, &[play(spark), Action::EndTurn]);
        assert_actions(&game, P1, &[]);
    }

    #[test]
    fn copies_of_one_card_are_distinct_objects() {
        let deck = vec![RECRUIT; 8];
        let game = Game::with_deck_order(0, [deck.clone(), deck]);

        let mut ids: Vec<ObjectId> = [P0, P1]
            .into_iter()
            .flat_map(|p| [game.hand(p), &game.deck(p)].concat())
            .collect();
        assert_eq!(ids.len(), 16);
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 16, "two cards share an ObjectId");
    }

    #[test]
    fn each_copy_in_hand_is_its_own_play_action() {
        let deck = vec![RECRUIT; 8];
        let mut game = Game::with_deck_order(0, [deck.clone(), deck]);
        turn_with_mana(&mut game, P0, 2);

        let mut expected: Vec<Action> = game.hand(P0).into_iter().map(|o| play(*o)).collect();
        assert_eq!(expected.len(), 5);
        expected.push(Action::EndTurn);
        assert_actions(&game, P0, &expected);
    }

    #[test]
    fn new_actually_shuffles() {
        let unshuffled = Game::with_deck_order(0, [sample_deck(), sample_deck()]);
        let differs = (0..20).any(|seed| {
            let g = Game::new(seed, [sample_deck(), sample_deck()]);
            hand_defs(&g, P0) != hand_defs(&unshuffled, P0)
                || deck_defs(&g, P0) != deck_defs(&unshuffled, P0)
        });
        assert!(differs, "20 seeds and never a different deck order");
    }

    #[test]
    fn end_turn_starts_the_opponents_turn() {
        let deck = vec![SPARK, SPARK, BOLT, BOLT, RECRUIT, RECRUIT];
        let mut game = Game::with_deck_order(0, [deck.clone(), deck]);

        game.apply(P0, Action::EndTurn).unwrap();
        assert_eq!(game.mana(P1), 1);
        assert_eq!(hand_defs(&game, P1), [SPARK, SPARK, BOLT, BOLT]);
        let hand = game.hand(P1);
        assert_actions(&game, P0, &[]);
        assert_actions(&game, P1, &[play(hand[0]), play(hand[1]), Action::EndTurn]);

        game.apply(P1, Action::EndTurn).unwrap();
        assert_eq!(game.mana(P0), 2, "max mana grows by one per turn");
        assert_eq!(hand_defs(&game, P0), [SPARK, SPARK, BOLT, BOLT, RECRUIT]);
        let mut expected: Vec<Action> = game.hand(P0).into_iter().map(|o| play(*o)).collect();
        expected.push(Action::EndTurn);
        assert_actions(&game, P0, &expected);
    }

    #[test]
    fn mana_caps_at_ten() {
        let deck = vec![BOLT; 30];
        let mut game = Game::with_deck_order(0, [deck.clone(), deck]);
        for _ in 0..24 {
            end_turn(&mut game);
        }
        assert_eq!(game.mana(P0), 10);
    }

    #[test]
    fn drawing_from_an_empty_deck_deals_one_damage() {
        let mut game = Game::with_deck_order(0, [vec![BOLT; 6], vec![SPARK; 3]]);

        game.apply(P0, Action::EndTurn).unwrap();
        assert_eq!(game.hero_health(P1), 9);
        assert_eq!(game.hand(P1).len(), 3);
    }

    #[test]
    fn fatigue_at_turn_start_can_end_the_game() {
        let mut game = Game::with_deck_order(0, [vec![BOLT; 30], vec![BOLT; 3]]);
        end_turns_until_over(&mut game);
        assert_eq!(game.outcome(), Some(Outcome::Won(P0)));
        assert!(game.hero_health(P1) <= 0);
        assert_actions(&game, P0, &[]);
        assert_actions(&game, P1, &[]);
    }

    #[test]
    fn applied_returns_a_new_game_and_leaves_the_original_alone() {
        let deck = vec![SPARK; 6];
        let game = Game::with_deck_order(0, [deck.clone(), deck]);
        let snapshot = game.clone();

        let next = game.applied(P0, play(in_hand(&game, P0, SPARK))).unwrap();
        assert_eq!(game, snapshot);
        assert_eq!(next.hero_health(P1), 9);

        assert_eq!(
            game.applied(P1, Action::EndTurn),
            Err(ApplyError::IllegalAction(IllegalAction {
                player_id: P1,
                action: Action::EndTurn
            }))
        );
    }
}
