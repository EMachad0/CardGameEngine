//! What `apply` reports to its observer.

use rules::static_card_definition::{
    BARRACKS, BLAST, BOLT, CAPTAIN, FORAGE, RECRUIT, SPARK, WILD_BOLT, ZAP,
};
use rules::{Action, Event, Game, ObjectId, Outcome, PlayerId, View};

use crate::support::*;

/// The events of each step, leaving out the steps with none.
fn event_steps(recorder: &Recorder) -> Vec<Vec<Event>> {
    recorder
        .steps
        .iter()
        .map(|step| step.events.clone())
        .filter(|events| !events.is_empty())
        .collect()
}

#[test]
fn ending_the_turn_reports_the_end_then_the_next_turn_start_and_its_draw() {
    let deck = vec![SPARK; 6];
    let mut game = Game::with_deck_order(0, [deck.clone(), deck]);
    let [p0, p1] = players(&game);
    let top = game.deck(p1)[0];

    let recorder = observe(&mut game, p0, Action::EndTurn);

    assert_eq!(
        recorder.events(),
        [
            Event::TurnEnded { player_id: p0 },
            Event::TurnStarted { player_id: p1 },
            Event::Drew {
                player_id: p1,
                object_id: top
            },
        ]
    );
}

#[test]
fn a_turn_change_and_its_draw_share_one_step() {
    let deck = vec![SPARK; 6];
    let mut game = Game::with_deck_order(0, [deck.clone(), deck]);
    let [p0, p1] = players(&game);
    let top = game.deck(p1)[0];

    let recorder = observe(&mut game, p0, Action::EndTurn);

    assert_eq!(
        event_steps(&recorder),
        [vec![
            Event::TurnEnded { player_id: p0 },
            Event::TurnStarted { player_id: p1 },
            Event::Drew {
                player_id: p1,
                object_id: top
            },
        ]]
    );
}

#[test]
fn a_draw_from_an_empty_deck_reports_fatigue_damage_instead() {
    let mut game = Game::with_deck_order(0, [vec![SPARK; 6], vec![SPARK; 3]]);
    let [p0, p1] = players(&game);
    assert!(
        game.deck(p1).is_empty(),
        "setup dealt player 1's whole deck"
    );

    let recorder = observe(&mut game, p0, Action::EndTurn);

    assert_eq!(
        recorder.events(),
        [
            Event::TurnEnded { player_id: p0 },
            Event::TurnStarted { player_id: p1 },
            Event::FatigueDamaged {
                amount: 1,
                player_id: p1,
                object_id: game.hero_id(p1),
            },
        ]
    );
}

#[test]
fn spark_reports_its_play_then_its_hit_on_the_enemy_hero_id() {
    let mut game = Game::with_deck_order(0, [vec![SPARK; 6], vec![SPARK; 6]]);
    let [p0, p1] = players(&game);
    let spark = in_hand(&game, p0, SPARK);

    let recorder = observe(&mut game, p0, play(spark));

    assert_eq!(
        recorder.events(),
        [
            Event::Played {
                player_id: p0,
                object_id: spark
            },
            Event::Damaged {
                target: game.hero_id(p1),
                amount: 1,
                source: spark
            },
        ]
    );
}

#[test]
fn zap_reports_its_hit_on_the_enemy_hero_then_its_draw() {
    let mut game = Game::with_deck_order(0, [deck_with_top(&[ZAP]), deck_with_top(&[])]);
    let [p0, p1] = players(&game);
    turn_with_mana(&mut game, p0, 2);
    let zap = in_hand(&game, p0, ZAP);
    let top = game.deck(p0)[0];

    let events = observe(&mut game, p0, play(zap)).events();

    assert_eq!(
        events,
        [
            Event::Played {
                player_id: p0,
                object_id: zap
            },
            Event::Damaged {
                target: game.hero_id(p1),
                amount: 1,
                source: zap
            },
            Event::Drew {
                player_id: p0,
                object_id: top
            },
        ]
    );
}

#[test]
fn zaps_hit_and_draw_are_separate_steps() {
    let mut game = Game::with_deck_order(0, [deck_with_top(&[ZAP]), deck_with_top(&[])]);
    let [p0, p1] = players(&game);
    turn_with_mana(&mut game, p0, 2);
    let zap = in_hand(&game, p0, ZAP);
    let top = game.deck(p0)[0];

    let recorder = observe(&mut game, p0, play(zap));

    assert_eq!(
        event_steps(&recorder),
        [
            vec![Event::Played {
                player_id: p0,
                object_id: zap
            }],
            vec![Event::Damaged {
                target: game.hero_id(p1),
                amount: 1,
                source: zap
            }],
            vec![Event::Drew {
                player_id: p0,
                object_id: top
            }],
        ],
        "two sentences of card text, so a checkpoint between them"
    );
}

