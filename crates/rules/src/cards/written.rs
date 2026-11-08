//! Card data as other crates write it, naming each card by its `DefCode`. The binder parses it into
//! the forms the rules use.

use crate::cards::binder::def_code::DefCode;
use crate::cards::card_selector::CardSelector;
use crate::cards::filter::CharacterFilter;
use crate::cards::hero::HeroCardDef;
use crate::cards::player_selector::PlayerSelector;
use crate::cards::spell::SpellCardDef;
use crate::choice::Choice;
use crate::effect::{CardVerb, EffectAmount};
use crate::target::{Declared, TargetForm, Triggered};
use crate::trigger::TriggerPattern;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardDef {
    pub name: String,
    pub kind: CardKindDef,
    pub mana_cost: u8,
    pub modifier_effects: Vec<ModifierEffect>,
    pub play_effects: Vec<Effect<Declared>>,
    pub triggers: Vec<TriggerDef>,
}

impl Default for CardDef {
    fn default() -> Self {
        Self {
            name: Default::default(),
            kind: CardKindDef::Minion(MinionCardDef::new(1, 1)),
            mana_cost: Default::default(),
            modifier_effects: Default::default(),
            play_effects: Default::default(),
            triggers: Default::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardKindDef {
    Spell(SpellCardDef),
    Minion(MinionCardDef),
    Hero(HeroCardDef),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinionCardDef {
    pub attack: i32,
    pub health: i32,
    pub friendly_aura_effects: Vec<ModifierEffect>,
    pub hostile_aura_effects: Vec<ModifierEffect>,
}

impl MinionCardDef {
    pub fn new(attack: i32, health: i32) -> Self {
        Self {
            attack,
            health,
            friendly_aura_effects: Default::default(),
            hostile_aura_effects: Default::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerDef {
    pub pattern: TriggerPattern,
    pub effects: Vec<Effect<Triggered>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CharacterSelector<S: TargetForm> {
    All(CharacterFilter),
    Itself,
    OwnerHero,
    Random(CharacterFilter),
    Targeted(S::CharacterTarget),
    /// `others` acts on the options the player didn't pick, in the order shown, after the
    /// effect's verbs act on the characters picked.
    Picked {
        choice: Choice<CharacterFilter>,
        others: Vec<CharacterVerb>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect<S: TargetForm> {
    Players {
        selector: PlayerSelector,
        verbs: Vec<PlayerVerb>,
    },
    Characters {
        selector: CharacterSelector<S>,
        verbs: Vec<CharacterVerb>,
    },
    Cards {
        selector: CardSelector<S>,
        verbs: Vec<CardVerb>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerVerb {
    Draw(EffectAmount),
    Summon(DefCode),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CharacterVerb {
    Damage(EffectAmount),
    Modify(ModifierEffect),
    AddFriendlyAura(ModifierEffect),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModifierEffect {
    BuffAtk { amount: EffectAmount },
    BuffHealth { amount: EffectAmount },
    ReduceManaCost { amount: EffectAmount },
    GrantTrigger(TriggerDef),
}
