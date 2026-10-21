//! What an effect acts on: the players or the characters its selector names, each of them in the
//! order they come, or one of them at random.

use crate::cards::definition::{CharacterSelector, PlayerSelector, PlayerSelectorFilter};
use crate::cards::filter::fulfill_side_filter;
use crate::cards::object::ObjectId;
use crate::game::cards::Cards;
use crate::game::player_input::PlayerInput;
use crate::ids::PlayerId;
use crate::rng::Rng;

/// What a selector names: each of some values, or one of them at random.
pub(super) struct Selection<T>(Named<T>);

enum Named<T> {
    Each(Vec<T>),
    OneOf(Vec<T>),
}

impl<T: Copy> Selection<T> {
    pub(super) fn resolve(self, rng: &mut Rng) -> Vec<T> {
        match self.0 {
            Named::Each(values) => values,
            Named::OneOf(values) => rng.choose(&values).copied().into_iter().collect(),
        }
    }
}

pub(super) fn select_players(
    players: &[PlayerId],
    owner: PlayerId,
    selector: &PlayerSelector,
) -> Selection<PlayerId> {
    let all = |&PlayerSelectorFilter { side }| {
        players
            .iter()
            .copied()
            .filter(|&player_id| fulfill_side_filter(side, owner, player_id))
            .collect::<Vec<_>>()
    };
    Selection(match selector {
        PlayerSelector::All(filter) => Named::Each(all(filter)),
        PlayerSelector::Owner => Named::Each(vec![owner]),
        PlayerSelector::Random(filter) => Named::OneOf(all(filter)),
    })
}

pub(super) fn select_characters(
    cards: Cards<'_>,
    player_input: &PlayerInput,
    owner: PlayerId,
    object_id: ObjectId,
    selector: &CharacterSelector,
) -> Selection<ObjectId> {
    Selection(match selector {
        CharacterSelector::All(filter) => {
            Named::Each(cards.scan_characters_with_filter(owner, filter))
        }
        CharacterSelector::Itself => Named::Each(vec![object_id]),
        CharacterSelector::OwnerHero => Named::Each(vec![cards.zones.hero(owner)]),
        CharacterSelector::Random(filter) => {
            Named::OneOf(cards.scan_characters_with_filter(owner, filter))
        }
        CharacterSelector::Targeted(target_id) => Named::Each(
            player_input
                .targeted(object_id, *target_id)
                .expect("Target not fulfilled")
                .to_vec(),
        ),
    })
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn each_value_named_comes_back_in_order() {
        let selection = Selection(Named::Each(vec![30, 10, 20]));

        assert_eq!(selection.resolve(&mut Rng::new(3)), vec![30, 10, 20]);
    }

    #[test]
    fn each_value_named_leaves_the_stream_as_it_was() {
        let mut rng = Rng::new(3);

        Selection(Named::Each(vec![30, 10, 20])).resolve(&mut rng);

        assert_eq!(rng, Rng::new(3));
    }

    #[test]
    fn one_value_at_random_is_one_of_those_named() {
        let values = [30, 10, 20];
        for seed in 0..100 {
            let resolved = Selection(Named::OneOf(values.to_vec())).resolve(&mut Rng::new(seed));

            assert!(
                resolved.len() == 1 && values.contains(&resolved[0]),
                "seed {seed} resolved {resolved:?}",
            );
        }
    }

    #[test]
    fn one_value_at_random_takes_one_draw_from_the_stream() {
        for seed in 0..100 {
            let mut resolver = Rng::new(seed);
            let mut drawer = Rng::new(seed);

            Selection(Named::OneOf(vec![30, 10, 20])).resolve(&mut resolver);
            drawer.next_u64();

            assert_eq!(resolver, drawer, "seed {seed}");
        }
    }
}
