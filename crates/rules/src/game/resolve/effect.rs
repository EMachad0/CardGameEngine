use crate::cards::binder::def_id::DefId;
use crate::cards::modifier::Modifier;
use crate::cards::object::{ObjectBag, ObjectId};
use crate::effect::{CharacterVerb, Effect, PickId, PlayerVerb, ZoneMove};
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

/// An effect read before anything is written: its amounts counted, what it acts on listed, and the
/// picks it moves found. Nothing has drawn from the stream yet.
pub(in crate::game) struct PreparedEffect(Prepared);

enum Prepared {
    Players {
        players: Selection<PlayerId>,
        verbs: Vec<PreparedPlayerVerb>,
    },
    Characters {
        characters: Selection<ObjectId>,
        verbs: Vec<PreparedCharacterVerb>,
    },
    Reveal {
        players: Selection<PlayerId>,
        amount: u8,
        source: ObjectId,
        pick_id: PickId,
    },
    ResolvePick {
        owner: PlayerId,
        picked: Vec<ObjectId>,
        others: Vec<ObjectId>,
        picked_move: ZoneMove,
        other_move: ZoneMove,
    },
}

enum PreparedPlayerVerb {
    Draw(u8),
    Summon(DefId),
}

enum PreparedCharacterVerb {
    Damage { amount: u8, source: ObjectId },
    Modify(Modifier),
    AddFriendlyAura(Modifier),
}

pub(in crate::game) fn prepare(
    cards: CardsRef,
    player_ids: &[PlayerId],
    player_input: &PlayerInput,
    next: NextEffect,
) -> PreparedEffect {
    let NextEffect { source, effect } = next;
    let owner = cards.objects.get(source).player_id;
    PreparedEffect(match effect {
        Effect::Players { selector, verbs } => Prepared::Players {
            players: select_players(player_ids, owner, &selector),
            verbs: verbs
                .into_iter()
                .map(|verb| match verb {
                    PlayerVerb::Draw(amount) => {
                        PreparedPlayerVerb::Draw(effect_amount(cards, &amount, owner))
                    }
                    PlayerVerb::Summon(def_id) => PreparedPlayerVerb::Summon(def_id),
                })
                .collect(),
        },
        Effect::Characters { selector, verbs } => Prepared::Characters {
            characters: select_characters(cards, owner, source, &selector),
            verbs: verbs
                .into_iter()
                .map(|verb| match verb {
                    CharacterVerb::Damage(amount) => PreparedCharacterVerb::Damage {
                        amount: effect_amount(cards, &amount, owner),
                        source,
                    },
                    CharacterVerb::Modify(modifier_effect) => {
                        PreparedCharacterVerb::Modify(Modifier {
                            granted_by: source,
                            effect: modifier_effect,
                        })
                    }
                    CharacterVerb::AddFriendlyAura(modifier_effect) => {
                        PreparedCharacterVerb::AddFriendlyAura(Modifier {
                            granted_by: source,
                            effect: modifier_effect,
                        })
                    }
                })
                .collect(),
        },
        Effect::Reveal {
            pick_id,
            selector,
            amount,
        } => Prepared::Reveal {
            amount: effect_amount(cards, &amount, owner),
            players: select_players(player_ids, owner, &selector),
            source,
            pick_id,
        },
        Effect::ResolvePick {
            pick_id,
            picked_move,
            other_move,
        } => {
            let (picked, others) = player_input
                .picks(source, pick_id)
                .map_or_else(Default::default, |picks| {
                    (picks.picked().to_vec(), picks.others().to_vec())
                });
            Prepared::ResolvePick {
                owner,
                picked,
                others,
                picked_move,
                other_move,
            }
        }
    })
}

pub(in crate::game) fn resolve(
    prepared: PreparedEffect,
    rng: &mut Rng,
    zones: &mut Zones,
    objects: &mut ObjectBag,
    player_input: &mut PlayerInput,
) -> Vec<Report> {
    let mut reports = Vec::new();
    match prepared.0 {
        Prepared::Players { players, verbs } => {
            let verb = only_verb(&verbs);
            for player_id in players.resolve(rng) {
                match verb {
                    PreparedPlayerVerb::Draw(amount) => {
                        reports.extend(draw(player_id, *amount, zones, objects));
                    }
                    PreparedPlayerVerb::Summon(def_id) => {
                        let object_id = objects.insert(*def_id, player_id);
                        reports.push(summon(player_id, object_id, zones));
                    }
                }
            }
        }
        Prepared::Characters { characters, verbs } => {
            let verb = only_verb(&verbs);
            for character in characters.resolve(rng) {
                match verb {
                    PreparedCharacterVerb::Damage { amount, source } => {
                        reports.push(damage(character, *amount, *source, objects));
                    }
                    PreparedCharacterVerb::Modify(modifier) => {
                        objects.grant(character, modifier.clone());
                    }
                    PreparedCharacterVerb::AddFriendlyAura(modifier) => {
                        objects.grant_friendly_aura(character, modifier.clone());
                    }
                }
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
            picked,
            others,
            picked_move,
            other_move,
        } => {
            for id in picked {
                reports.push(zone_move(owner, id, picked_move, zones));
            }
            for id in others {
                reports.push(zone_move(owner, id, other_move, zones));
            }
        }
    }
    reports
}

fn only_verb<V>(verbs: &[V]) -> &V {
    let [verb] = verbs else {
        panic!("card data should give every effect exactly one verb");
    };
    verb
}

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