#[test]
fn wild_bolt_reports_its_hit_on_the_hero_that_lost_health() {
    for seed in 0..50 {
        let mut game = Game::with_deck_order(seed, [vec![WILD_BOLT; 6], vec![SPARK; 6]]);
        let [p0, _] = players(&game);
        let wild_bolt = in_hand(&game, p0, WILD_BOLT);

        let events = observe(&mut game, p0, play(wild_bolt)).events();

        let hit = players(&game)
            .into_iter()
            .find(|&p| game.hero_health(p) == 7)
            .unwrap_or_else(|| panic!("seed {seed}: no hero took 3"));
        assert_eq!(
            events,
            [
                Event::Played {
                    player_id: p0,
                    object_id: wild_bolt
                },
                Event::Damaged {
                    target: game.hero_id(hit),
                    amount: 3,
                    source: wild_bolt
                },
            ],
            "seed {seed}"
        );
    }
}

#[test]
fn a_checkpoint_follows_a_card_leaving_the_hand() {
    for (def, mana) in [(SPARK, 1), (RECRUIT, 2)] {
        let mut game = Game::with_deck_order(0, [deck_with_top(&[def]), deck_with_top(&[])]);
        let [p0, _] = players(&game);
        turn_with_mana(&mut game, p0, mana);
        let card = in_hand(&game, p0, def);

        let recorder = observe(&mut game, p0, play(card));

        assert_eq!(
            event_steps(&recorder).first(),
            Some(&vec![Event::Played {
                player_id: p0,
                object_id: card
            }]),
            "{def:?}"
        );
    }
}

