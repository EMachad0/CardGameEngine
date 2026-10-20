use crate::cards::filter::CharacterFilter;
use crate::cards::object::ObjectId;
use crate::game::Game;
use crate::ids::PlayerId;

impl Game {
    pub(crate) fn scan_characters_with_filter(
        &self,
        asking: PlayerId,
        filter: &CharacterFilter,
    ) -> Vec<ObjectId> {
        self.zones
            .characters_in_play()
            .filter(|&object_id| {
                let owner = self.objects.get(object_id).player_id;
                filter.matches(asking, owner, self.card_def(object_id).character_kind())
            })
            .collect()
    }
}
