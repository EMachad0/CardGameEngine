//! What an effect acts on, as its selector says: players or characters, each of them in the order
//! they come or one of them at random, or the cards targeted.

use crate::cards::definition::{
    CardSelector, CharacterSelector, PlayerSelector, PlayerSelectorFilter,
};
use crate::cards::filter::fulfill_side_filter;
use crate::cards::object::ObjectId;
use crate::choice::CardTargetZone;
use crate::game::cards_ref::CardsRef;
use crate::ids::PlayerId;
use crate::rng::Rng;
use crate::target::Bound;

pub(super) struct Selection<T>(Values<T>);

pub(super) struct CardSelection {
    pub(super) from: CardTargetZone,
    /// Whose `from` zone holds the cards.
    pub(super) chooser: PlayerId,
    pub(super) cards: Selection<ObjectId>,
}

enum Values<T> {
    Each(Vec<T>),
    OneOf(Vec<T>),
}

impl<T: Copy> Selection<T> {
    pub(super) fn resolve(self, rng: &mut Rng) -> Vec<T> {
        match self.0 {
            Values::Each(values) => values,
            Values::OneOf(values) => rng.choose(&values).copied().into_iter().collect(),
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
        PlayerSelector::All(filter) => Values::Each(all(filter)),
        PlayerSelector::Owner => Values::Each(vec![owner]),
        PlayerSelector::Random(filter) => Values::OneOf(all(filter)),
    })
}

pub(super) fn select_characters(
    cards: CardsRef,
    owner: PlayerId,
    source: ObjectId,
    selector: &CharacterSelector<Bound>,
) -> Selection<ObjectId> {
    Selection(match selector {
        CharacterSelector::All(filter) => {
            Values::Each(cards.scan_characters_with_filter(owner, filter))
        }
        CharacterSelector::Itself => Values::Each(vec![source]),
        CharacterSelector::OwnerHero => Values::Each(vec![cards.zones.hero(owner)]),
        CharacterSelector::Random(filter) => {
            Values::OneOf(cards.scan_characters_with_filter(owner, filter))
        }
        CharacterSelector::Targeted(targeted) => Values::Each(targeted.clone()),
    })
}

pub(super) fn select_cards(
    _cards: CardsRef,
    owner: PlayerId,
    _source: ObjectId,
    selector: &CardSelector<Bound>,
) -> CardSelection {
    match selector {
        CardSelector::Targeted(targeted) => CardSelection {
            from: targeted.zone,
            chooser: owner,
            cards: Selection(Values::Each(targeted.object_ids.clone())),
        },
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn resolving_each_value_keeps_their_order() {
        let selection = Selection(Values::Each(vec![30, 10, 20]));

        assert_eq!(selection.resolve(&mut Rng::new(3)), vec![30, 10, 20]);
    }

    #[test]
    fn resolving_each_value_takes_no_draw_from_the_stream() {
        let mut rng = Rng::new(3);

        Selection(Values::Each(vec![30, 10, 20])).resolve(&mut rng);

        assert_eq!(rng, Rng::new(3));
    }

    #[test]
    fn resolving_one_at_random_gives_one_of_the_values() {
        let values = [30, 10, 20];
        for seed in 0..100 {
            let resolved = Selection(Values::OneOf(values.to_vec())).resolve(&mut Rng::new(seed));

            assert!(
                resolved.len() == 1 && values.contains(&resolved[0]),
                "seed {seed} resolved {resolved:?}",
            );
        }
    }

    #[test]
    fn resolving_one_at_random_from_nothing_gives_nothing() {
        let selection = Selection::<u32>(Values::OneOf(Vec::new()));

        assert_eq!(selection.resolve(&mut Rng::new(3)), Vec::new());
    }

    #[test]
    fn resolving_one_at_random_takes_one_draw_from_the_stream() {
        for seed in 0..100 {
            let mut resolver = Rng::new(seed);
            let mut drawer = Rng::new(seed);

            Selection(Values::OneOf(vec![30, 10, 20])).resolve(&mut resolver);
            drawer.next_u64();

            assert_eq!(resolver, drawer, "seed {seed}");
        }
    }
}
