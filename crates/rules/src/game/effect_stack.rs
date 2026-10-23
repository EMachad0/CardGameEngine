//! The effect stack and the resolving effect. The newest pending effect resolves first, and one
//! resolves at a time: the next is taken off the stack only once the resolving one finishes.

use crate::cards::object::ObjectId;
use crate::effect::Effect;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) struct EffectStack {
    pending: Vec<PendingEffect>,
    resolving: Option<ResolvingEffect>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingEffect {
    source: ObjectId,
    effects: Vec<Effect<Vec<ObjectId>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvingEffect {
    pending: PendingEffect,
    idx: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct NextEffect {
    pub(super) source: ObjectId,
    pub(super) effect: Effect<Vec<ObjectId>>,
}

impl ResolvingEffect {
    fn next_effect(&mut self) -> Option<NextEffect> {
        let effect = self.pending.effects.get(self.idx)?.clone();
        self.idx += 1;
        Some(NextEffect {
            source: self.pending.source,
            effect,
        })
    }

    fn is_done(&self) -> bool {
        self.pending.effects.len() <= self.idx
    }
}

impl EffectStack {
    pub(super) fn push(&mut self, source: ObjectId, effects: Vec<Effect<Vec<ObjectId>>>) {
        self.pending.push(PendingEffect { source, effects });
    }

    /// Takes the newest pending effect off the stack to resolve it. Panics while another one is
    /// resolving.
    pub(super) fn start_next(&mut self) {
        assert!(
            self.resolving.is_none(),
            "the resolving effect should finish before the next one starts",
        );
        self.resolving = self
            .pending
            .pop()
            .map(|pending| ResolvingEffect { pending, idx: 0 });
    }

    /// The resolving effect's next effect. Moves the resolving effect past it.
    pub(super) fn next_effect(&mut self) -> Option<NextEffect> {
        self.resolving.as_mut()?.next_effect()
    }

    /// Ends the resolving effect if it has no effects left, and returns whether it did.
    pub(super) fn finish_if_done(&mut self) -> bool {
        self.resolving
            .take_if(|resolving| resolving.is_done())
            .is_some()
    }

    pub(super) fn is_resolving(&self) -> bool {
        self.resolving.is_some()
    }

    pub(super) fn is_idle(&self) -> bool {
        self.pending.is_empty() && !self.is_resolving()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::cards::binder::sets::base::static_card_definition::SPARK;
    use crate::cards::definition::PlayerSelector;
    use crate::cards::object::ObjectBag;
    use crate::effect::EffectAmount;
    use crate::ids::PlayerId;

    fn sources<const N: usize>() -> [ObjectId; N] {
        let mut objects = ObjectBag::default();
        std::array::from_fn(|_| objects.insert(SPARK, PlayerId::new(0)))
    }

    fn draw(amount: u8) -> Effect<Vec<ObjectId>> {
        Effect::Draw {
            selector: PlayerSelector::Owner,
            amount: EffectAmount::Static(amount),
        }
    }

    fn next(source: ObjectId, effect: Effect<Vec<ObjectId>>) -> Option<NextEffect> {
        Some(NextEffect { source, effect })
    }

    #[test]
    fn the_newest_pending_effect_resolves_first() {
        let [older, newer] = sources();
        let mut stack = EffectStack::default();
        stack.push(older, vec![draw(1)]);
        stack.push(newer, vec![draw(2)]);

        stack.start_next();

        assert_eq!(stack.next_effect(), next(newer, draw(2)));
    }

    #[test]
    fn a_resolving_effect_hands_out_its_effects_in_order_with_its_source() {
        let [source] = sources();
        let mut stack = EffectStack::default();
        stack.push(source, vec![draw(1), draw(2)]);
        stack.start_next();

        let steps = [
            stack.next_effect(),
            stack.next_effect(),
            stack.next_effect(),
        ];

        assert_eq!(steps, [next(source, draw(1)), next(source, draw(2)), None]);
    }

    #[test]
    fn nothing_resolves_before_a_pending_effect_is_started() {
        let [source] = sources();
        let mut stack = EffectStack::default();
        stack.push(source, vec![draw(1)]);

        assert_eq!(stack.next_effect(), None);
    }

    #[test]
    #[should_panic(expected = "the resolving effect should finish before the next one starts")]
    fn no_effect_starts_while_another_resolves() {
        let [older, newer] = sources();
        let mut stack = EffectStack::default();
        stack.push(older, vec![draw(1)]);
        stack.start_next();
        stack.push(newer, vec![draw(2)]);

        stack.start_next();
    }

    #[test]
    fn the_next_pending_effect_starts_once_the_resolving_one_finishes() {
        let [older, newer] = sources();
        let mut stack = EffectStack::default();
        stack.push(older, vec![draw(1)]);
        stack.push(newer, vec![draw(2)]);
        stack.start_next();
        stack.next_effect();

        assert!(stack.finish_if_done());
        stack.start_next();

        assert_eq!(stack.next_effect(), next(older, draw(1)));
    }

    #[test]
    #[should_panic(expected = "the resolving effect should finish before the next one starts")]
    fn no_effect_starts_before_a_resolving_one_with_no_effects_left_finishes() {
        let [older, newer] = sources();
        let mut stack = EffectStack::default();
        stack.push(older, vec![draw(1)]);
        stack.push(newer, vec![draw(2)]);
        stack.start_next();
        stack.next_effect();

        stack.start_next();
    }

    #[test]
    fn a_resolving_effect_with_effects_left_does_not_finish() {
        let [source] = sources();
        let mut stack = EffectStack::default();
        stack.push(source, vec![draw(1), draw(2)]);
        stack.start_next();
        stack.next_effect();

        assert!(!stack.finish_if_done());
        assert_eq!(stack.next_effect(), next(source, draw(2)));
    }

    #[test]
    fn finishing_with_nothing_resolving_finishes_nothing() {
        let [source] = sources();
        let mut stack = EffectStack::default();
        stack.push(source, vec![draw(1)]);

        assert!(!stack.finish_if_done());
    }

    #[test]
    fn a_started_effect_is_resolving_until_it_finishes() {
        let [source] = sources();
        let mut stack = EffectStack::default();
        stack.push(source, vec![draw(1)]);
        assert!(!stack.is_resolving(), "a pending effect is not resolving");

        stack.start_next();
        stack.next_effect();
        assert!(stack.is_resolving(), "no effects left, but not finished");

        stack.finish_if_done();
        assert!(!stack.is_resolving());
    }

    #[test]
    fn a_stack_is_idle_only_with_nothing_pending_or_resolving() {
        let [source] = sources();
        let mut stack = EffectStack::default();
        assert!(stack.is_idle());

        stack.push(source, vec![draw(1)]);
        assert!(!stack.is_idle(), "an effect is pending");

        stack.start_next();
        assert!(!stack.is_idle(), "an effect is resolving");

        stack.next_effect();
        stack.finish_if_done();
        assert!(stack.is_idle());
    }
}
