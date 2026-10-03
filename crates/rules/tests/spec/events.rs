//! What `apply` reports to its observer (node D).

use rules::cards::{BLAST, BOLT, CAPTAIN, FORAGE, RECRUIT, SPARK, WILD_BOLT};
use rules::{Action, Event, Game, ObjectId, PlayerId, Target, View};

use crate::support::*;

#[test]
fn ending_the_turn_reports_the_end_then_the_next_turn_start_and_its_draw() {
    let deck = vec![SPARK; 6];
    let mut game = Game::with_deck_order(0, [deck.clone(), deck]);
    let top = game.deck(P1)[0];

    let recorder = observe(&mut game, P0, Action::EndTurn);

    assert_eq!(
        recorder.events(),
        [
            Event::TurnEnded { player_id: P0 },
            Event::TurnStarted { player_id: P1 },
            Event::Drew {
                player_id: P1,
                object_id: top
            },
        ]
    );
}

#[test]
fn a_draw_from_an_empty_deck_reports_fatigue_damage_instead() {
    let mut game = Game::with_deck_order(0, [vec![SPARK; 6], vec![SPARK; 3]]);
    assert!(
        game.deck(P1).is_empty(),
        "setup dealt player 1's whole deck"
    );

    let recorder = observe(&mut game, P0, Action::EndTurn);

    assert_eq!(
        recorder.events(),
        [
            Event::TurnEnded { player_id: P0 },
            Event::TurnStarted { player_id: P1 },
            Event::FatigueDamaged {
                amount: 1,
                player_id: P1
            },
        ]
    );
}

#[test]
fn spark_reports_its_play_then_its_hit_on_the_enemy_hero() {
    let mut game = Game::with_deck_order(0, [vec![SPARK; 6], vec![SPARK; 6]]);
    let spark = in_hand(&game, P0, SPARK);

    let recorder = observe(&mut game, P0, play(spark));

    assert_eq!(
        recorder.events(),
        [
            Event::Played {
                player_id: P0,
                object_id: spark
            },
            Event::Damaged {
                target: Target::Hero(P1),
                amount: 1,
                source: spark
            },
        ]
    );
}

#[test]
fn wild_bolt_reports_its_hit_on_the_hero_that_lost_health() {
    for seed in 0..50 {
        let mut game = Game::with_deck_order(seed, [vec![WILD_BOLT; 6], vec![SPARK; 6]]);
        let wild_bolt = in_hand(&game, P0, WILD_BOLT);

        let events = observe(&mut game, P0, play(wild_bolt)).events();

        let hit = PLAYERS
            .into_iter()
            .find(|&p| game.hero_health(p) == 7)
            .unwrap_or_else(|| panic!("seed {seed}: no hero took 3"));
        assert_eq!(
            events,
            [
                Event::Played {
                    player_id: P0,
                    object_id: wild_bolt
                },
                Event::Damaged {
                    target: Target::Hero(hit),
                    amount: 3,
                    source: wild_bolt
                },
            ],
            "seed {seed}"
        );
    }
}

#[test]
fn a_minion_is_reported_entering_the_board_after_its_play() {
    let mut game = Game::with_deck_order(0, [deck_with_top(&[RECRUIT]), deck_with_top(&[])]);
    turn_with_mana(&mut game, P0, 2);
    let recruit = in_hand(&game, P0, RECRUIT);

    let recorder = observe(&mut game, P0, play(recruit));

    assert_eq!(
        recorder.events(),
        [
            Event::Played {
                player_id: P0,
                object_id: recruit
            },
            Event::BoardEntered {
                player_id: P0,
                object_id: recruit
            },
        ]
    );
}

