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
    use crate::cards::{BLAST, BOLT, CAPTAIN, FORAGE, GIANT, RECRUIT, SPARK, WILD_BOLT};
    use crate::testkit::*;
    use crate::{Action, Game};

    #[test]
    fn playing_a_spark_pays_mana_and_hits_the_enemy() {
        let deck = vec![SPARK, BOLT, RECRUIT, CAPTAIN, BLAST];
        let mut game = Game::with_deck_order(0, [deck.clone(), deck]);

        play_def(&mut game, P0, SPARK);

        assert_eq!(game.hero_health(P1), 9);
        assert_eq!(game.hero_health(P0), 10);
        assert_eq!(game.mana(P0), 0);
        assert_eq!(hand_defs(&game, P0), [BOLT, RECRUIT, CAPTAIN]);
        assert_actions(&game, P0, &[Action::EndTurn]);
    }

    #[test]
    fn damage_spells_hit_the_casters_enemy() {
        for (spell, damage) in [(SPARK, 1), (BOLT, 2)] {
            for (caster, enemy) in [(P0, P1), (P1, P0)] {
                let deck = deck_with_top(&[spell]);
                let mut game = Game::with_deck_order(0, [deck.clone(), deck]);
                turn_with_mana(&mut game, caster, 2);

                play_def(&mut game, caster, spell);

                assert_eq!(
                    game.hero_health(enemy),
                    10 - damage,
                    "{spell:?} by {caster:?}"
                );
                assert_eq!(game.hero_health(caster), 10, "{spell:?} by {caster:?}");
            }
        }
    }

    #[test]
    fn every_card_pays_its_printed_cost() {
        // Written out from SPEC.md, so a wrong cost in the core can't check itself.
        let cases = [
            (SPARK, 1),
            (BOLT, 2),
            (WILD_BOLT, 1),
            (FORAGE, 1),
            (BLAST, 3),
            (RECRUIT, 2),
            (CAPTAIN, 3),
            (GIANT, 8),
        ];
        for (def, cost) in cases {
            let mut game = Game::with_deck_order(0, [deck_with_top(&[def]), deck_with_top(&[])]);
            turn_with_mana(&mut game, P0, cost);
            let card = in_hand(&game, P0, def);
            assert_eq!(game.cost(card), Some(cost), "{def:?}");

            let before = game.mana(P0);
            game.apply(P0, play(card)).unwrap();
            assert_eq!(game.mana(P0), before - cost, "{def:?}");
        }
    }

    #[test]
    fn minions_enter_at_the_right_end_of_their_owners_board() {
        let deck0 = deck_with_top(&[RECRUIT, CAPTAIN, RECRUIT]);
        let mut game = Game::with_deck_order(0, [deck0, deck_with_top(&[])]);

        turn_with_mana(&mut game, P0, 2);
        let hand_size = game.hand(P0).len();
        play_def(&mut game, P0, RECRUIT);
        assert_eq!(board_defs(&game, P0), [RECRUIT]);
        assert_eq!(game.hand(P0).len(), hand_size - 1);
        assert_eq!(game.mana(P0), 0);

        turn_with_mana(&mut game, P0, 3);
        play_def(&mut game, P0, CAPTAIN);
        turn_with_mana(&mut game, P0, 2);
        play_def(&mut game, P0, RECRUIT);

        assert_eq!(board_defs(&game, P0), [RECRUIT, CAPTAIN, RECRUIT]);
        assert!(game.board(P1).is_empty());
    }

    #[test]
    fn blast_deals_two_damage_to_every_character() {
        let deck0 = deck_with_top(&[GIANT, BLAST]);
        let mut game = Game::with_deck_order(0, [deck0, deck_with_top(&[GIANT])]);
        turn_with_mana(&mut game, P0, 8);
        let mine = summon(&mut game, P0, GIANT);
        turn_with_mana(&mut game, P1, 8);
        let theirs = summon(&mut game, P1, GIANT);
        turn_with_mana(&mut game, P0, 3);

        play_def(&mut game, P0, BLAST);

        assert_eq!(game.hero_health(P0), 8);
        assert_eq!(game.hero_health(P1), 8);
        assert_eq!(game.health(mine), Some(3));
        assert_eq!(game.health(theirs), Some(3));
    }

    #[test]
    fn forage_offers_only_picks_until_one_is_made() {
        let deck0 = vec![FORAGE, SPARK, SPARK, SPARK, RECRUIT, CAPTAIN, BOLT];
        let mut game = Game::with_deck_order(0, [deck0, vec![SPARK; 6]]);

        play_def(&mut game, P0, FORAGE);

        assert_eq!(revealed_defs(&game, P0), [RECRUIT, CAPTAIN]);
        assert_eq!(deck_defs(&game, P0), [BOLT]);
        let revealed = game.revealed(P0);
        assert_actions(&game, P0, &[pick(revealed[0]), pick(revealed[1])]);
        assert_actions(&game, P1, &[]);

        game.apply(P0, pick(revealed[1])).unwrap();

        assert!(game.revealed(P0).is_empty());
        assert_eq!(hand_defs(&game, P0), [SPARK, SPARK, SPARK, CAPTAIN]);
        assert_eq!(
            deck_defs(&game, P0),
            [BOLT, RECRUIT],
            "the unpicked card goes to the bottom"
        );
        assert_actions(&game, P0, &[Action::EndTurn]);
    }

    #[test]
    fn forage_with_one_card_left_still_asks_for_the_pick() {
        let deck0 = vec![FORAGE, SPARK, SPARK, SPARK, RECRUIT];
        let mut game = Game::with_deck_order(0, [deck0, vec![SPARK; 6]]);

        play_def(&mut game, P0, FORAGE);
        assert_eq!(revealed_defs(&game, P0), [RECRUIT]);
        let revealed = game.revealed(P0);
        assert_actions(&game, P0, &[pick(revealed[0])]);

        game.apply(P0, pick(revealed[0])).unwrap();
        assert_eq!(hand_defs(&game, P0), [SPARK, SPARK, SPARK, RECRUIT]);
        assert!(game.deck(P0).is_empty());
    }

    #[test]
    fn forage_on_an_empty_deck_does_nothing() {
        let deck0 = vec![FORAGE, SPARK, SPARK, SPARK];
        let mut game = Game::with_deck_order(0, [deck0, vec![SPARK; 6]]);

        play_def(&mut game, P0, FORAGE);
        assert!(game.revealed(P0).is_empty());
        assert_eq!(hand_defs(&game, P0), [SPARK, SPARK, SPARK]);
        assert_actions(&game, P0, &[Action::EndTurn]);
    }

    /// WildBolt's target is random, so its tests assert what holds for every seed.
    /// A test that pins one seed to an outcome breaks as soon as unrelated code
    /// takes an earlier number from the RNG.
    fn cast_wild_bolt(seed: u64) -> (i32, i32) {
        let mut game = Game::with_deck_order(seed, [vec![WILD_BOLT; 6], vec![SPARK; 6]]);
        play_def(&mut game, P0, WILD_BOLT);
        assert_eq!(game.mana(P0), 0, "seed {seed}: WildBolt costs 1");
        (10 - game.hero_health(P0), 10 - game.hero_health(P1))
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
