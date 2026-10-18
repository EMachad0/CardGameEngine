use crate::{
    cards::object::ObjectId,
    effect::{PickId, ZoneMove},
    event::Event,
    game::{Game, reporting::Report, state_check::Death, triggering::MatchedTrigger},
    ids::PlayerId,
    observer::{Observer, Views},
};

pub(super) struct Play {
    player_id: PlayerId,
    object_id: ObjectId,
}

impl From<Play> for Event {
    fn from(play: Play) -> Self {
        Event::Played {
            player_id: play.player_id,
            object_id: play.object_id,
        }
    }
}

impl Game {
    pub(crate) fn zone_move(
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
    pub(crate) fn draw(&mut self, player_id: PlayerId, count: u8, obs: &mut impl Observer) {
        for _ in 0..count {
            if let Some(object_id) = self.zones.draw(player_id) {
                let report = Report::Drew {
                    player_id,
                    object_id,
                };
                self.report(report, obs);
            } else {
                let hero_id = self.zones.hero(player_id);
                let damage = 1;
                let object = self.objects.get_mut(hero_id);
                object.damage += damage;
                let report = Report::FatigueDamaged {
                    amount: damage,
                    player_id,
                    object_id: hero_id,
                };
                self.report(report, obs);
            };
        }
    }

    pub(crate) fn play(
        &mut self,
        player_id: PlayerId,
        object_id: ObjectId,
        obs: &mut impl Observer,
    ) {
        self.zones
            .take_from_hand(player_id, object_id)
            .expect("legal actions guarantee playable cards are in hand");

        if !self.play_effects(object_id).is_empty() {
            self.effect_stack
                .push(object_id, self.play_effects(object_id).to_vec());
        }

        let report = Report::Played(Play {
            player_id,
            object_id,
        });
        self.report(report, obs);

        obs.checkpoint(Views::new(self));
        if self.card_def(object_id).has_board_presence() {
            self.summon(player_id, object_id, obs);
            obs.checkpoint(Views::new(self));
        }
    }

    pub(crate) fn summon(
        &mut self,
        player_id: PlayerId,
        object_id: ObjectId,
        obs: &mut impl Observer,
    ) {
        self.zones.enter_board(player_id, object_id);
        self.character_entered(player_id, object_id, obs);
    }

    pub(super) fn kill(&mut self, deaths: Vec<Death>, obs: &mut impl Observer) {
        // Enqueue death rattles
        for death in deaths.iter() {
            for MatchedTrigger { source, effects } in self.death_rattles(death.object_id()) {
                self.effect_stack.push(source, effects);
            }
        }

        for death in deaths.into_iter() {
            let object_id = death.object_id();
            let object = self.objects.get(object_id);
            if self.card_def(object_id).leaves_zone_on_death() {
                self.destroy(object.player_id, object_id, obs);
            }
            self.report(Report::Died(death), obs);
        }
    }

    pub(crate) fn destroy(
        &mut self,
        player_id: PlayerId,
        object_id: ObjectId,
        obs: &mut impl Observer,
    ) {
        self.zones
            .leave_board(player_id, object_id)
            .expect("object not in board");

        if self.card_def(object_id).goes_to_graveyard() {
            self.entomb(player_id, object_id, obs);
        }
    }

    pub(crate) fn entomb(
        &mut self,
        player_id: PlayerId,
        object_id: ObjectId,
        _obs: &mut impl Observer,
    ) {
        self.zones.entomb(player_id, object_id);
    }

    pub(crate) fn reveal(
        &mut self,
        player_id: PlayerId,
        source: ObjectId,
        pick_id: PickId,
        count: u8,
        obs: &mut impl Observer,
    ) {
        #[expect(
            clippy::disallowed_methods,
            reason = "the one reveal, which posts the revealed cards as a pending pick"
        )]
        let options = self.zones.reveal(player_id, count);
        let report = Report::Revealed {
            player_id,
            object_ids: options.clone(),
        };
        if self
            .player_input
            .ask_pick(player_id, source, pick_id, options)
        {
            self.report(report, obs);
        }
    }

    pub(crate) fn pick(
        &mut self,
        player_id: PlayerId,
        object_id: ObjectId,
        obs: &mut impl Observer,
    ) {
        self.player_input.pick(player_id, object_id);

        let report = Report::Picked {
            player_id,
            object_id,
        };
        self.report(report, obs);
    }

    pub(crate) fn add_to_hand(
        &mut self,
        player_id: PlayerId,
        object_id: ObjectId,
        obs: &mut impl Observer,
    ) {
        self.zones.add_to_hand(player_id, object_id);
        let report = Report::AddedToHand {
            player_id,
            object_id,
        };
        self.report(report, obs);
    }

    pub(crate) fn bury(
        &mut self,
        player_id: PlayerId,
        object_id: ObjectId,
        obs: &mut impl Observer,
    ) {
        self.zones.bury(player_id, object_id);
        let report = Report::Buried {
            player_id,
            object_id,
        };
        self.report(report, obs);
    }

    fn character_entered(
        &mut self,
        player_id: PlayerId,
        object_id: ObjectId,
        obs: &mut impl Observer,
    ) {
        let report = Report::BoardEntered {
            player_id,
            object_id,
        };
        self.report(report, obs);
    }

    pub(crate) fn damage(
        &mut self,
        target: ObjectId,
        amount: u8,
        source: ObjectId,
        obs: &mut impl Observer,
    ) {
        let object = self.objects.get_mut(target);
        object.damage += amount;
        let report = Report::Damaged {
            target,
            amount,
            source,
        };
        self.report(report, obs);
    }
}
