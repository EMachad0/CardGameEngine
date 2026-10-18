//! One player's mana and max mana, and the rules for paying and refilling it.

const MAX_MANA_CAP: u8 = 10;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) struct Mana {
    current: u8,
    max: u8,
}

impl Mana {
    pub(super) fn current(&self) -> u8 {
        self.current
    }

    pub(super) fn max(&self) -> u8 {
        self.max
    }

    pub(super) fn can_pay(&self, cost: u8) -> bool {
        cost <= self.current
    }

    pub(super) fn pay(&mut self, cost: u8) {
        self.current -= cost;
    }

    pub(super) fn refill(&mut self) {
        self.max = (self.max + 1).min(MAX_MANA_CAP);
        self.current = self.max;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn a_refill_fills_mana_to_a_max_one_higher() {
        let mut mana = Mana::default();
        mana.refill();
        mana.refill();

        assert_eq!(
            (mana.current(), mana.max()),
            (2, 2),
            "(mana, max mana) after two refills",
        );
    }

    #[test]
    fn max_mana_stops_rising_at_ten() {
        let mut mana = Mana::default();
        for _ in 0..11 {
            mana.refill();
        }

        assert_eq!(
            (mana.current(), mana.max()),
            (10, 10),
            "(mana, max mana) after eleven refills",
        );
    }

    #[test]
    fn paying_spends_the_cost() {
        let mut mana = Mana::default();
        for _ in 0..3 {
            mana.refill();
        }
        mana.pay(2);

        assert_eq!(
            (mana.current(), mana.max()),
            (1, 3),
            "(mana, max mana) after paying 2 of 3",
        );
    }

    #[test]
    fn a_cost_equal_to_mana_can_be_paid() {
        let mut mana = Mana::default();
        mana.refill();
        mana.refill();

        assert!(mana.can_pay(2), "2 mana pays a cost of 2");
    }

    #[test]
    fn a_cost_above_mana_cannot_be_paid() {
        let mut mana = Mana::default();
        mana.refill();
        mana.refill();

        assert!(!mana.can_pay(3), "2 mana does not pay a cost of 3");
    }
}
