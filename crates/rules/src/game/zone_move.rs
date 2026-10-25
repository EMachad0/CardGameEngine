use crate::cards::object::{ObjectBag, ObjectId};
use crate::effect::{CardDestinationZone, ZoneMove};
use crate::game::reporting::Report;
use crate::ids::PlayerId;
use crate::target::CardTargetZone;
use crate::zones::{WasInHand, Zones};

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

/// Takes the card out of `holder`'s `from` zone and puts it at `to` among `owner`'s zones.
/// `None`, with nothing moved, if the card isn't in `from`.
pub(super) fn move_card(
    card: ObjectId,
    holder: PlayerId,
    from: CardTargetZone,
    owner: PlayerId,
    to: CardDestinationZone,
    zones: &mut Zones,
) -> Option<Report> {
    match from {
        CardTargetZone::Hand => {
            let in_hand = WasInHand::from(zones.find_in_hand(holder, card)?);
            zones.take_from_hand(in_hand);
        }
    }
    Some(zone_move(owner, card, to.into(), zones))
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::cards::binder::sets::base::static_card_definition::SPARK;

    #[test]
    fn a_card_moves_from_its_holders_zone_into_its_owners() {
        let (holder, owner) = (PlayerId::new(0), PlayerId::new(1));
        let mut bag = ObjectBag::default();
        let [holder_hero, owner_hero, card] = std::array::from_fn(|_| bag.insert(SPARK, owner));
        let mut zones = Zones::new([(holder_hero, Vec::new()), (owner_hero, Vec::new())]);
        zones.add_to_hand(holder, card);

        let report = move_card(
            card,
            holder,
            CardTargetZone::Hand,
            owner,
            CardDestinationZone::DeckBottom,
            &mut zones,
        );

        assert!(matches!(
            report,
            Some(Report::Buried { player_id, object_id }) if player_id == owner && object_id == card
        ));
        assert_eq!(zones.hand(holder), []);
        assert_eq!(zones.deck(owner), [card]);
    }
}