/// Player 0 has a Recruit and a Captain on the board, in that order, and can cast the Blast
/// in hand. Returns the game, the Recruit and the Captain.
fn recruit_and_captain_facing_blast() -> (Game, ObjectId, ObjectId) {
    let deck0 = deck_with_top(&[RECRUIT, CAPTAIN, BLAST]);
    let mut game = Game::with_deck_order(0, [deck0, deck_with_top(&[])]);
    turn_with_mana(&mut game, P0, 2);
    let recruit = summon(&mut game, P0, RECRUIT);
    turn_with_mana(&mut game, P0, 3);
    let captain = summon(&mut game, P0, CAPTAIN);
    turn_with_mana(&mut game, P0, 3);
    (game, recruit, captain)
}

#[test]
fn blast_reports_its_play_then_a_hit_on_every_character() {
    let (mut game, recruit, captain) = recruit_and_captain_facing_blast();
    let blast = in_hand(&game, P0, BLAST);

    let events = observe(&mut game, P0, play(blast)).events();

    assert_eq!(
        events.first(),
        Some(&Event::Played {
            player_id: P0,
            object_id: blast
        })
    );
    let hits: Vec<&Event> = events
        .iter()
        .filter(|e| matches!(e, Event::Damaged { .. }))
        .collect();
    let targets = [
        Target::Monster(recruit),
        Target::Monster(captain),
        Target::Hero(P0),
        Target::Hero(P1),
    ];
    assert_eq!(
        hits.len(),
        targets.len(),
        "one hit per character: {events:?}"
    );
    for target in targets {
        let hit = Event::Damaged {
            target,
            amount: 2,
            source: blast,
        };
        assert!(hits.contains(&&hit), "missing {hit:?} in {events:?}");
    }
}

#[test]
fn blast_reports_every_hit_before_any_death() {
    let (mut game, _, _) = recruit_and_captain_facing_blast();
    let blast = in_hand(&game, P0, BLAST);

    let events = observe(&mut game, P0, play(blast)).events();

    let last_hit = events
        .iter()
        .rposition(|e| matches!(e, Event::Damaged { .. }))
        .expect("Blast hit something");
    let first_death = events
        .iter()
        .position(|e| matches!(e, Event::Died { .. }))
        .expect("Blast killed something");
    assert!(
        last_hit < first_death,
        "deaths come from the state check, after the spell: {events:?}"
    );
}

#[test]
fn the_captain_is_reported_dead_before_the_recruit_it_held_up() {
    let (mut game, recruit, captain) = recruit_and_captain_facing_blast();
    let blast = in_hand(&game, P0, BLAST);

    let events = observe(&mut game, P0, play(blast)).events();

    let deaths: Vec<Event> = events
        .into_iter()
        .filter(|e| matches!(e, Event::Died { .. }))
        .collect();
    assert_eq!(
        deaths,
        [
            Event::Died { object_id: captain },
            Event::Died { object_id: recruit },
        ],
        "the Recruit survives the first pass and dies in the second, once the buff is gone"
    );
}

/// The health `view` shows for minion `id` on `p`'s board, or `None` if it isn't there.
fn board_health(view: &View, p: PlayerId, id: ObjectId) -> Option<i32> {
    view.players[p.idx()]
        .board
        .iter()
        .find(|card| card.object_id == id)
        .map(|card| card.health)
}

#[test]
fn checkpoints_show_the_recruit_hit_then_unbuffed_before_it_leaves() {
    let (mut game, recruit, _) = recruit_and_captain_facing_blast();
    let blast = in_hand(&game, P0, BLAST);

    let recorder = observe(&mut game, P0, play(blast));

    let mut seen: Vec<Option<i32>> = recorder
        .steps
        .iter()
        .map(|step| board_health(&step.views[P0.idx()], P0, recruit))
        .collect();
    seen.dedup();
    assert_eq!(
        seen,
        [Some(3), Some(1), Some(0), None],
        "the Recruit's health across checkpoints: before the hit, after it, \
         after the Captain's death pass, then removed"
    );
}

/// Player 0 holds a Forage, with a Recruit and a Captain on top of the deck and a Bolt under them.
fn forage_over_recruit_and_captain() -> Game {
    let deck0 = vec![FORAGE, SPARK, SPARK, SPARK, RECRUIT, CAPTAIN, BOLT];
    Game::with_deck_order(0, [deck0, vec![SPARK; 6]])
}

