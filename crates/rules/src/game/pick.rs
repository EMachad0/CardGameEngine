use crate::{
    cards::effect::PickId,
    choice::PickTarget,
    event::Event,
    game::{Game, PlayerInteractionState},
    ids::PlayerId,
    object::ObjectId,
    observer::Observer,
};

impl Game {
    pub(super) fn reveal(
        &mut self,
        player_id: PlayerId,
        source: ObjectId,
        pick_id: PickId,
        count: u8,
        obs: &mut impl Observer,
    ) {
        let options = {
            let player = self.get_player_mut(player_id);
            let mut options = Vec::new();
            for _ in 0..count {
                let Some(card) = player.zones.deck.pop_front() else {
                    break;
                };

                options.push(card);
            }
            options
        };

        if !options.is_empty() {
            let event = Event::Revealed {
                player_id,
                object_ids: options.clone(),
            };
            let player = self.get_player_mut(player_id);
            player.interaction_state = PlayerInteractionState::PendingPick {
                options,
                source,
                pick_id,
            };
            self.emit(event, obs);
        }
    }

    pub(super) fn pick(
        &mut self,
        player_id: PlayerId,
        object_id: ObjectId,
        obs: &mut impl Observer,
    ) {
        let player = self.get_player_mut(player_id);
        let PlayerInteractionState::PendingPick {
            mut options,
            source,
            pick_id,
        } = std::mem::take(&mut player.interaction_state)
        else {
            unreachable!();
        };
        let idx = options
            .iter()
            .position(|id| *id == object_id)
            .expect("a legal pick should name one of the options");
        let picked = options.remove(idx);

        let obj = self.objects.get_mut(source);
        obj.pick_targets.push(PickTarget {
            pick_id,
            picked: vec![picked],
            others: options,
        });

        let event = Event::Picked {
            player_id,
            object_id,
        };
        self.emit(event, obs);
    }
}
