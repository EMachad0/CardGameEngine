//! One player's record inside `Game`: resources, zones, and any pending choice.
//!
//! Fields are `pub(super)`: the rules that change them live in `Game`'s
//! implementation, and nothing outside `game` can see them.

use crate::cards::Card;
use crate::ids::PlayerId;
use crate::zones::{Deck, Hand};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) enum PlayerInteractionState {
    #[default]
    Board,
    Picker {
        options: Vec<Card>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Player {
    pub(super) id: PlayerId,
    pub(super) mana: u8,
    pub(super) max_mana: u8,
    pub(super) health: i32,
    pub(super) hand: Hand,
    pub(super) deck: Deck,
    pub(super) interaction_state: PlayerInteractionState,
}

impl Player {
    pub(super) fn new(id: PlayerId, deck: Deck) -> Self {
        Self {
            id,
            mana: 0,
            max_mana: 0,
            health: 10,
            hand: Hand::empty(),
            deck,
            interaction_state: PlayerInteractionState::default(),
        }
    }

    /// Top of the deck to the end of the hand, `count` times.
    /// Each draw from an empty deck costs 1 health instead.
    pub(super) fn draw(&mut self, count: u8) {
        for _ in 0..count {
            if let Some(card) = self.deck.pop_front() {
                self.hand.add(card);
            } else {
                self.health -= 1;
            };
        }
    }

    /// Takes up to `count` cards off the top into a pending pick.
    /// If the deck is empty, nothing becomes pending.
    pub(super) fn reveal(&mut self, count: u8) {
        let mut options = Vec::new();
        for _ in 0..count {
            let Some(card) = self.deck.pop_front() else {
                break;
            };

            options.push(card);
        }
        if !options.is_empty() {
            self.interaction_state = PlayerInteractionState::Picker { options }
        }
    }

    /// Ends the pending pick: returns the chosen card and the rest in reveal order.
    /// Only valid while a pick is pending; `legal_actions` guarantees that.
    pub(super) fn pick_revealed(&mut self, index: usize) -> (Card, Vec<Card>) {
        let PlayerInteractionState::Picker { mut options } =
            std::mem::take(&mut self.interaction_state)
        else {
            unreachable!();
        };
        let picked = options.remove(index);
        (picked, options)
    }
}

#[cfg(test)]
mod tests {
    use super::{Player, PlayerInteractionState};
    use crate::cards::Card;
    use crate::ids::PlayerId;
    use crate::zones::Deck;

    fn bolt(damage: u8) -> Card {
        Card::Bolt { damage }
    }

    fn player_with(deck: Vec<Card>) -> Player {
        Player::new(PlayerId::new(0), Deck::new(deck))
    }

    #[test]
    fn draw_moves_the_top_card_to_the_end_of_the_hand() {
        let mut p = player_with(vec![bolt(1), bolt(2), bolt(3)]);
        p.draw(2);
        assert_eq!(p.hand.as_slice(), [bolt(1), bolt(2)]);
        assert_eq!(p.deck.to_vec(), [bolt(3)]);
        assert_eq!(p.health, 10);
    }

    #[test]
    fn each_draw_from_an_empty_deck_costs_one_health() {
        let mut p = player_with(vec![bolt(1)]);
        p.draw(3);
        assert_eq!(p.hand.as_slice(), [bolt(1)]);
        assert_eq!(p.health, 8);
    }

    #[test]
    fn reveal_takes_up_to_count_from_the_top() {
        let mut p = player_with(vec![bolt(1), bolt(2), bolt(3)]);
        p.reveal(2);
        assert_eq!(
            p.interaction_state,
            PlayerInteractionState::Picker {
                options: vec![bolt(1), bolt(2)]
            }
        );
        assert_eq!(p.deck.to_vec(), [bolt(3)]);

        let mut short = player_with(vec![bolt(1)]);
        short.reveal(2);
        assert_eq!(
            short.interaction_state,
            PlayerInteractionState::Picker {
                options: vec![bolt(1)]
            }
        );
    }

    #[test]
    fn reveal_on_an_empty_deck_leaves_nothing_pending() {
        let mut p = player_with(Vec::new());
        p.reveal(2);
        assert_eq!(p.interaction_state, PlayerInteractionState::Board);
    }

    #[test]
    fn pick_returns_the_choice_and_the_rest_in_reveal_order() {
        let mut p = player_with(vec![bolt(1), bolt(2), bolt(3), bolt(4)]);
        p.reveal(3);
        assert_eq!(p.pick_revealed(1), (bolt(2), vec![bolt(1), bolt(3)]));
        assert_eq!(p.interaction_state, PlayerInteractionState::Board);
    }
}
