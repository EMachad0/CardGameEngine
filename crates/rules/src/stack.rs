//! The effect stack, newest first, and the resolving effect taken off it.

use crate::{
    cards::effect::{Effect, EffectSequence},
    object::ObjectId,
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct EffectStack {
    pending: Vec<PendingEffect>,
    resolving: Option<ResolvingEffect>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingEffect {
    source: ObjectId,
    effects: EffectSequence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvingEffect {
    pending: PendingEffect,
    next: usize,
}

impl ResolvingEffect {
    fn peek(&self) -> Option<&Effect> {
        self.pending.effects.as_slice().get(self.next)
    }

    fn next_effect(&mut self) -> Option<(ObjectId, Effect)> {
        let effect = self.peek()?.clone();
        self.next += 1;
        Some((self.pending.source, effect))
    }

    fn is_done(&self) -> bool {
        self.peek().is_none()
    }
}

impl EffectStack {
    pub(crate) fn push(&mut self, source: ObjectId, effects: impl Into<EffectSequence>) {
        self.pending.push(PendingEffect {
            source,
            effects: effects.into(),
        });
    }

    /// Starts resolving the newest pending effect. Does nothing while one is resolving.
    pub(crate) fn start_next(&mut self) {
        if self.resolving.is_none() {
            self.resolving = self
                .pending
                .pop()
                .map(|pending| ResolvingEffect { pending, next: 0 });
        }
    }

    /// Returns the resolving effect's next effect and the object it comes from, and moves the
    /// place past that effect.
    pub(crate) fn next_effect(&mut self) -> Option<(ObjectId, Effect)> {
        self.resolving.as_mut()?.next_effect()
    }

    /// Ends the resolving effect when it has no effects left, and returns whether it did.
    pub(crate) fn finish_if_done(&mut self) -> bool {
        self.resolving
            .take_if(|resolving| resolving.is_done())
            .is_some()
    }

    pub(crate) fn is_resolving(&self) -> bool {
        self.resolving.is_some()
    }

    pub(crate) fn is_idle(&self) -> bool {
        self.pending.is_empty() && !self.is_resolving()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    use crate::cards::{
        amount::EffectAmount, def_ids::static_card_definition::SPARK, selector::PlayerSelector,
    };
    use crate::ids::PlayerId;
    use crate::object::ObjectBag;

    fn ids<const N: usize>() -> [ObjectId; N] {
        let mut bag = ObjectBag::default();
        std::array::from_fn(|_| bag.insert(SPARK, PlayerId::new(0)))
    }

    fn draw(amount: u8) -> Effect {
        Effect::Draw {
            selector: PlayerSelector::Owner,
            amount: EffectAmount::Static(amount),
        }
    }

    #[test]
    fn the_newest_pending_effect_resolves_first() {
        let [a, b] = ids();
        let mut stack = EffectStack::default();
        stack.push(a, vec![draw(1)]);
        stack.push(b, vec![draw(2)]);

        stack.start_next();

        assert_eq!(stack.next_effect(), Some((b, draw(2))));
    }

    #[test]
    fn a_resolving_effect_hands_out_its_effects_in_order_with_its_source() {
        let [a] = ids();
        let mut stack = EffectStack::default();
        stack.push(a, vec![draw(1), draw(2)]);
        stack.start_next();

        let steps = [
            stack.next_effect(),
            stack.next_effect(),
            stack.next_effect(),
        ];

        assert_eq!(steps, [Some((a, draw(1))), Some((a, draw(2))), None]);
    }

    #[test]
    fn nothing_resolves_before_a_pending_effect_is_started() {
        let [a] = ids();
        let mut stack = EffectStack::default();
        stack.push(a, vec![draw(1)]);

        assert_eq!(stack.next_effect(), None);
    }

    #[test]
    fn starting_while_an_effect_resolves_leaves_the_rest_pending() {
        let [a, b] = ids();
        let mut stack = EffectStack::default();
        stack.push(a, vec![draw(1), draw(2)]);
        stack.start_next();
        stack.next_effect();
        stack.push(b, vec![draw(3)]);

        stack.start_next();

        assert_eq!(
            stack.next_effect(),
            Some((a, draw(2))),
            "only one effect resolves at a time, so the newer one waits",
        );
    }

    #[test]
    fn starting_before_a_done_effect_finishes_leaves_the_rest_pending() {
        let [a, b] = ids();
        let mut stack = EffectStack::default();
        stack.push(a, vec![draw(1)]);
        stack.push(b, vec![draw(2)]);
        stack.start_next();
        stack.next_effect();

        stack.start_next();

        assert_eq!(
            stack.next_effect(),
            None,
            "a resolving effect with no effects left still blocks the next one until it finishes",
        );
    }

    #[test]
    fn the_next_pending_effect_starts_once_the_resolving_one_finishes() {
        let [a, b] = ids();
        let mut stack = EffectStack::default();
        stack.push(a, vec![draw(1)]);
        stack.push(b, vec![draw(2)]);
        stack.start_next();
        stack.next_effect();
        stack.finish_if_done();

        stack.start_next();

        assert_eq!(stack.next_effect(), Some((a, draw(1))));
    }

    #[test]
    fn finish_if_done_ends_a_resolving_effect_with_no_effects_left() {
        let [a] = ids();
        let mut stack = EffectStack::default();
        stack.push(a, vec![draw(1)]);
        stack.start_next();
        stack.next_effect();

        assert!(stack.finish_if_done());
        assert!(!stack.is_resolving());
    }

    #[test]
    fn finish_if_done_keeps_a_resolving_effect_with_effects_left() {
        let [a] = ids();
        let mut stack = EffectStack::default();
        stack.push(a, vec![draw(1), draw(2)]);
        stack.start_next();
        stack.next_effect();

        assert!(!stack.finish_if_done());
        assert_eq!(stack.next_effect(), Some((a, draw(2))));
    }

    #[test]
    fn finish_if_done_with_nothing_resolving_ends_nothing() {
        let [a] = ids();
        let mut stack = EffectStack::default();
        stack.push(a, vec![draw(1)]);

        assert!(!stack.finish_if_done());
        assert!(!stack.is_idle(), "the pending effect still waits");
    }

    #[test]
    fn a_stack_is_idle_only_with_nothing_pending_or_resolving() {
        let [a] = ids();
        let mut stack = EffectStack::default();
        assert!(stack.is_idle());

        stack.push(a, vec![draw(1)]);
        assert!(!stack.is_idle(), "an effect is pending");

        stack.start_next();
        assert!(!stack.is_idle(), "an effect is resolving");

        stack.next_effect();
        stack.finish_if_done();
        assert!(stack.is_idle());
    }
}
