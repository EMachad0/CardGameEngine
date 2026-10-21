//! One player's mana and max mana, and the rules for paying and refilling it.

const MAX_MANA_CAP: u8 = 10;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) struct Mana {
    current: u8,
    max: u8,
}

/// A cost the mana could pay when asked. It holds no borrow, so it stays true only until the
/// mana changes.
pub(super) struct Payment {
    cost: u8,
}

impl Mana {
    pub(super) fn current(&self) -> u8 {
        self.current
    }

    pub(super) fn max(&self) -> u8 {
        self.max
    }

    /// `None` if the cost is more than the current mana.
    pub(super) fn payment(&self, cost: u8) -> Option<Payment> {
        (cost <= self.current).then_some(Payment { cost })
    }

    pub(super) fn pay(&mut self, payment: Payment) {
        assert!(
            payment.cost <= self.current,
            "a payment is spent before its mana drops below it",
        );
        self.current -= payment.cost;
    }

    pub(super) fn refill(&mut self) {
        self.max = (self.max + 1).min(MAX_MANA_CAP);
        self.current = self.max;
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn a_refill_fills_mana_to_a_max_one_higher() {
        let mut mana = Mana::default();
        mana.refill();
        mana.pay(mana.payment(1).unwrap());
        mana.refill();

        assert_eq!(
            (mana.current(), mana.max()),
            (2, 2),
            "a refill raises max mana by one and fills spent mana to it",
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
            "max mana rises to 10 and no further",
        );
    }

    #[test]
    fn paying_spends_the_cost() {
        let mut mana = Mana::default();
        for _ in 0..3 {
            mana.refill();
        }
        mana.pay(mana.payment(2).unwrap());

        assert_eq!(
            (mana.current(), mana.max()),
            (1, 3),
            "paying 2 of 3 mana leaves 1, and max mana stays 3",
        );
    }

    #[test]
    fn a_cost_equal_to_mana_can_be_paid() {
        let mut mana = Mana::default();
        mana.refill();
        mana.refill();

        assert!(mana.payment(2).is_some(), "2 mana pays a cost of 2");
    }

    #[test]
    #[should_panic(expected = "a payment is spent before its mana drops below it")]
    fn a_payment_cannot_be_spent_once_mana_drops_below_it() {
        let mut mana = Mana::default();
        mana.refill();
        let first = mana.payment(1).unwrap();
        let second = mana.payment(1).unwrap();
        mana.pay(first);

        mana.pay(second);
    }

    #[test]
    fn a_cost_above_mana_cannot_be_paid() {
        let mut mana = Mana::default();
        mana.refill();
        mana.refill();

        assert!(mana.payment(3).is_none(), "2 mana does not pay a cost of 3");
    }
}
