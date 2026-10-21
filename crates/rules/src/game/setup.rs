//! A new game, up to the start of its first turn.

use crate::cards::binder::sets::base::static_card_definition::HERO;
use crate::cards::binder::store::Binder;
use crate::cards::definition::DefId;
use crate::cards::object::ObjectBag;
use crate::game::player::Player;
use crate::game::resolve::aftermath::standings::Standings;
use crate::game::state::Game;
use crate::history::History;
use crate::ids::PlayerId;
use crate::rng::Rng;
use crate::turn::TurnOrder;
use crate::zones::Zones;

pub(super) fn setup(seed: u64, decks: [Vec<DefId>; 2], shuffle: bool) -> Game {
    let rng = Rng::new(seed);

    // Fails only on invalid built-in definitions.
    let binder = Binder::built_in().unwrap();

    let mut objects = ObjectBag::default();

    let player_count = decks.len();
    let player_ids = (0..player_count).map(PlayerId::new).collect::<Vec<_>>();
    let players = player_ids
        .iter()
        .map(|&player_id| Player::new(player_id))
        .collect::<Vec<_>>();
    let heroes = player_ids
        .iter()
        .map(|&player_id| objects.insert(HERO, player_id))
        .collect::<Vec<_>>();
    let decks = player_ids
        .iter()
        .zip(decks)
        .map(|(&player_id, deck_defs)| {
            deck_defs
                .into_iter()
                .map(|def_id| objects.insert(def_id, player_id))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let zones = Zones::new(heroes.into_iter().zip(decks));
    let turn_order = TurnOrder::new(player_ids.clone());
    let history = History::default();

    let mut game = Game {
        rng,
        turn_order,
        players,
        zones,
        player_input: Default::default(),
        objects,
        binder,
        history,
        #[expect(
            clippy::disallowed_methods,
            reason = "the one start of the standings, as the game is set up"
        )]
        standings: Standings::new(player_ids.clone()),
        effect_stack: Default::default(),
    };

    if shuffle {
        for &player_id in &player_ids {
            game.zones.shuffle_deck(player_id, &mut game.rng);
        }
    }

    for player_id in player_ids {
        game.draw(player_id, 3, &mut ());
    }

    game.start_turn(&mut ());
    game
}
