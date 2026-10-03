use crate::{
    Event, Game, ObjectId, Observer, PlayerId, Views,
    game::PlayerInteractionState,
    history::{HistoryEntry, HistoryKind},
};

impl Game {
    /// Moves the top card to the end of the hand, `count` times.
    /// Each draw from an empty deck costs 1 health instead.
    pub(crate) fn draw(&mut self, player_id: PlayerId, count: usize, obs: &mut impl Observer) {
        let player = self.get_player_mut(player_id);
        for _ in 0..count {
            if let Some(object_id) = player.zones.deck.pop_front() {
                obs.event(&Event::Drew {
                    player_id,
                    object_id,
                });
                player.zones.hand.add(object_id);
            } else {
                obs.event(&Event::FatigueDamaged {
                    amount: 1,
                    player_id,
                });
                player.health -= 1;
            };
        }
    }

    pub(crate) fn play(
        &mut self,
        player_id: PlayerId,
        object_id: ObjectId,
        obs: &mut impl Observer,
    ) {
        self.get_player_mut(player_id)
            .zones
            .hand
            .remove(object_id)
            .expect("object not in hand");

        if let Some(object) = self.objects.get(object_id) {
            self.history.entries.push(HistoryEntry::new(
                player_id,
                HistoryKind::CardPlayed {
                    object: object.clone(),
                },
                self.turn_order.turn_count(),
            ));
        }
        obs.event(&Event::Played {
            player_id,
            object_id,
        });
        obs.checkpoint(Views::new(self));
        if let Ok(effects) = self.on_play(object_id) {
            self.apply_effects(player_id, object_id, effects, obs);
        }

        let def_id = self.def_id(object_id).expect("unexpected lookup error");
        if self.binder.has_board_presence(def_id) {
            self.spawn(player_id, object_id, obs);
        }
    }

    pub(crate) fn spawn(
        &mut self,
        player_id: PlayerId,
        object_id: ObjectId,
        obs: &mut impl Observer,
    ) {
        self.get_player_mut(player_id).zones.board.add(object_id);
        if let Ok(effects) = self.on_board_enter(object_id) {
            self.apply_effects(player_id, object_id, effects, obs);
        }
        obs.event(&Event::BoardEntered {
            player_id,
            object_id,
        });
    }

    pub(crate) fn kill(
        &mut self,
        player_id: PlayerId,
        object_id: ObjectId,
        obs: &mut impl Observer,
    ) {
        if let Ok(effects) = self.on_death(object_id) {
            self.apply_effects(player_id, object_id, effects, obs);
        }
        if let Some(object) = self.objects.get(object_id) {
            self.history.entries.push(HistoryEntry::new(
                player_id,
                HistoryKind::MonsterDied {
                    object: object.clone(),
                },
                self.turn_order.turn_count(),
            ));
        }
        obs.event(&Event::Died { object_id });
        self.destroy(player_id, object_id, obs);
    }

    pub(crate) fn destroy(
        &mut self,
        player_id: PlayerId,
        object_id: ObjectId,
        obs: &mut impl Observer,
    ) {
        if let Ok(effects) = self.on_board_leave(object_id) {
            self.apply_effects(player_id, object_id, effects, obs);
        }
        self.get_player_mut(player_id)
            .zones
            .board
            .remove(object_id)
            .expect("object not in board");

        let def_id = self.def_id(object_id).expect("unexpected lookup error");
        if self.binder.has_deck_presence(def_id) {
            self.emtomb(player_id, object_id, obs);
        }
    }

    pub(crate) fn emtomb(
        &mut self,
        player_id: PlayerId,
        object_id: ObjectId,
        _obs: &mut impl Observer,
    ) {
        self.get_player_mut(player_id)
            .zones
            .graveyard
            .add(object_id);
    }

    pub(crate) fn reveal(&mut self, player_id: PlayerId, count: usize, obs: &mut impl Observer) {
        let player = self.get_player_mut(player_id);

        let mut options = Vec::new();
        for _ in 0..count {
            let Some(card) = player.zones.deck.pop_front() else {
                break;
            };

            options.push(card);
        }
        if !options.is_empty() {
            let event = Event::Revealed {
                player_id,
                object_ids: options,
            };
            obs.event(&event);
            if let Event::Revealed {
                object_ids: options,
                ..
            } = event
            {
                player.interaction_state = PlayerInteractionState::Picker { options }
            } else {
                unreachable!();
            };
        }
    }

    pub(crate) fn pick(
        &mut self,
        player_id: PlayerId,
        object_id: ObjectId,
        obs: &mut impl Observer,
    ) {
        let player = self.get_player_mut(player_id);
        let PlayerInteractionState::Picker { mut options } =
            std::mem::take(&mut player.interaction_state)
        else {
            unreachable!();
        };
        if let Some(idx) = options.iter().position(|id| *id == object_id) {
            let picked = options.remove(idx);
            player.zones.hand.add(picked);

            obs.event(&Event::Picked {
                player_id,
                object_id,
            });

            options
                .into_iter()
                .for_each(|id| self.bury(player_id, id, obs));
        }
    }

    pub(crate) fn bury(
        &mut self,
        player_id: PlayerId,
        object_id: ObjectId,
        obs: &mut impl Observer,
    ) {
        let player = self.get_player_mut(player_id);
        player.zones.deck.push_back(object_id);
        obs.event(&Event::Buried {
            player_id,
            object_id,
        });
    }
}
