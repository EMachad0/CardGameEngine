//! What happens as a turn ends and the next one starts.

use crate::cards::object::ObjectBag;
use crate::game::player::Players;
use crate::game::reporting::Report;
use crate::game::zone_move::draw;
use crate::turn::TurnOrder;
use crate::zones::Zones;

/// Ends the active player's turn and starts the next player's.
pub(super) fn end_turn(
    turn_order: &mut TurnOrder,
    players: &mut Players,
    zones: &mut Zones,
    objects: &mut ObjectBag,
) -> Vec<Report> {
    let active_player_id = turn_order.get_active_player_id();
    let mut reports = vec![Report::TurnEnded {
        player_id: active_player_id,
    }];
    turn_order.end_turn();
    reports.extend(start_turn(turn_order, players, zones, objects));
    reports
}

pub(super) fn start_turn(
    turn_order: &TurnOrder,
    players: &mut Players,
    zones: &mut Zones,
    objects: &mut ObjectBag,
) -> Vec<Report> {
    let active_player_id = turn_order.get_active_player_id();
    let mut reports = vec![Report::TurnStarted {
        player_id: active_player_id,
    }];
    players.mana_mut(active_player_id).refill();
    reports.extend(draw(active_player_id, 1, zones, objects));
    reports
}
