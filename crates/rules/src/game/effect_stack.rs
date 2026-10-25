//! The effect stack and the resolving effect. The newest pending effect resolves first, and one
//! resolves at a time: the next is taken off the stack only once the resolving one finishes. The
//! resolving effect holds the picking its last effect opened until the picking finishes.

use crate::cards::object::ObjectId;
use crate::effect::Effect;
use crate::game::picking::Picking;
use crate::target::Bound;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) struct EffectStack {
    pending: Vec<PendingEffect>,
    resolving: Option<ResolvingEffect>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingEffect {
    source: ObjectId,
    effects: Vec<Effect<Bound>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvingEffect {
    pending: PendingEffect,
    idx: usize,
    picking: Option<Picking>,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct NextEffect {
    pub(super) source: ObjectId,
    pub(super) effect: Effect<Bound>,
}

/// A picking that holds its count, taken off its resolving effect to finish.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct CompletePicking {
    pub(super) source: ObjectId,
    pub(super) picking: Picking,
}

/// What the resolving effect does next.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Advance {
    Effect(NextEffect),
    FinishPicking(CompletePicking),
    AwaitPicking,
    /// Nothing is resolving.
    Done,
}

/// That the resolving effect's picking waits on an answer. It borrows the effect stack, so the
/// picking stays open while this lives.
#[derive(Debug, Clone, Copy)]
pub(super) struct AwaitsPicking<'a> {
    source: ObjectId,
    picking: &'a Picking,
}

impl<'a> AwaitsPicking<'a> {
    /// The object whose text opened the pick.
    pub(super) fn source(self) -> ObjectId {
        self.source
    }

    pub(super) fn picking(self) -> &'a Picking {
        self.picking
    }

    /// One per option left, in the order shown.
    pub(super) fn options(self) -> impl Iterator<Item = WasAwaitingPicking> + 'a {
        self.picking
            .options_left()
            .map(|object_id| WasAwaitingPicking { object_id })
    }
}

/// That the resolving effect's picking was waiting on an answer, and one of its options left. It
/// holds no borrow, so the picking may have closed since.
pub(super) struct WasAwaitingPicking {
    object_id: ObjectId,
}

impl WasAwaitingPicking {
    /// The option.
    pub(super) fn object_id(&self) -> ObjectId {
        self.object_id
    }
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
}

impl EffectStack {
    pub(super) fn push(&mut self, source: ObjectId, effects: Vec<Effect<Bound>>) {
        self.pending.push(PendingEffect { source, effects });
    }

    /// Takes the newest pending effect off the stack to resolve it. Panics while another one is
    /// resolving.
    pub(super) fn start_next(&mut self) {
        assert!(
            self.resolving.is_none(),
            "the resolving effect should finish before the next one starts",
        );
        self.resolving = self.pending.pop().map(|pending| ResolvingEffect {
            pending,
            idx: 0,
            picking: None,
        });
    }

    /// Moves the resolving effect on to what it does next: off its complete picking, past its next
    /// effect, or, with neither left, off the stack.
    pub(super) fn advance(&mut self) -> Advance {
        let Some(resolving) = &mut self.resolving else {
            return Advance::Done;
        };
        if let Some(picking) = resolving.picking.take_if(|picking| picking.is_complete()) {
            return Advance::FinishPicking(CompletePicking {
                source: resolving.pending.source,
                picking,
            });
        }
        if resolving.picking.is_some() {
            return Advance::AwaitPicking;
        }
        if let Some(next) = resolving.next_effect() {
            return Advance::Effect(next);
        }
        self.resolving = None;
        Advance::Done
    }

    /// Gives the resolving effect the picking its last effect opened. Panics unless an effect is
    /// resolving with no picking.
    pub(super) fn open_picking(&mut self, picking: Picking) {
        let resolving = self
            .resolving
            .as_mut()
            .expect("a picking should open only while an effect resolves");
        assert!(
            resolving.picking.is_none(),
            "a resolving effect should finish its picking before it opens another",
        );
        resolving.picking = Some(picking);
    }

