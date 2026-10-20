use crate::cards::object::ObjectId;
use crate::effect::ZoneMove;
use crate::game::Game;
use crate::game::reporting::Report;
use crate::ids::PlayerId;
use crate::observer::Observer;

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
            if let Some(object_id) = self.zones.draw(player_id) {
                let report = Report::Drew {
                    player_id,
                    object_id,
                };
                self.report(report, obs);
            } else {
                let hero_id = self.zones.hero(player_id);
                let damage = 1;
                self.objects.add_damage(hero_id, damage);
                let report = Report::FatigueDamaged {
                    amount: damage,
                    player_id,
                    object_id: hero_id,
                };
                self.report(report, obs);
            };
        }
    }

    pub(super) fn summon(
        &mut self,
        player_id: PlayerId,
        object_id: ObjectId,
        obs: &mut impl Observer,
    ) {
        self.zones.enter_board(player_id, object_id);
        self.character_entered(player_id, object_id, obs);
    }

    fn add_to_hand(&mut self, player_id: PlayerId, object_id: ObjectId, obs: &mut impl Observer) {
        self.zones.add_to_hand(player_id, object_id);
        let report = Report::AddedToHand {
            player_id,
            object_id,
        };
        self.report(report, obs);
    }

    fn bury(&mut self, player_id: PlayerId, object_id: ObjectId, obs: &mut impl Observer) {
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
}
