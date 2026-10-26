use crate::cards::binder::def_id::DefId;
use crate::cards::filter::CharacterFilter;
use crate::cards::modifier::Modifier;
use crate::cards::object::{ObjectBag, ObjectId};
use crate::choice::Choice;
use crate::effect::{CardDestinationZone, CardVerb, CharacterVerb, Effect, PlayerVerb};
use crate::game::amount::effect_amount;
use crate::game::cards_ref::CardsRef;
use crate::game::effect_stack::{FinishedPick, NextEffect};
use crate::game::picking::{CardPicking, CharacterPicking, PickedObjects, Picking};
use crate::game::reporting::Report;
use crate::game::selector::{
    CardSelection, SelectedOrPick, Selection, select_cards, select_characters, select_picked_cards,
    select_picked_characters, select_players,
};
use crate::game::zone_move::{draw, move_card, summon};
use crate::ids::PlayerId;
use crate::pick::CardPickOptions;
use crate::rng::Rng;
use crate::zones::Zones;

/// An effect read before its first write: ready to resolve, or a pick to open.
pub(in crate::game) enum PreparedOrOpen {
    Prepared(PreparedEffect),
    Open(Picking),
}

/// An effect with its amounts counted and what it may act on listed, both before its first write.
pub(in crate::game) struct PreparedEffect(Prepared);

enum Prepared {
    Players {
        players: Selection<PlayerId>,
        verbs: Vec<PreparedPlayerVerb>,
    },
    Characters(PreparedCharacters),
    Cards(PreparedCards),
    /// In the order their verbs run.
    PickedCharacters([PreparedCharacters; 2]),
    /// In the order their verbs run.
    PickedCards([PreparedCards; 2]),
}

struct PreparedCharacters {
    characters: Selection<ObjectId>,
    verbs: Vec<PreparedCharacterVerb>,
}

