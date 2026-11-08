#[derive(Debug, enumset::EnumSetType)]
pub enum CardKind {
    Spell,
    Minion,
    Hero,
}

#[derive(Debug, enumset::EnumSetType)]
pub enum CharacterKind {
    Hero,
    Minion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellCardDef;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeroCardDef {
    pub health: i32,
}
