use crate::cards::binder::def_id::DefId;
use crate::cards::binder::sets::base::static_card_definition::HERO;
use crate::cards::binder::store::Binder;
use crate::cards::object::ObjectBag;
use crate::game::player::{Player, Players};
use crate::game::resolve::aftermath::standings::Standings;
use crate::game::state::{Game, Opening};
use crate::rng::Rng;
use crate::turn::TurnOrder;
use crate::zones::Zones;

pub(super) fn set_up(seed: u64, decks: [Vec<DefId>; 2], shuffle: bool) -> Game {
    let mut rng = Rng::new(seed);

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
    let mut zones = Zones::new(heroes.into_iter().zip(decks));
    let turn_order = TurnOrder::new(player_ids.clone());
    #[expect(
        clippy::disallowed_methods,
        reason = "the one start of the standings, as the game is set up"
    )]
    let standings = Standings::new(player_ids.clone());

    if shuffle {
        for &player_id in &player_ids {
            zones.shuffle_deck(player_id, &mut rng);
        }
    }

    #[expect(
        clippy::disallowed_methods,
        reason = "the one opening of a game, as it is set up"
    )]
    let mut opening = Opening::new(rng, turn_order, players, zones, objects, binder, standings);
    for player_id in player_ids {
        opening.deal(player_id, 3);
    }
    opening.start_first_turn()
}