struct PreparedCards {
    cards: CardSelection,
    verbs: Vec<PreparedCardVerb>,
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

enum PreparedCardVerb {
    Move(CardDestinationZone),
}

pub(in crate::game) fn prepare(
    cards: CardsRef,
    player_ids: &[PlayerId],
    next: NextEffect,
) -> PreparedOrOpen {
    let NextEffect { source, effect } = next;
    let owner = cards.objects.get(source).player_id;
    PreparedOrOpen::Prepared(PreparedEffect(match effect {
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
        Effect::Characters { selector, verbs } => {
            match select_characters(cards, owner, source, selector) {
                SelectedOrPick::Selected(characters) => Prepared::Characters(PreparedCharacters {
                    characters,
                    verbs: prepare_character_verbs(cards, owner, source, verbs),
                }),
                SelectedOrPick::Pick { choice, others } => {
                    return PreparedOrOpen::Open(open_character_pick(
                        cards, owner, source, choice, verbs, others,
                    ));
                }
            }
        }
        Effect::Cards { selector, verbs } => match select_cards(cards, source, selector) {
            SelectedOrPick::Selected(selection) => Prepared::Cards(PreparedCards {
                cards: selection,
                verbs: prepare_card_verbs(verbs),
            }),
            SelectedOrPick::Pick { choice, others } => {
                return PreparedOrOpen::Open(open_card_pick(
                    cards, owner, source, choice, verbs, others,
                ));
            }
        },
    }))
}

pub(in crate::game) fn prepare_finish(cards: CardsRef, finished: FinishedPick) -> PreparedEffect {
    let FinishedPick { source, picked } = finished;
    let owner = cards.objects.get(source).player_id;
    PreparedEffect(match picked {
        PickedObjects::Characters(picked) => {
            Prepared::PickedCharacters(picked.into_groups().map(|(covered, verbs)| {
                PreparedCharacters {
                    characters: select_picked_characters(covered),
                    verbs: prepare_character_verbs(cards, owner, source, verbs),
                }
            }))
        }
        PickedObjects::Cards(picked) => {
            let (holder, from, groups) = picked.into_groups();
            Prepared::PickedCards(groups.map(|(covered, verbs)| PreparedCards {
                cards: select_picked_cards(holder, from, covered),
                verbs: prepare_card_verbs(verbs),
            }))
        }
    })
}

pub(in crate::game) fn resolve(
    prepared: PreparedEffect,
    rng: &mut Rng,
    zones: &mut Zones,
    objects: &mut ObjectBag,
) -> Vec<Report> {
    let mut reports = Vec::new();
    match prepared.0 {
        Prepared::Players { players, verbs } => {
            for (verb, player_id) in verbs_on(players, &verbs, rng) {
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
        Prepared::Characters(prepared) => {
            reports.extend(resolve_characters(prepared, rng, objects));
        }
        Prepared::Cards(prepared) => {
            reports.extend(resolve_cards(prepared, rng, zones, objects));
        }
        Prepared::PickedCharacters(groups) => {
            for group in groups {
                reports.extend(resolve_characters(group, rng, objects));
            }
        }
        Prepared::PickedCards(groups) => {
            for group in groups {
                reports.extend(resolve_cards(group, rng, zones, objects));
            }
        }
    }
    reports
}

/// A pick of `choice` among the characters in play, counted before anything is picked.
fn open_character_pick(
    cards: CardsRef,
    owner: PlayerId,
    source: ObjectId,
    choice: Choice<CharacterFilter>,
    verbs: Vec<CharacterVerb>,
    others: Vec<CharacterVerb>,
) -> Picking {
    let Choice {
        options,
        count,
        bound,
        different: _,
    } = choice;
    Picking::Characters(CharacterPicking::new(
        cards.scan_characters_with_filter(cards.chooser(source), &options),
        effect_amount(cards, &count, owner),
        bound,
        verbs,
        others,
    ))
}

/// A pick of `choice` from the chooser's zone, counted before anything is picked.
fn open_card_pick(
    cards: CardsRef,
    owner: PlayerId,
    source: ObjectId,
    choice: Choice<CardPickOptions>,
    verbs: Vec<CardVerb>,
    others: Vec<CardVerb>,
) -> Picking {
    let Choice {
        options,
        count,
        bound,
        different: _,
    } = choice;
    Picking::Cards(CardPicking::new(
        cards.scan_pick_window(cards.chooser(source), &options),
        effect_amount(cards, &count, owner),
        bound,
        verbs,
        others,
    ))
}

fn prepare_character_verbs(
    cards: CardsRef,
    owner: PlayerId,
    source: ObjectId,
    verbs: Vec<CharacterVerb>,
) -> Vec<PreparedCharacterVerb> {
    verbs
        .into_iter()
        .map(|verb| match verb {
            CharacterVerb::Damage(amount) => PreparedCharacterVerb::Damage {
                amount: effect_amount(cards, &amount, owner),
                source,
            },
            CharacterVerb::Modify(modifier_effect) => PreparedCharacterVerb::Modify(Modifier {
                granted_by: source,
                effect: modifier_effect,
            }),
            CharacterVerb::AddFriendlyAura(modifier_effect) => {
                PreparedCharacterVerb::AddFriendlyAura(Modifier {
                    granted_by: source,
                    effect: modifier_effect,
                })
            }
        })
        .collect()
}

fn prepare_card_verbs(verbs: Vec<CardVerb>) -> Vec<PreparedCardVerb> {
    verbs
        .into_iter()
        .map(|verb| match verb {
            CardVerb::Move(to) => PreparedCardVerb::Move(to),
        })
        .collect()
}

fn resolve_characters(
    prepared: PreparedCharacters,
    rng: &mut Rng,
    objects: &mut ObjectBag,
) -> Vec<Report> {
    let PreparedCharacters { characters, verbs } = prepared;
    let mut reports = Vec::new();
    for (verb, character) in verbs_on(characters, &verbs, rng) {
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
    reports
}

fn resolve_cards(
    prepared: PreparedCards,
    rng: &mut Rng,
    zones: &mut Zones,
    objects: &ObjectBag,
) -> Vec<Report> {
    let PreparedCards {
        cards: CardSelection {
            from,
            chooser,
            cards,
        },
        verbs,
    } = prepared;
    let mut reports = Vec::new();
    for (verb, card) in verbs_on(cards, &verbs, rng) {
        match verb {
            PreparedCardVerb::Move(to) => {
                let owner = objects.get(card).player_id;
                reports.extend(move_card(card, chooser, from, owner, *to, zones));
            }
        }
    }
    reports
}

/// Pairs each verb, in order, with every value of the selection. The selection is read once, so a
/// random one gives every verb the same value. With no verbs it isn't read, and takes nothing from
/// the stream.
fn verbs_on<'v, T: Copy, V>(
    selection: Selection<T>,
    verbs: &'v [V],
    rng: &mut Rng,
) -> Vec<(&'v V, T)> {
    if verbs.is_empty() {
        return Vec::new();
    }
    let selected = selection.resolve(rng);
    verbs
        .iter()
        .flat_map(|verb| selected.iter().map(move |&value| (verb, value)))
        .collect()
}

fn damage(target: ObjectId, amount: u8, source: ObjectId, objects: &mut ObjectBag) -> Report {
    objects.add_damage(target, amount);
    Report::Damaged {
        target,
        amount,
        source,
    }
}
