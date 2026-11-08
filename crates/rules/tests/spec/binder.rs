//! Building a binder from sets: the codes it composes and the references it parses.

use pretty_assertions::assert_eq;
use rules::prelude::{
    Binder, BinderError, CardDef, CardKindDef, CharacterFilter, CharacterKind, CharacterSelector,
    CharacterVerb, DefCode, Effect, Game, MinionCardDef, ModifierEffect, PlayerSelector,
    PlayerVerb, Set, SetCard, Side, SpellCardDef, TargetForm, TriggerDef, TriggerPattern,
    TriggerSubject,
};

use crate::support::*;
use crate::test_set::{BOLT, HERO, RECRUIT, test_set};

const MISSING: DefCode = DefCode {
    set: "test",
    name: "missing",
    version: 0,
};

fn set(prefix: &'static str, cards: Vec<SetCard>) -> Set {
    Set { prefix, cards }
}

fn minion(name: &'static str, version: u16) -> SetCard {
    SetCard {
        name,
        version,
        def: CardDef {
            kind: CardKindDef::Minion(MinionCardDef::new(1, 1)),
            mana_cost: 1,
            ..Default::default()
        },
    }
}

fn summon<S: TargetForm>(code: DefCode) -> Effect<S> {
    Effect::Players {
        selector: PlayerSelector::Owner,
        verbs: vec![PlayerVerb::Summon(code)],
    }
}

/// A spell named `name` that costs 1 and summons `code` for its caster.
fn summoning_spell(name: &'static str, code: DefCode) -> SetCard {
    SetCard {
        name,
        version: 0,
        def: CardDef {
            kind: CardKindDef::Spell(SpellCardDef),
            mana_cost: 1,
            play_effects: vec![summon(code)],
            ..Default::default()
        },
    }
}

#[test]
fn a_card_is_found_by_its_sets_prefix_its_name_and_its_version() {
    let code = DefCode {
        set: "p",
        name: "n",
        version: 3,
    };
    let binder = binder_with(set("p", vec![minion("n", 3)]));

    let game = Game::with_deck_order(0, binder, [deck(vec![code; 4]), deck(vec![BOLT; 4])])
        .expect("the deck names a card in the binder");

    let [p0, _] = players(&game);
    assert_eq!(hand_defs(&game, p0), [code; 4]);
}

#[test]
fn two_sets_holding_one_code_are_a_duplicate() {
    let result = Binder::new([
        set("p", vec![minion("n", 0)]),
        set("p", vec![minion("n", 0)]),
    ]);

    assert_eq!(
        result.map(drop),
        Err(BinderError::Duplicate(DefCode {
            set: "p",
            name: "n",
            version: 0,
        })),
    );
}

#[test]
fn sets_with_different_prefixes_may_hold_one_name_and_version() {
    let result = Binder::new([
        set("p", vec![minion("n", 0)]),
        set("q", vec![minion("n", 0)]),
    ]);

    assert_eq!(result.map(drop), Ok(()));
}

#[test]
fn a_summon_that_names_no_card_is_not_found() {
    let result = Binder::new([test_set(), set("p", vec![summoning_spell("call", MISSING)])]);

    assert_eq!(result.map(drop), Err(BinderError::NotFound(MISSING)));
}

#[test]
fn a_summon_that_names_a_spell_or_a_hero_is_not_a_minion() {
    for not_a_minion in [BOLT, HERO] {
        let result = Binder::new([
            test_set(),
            set("p", vec![summoning_spell("call", not_a_minion)]),
        ]);

        assert_eq!(
            result.map(drop),
            Err(BinderError::NotAMinion(not_a_minion)),
            "{not_a_minion}"
        );
    }
}

#[test]
fn a_summon_in_a_trigger_granted_by_an_aura_that_names_no_card_is_not_found() {
    let granted = TriggerDef {
        pattern: TriggerPattern::Died(TriggerSubject::Itself),
        effects: vec![summon(MISSING)],
    };
    let mut aura = MinionCardDef::new(1, 1);
    aura.friendly_aura_effects = vec![ModifierEffect::GrantTrigger(granted)];
    let banner = SetCard {
        name: "banner",
        version: 0,
        def: CardDef {
            kind: CardKindDef::Minion(aura),
            ..Default::default()
        },
    };

    let result = Binder::new([test_set(), set("p", vec![banner])]);

    assert_eq!(result.map(drop), Err(BinderError::NotFound(MISSING)));
}

#[test]
fn a_summon_of_a_card_in_another_set_summons_a_minion_with_that_cards_code() {
    let call = DefCode {
        set: "p",
        name: "call",
        version: 0,
    };
    let binder = binder_with(set("p", vec![summoning_spell("call", RECRUIT)]));
    let mut game = Game::with_deck_order(0, binder, [deck(vec![call; 4]), deck(vec![BOLT; 4])])
        .expect("the decks name cards in the binder");
    let [p0, _] = players(&game);

    play_def(&mut game, p0, call);

    assert_eq!(board_defs(&game, p0), [RECRUIT]);
}

#[test]
fn a_test_plays_a_card_only_its_own_set_defines() {
    let jab = DefCode {
        set: "p",
        name: "jab",
        version: 0,
    };
    let jab_card = SetCard {
        name: "jab",
        version: 0,
        def: CardDef {
            kind: CardKindDef::Spell(SpellCardDef),
            mana_cost: 1,
            play_effects: vec![Effect::Characters {
                selector: CharacterSelector::All(CharacterFilter {
                    kind: CharacterKind::Hero.into(),
                    side: Side::Enemy.into(),
                    include_itself: true,
                }),
                verbs: vec![CharacterVerb::Damage(3.into())],
            }],
            ..Default::default()
        },
    };
    let binder = binder_with(set("p", vec![jab_card]));
    let mut game = Game::with_deck_order(0, binder, [deck(vec![jab; 4]), deck(vec![BOLT; 4])])
        .expect("the decks name cards in the binder");
    let [p0, p1] = players(&game);

    play_def(&mut game, p0, jab);

    assert_eq!(game.hero_health(p1), 7, "Jab deals 3 to the enemy hero");
}
