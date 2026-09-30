//! The mutation half of `Game::apply`. It runs an action and resolves any card
//! the action plays.
//!
//! Every action that reaches this module is already in `legal_actions`, so
//! nothing here re-checks legality.

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::*;

    #[test]
    fn playing_a_bolt_pays_mana_and_hits_the_enemy() {
        let deck = vec![bolt(1), bolt(2), bolt(3), bolt(4), bolt(5)];
        let mut game = Game::with_deck_order(0, [deck.clone(), deck]);

        game.apply(P0, play(0)).unwrap();

        assert_eq!(game.health(P1), 9);
        assert_eq!(game.mana(P0), 0);
        assert_eq!(game.hand(P0), &[bolt(2), bolt(3), bolt(4)]);
        assert_actions(&game, P0, &[Action::EndTurn]);
    }

    #[test]
    fn every_card_pays_its_cost() {
        // Written out from SPEC.md, so a wrong cost in the core can't check itself.
        let cases = [
            (bolt(1), 1),
            (bolt(3), 3),
            (Card::WildBolt, 1),
            (Card::Forage, 1),
        ];
        for (card, cost) in cases {
            // The card is on top, so it's `hand[0]` after the opening draw.
            let mut deck0 = vec![card];
            deck0.extend(vec![bolt(9); 20]);
            let mut game = Game::with_deck_order(0, [deck0, vec![bolt(9); 20]]);
            while game.mana(P0) < cost {
                game.apply(P0, Action::EndTurn).unwrap();
                game.apply(P1, Action::EndTurn).unwrap();
            }
            assert_eq!(game.hand(P0)[0], card);

            let before = game.mana(P0);
            game.apply(P0, play(0)).unwrap();
            assert_eq!(game.mana(P0), before - cost, "{card:?}");
        }
    }

    #[test]
    fn forage_offers_only_picks_until_one_is_made() {
        let deck0 = vec![
            Card::Forage,
            bolt(1),
            bolt(1),
            bolt(1),
            bolt(5),
            bolt(6),
            bolt(2),
        ];
        let deck1 = vec![bolt(1); 6];
        let mut game = Game::with_deck_order(0, [deck0, deck1]);

        game.apply(P0, play(0)).unwrap();

        assert_eq!(game.revealed(P0), &[bolt(5), bolt(6)]);
        assert_eq!(game.deck(P0), &[bolt(2)]);
        assert_actions(&game, P0, &[pick(0), pick(1)]);
        assert_actions(&game, P1, &[]);

        game.apply(P0, pick(1)).unwrap();

        assert!(game.revealed(P0).is_empty());
        assert_eq!(game.hand(P0), &[bolt(1), bolt(1), bolt(1), bolt(6)]);
        assert_eq!(
            game.deck(P0),
            &[bolt(2), bolt(5)],
            "unpicked card goes to the bottom"
        );
        assert_actions(&game, P0, &[Action::EndTurn]);
    }

    #[test]
    fn forage_with_one_card_left_still_asks_for_the_pick() {
        let deck0 = vec![Card::Forage, bolt(1), bolt(1), bolt(1), bolt(5)];
        let mut game = Game::with_deck_order(0, [deck0, vec![bolt(1); 6]]);

        game.apply(P0, play(0)).unwrap();
        assert_eq!(game.revealed(P0), &[bolt(5)]);
        assert_actions(&game, P0, &[pick(0)]);

        game.apply(P0, pick(0)).unwrap();
        assert_eq!(game.hand(P0), &[bolt(1), bolt(1), bolt(1), bolt(5)]);
        assert!(game.deck(P0).is_empty());
    }

    #[test]
    fn forage_on_an_empty_deck_does_nothing() {
        let deck0 = vec![Card::Forage, bolt(1), bolt(1), bolt(1)];
        let mut game = Game::with_deck_order(0, [deck0, vec![bolt(1); 6]]);

        game.apply(P0, play(0)).unwrap();
        assert!(game.revealed(P0).is_empty());
        assert_eq!(game.hand(P0), &[bolt(1), bolt(1), bolt(1)]);
        assert_actions(&game, P0, &[Action::EndTurn]);
    }

    /// WildBolt's target is random, so its tests assert what holds for every seed.
    /// A test that pins one seed to an outcome breaks as soon as unrelated code
    /// takes an earlier number from the RNG.
    fn cast_wild_bolt(seed: u64) -> (i32, i32) {
        let deck0 = vec![Card::WildBolt; 6];
        let mut game = Game::with_deck_order(seed, [deck0, vec![bolt(1); 6]]);
        game.apply(P0, play(0)).unwrap();
        assert_eq!(game.mana(P0), 0, "seed {seed}: WildBolt costs 1");
        (10 - game.health(P0), 10 - game.health(P1))
    }

    #[test]
    fn wild_bolt_hits_exactly_one_hero_for_three() {
        for seed in 0..100 {
            let lost = cast_wild_bolt(seed);
            assert!(
                lost == (3, 0) || lost == (0, 3),
                "seed {seed}: health lost (caster, enemy) = {lost:?}"
            );
        }
    }

    #[test]
    fn wild_bolt_can_hit_either_hero() {
        let outcomes: Vec<(i32, i32)> = (0..100).map(cast_wild_bolt).collect();
        assert!(
            outcomes.contains(&(3, 0)),
            "never hit the caster in 100 seeds"
        );
        assert!(
            outcomes.contains(&(0, 3)),
            "never hit the enemy in 100 seeds"
        );
    }
}
