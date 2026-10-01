//! The state check at the end of every `apply` (node C). It removes dead minions
//! until a pass removes nothing, then decides the outcome.

#[cfg(test)]
mod tests {
    use crate::cards::{BLAST, CAPTAIN, RECRUIT};
    use crate::testkit::*;
    use crate::{Game, Outcome};

    #[test]
    fn blast_removes_every_minion_it_brings_to_zero_health() {
        let deck = deck_with_top(&[RECRUIT, BLAST]);
        let mut game = Game::with_deck_order(0, [deck.clone(), deck]);
        turn_with_mana(&mut game, P0, 2);
        let mine = summon(&mut game, P0, RECRUIT);
        turn_with_mana(&mut game, P1, 2);
        let theirs = summon(&mut game, P1, RECRUIT);
        turn_with_mana(&mut game, P0, 3);

        play_def(&mut game, P0, BLAST);

        assert!(game.board(P0).is_empty());
        assert!(game.board(P1).is_empty());
        assert_eq!(game.health(mine), None);
        assert_eq!(game.health(theirs), None);
        assert_eq!(game.outcome(), None);
    }

    #[test]
    fn the_check_repeats_until_a_pass_removes_nothing() {
        // Blast deals 2 to each. The Captain has 1 max health and dies in the first
        // pass. The Recruit has 3 with the buff and survives it, then dies in the
        // second pass once the buff is gone.
        let deck0 = deck_with_top(&[RECRUIT, CAPTAIN, BLAST]);
        let mut game = Game::with_deck_order(0, [deck0, deck_with_top(&[])]);
        turn_with_mana(&mut game, P0, 2);
        let recruit = summon(&mut game, P0, RECRUIT);
        turn_with_mana(&mut game, P0, 3);
        summon(&mut game, P0, CAPTAIN);
        turn_with_mana(&mut game, P0, 3);
        assert_eq!(game.health(recruit), Some(3));

        play_def(&mut game, P0, BLAST);

        assert!(
            game.board(P0).is_empty(),
            "left {:?}",
            board_defs(&game, P0)
        );
        assert_eq!(game.health(recruit), None);
    }

    #[test]
    fn a_blast_that_drops_both_heroes_to_zero_is_a_draw() {
        // Both decks are empty after player 0's first draw, so every later turn
        // start costs the active player 1 health. Both heroes are at 2 when
        // player 0's ninth turn starts.
        let mut game = Game::with_deck_order(0, [vec![BLAST; 4], vec![BLAST; 3]]);
        turn_with_mana(&mut game, P0, 9);
        assert_eq!(game.hero_health(P0), 2);
        assert_eq!(game.hero_health(P1), 2);

        play_def(&mut game, P0, BLAST);

        assert_eq!(game.outcome(), Some(Outcome::Draw));
        assert_actions(&game, P0, &[]);
        assert_actions(&game, P1, &[]);
    }

    #[test]
    fn a_blast_that_drops_only_its_casters_hero_to_zero_loses() {
        let mut game = Game::with_deck_order(0, [vec![BLAST; 4], deck_with_top(&[])]);
        turn_with_mana(&mut game, P0, 9);
        assert_eq!(game.hero_health(P0), 2);
        assert_eq!(game.hero_health(P1), 10);

        play_def(&mut game, P0, BLAST);

        assert_eq!(game.outcome(), Some(Outcome::Won(P1)));
        assert_actions(&game, P0, &[]);
        assert_actions(&game, P1, &[]);
    }
}
