use crate::cards::object::{ObjectBag, ObjectId};
use crate::effect::ZoneMove;
use crate::game::reporting::Report;
use crate::ids::PlayerId;
use crate::zones::Zones;

pub(super) fn zone_move(
    player_id: PlayerId,
    object_id: ObjectId,
    zone_move: ZoneMove,
    zones: &mut Zones,
) -> Report {
    match zone_move {
        ZoneMove::AddToHand => add_to_hand(player_id, object_id, zones),
        ZoneMove::Bury => bury(player_id, object_id, zones),
    }
}

/// Moves the top card to the end of the hand, `count` times.
/// Each draw from an empty deck costs 1 health instead.
pub(super) fn draw(
    player_id: PlayerId,
    count: u8,
    zones: &mut Zones,
    objects: &mut ObjectBag,
) -> Vec<Report> {
    (0..count)
        .map(|_| {
            if let Some(object_id) = zones.draw(player_id) {
                Report::Drew {
                    player_id,
                    object_id,
                }
            } else {
                let hero_id = zones.hero(player_id);
                let damage = 1;
                objects.add_damage(hero_id, damage);
                Report::FatigueDamaged {
                    amount: damage,
                    player_id,
                    object_id: hero_id,
                }
            }
        })
        .collect()
}

pub(super) fn summon(player_id: PlayerId, object_id: ObjectId, zones: &mut Zones) -> Report {
    zones.enter_board(player_id, object_id);
    Report::BoardEntered {
        player_id,
        object_id,
    }
}

fn add_to_hand(player_id: PlayerId, object_id: ObjectId, zones: &mut Zones) -> Report {
    zones.add_to_hand(player_id, object_id);
    Report::AddedToHand {
        player_id,
        object_id,
    }
}

fn bury(player_id: PlayerId, object_id: ObjectId, zones: &mut Zones) -> Report {
    zones.bury(player_id, object_id);
    Report::Buried {
        player_id,
        object_id,
    }
}