    /// The resolving effect's picking, unless it holds its count or there is none.
    pub(super) fn awaited_picking(&self) -> Option<AwaitsPicking<'_>> {
        let resolving = self.resolving.as_ref()?;
        let picking = resolving.picking.as_ref()?;
        (!picking.is_complete()).then_some(AwaitsPicking {
            source: resolving.pending.source,
            picking,
        })
    }

    /// Adds the option to the resolving effect's picking. Panics unless that picking still awaits
    /// an answer with the option left.
    pub(super) fn pick(&mut self, awaiting_picking: WasAwaitingPicking) {
        self.resolving
            .as_mut()
            .and_then(|resolving| resolving.picking.as_mut())
            .expect("a picking found awaiting an answer should stay open until it finishes")
            .pick(awaiting_picking.object_id);
    }

    pub(super) fn is_idle(&self) -> bool {
        self.pending.is_empty() && self.resolving.is_none()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::cards::binder::sets::base::static_card_definition::SPARK;
    use crate::cards::definition::PlayerSelector;
    use crate::cards::object::ObjectBag;
    use crate::choice::AmountBound;
    use crate::effect::{CardDestinationZone, CardVerb, EffectAmount, PlayerVerb};
    use crate::game::picking::CardPicking;
    use crate::ids::PlayerId;
    use crate::pick::{CardPickZone, DeckWindow};

    fn sources<const N: usize>() -> [ObjectId; N] {
        let mut objects = ObjectBag::default();
        std::array::from_fn(|_| objects.insert(SPARK, PlayerId::new(0)))
    }

    fn draw(amount: u8) -> Effect<Bound> {
        Effect::Players {
            selector: PlayerSelector::Owner,
            verbs: vec![PlayerVerb::Draw(EffectAmount::Static(amount))],
        }
    }

    /// A pick of one of `options`, all pickable.
    fn picking(options: &[ObjectId]) -> Picking {
        Picking::Cards(CardPicking::new(
            CardPickZone::YourDeck(DeckWindow::Top(2)),
            options.iter().map(|&object_id| (object_id, true)),
            1,
            AmountBound::Exactly,
            vec![CardVerb::Move(CardDestinationZone::Hand)],
            Vec::new(),
        ))
    }

    fn effect(source: ObjectId, effect: Effect<Bound>) -> Advance {
        Advance::Effect(NextEffect { source, effect })
    }

    /// A stack resolving `source`'s one effect, which has opened a pick of one of `options`.
    fn picking_stack(source: ObjectId, options: &[ObjectId]) -> EffectStack {
        let mut stack = EffectStack::default();
        stack.push(source, vec![draw(1), draw(2)]);
        stack.start_next();
        stack.advance();
        stack.open_picking(picking(options));
        stack
    }

    fn options(stack: &EffectStack) -> Vec<ObjectId> {
        stack
            .awaited_picking()
            .map(|awaits| awaits.options().map(|option| option.object_id()).collect())
            .unwrap_or_default()
    }

    fn pick(stack: &mut EffectStack, object_id: ObjectId) {
        let awaiting_picking = stack
            .awaited_picking()
            .unwrap()
            .options()
            .find(|option| option.object_id() == object_id)
            .unwrap();
        stack.pick(awaiting_picking);
    }

    #[test]
    fn the_newest_pending_effect_resolves_first() {
        let [older, newer] = sources();
        let mut stack = EffectStack::default();
        stack.push(older, vec![draw(1)]);
        stack.push(newer, vec![draw(2)]);

        stack.start_next();

        assert_eq!(stack.advance(), effect(newer, draw(2)));
    }

    #[test]
    fn a_resolving_effect_hands_out_its_effects_in_order_with_its_source_then_is_done() {
        let [source] = sources();
        let mut stack = EffectStack::default();
        stack.push(source, vec![draw(1), draw(2)]);
        stack.start_next();

        let steps = [stack.advance(), stack.advance(), stack.advance()];

        assert_eq!(
            steps,
            [
                effect(source, draw(1)),
                effect(source, draw(2)),
                Advance::Done
            ]
        );
    }

    #[test]
    fn nothing_resolves_before_a_pending_effect_is_started() {
        let [source] = sources();
        let mut stack = EffectStack::default();
        stack.push(source, vec![draw(1)]);

        assert_eq!(stack.advance(), Advance::Done);
        assert!(!stack.is_idle(), "the pending effect still waits");
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
    fn the_next_pending_effect_starts_once_the_resolving_one_is_done() {
        let [older, newer] = sources();
        let mut stack = EffectStack::default();
        stack.push(older, vec![draw(1)]);
        stack.push(newer, vec![draw(2)]);
        stack.start_next();
        stack.advance();

        assert_eq!(stack.advance(), Advance::Done);
        stack.start_next();

        assert_eq!(stack.advance(), effect(older, draw(1)));
    }

    #[test]
    #[should_panic(expected = "the resolving effect should finish before the next one starts")]
    fn no_effect_starts_before_a_resolving_one_with_no_effects_left_is_done() {
        let [older, newer] = sources();
        let mut stack = EffectStack::default();
        stack.push(older, vec![draw(1)]);
        stack.push(newer, vec![draw(2)]);
        stack.start_next();
        stack.advance();

        stack.start_next();
    }

    #[test]
    fn a_stack_is_idle_only_with_nothing_pending_or_resolving() {
        let [source] = sources();
        let mut stack = EffectStack::default();
        assert!(stack.is_idle());

        stack.push(source, vec![draw(1)]);
        assert!(!stack.is_idle(), "an effect is pending");

        stack.start_next();
        stack.advance();
        assert!(!stack.is_idle(), "no effects left, but not done");

        stack.advance();
        assert!(stack.is_idle());
    }

    #[test]
    fn an_open_picking_awaits_its_options() {
        let [source, a, b] = sources();

        let stack = picking_stack(source, &[a, b]);

        assert_eq!(options(&stack), [a, b]);
        assert_eq!(stack.awaited_picking().unwrap().source(), source);
    }

    #[test]
    fn an_open_picking_holds_back_the_next_effect() {
        let [source, a] = sources();
        let mut stack = picking_stack(source, &[a]);

        assert_eq!(stack.advance(), Advance::AwaitPicking);
        assert_eq!(stack.advance(), Advance::AwaitPicking);
    }

    #[test]
    fn a_picking_opened_by_the_last_effect_keeps_its_effect_resolving() {
        let [source, a] = sources();
        let mut stack = EffectStack::default();
        stack.push(source, vec![draw(1)]);
        stack.start_next();
        stack.advance();
        stack.open_picking(picking(&[a]));

        assert_eq!(stack.advance(), Advance::AwaitPicking);
        assert!(!stack.is_idle());
    }

    #[test]
    fn a_complete_picking_finishes_before_the_next_effect() {
        let [source, a, b] = sources();
        let mut stack = picking_stack(source, &[a, b]);
        pick(&mut stack, b);
        let mut answered = picking(&[a, b]);
        answered.pick(b);

        assert_eq!(
            stack.advance(),
            Advance::FinishPicking(CompletePicking {
                source,
                picking: answered
            })
        );
        assert_eq!(stack.advance(), effect(source, draw(2)));
    }

    #[test]
    fn a_complete_picking_is_not_awaited() {
        let [source, a] = sources();
        let mut stack = picking_stack(source, &[a]);

        pick(&mut stack, a);

        assert!(stack.awaited_picking().is_none());
    }

    #[test]
    fn a_picking_with_no_options_is_not_awaited_and_finishes_next() {
        let [source] = sources();
        let mut stack = picking_stack(source, &[]);

        assert!(stack.awaited_picking().is_none());
        assert!(matches!(stack.advance(), Advance::FinishPicking(_)));
    }

    #[test]
    #[should_panic(
        expected = "a picking found awaiting an answer should stay open until it finishes"
    )]
    fn an_option_cannot_be_picked_once_its_picking_finishes() {
        let [source, a] = sources();
        let mut stack = picking_stack(source, &[a]);
        let stale = stack.awaited_picking().unwrap().options().next().unwrap();
        pick(&mut stack, a);
        stack.advance();

        stack.pick(stale);
    }

    #[test]
    #[should_panic(expected = "a picking should open only while an effect resolves")]
    fn no_picking_opens_with_nothing_resolving() {
        let [a] = sources();

        EffectStack::default().open_picking(picking(&[a]));
    }

    #[test]
    #[should_panic(
        expected = "a resolving effect should finish its picking before it opens another"
    )]
    fn a_picking_cannot_open_over_another() {
        let [source, a] = sources();
        let mut stack = picking_stack(source, &[a]);

        stack.open_picking(picking(&[a]));
    }
}