#[test]
fn forage_reports_the_cards_it_revealed_in_order() {
    let mut game = forage_over_recruit_and_captain();
    let forage = in_hand(&game, P0, FORAGE);
    let top_two = game.deck(P0)[..2].to_vec();

    let recorder = observe(&mut game, P0, play(forage));

    assert_eq!(
        recorder.events(),
        [
            Event::Played {
                player_id: P0,
                object_id: forage
            },
            Event::Revealed {
                player_id: P0,
                object_ids: top_two
            },
        ]
    );
}

#[test]
fn a_forage_that_reveals_nothing_reports_only_its_play() {
    let deck0 = vec![FORAGE, SPARK, SPARK, SPARK];
    let mut game = Game::with_deck_order(0, [deck0, vec![SPARK; 6]]);
    let forage = in_hand(&game, P0, FORAGE);

    let recorder = observe(&mut game, P0, play(forage));

    assert_eq!(
        recorder.events(),
        [Event::Played {
            player_id: P0,
            object_id: forage
        }]
    );
}

#[test]
fn a_pick_reports_the_picked_card_then_buries_the_rest() {
    let mut game = forage_over_recruit_and_captain();
    play_def(&mut game, P0, FORAGE);
    let &[recruit, captain] = game.revealed(P0) else {
        panic!("Forage revealed {:?}", game.revealed(P0));
    };

    let recorder = observe(&mut game, P0, pick(captain));

    assert_eq!(
        recorder.events(),
        [
            Event::Picked {
                player_id: P0,
                object_id: captain
            },
            Event::Buried {
                player_id: P0,
                object_id: recruit
            },
        ]
    );
}

#[test]
fn only_the_apply_that_ends_the_game_reports_its_outcome() {
    // Neither player plays anything, so fatigue ends the game.
    let mut game = Game::with_deck_order(0, [vec![SPARK; 3], vec![SPARK; 3]]);
    let mut reports = Vec::new();
    while game.outcome().is_none() {
        assert!(reports.len() < 60, "no outcome after 60 turns");
        let p = PLAYERS
            .into_iter()
            .find(|&p| game.legal_actions(p).contains(&Action::EndTurn))
            .expect("someone can end their turn");
        reports.push(observe(&mut game, p, Action::EndTurn).events());
    }

    let outcome = game.outcome().unwrap();
    let ends: Vec<(usize, &Event)> = reports
        .iter()
        .enumerate()
        .flat_map(|(i, events)| events.iter().map(move |e| (i, e)))
        .filter(|(_, e)| matches!(e, Event::GameEnded { .. }))
        .collect();
    assert_eq!(
        ends,
        [(reports.len() - 1, &Event::GameEnded { outcome })],
        "one GameEnded, in the last apply"
    );
}

#[test]
fn a_rejected_action_reports_nothing() {
    let mut game = Game::with_deck_order(0, [vec![SPARK; 6], vec![SPARK; 6]]);
    let mut recorder = Recorder::default();

    let result = game.apply(P1, Action::EndTurn, &mut recorder);

    assert!(result.is_err(), "it is player 0's turn");
    assert_eq!(recorder, Recorder::default(), "no event and no checkpoint");
}

#[test]
fn an_apply_ends_with_a_checkpoint_showing_the_game_it_leaves() {
    let mut game = Game::with_deck_order(0, [vec![SPARK; 6], vec![SPARK; 6]]);
    let spark = in_hand(&game, P0, SPARK);

    let recorder = observe(&mut game, P0, play(spark));

    assert!(
        recorder.trailing.is_empty(),
        "events after the last checkpoint: {:?}",
        recorder.trailing
    );
    let last = recorder.steps.last().expect("at least one checkpoint");
    assert_eq!(last.views, PLAYERS.map(|p| game.view(p)));
}
