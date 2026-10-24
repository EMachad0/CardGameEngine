use crate::cards::binder::def_id::DefId;
use crate::cards::binder::sets::base::static_card_definition::HERO;
use crate::cards::binder::store::Binder;
use crate::game::state::{Game, Seating};

pub(super) fn set_up(seed: u64, decks: [Vec<DefId>; 2], shuffle: bool) -> Game {
    #[expect(clippy::disallowed_methods, reason = "the one start of a game's setup")]
    let mut seating = Seating::new(seed, Binder::built_in(), decks.len());

    let player_ids = seating.player_ids();
    let heroes = player_ids
        .iter()
        .map(|&player_id| seating.make_object(HERO, player_id))
        .collect::<Vec<_>>();
    let decks = player_ids
        .iter()
        .zip(decks)
        .map(|(&player_id, deck_defs)| {
            deck_defs
                .into_iter()
                .map(|def_id| seating.make_object(def_id, player_id))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let mut setting_up = seating.seat(player_ids.clone(), heroes.into_iter().zip(decks));

    if shuffle {
        for &player_id in &player_ids {
            setting_up.shuffle_deck(player_id);
        }
    }

    for player_id in player_ids {
        setting_up.draw(player_id, 3);
    }
    setting_up.start_first_turn()
}
