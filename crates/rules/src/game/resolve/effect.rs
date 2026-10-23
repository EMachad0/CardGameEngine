use crate::cards::binder::def_id::DefId;
use crate::cards::modifier::Modifier;
use crate::cards::object::{ObjectBag, ObjectId};
use crate::effect::{Effect, PickId, ZoneMove};
use crate::game::amount::effect_amount;
use crate::game::cards_ref::CardsRef;
use crate::game::effect_stack::NextEffect;
use crate::game::player_input::awaiting::PlayerInput;
use crate::game::reporting::Report;
use crate::game::selector::{Selection, select_characters, select_players};
use crate::game::zone_move::{draw, summon, zone_move};
use crate::ids::PlayerId;
use crate::rng::Rng;
use crate::zones::Zones;

/// An effect read before anything is written: its amount counted, what it acts on listed, and the
/// picks it moves found. Nothing has drawn from the stream yet.
pub(in crate::game) struct PreparedEffect(Prepared);

enum Prepared {
    Draw {
        players: Selection<PlayerId>,
        amount: u8,
    },
    Reveal {
        players: Selection<PlayerId>,
        amount: u8,
        source: ObjectId,
        pick_id: PickId,
    },
    ResolvePick {
        owner: PlayerId,
        picks: Option<(Vec<ObjectId>, Vec<ObjectId>)>,
        picked_move: ZoneMove,
        other_move: ZoneMove,
    },
    Damage {
        characters: Selection<ObjectId>,
        amount: u8,
        source: ObjectId,
    },
    Buff {
        characters: Selection<ObjectId>,
        modifier: Modifier,
    },
    AddFriendlyAura {
        characters: Selection<ObjectId>,
        modifier: Modifier,
    },
    Summon {
        players: Selection<PlayerId>,
        def_id: DefId,
    },
}

pub(in crate::game) fn prepare(
    cards: CardsRef,
    players: &[PlayerId],
    player_input: &PlayerInput,
    next: NextEffect,
) -> PreparedEffect {
    let NextEffect { source, effect } = next;
    let owner = cards.objects.get(source).player_id;
    PreparedEffect(match effect {
        Effect::Draw { selector, amount } => Prepared::Draw {
            amount: effect_amount(cards, &amount, owner),
            players: select_players(players, owner, &selector),
        },
        Effect::Reveal {
            pick_id,
            selector,
            amount,
        } => Prepared::Reveal {
            amount: effect_amount(cards, &amount, owner),
            players: select_players(players, owner, &selector),
            source,
            pick_id,
        },
        Effect::ResolvePick {
            pick_id,
            picked_move,
            other_move,
        } => Prepared::ResolvePick {
            owner,
            picks: player_input
                .picks(source, pick_id)
                .map(|picks| (picks.picked().to_vec(), picks.others().to_vec())),
            picked_move,
            other_move,
        },
        Effect::Damage { selector, amount } => Prepared::Damage {
            amount: effect_amount(cards, &amount, owner),
            characters: select_characters(cards, owner, source, &selector),
            source,
        },
        Effect::Buff { selector, effect } => Prepared::Buff {
            characters: select_characters(cards, owner, source, &selector),
            modifier: Modifier {
                granted_by: source,
                effect,
            },
        },
        Effect::AddFriendlyAura { selector, effect } => Prepared::AddFriendlyAura {
            characters: select_characters(cards, owner, source, &selector),
            modifier: Modifier {
                granted_by: source,
                effect,
            },
        },
        Effect::Summon { selector, def_id } => Prepared::Summon {
            players: select_players(players, owner, &selector),
            def_id,
        },
    })
}

pub(in crate::game) fn apply(
    prepared: PreparedEffect,
    rng: &mut Rng,
    zones: &mut Zones,
    objects: &mut ObjectBag,
    player_input: &mut PlayerInput,
) -> Vec<Report> {
    let mut reports = Vec::new();
    match prepared.0 {
        Prepared::Draw { players, amount } => {
            for target in players.resolve(rng) {
                reports.extend(draw(target, amount, zones, objects));
            }
        }
        Prepared::Reveal {
            players,
            amount,
            source,
            pick_id,
        } => {
            for target in players.resolve(rng) {
                reports.extend(reveal(target, source, pick_id, amount, zones, player_input));
            }
        }
        Prepared::ResolvePick {
            owner,
            picks,
            picked_move,
            other_move,
        } => {
            if let Some((picked, others)) = picks {
                for id in picked {
                    reports.push(zone_move(owner, id, picked_move, zones));
                }
                for id in others {
                    reports.push(zone_move(owner, id, other_move, zones));
                }
            }
        }
        Prepared::Damage {
            characters,
            amount,
            source,
        } => {
            for target in characters.resolve(rng) {
                reports.push(damage(target, amount, source, objects));
            }
        }
        Prepared::Buff {
            characters,
            modifier,
        } => {
            for target in characters.resolve(rng) {
                objects.grant(target, modifier.clone());
            }
        }
        Prepared::AddFriendlyAura {
            characters,
            modifier,
        } => {
            for target in characters.resolve(rng) {
                objects.grant_friendly_aura(target, modifier.clone());
            }
        }
        Prepared::Summon { players, def_id } => {
            for player_id in players.resolve(rng) {
                let object_id = objects.insert(def_id, player_id);
                reports.push(summon(player_id, object_id, zones));
            }
        }
    }
    reports
}

/// `None` if nothing was revealed, so no picking starts.
fn reveal(
    player_id: PlayerId,
    source: ObjectId,
    pick_id: PickId,
    count: u8,
    zones: &mut Zones,
    player_input: &mut PlayerInput,
) -> Option<Report> {
    #[expect(
        clippy::disallowed_methods,
        reason = "the one reveal, which starts a picking"
    )]
    let options = zones.reveal(player_id, count);
    let report = Report::Revealed {
        player_id,
        object_ids: options.clone(),
    };
    player_input
        .start_picking(player_id, source, pick_id, options)
        .then_some(report)
}

fn damage(target: ObjectId, amount: u8, source: ObjectId, objects: &mut ObjectBag) -> Report {
    objects.add_damage(target, amount);
    Report::Damaged {
        target,
        amount,
        source,
    }
}
