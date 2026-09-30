//! Printed card data: which cards exist and what they cost.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Card {
    Bolt { damage: u8 },
    WildBolt,
    Forage,
}

impl Card {
    pub(crate) fn mana_cost(&self) -> u8 {
        match self {
            Card::Bolt { damage } => *damage,
            Card::WildBolt => 1,
            Card::Forage => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn costs_match_the_spec_table() {
        assert_eq!(Card::Bolt { damage: 1 }.mana_cost(), 1);
        assert_eq!(Card::Bolt { damage: 4 }.mana_cost(), 4);
        assert_eq!(Card::WildBolt.mana_cost(), 1);
        assert_eq!(Card::Forage.mana_cost(), 1);
    }
}
