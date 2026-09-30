//! `Game`: the whole state, and the interface a shell drives it through.
//!
//! `legal_actions` is the one definition of legality. `apply` accepts exactly
//! what it lists, and an `Err` leaves the game unchanged. Child modules:
//! - `player`: one player's resources, zones and pending choice.
//! - `resolve`: the mutation half of `apply`.

mod player;
mod resolve;

use crate::action::{Action, Illegal};
use crate::cards::Card;
use crate::ids::PlayerId;
use crate::rng::Rng;
use crate::turn::TurnOrder;
use crate::zones::Deck;
use player::{Player, PlayerInteractionState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    rng: Rng,
    turn_order: TurnOrder,
    players: Vec<Player>,
}

impl Game {
    /// Seeds the `Rng`, shuffles both decks, deals 3 cards each and starts player 0's turn.
    pub fn new(seed: u64, decks: [Vec<Card>; 2]) -> Self {
        Self::setup(seed, decks, true)
    }

    /// Like `new`, but keeps each deck in the given order, index 0 on top.
    pub fn with_deck_order(seed: u64, decks: [Vec<Card>; 2]) -> Self {
        Self::setup(seed, decks, false)
    }

    fn setup(seed: u64, decks: [Vec<Card>; 2], shuffle: bool) -> Self {
        let mut rng = Rng::new(seed);
        let turn_order = TurnOrder::new(decks.len());

        let mut players = decks
            .into_iter()
            .enumerate()
            .map(|(i, deck)| Player::new(PlayerId::new(i), Deck::new(deck)))
            .collect::<Vec<_>>();

        for player in players.iter_mut() {
            if shuffle {
                player.deck.shuffle(&mut rng);
            }
            player.draw(3);
        }

        let mut game = Self {
            rng,
            turn_order,
            players,
        };
        game.start_turn();
        game
    }

    pub fn legal_actions(&self, player_id: PlayerId) -> Vec<Action> {
        if self.winner().is_some() {
            return Vec::new();
        }
        if self.turn_order.get_current_player_id() != player_id {
            return Vec::new();
        }

        let mut actions = Vec::new();
        match &self.get_player(player_id).interaction_state {
            PlayerInteractionState::Board => {
                let mana = self.mana(player_id);
                let hand = self.hand(player_id);
                for (i, card) in hand.iter().enumerate() {
                    if card.mana_cost() <= mana {
                        actions.push(Action::Play { hand_index: i });
                    }
                }

                actions.push(Action::EndTurn);
            }
            PlayerInteractionState::Picker { options } => {
                actions.extend((0..options.len()).map(|i| Action::Pick { index: i }));
            }
        }
        actions
    }

    /// `Ok` if and only if `action` is in `legal_actions(player_id)`.
    /// An `Err` leaves the game unchanged.
    pub fn apply(&mut self, player_id: PlayerId, action: Action) -> Result<(), Illegal> {
        if !self.legal_actions(player_id).contains(&action) {
            Err(Illegal::new(player_id, action))
        } else {
            self.apply_action(player_id, action);
            Ok(())
        }
    }

    pub fn applied(&self, player_id: PlayerId, action: Action) -> Result<Self, Illegal> {
        let mut game = self.clone();
        game.apply(player_id, action)?;
        Ok(game)
    }

    pub fn hand(&self, player_id: PlayerId) -> &[Card] {
        self.get_player(player_id).hand.as_slice()
    }

    /// A copy, top first.
    pub fn deck(&self, player_id: PlayerId) -> Vec<Card> {
        self.get_player(player_id).deck.to_vec()
    }

    pub fn mana(&self, player_id: PlayerId) -> u8 {
        self.get_player(player_id).mana
    }

    pub fn health(&self, player_id: PlayerId) -> i32 {
        self.get_player(player_id).health
    }

    /// The cards a pending Forage revealed. Empty if nothing is pending.
    pub fn revealed(&self, player_id: PlayerId) -> &[Card] {
        match &self.get_player(player_id).interaction_state {
            PlayerInteractionState::Board => &[],
            PlayerInteractionState::Picker { options } => options,
        }
    }

    pub fn winner(&self) -> Option<PlayerId> {
        let mut alive = self.players.iter().filter(|s| s.health > 0);
        match (alive.next(), alive.next()) {
            (Some(player), None) => Some(player.id),
            _ => None,
        }
    }

