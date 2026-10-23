use crate::cards::object::ObjectId;
use crate::event::Event;
use crate::game::cards_ref::CardsRef;
use crate::game::effect_stack::EffectStack;
use crate::game::reporting::Report;
use crate::game::resolve::aftermath::standings::Standings;
use crate::game::stats::{character_health, hero_health};
use crate::game::triggering::{MatchedTrigger, death_rattles};
use crate::ids::PlayerId;
use crate::zones::Zones;

pub(in crate::game) struct Death {
    object_id: ObjectId,
}

impl From<Death> for Event {
    fn from(death: Death) -> Self {
        Event::Died {
            object_id: death.object_id,
        }
    }
}

/// The dead one state check pass found, read before any of them leaves: each one's own death
/// rattles, where each goes, and the players whose heroes died.
pub(in crate::game) struct DeathPass {
    /// In death order.
    deaths: Vec<Death>,
    /// In death order.
    death_rattles: Vec<MatchedTrigger>,
    /// The dead that leave their zone, in death order.
    removals: Vec<Removal>,
    dead_players: Vec<PlayerId>,
}

struct Removal {
    player_id: PlayerId,
    object_id: ObjectId,
    to_graveyard: bool,
}

/// `None` if no character is dead.
pub(in crate::game) fn find_deaths(
    cards: CardsRef,
    players: &[PlayerId],
    standings: &Standings,
) -> Option<DeathPass> {
    let dead_minions = find_dead(cards, players);
    let dead_players = standings
        .playing()
        .filter(|&player_id| hero_health(cards, player_id) <= 0)
        .collect::<Vec<_>>();
    if dead_minions.is_empty() && dead_players.is_empty() {
        return None;
    }
    let dead_heroes = dead_players.iter().map(|&p| cards.zones.hero(p));

    let mut death_pass = dead_minions
        .into_iter()
        .chain(dead_heroes)
        .collect::<Vec<_>>();
    death_pass.sort_by_key(|&o| cards.zones.death_order(o));

    let death_rattles = death_pass
        .iter()
        .flat_map(|&object_id| death_rattles(cards, object_id))
        .collect();
    let removals = death_pass
        .iter()
        .filter_map(|&object_id| {
            let card_def = cards.card_def(object_id);
            card_def.leaves_zone_on_death().then(|| Removal {
                player_id: cards.objects.get(object_id).player_id,
                object_id,
                to_graveyard: card_def.goes_to_graveyard(),
            })
        })
        .collect();
    let deaths = death_pass
        .into_iter()
        .map(|object_id| Death { object_id })
        .collect();
    Some(DeathPass {
        deaths,
        death_rattles,
        removals,
        dead_players,
    })
}

/// Pushes the death rattles, removes the dead and eliminates the players whose heroes died, all
/// before any death is reported.
pub(in crate::game) fn kill(
    death_pass: DeathPass,
    zones: &mut Zones,
    effect_stack: &mut EffectStack,
    standings: &mut Standings,
) -> Vec<Report> {
    let DeathPass {
        deaths,
        death_rattles,
        removals,
        dead_players,
    } = death_pass;
    for MatchedTrigger { source, effects } in death_rattles {
        effect_stack.push(source, effects);
    }
    for Removal {
        player_id,
        object_id,
        to_graveyard,
    } in removals
    {
        destroy(zones, player_id, object_id, to_graveyard);
    }
    standings.eliminate(&dead_players);
    deaths.into_iter().map(Report::Died).collect()
}

fn find_dead(cards: CardsRef, players: &[PlayerId]) -> Vec<ObjectId> {
    let mut death_pass = Vec::new();
    for &player_id in players {
        for card in cards.zones.board_cards(player_id) {
            if character_health(cards, card.into()) <= 0 {
                death_pass.push(card.object_id());
            }
        }
    }
    death_pass
}

fn destroy(zones: &mut Zones, player_id: PlayerId, object_id: ObjectId, to_graveyard: bool) {
    #[expect(
        clippy::disallowed_methods,
        reason = "the one removal from the board, made as the state check removes the dead"
    )]
    zones
        .leave_board(player_id, object_id)
        .expect("object not in board");

    if to_graveyard {
        entomb(zones, player_id, object_id);
    }
}

fn entomb(zones: &mut Zones, player_id: PlayerId, object_id: ObjectId) {
    #[expect(
        clippy::disallowed_methods,
        reason = "the one entry into a graveyard, made as the state check removes the dead"
    )]
    zones.entomb(player_id, object_id);
}
