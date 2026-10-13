use crate::{
    cards::effect::ZoneMove,
    effect::PendingEffect,
    event::Event,
    game::Game,
    ids::PlayerId,
    object::ObjectId,
    observer::{Observer, Views},
};

impl Game {
    pub(super) fn zone_move(
        &mut self,
        player_id: PlayerId,
        object_id: ObjectId,
        zone_move: ZoneMove,
        obs: &mut impl Observer,
    ) {
        match zone_move {
            ZoneMove::AddToHand => self.add_to_hand(player_id, object_id, obs),
            ZoneMove::Bury => self.bury(player_id, object_id, obs),
        }
    }

    /// Moves the top card to the end of the hand, `count` times.
    /// Each draw from an empty deck costs 1 health instead.
    pub(super) fn draw(&mut self, player_id: PlayerId, count: u8, obs: &mut impl Observer) {
        for _ in 0..count {
            let player = self.get_player_mut(player_id);
            let card = player.zones.deck.pop_front();
            if let Some(object_id) = card {
                player.zones.hand.add(object_id);
                let event = Event::Drew {
                    player_id,
                    object_id,
                };
                self.emit(event, obs);
                self.objects.update_order(object_id);
            } else {
                let hero_id = player.zones.hero;
                let damage = 1;
                let object = self.objects.get_mut(hero_id);
                object.damage += damage;
                let event = Event::FatigueDamaged {
                    amount: damage,
                    player_id,
                    object_id: hero_id,
                };
                self.emit(event, obs);
            };
        }
    }

    pub(super) fn play(
        &mut self,
        player_id: PlayerId,
        object_id: ObjectId,
        obs: &mut impl Observer,
    ) {
        self.get_player_mut(player_id)
            .zones
            .hand
            .remove(object_id)
            .expect("legal actions guarantee playable cards are in hand");

        if !self.play_effects(object_id).is_empty() {
            self.effect_stack.push(PendingEffect::new(
                object_id,
                self.play_effects(object_id).to_vec(),
            ));
        }

        let event = Event::Played {
            player_id,
            object_id,
        };
        self.emit(event, obs);

        obs.checkpoint(Views::new(self));
        if self.card_def(object_id).has_board_presence() {
            self.summon(player_id, object_id, obs);
            obs.checkpoint(Views::new(self));
        }
    }

    pub(super) fn summon(
        &mut self,
        player_id: PlayerId,
        object_id: ObjectId,
        obs: &mut impl Observer,
    ) {
        self.get_player_mut(player_id).zones.board.add(object_id);
        self.character_entered(player_id, object_id, obs);
    }

    pub(super) fn kill(&mut self, object_ids: &[ObjectId], obs: &mut impl Observer) {
        // Enqueue death rattles
        for &object_id in object_ids.iter() {
            for death_rattle in self.death_rattles(object_id).into_iter() {
                self.effect_stack.push(PendingEffect {
                    source: object_id,
                    effects: death_rattle.def.effects,
                });
            }
        }

        for &object_id in object_ids.iter() {
            let object = self.objects.get(object_id);
            let is_hero = self.card_def(object_id).is_hero();
            if !is_hero {
                self.destroy(object.player_id, object_id, obs);
            }
            self.emit(Event::Died { object_id }, obs);
        }
    }

    fn destroy(&mut self, player_id: PlayerId, object_id: ObjectId, obs: &mut impl Observer) {
        self.get_player_mut(player_id)
            .zones
            .board
            .remove(object_id)
            .expect("object not in board");

        if self.card_def(object_id).has_deck_presence() {
            self.emtomb(player_id, object_id, obs);
        }
    }

    fn emtomb(&mut self, player_id: PlayerId, object_id: ObjectId, _obs: &mut impl Observer) {
        self.objects.update_order(object_id);
        self.get_player_mut(player_id)
            .zones
            .graveyard
            .add(object_id);
    }

    fn add_to_hand(&mut self, player_id: PlayerId, object_id: ObjectId, obs: &mut impl Observer) {
        let player = self.get_player_mut(player_id);
        player.zones.hand.add(object_id);
        self.objects.update_order(object_id);
        let event = Event::AddedToHand {
            player_id,
            object_id,
        };
        self.emit(event, obs);
    }

    fn bury(&mut self, player_id: PlayerId, object_id: ObjectId, obs: &mut impl Observer) {
        let player = self.get_player_mut(player_id);
        player.zones.deck.push_back(object_id);
        self.objects.update_order(object_id);
        let event = Event::Buried {
            player_id,
            object_id,
        };
        self.emit(event, obs);
    }

    fn character_entered(
        &mut self,
        player_id: PlayerId,
        object_id: ObjectId,
        obs: &mut impl Observer,
    ) {
        self.objects.update_order(object_id);
        let event = Event::BoardEntered {
            player_id,
            object_id,
        };
        self.emit(event, obs);
    }

    pub(super) fn damage(
        &mut self,
        target: ObjectId,
        amount: u8,
        source: ObjectId,
        obs: &mut impl Observer,
    ) {
        let object = self.objects.get_mut(target);
        object.damage += amount;
        let event = Event::Damaged {
            target,
            amount,
            source,
        };
        self.emit(event, obs);
    }
}
