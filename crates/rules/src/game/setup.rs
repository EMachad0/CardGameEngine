use crate::cards::binder::store::Binder;
use crate::game::deck::{Deck, DeckError, ParsedDeck};
use crate::game::state::{Game, Seating};

/// Parses every deck before it makes the first object.
pub(super) fn set_up(
    seed: u64,
    binder: Binder,
    decks: [Deck; 2],
    shuffle: bool,
) -> Result<Game, DeckError> {
    let decks = decks
        .into_iter()
        .map(|deck| deck.parse(&binder).map(ParsedDeck::into_parts))
        .collect::<Result<Vec<_>, _>>()?;

    #[expect(clippy::disallowed_methods, reason = "the one start of a game's setup")]
    let mut seating = Seating::new(seed, binder, decks.len());

    let player_ids = seating.player_ids();
    let heroes = player_ids
        .iter()
        .zip(&decks)
        .map(|(&player_id, &(hero, _))| seating.make_object(hero, player_id))
        .collect::<Vec<_>>();
    let decks = player_ids
        .iter()
        .zip(decks)
        .map(|(&player_id, (_, cards))| {
            cards
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
    Ok(setting_up.start_first_turn())
}