#[test]
fn a_minion_is_reported_entering_the_board_after_its_play() {
    let mut game = Game::with_deck_order(0, [deck_with_top(&[RECRUIT]), deck_with_top(&[])]);
    let [p0, _] = players(&game);
    turn_with_mana(&mut game, p0, 2);
    let recruit = in_hand(&game, p0, RECRUIT);

    let recorder = observe(&mut game, p0, play(recruit));

    assert_eq!(
        recorder.events(),
        [
            Event::Played {
                player_id: p0,
                object_id: recruit
            },
            Event::BoardEntered {
                player_id: p0,
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
    let [p0, _] = players(&game);
    turn_with_mana(&mut game, p0, 2);
    let recruit = summon(&mut game, p0, RECRUIT);
    turn_with_mana(&mut game, p0, 3);
    let captain = summon(&mut game, p0, CAPTAIN);
    turn_with_mana(&mut game, p0, 3);
    (game, recruit, captain)
}

#[test]
fn blast_reports_its_play_then_a_hit_on_every_character() {
    let (mut game, recruit, captain) = recruit_and_captain_facing_blast();
    let [p0, p1] = players(&game);
    let blast = in_hand(&game, p0, BLAST);

    let events = observe(&mut game, p0, play(blast)).events();

    assert_eq!(
        events.first(),
        Some(&Event::Played {
            player_id: p0,
            object_id: blast
        })
    );
    let hits: Vec<&Event> = events
        .iter()
        .filter(|e| matches!(e, Event::Damaged { .. }))
        .collect();
    let targets = [recruit, captain, game.hero_id(p0), game.hero_id(p1)];
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
fn blast_hits_the_heroes_before_any_minion() {
    let (mut game, _, _) = recruit_and_captain_facing_blast();
    let [p0, p1] = players(&game);
    let blast = in_hand(&game, p0, BLAST);

    let events = observe(&mut game, p0, play(blast)).events();

    let targets: Vec<ObjectId> = events
        .iter()
        .filter_map(|e| match e {
            Event::Damaged { target, .. } => Some(*target),
            _ => None,
        })
        .collect();
    let heroes = [game.hero_id(p0), game.hero_id(p1)];
    assert_eq!(targets.len(), 4, "{events:?}");
    assert!(
        targets[..2].iter().all(|t| heroes.contains(t)),
        "heroes come first: {targets:?}"
    );
}

#[test]
fn blasts_hits_land_together_in_one_step() {
    let (mut game, _, _) = recruit_and_captain_facing_blast();
    let [p0, _] = players(&game);
    let blast = in_hand(&game, p0, BLAST);

    let recorder = observe(&mut game, p0, play(blast));

    let hits_per_step: Vec<usize> = recorder
        .steps
        .iter()
        .map(|step| {
            step.events
                .iter()
                .filter(|e| matches!(e, Event::Damaged { .. }))
                .count()
        })
        .filter(|&hits| hits > 0)
        .collect();
    assert_eq!(
        hits_per_step,
        [4],
        "one sentence of card text, so no checkpoint between its hits"
    );
}

#[test]
fn blast_reports_every_hit_before_any_death() {
    let (mut game, _, _) = recruit_and_captain_facing_blast();
    let [p0, _] = players(&game);
    let blast = in_hand(&game, p0, BLAST);

    let events = observe(&mut game, p0, play(blast)).events();

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
    let [p0, _] = players(&game);
    let blast = in_hand(&game, p0, BLAST);

    let events = observe(&mut game, p0, play(blast)).events();

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

#[test]
fn barracks_reports_its_play_then_the_squire_entering_the_board() {
    let mut game = Game::with_deck_order(0, [deck_with_top(&[BARRACKS]), deck_with_top(&[])]);
    let [p0, _] = players(&game);
    turn_with_mana(&mut game, p0, 2);
    let barracks = in_hand(&game, p0, BARRACKS);

    let events = observe(&mut game, p0, play(barracks)).events();

    let &[squire] = game.board(p0) else {
        panic!("Barracks left {:?}", board_defs(&game, p0));
    };
    assert_eq!(
        events,
        [
            Event::Played {
                player_id: p0,
                object_id: barracks
            },
            Event::BoardEntered {
                player_id: p0,
                object_id: squire
            },
        ],
        "a summoned Squire enters the board without being played"
    );
}

/// Player 0's deck runs out at its first draw, so each later turn start costs its hero 1.
/// Player 0 summons a Recruit on its second turn and waits until its ninth, when its hero is
/// at 2 and it can cast the Blast in hand. Returns the game and the Recruit.
fn recruit_and_low_hero_facing_blast() -> (Game, ObjectId) {
    let deck0 = vec![RECRUIT, BLAST, BLAST, BLAST];
    let mut game = Game::with_deck_order(0, [deck0, deck_with_top(&[])]);
    let [p0, _] = players(&game);
    turn_with_mana(&mut game, p0, 2);
    let recruit = summon(&mut game, p0, RECRUIT);
    turn_with_mana(&mut game, p0, 9);
    assert_eq!(game.hero_health(p0), 2, "fatigue left player 0 at 2");
    (game, recruit)
}

#[test]
fn a_hero_brought_to_zero_is_reported_dead_once_before_the_game_ends() {
    let (mut game, _) = recruit_and_low_hero_facing_blast();
    let [p0, p1] = players(&game);
    let hero = game.hero_id(p0);
    let blast = in_hand(&game, p0, BLAST);

    let events = observe(&mut game, p0, play(blast)).events();

    let hero_deaths: Vec<usize> = events
        .iter()
        .enumerate()
        .filter(|(_, e)| **e == Event::Died { object_id: hero })
        .map(|(i, _)| i)
        .collect();
    assert_eq!(hero_deaths.len(), 1, "one Died for the hero: {events:?}");
    assert_eq!(
        &events[hero_deaths[0] + 1..],
        [Event::GameEnded {
            outcome: Outcome::Won(p1)
        }],
        "the game ends right after the hero's death"
    );
}

#[test]
fn a_minion_dying_with_a_hero_is_reported_dead_first() {
    let (mut game, recruit) = recruit_and_low_hero_facing_blast();
    let [p0, _] = players(&game);
    let hero = game.hero_id(p0);
    let blast = in_hand(&game, p0, BLAST);

    let events = observe(&mut game, p0, play(blast)).events();

    let deaths: Vec<Event> = events
        .into_iter()
        .filter(|e| matches!(e, Event::Died { .. }))
        .collect();
    assert_eq!(
        deaths,
        [
            Event::Died { object_id: recruit },
            Event::Died { object_id: hero },
        ],
        "dead minions leave the board before the outcome is decided"
    );
}

#[test]
fn heroes_brought_to_zero_together_are_each_reported_dead_before_the_draw() {
    let mut game = Game::with_deck_order(0, [vec![BLAST; 4], vec![BLAST; 3]]);
    let [p0, p1] = players(&game);
    turn_with_mana(&mut game, p0, 9);
    let heroes = [game.hero_id(p0), game.hero_id(p1)];
    let blast = in_hand(&game, p0, BLAST);

    let events = observe(&mut game, p0, play(blast)).events();

    let Some((Event::GameEnded { outcome }, before)) = events.split_last() else {
        panic!("the last event is not GameEnded: {events:?}");
    };
    assert_eq!(*outcome, Outcome::Draw);
    let deaths: Vec<&Event> = before
        .iter()
        .filter(|e| matches!(e, Event::Died { .. }))
        .collect();
    assert_eq!(deaths.len(), 2, "one Died per hero: {events:?}");
    for object_id in heroes {
        assert!(
            deaths.contains(&&Event::Died { object_id }),
            "no Died for {object_id:?}: {events:?}"
        );
    }
}

/// The health `view` shows for minion `id` on `p`'s board, or `None` if it isn't there.
fn board_health(view: &View, p: PlayerId, id: ObjectId) -> Option<i32> {
    player_view(view, p)
        .board
        .iter()
        .find(|card| card.object_id == id)
        .map(|card| card.health)
}

#[test]
fn checkpoints_show_the_recruit_hit_then_unbuffed_before_it_leaves() {
    let (mut game, recruit, _) = recruit_and_captain_facing_blast();
    let [p0, _] = players(&game);
    let blast = in_hand(&game, p0, BLAST);

    let recorder = observe(&mut game, p0, play(blast));

    let mut seen: Vec<Option<i32>> = recorder
        .steps
        .iter()
        .map(|step| board_health(step.view(p0), p0, recruit))
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
    let [p0, _] = players(&game);
    let forage = in_hand(&game, p0, FORAGE);
    let top_two = game.deck(p0)[..2].to_vec();

    let recorder = observe(&mut game, p0, play(forage));

    assert_eq!(
        recorder.events(),
        [
            Event::Played {
                player_id: p0,
                object_id: forage
            },
            Event::Revealed {
                player_id: p0,
                object_ids: top_two
            },
        ]
    );
}

#[test]
fn a_forage_that_reveals_nothing_reports_only_its_play() {
    let deck0 = vec![FORAGE, SPARK, SPARK, SPARK];
    let mut game = Game::with_deck_order(0, [deck0, vec![SPARK; 6]]);
    let [p0, _] = players(&game);
    let forage = in_hand(&game, p0, FORAGE);

    let recorder = observe(&mut game, p0, play(forage));

    assert_eq!(
        recorder.events(),
        [Event::Played {
            player_id: p0,
            object_id: forage
        }]
    );
}

#[test]
fn a_pick_reports_the_picked_card_then_buries_the_rest() {
    let mut game = forage_over_recruit_and_captain();
    let [p0, _] = players(&game);
    play_def(&mut game, p0, FORAGE);
    let &[recruit, captain] = game.revealed(p0) else {
        panic!("Forage revealed {:?}", game.revealed(p0));
    };

    let recorder = observe(&mut game, p0, pick(captain));

    assert_eq!(
        recorder.events(),
        [
            Event::Picked {
                player_id: p0,
                object_id: captain
            },
            Event::Buried {
                player_id: p0,
                object_id: recruit
            },
        ]
    );
}

#[test]
fn a_pick_and_its_burials_share_one_step() {
    let mut game = forage_over_recruit_and_captain();
    let [p0, _] = players(&game);
    play_def(&mut game, p0, FORAGE);
    let &[recruit, captain] = game.revealed(p0) else {
        panic!("Forage revealed {:?}", game.revealed(p0));
    };

    let recorder = observe(&mut game, p0, pick(captain));

    assert_eq!(
        event_steps(&recorder),
        [vec![
            Event::Picked {
                player_id: p0,
                object_id: captain
            },
            Event::Buried {
                player_id: p0,
                object_id: recruit
            },
        ]]
    );
}

#[test]
fn only_the_apply_that_ends_the_game_reports_its_outcome() {
    // Neither player plays anything, so fatigue ends the game.
    let mut game = Game::with_deck_order(0, [vec![SPARK; 3], vec![SPARK; 3]]);
    let mut reports = Vec::new();
    while game.outcome().is_none() {
        assert!(reports.len() < 60, "no outcome after 60 turns");
        let p = players(&game)
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
    let [_, p1] = players(&game);
    let mut recorder = Recorder::new(&game);

    let result = game.apply(p1, Action::EndTurn, &mut recorder);

    assert!(result.is_err(), "it is player 0's turn");
    assert_eq!(recorder, Recorder::new(&game), "no event and no checkpoint");
}

#[test]
fn an_apply_ends_with_a_checkpoint_showing_the_game_it_leaves() {
    let mut game = Game::with_deck_order(0, [vec![SPARK; 6], vec![SPARK; 6]]);
    let [p0, _] = players(&game);
    let spark = in_hand(&game, p0, SPARK);

    let recorder = observe(&mut game, p0, play(spark));

    assert!(
        recorder.trailing.is_empty(),
        "events after the last checkpoint: {:?}",
        recorder.trailing
    );
    let last = recorder.steps.last().expect("at least one checkpoint");
    assert_eq!(last.views, players(&game).map(|p| game.view(p)));
}
