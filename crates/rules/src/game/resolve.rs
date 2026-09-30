//! The mutation half of `Game::apply`: runs an action and resolves the card it plays.
//!
//! Only called after `legal_actions` has listed the action, so nothing here
//! re-checks legality. Tested through `Game`'s interface (tests/contract.rs).

use super::Game;
use crate::action::Action;
use crate::cards::Card;
use crate::ids::PlayerId;

impl Game {
    pub(super) fn apply_action(&mut self, player_id: PlayerId, action: Action) {
        match action {
            Action::Play { hand_index } => {
                let player = self.get_player_mut(player_id);
                let card = player.hand.remove(hand_index);
                player.mana -= card.mana_cost();
                self.apply_card(player_id, card);
            }
            Action::Pick { index } => {
                let player = self.get_player_mut(player_id);
                let (picked, other_cards) = player.pick_revealed(index);
                player.hand.add(picked);
                other_cards
                    .into_iter()
                    .for_each(|c| player.deck.push_back(c));
            }
            Action::EndTurn => {
                self.turn_order.end_turn();
                self.start_turn();
            }
        }
    }

    fn apply_card(&mut self, player_id: PlayerId, card: Card) {
        match card {
            Card::Bolt { damage } => {
                let target_player_id = PlayerId::new((player_id.idx() + 1) % self.players.len());
                let target_player = self.get_player_mut(target_player_id);
                target_player.health -= damage as i32;
            }
            Card::WildBolt => {
                let target_player_id = PlayerId::new(self.rng.below(self.players.len()));
                let target_player = self.get_player_mut(target_player_id);
                target_player.health -= 3;
            }
            Card::Forage => {
                let player = self.get_player_mut(player_id);
                player.reveal(2);
            }
        }
    }
}