    fn start_turn(&mut self) {
        let current_player_id = self.turn_order.get_current_player_id();
        let player = self.get_player_mut(current_player_id);
        player.max_mana = (player.max_mana + 1).min(10);
        player.mana = player.max_mana;
        player.draw(1);
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
    use super::*;
    use crate::testkit::*;

    fn sample_deck() -> Vec<Card> {
        vec![
            bolt(1),
            bolt(2),
            Card::Forage,
            bolt(3),
            Card::WildBolt,
            bolt(1),
            Card::Forage,
            bolt(2),
            bolt(4),
            Card::WildBolt,
        ]
    }

    #[test]
    fn setup_draws_three_each_then_starts_player_0s_turn() {
        let deck0 = vec![bolt(1), bolt(2), bolt(3), bolt(4), bolt(5)];
        let deck1 = vec![bolt(1), bolt(1), bolt(1), bolt(2), bolt(2)];
        let game = Game::with_deck_order(0, [deck0, deck1]);

        assert_eq!(game.hand(P0), &[bolt(1), bolt(2), bolt(3), bolt(4)]);
        assert_eq!(game.deck(P0), &[bolt(5)]);
        assert_eq!(game.hand(P1), &[bolt(1), bolt(1), bolt(1)]);
        assert_eq!(game.deck(P1), &[bolt(2), bolt(2)]);
        assert_eq!(game.mana(P0), 1);
        assert_eq!(game.mana(P1), 0);
        assert_eq!(game.health(P0), 10);
        assert_eq!(game.health(P1), 10);
        assert_eq!(game.winner(), None);
        assert!(game.revealed(P0).is_empty());

        assert_actions(&game, P0, &[play(0), Action::EndTurn]);
        assert_actions(&game, P1, &[]);
    }

    #[test]
    fn new_actually_shuffles() {
        let unshuffled = Game::with_deck_order(0, [sample_deck(), sample_deck()]);
        let differs = (0..20).any(|seed| {
            let g = Game::new(seed, [sample_deck(), sample_deck()]);
            g.hand(P0) != unshuffled.hand(P0) || g.deck(P0) != unshuffled.deck(P0)
        });
        assert!(differs, "20 seeds and never a different deck order");
    }

    #[test]
    fn end_turn_starts_the_opponents_turn() {
        let deck = vec![bolt(1), bolt(2), bolt(3), bolt(4), bolt(5), bolt(6)];
        let mut game = Game::with_deck_order(0, [deck.clone(), deck]);

        game.apply(P0, Action::EndTurn).unwrap();
        assert_eq!(game.mana(P1), 1);
        assert_eq!(game.hand(P1), &[bolt(1), bolt(2), bolt(3), bolt(4)]);
        assert_actions(&game, P0, &[]);
        assert_actions(&game, P1, &[play(0), Action::EndTurn]);

        game.apply(P1, Action::EndTurn).unwrap();
        assert_eq!(game.mana(P0), 2, "max mana grows by one per turn");
        assert_eq!(
            game.hand(P0),
            &[bolt(1), bolt(2), bolt(3), bolt(4), bolt(5)]
        );
        assert_actions(&game, P0, &[play(0), play(1), Action::EndTurn]);
    }

    #[test]
    fn mana_caps_at_ten() {
        let deck = vec![bolt(9); 20];
        let mut game = Game::with_deck_order(0, [deck.clone(), deck]);
        for _ in 0..12 {
            game.apply(P0, Action::EndTurn).unwrap();
            game.apply(P1, Action::EndTurn).unwrap();
        }
        assert_eq!(game.mana(P0), 10);
    }

    #[test]
    fn drawing_from_an_empty_deck_deals_one_damage() {
        let deck1 = vec![bolt(1), bolt(1), bolt(1)];
        let mut game = Game::with_deck_order(0, [vec![bolt(1); 6], deck1]);

        game.apply(P0, Action::EndTurn).unwrap();
        assert_eq!(game.health(P1), 9);
        assert_eq!(game.hand(P1).len(), 3);
    }

    #[test]
    fn lethal_ends_the_game_mid_turn() {
        // Player 1's deck runs out at setup, so they take 1 fatigue each turn.
        // Player 0 plays Bolt 9 as soon as mana reaches 9.
        let deck0 = vec![bolt(9); 10];
        let deck1 = vec![bolt(9); 3];
        let mut game = Game::with_deck_order(0, [deck0, deck1]);

        for _ in 0..8 {
            game.apply(P0, Action::EndTurn).unwrap();
            game.apply(P1, Action::EndTurn).unwrap();
        }
        assert_eq!(game.health(P1), 2);
        assert_eq!(game.mana(P0), 9);

        game.apply(P0, play(0)).unwrap();

        assert_eq!(game.winner(), Some(P0));
        assert_actions(&game, P0, &[]);
        assert_actions(&game, P1, &[]);
    }

    #[test]
    fn fatigue_at_turn_start_can_end_the_game() {
        let mut game = Game::with_deck_order(0, [vec![bolt(9); 30], vec![bolt(9); 3]]);
        while game.winner().is_none() {
            game.apply(P0, Action::EndTurn).unwrap();
            if game.winner().is_none() {
                game.apply(P1, Action::EndTurn).unwrap();
            }
        }
        assert_eq!(game.winner(), Some(P0));
        assert!(game.health(P1) <= 0);
    }

    #[test]
    fn applied_returns_a_new_game_and_leaves_the_original_alone() {
        let deck = vec![bolt(1); 6];
        let game = Game::with_deck_order(0, [deck.clone(), deck]);
        let snapshot = game.clone();

        let next = game.applied(P0, play(0)).unwrap();
        assert_eq!(game, snapshot);
        assert_eq!(next.health(P1), 9);

        assert_eq!(
            game.applied(P1, Action::EndTurn),
            Err(Illegal {
                player: P1,
                action: Action::EndTurn
            })
        );
    }
}
