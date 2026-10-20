//! What an effect acts on: the players or the characters its selector names, in the order they
//! come, or one of them at random.

use crate::cards::definition::{CharacterSelector, PlayerSelector, PlayerSelectorFilter};
use crate::cards::filter::fulfill_side_filter;
use crate::cards::object::ObjectId;
use crate::game::Game;
use crate::ids::PlayerId;

impl Game {
    pub(super) fn resolve_player_selector(
        &mut self,
        owner: PlayerId,
        selector: &PlayerSelector,
    ) -> Vec<PlayerId> {
        let all = |&PlayerSelectorFilter { side }| {
            self.players
                .iter()
                .map(|p| p.id)
                .filter(|&player_id| fulfill_side_filter(side, owner, player_id))
                .collect::<Vec<_>>()
        };
        match selector {
            PlayerSelector::All(filter) => all(filter),
            PlayerSelector::Owner => vec![owner],
            PlayerSelector::Random(filter) => {
                let candidates = all(filter);
                self.rng.choose(&candidates).copied().into_iter().collect()
            }
        }
    }

    pub(super) fn resolve_character_selector(
        &mut self,
        owner: PlayerId,
        object_id: ObjectId,
        selector: &CharacterSelector,
    ) -> Vec<ObjectId> {
        match selector {
            CharacterSelector::All(filter) => self.scan_characters_with_filter(owner, filter),
            CharacterSelector::Itself => vec![object_id],
            CharacterSelector::OwnerHero => vec![self.hero_id(owner)],
            CharacterSelector::Random(filter) => {
                let candidates = self.scan_characters_with_filter(owner, filter);
                self.rng.choose(&candidates).copied().into_iter().collect()
            }
            CharacterSelector::Targeted(target_id) => self
                .player_input
                .targeted(object_id, *target_id)
                .expect("Target not fulfilled")
                .to_vec(),
        }
    }
}
