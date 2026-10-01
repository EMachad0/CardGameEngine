//! One player's record inside `Game`: resources, zones, and any pending choice.
//!
//! Fields are `pub(super)`, so `Game` can change them and nothing outside
//! `game` can see them.

use crate::ObjectId;
use crate::ids::PlayerId;
use crate::zones::Zones;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) enum PlayerInteractionState {
    #[default]
    Board,
    Picker {
        options: Vec<ObjectId>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Player {
    pub(super) id: PlayerId,
    pub(super) mana: u8,
    pub(super) max_mana: u8,
    pub(super) health: i32,
    pub(super) interaction_state: PlayerInteractionState,
    pub(super) zones: Zones,
}

impl Player {
    pub(super) fn new(id: PlayerId) -> Self {
        Self {
            id,
            mana: 0,
            max_mana: 0,
            health: 10,
            interaction_state: PlayerInteractionState::default(),
            zones: Zones::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::testkit::*;
    //
    // fn player_with(deck: Vec<Card>) -> Player {
    //     Player::new(P0, Deck::new(deck))
    // }

    // #[test]
    // fn draw_moves_the_top_card_to_the_end_of_the_hand() {
    //     let mut p = player_with(vec![bolt(1), bolt(2), bolt(3)]);
    //     p.draw(2);
    //     assert_eq!(p.hand.as_slice(), [bolt(1), bolt(2)]);
    //     assert_eq!(p.deck.to_vec(), [bolt(3)]);
    //     assert_eq!(p.health, 10);
    // }

    // #[test]
    // fn each_draw_from_an_empty_deck_costs_one_health() {
    //     let mut p = player_with(vec![bolt(1)]);
    //     p.draw(3);
    //     assert_eq!(p.hand.as_slice(), [bolt(1)]);
    //     assert_eq!(p.health, 8);
    // }
    //
    // #[test]
    // fn reveal_takes_up_to_count_from_the_top() {
    //     let mut p = player_with(vec![bolt(1), bolt(2), bolt(3)]);
    //     p.reveal(2);
    //     assert_eq!(
    //         p.interaction_state,
    //         PlayerInteractionState::Picker {
    //             options: vec![bolt(1), bolt(2)]
    //         }
    //     );
    //     assert_eq!(p.deck.to_vec(), [bolt(3)]);
    //
    //     let mut short = player_with(vec![bolt(1)]);
    //     short.reveal(2);
    //     assert_eq!(
    //         short.interaction_state,
    //         PlayerInteractionState::Picker {
    //             options: vec![bolt(1)]
    //         }
    //     );
    // }
    //
    // #[test]
    // fn reveal_on_an_empty_deck_leaves_nothing_pending() {
    //     let mut p = player_with(Vec::new());
    //     p.reveal(2);
    //     assert_eq!(p.interaction_state, PlayerInteractionState::Board);
    // }
    //
    // #[test]
    // fn pick_returns_the_choice_and_the_rest_in_reveal_order() {
    //     let mut p = player_with(vec![bolt(1), bolt(2), bolt(3), bolt(4)]);
    //     p.reveal(3);
    //     assert_eq!(p.pick_revealed(1), (bolt(2), vec![bolt(1), bolt(3)]));
    //     assert_eq!(p.interaction_state, PlayerInteractionState::Board);
    // }
}
