//! A new game, up to the start of its first turn.

use crate::cards::binder::def_id::DefId;
use crate::cards::binder::sets::base::static_card_definition::HERO;
use crate::cards::binder::store::Binder;
use crate::cards::object::ObjectBag;
use crate::game::player::{Player, Players};
use crate::game::resolve::aftermath::standings::Standings;
use crate::game::state::Game;
use crate::history::History;
use crate::rng::Rng;
use crate::turn::TurnOrder;
use crate::zones::Zones;

pub(super) fn setup(seed: u64, decks: [Vec<DefId>; 2], shuffle: bool) -> Game {
    let rng = Rng::new(seed);

    let binder = Binder::built_in();

    let mut objects = ObjectBag::default();

    let players = Players::new(decks.len());
    let player_ids = players.iter().map(Player::id).collect::<Vec<_>>();
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
