---
status: accepted
---

# An effect applies a list of verbs to one selection, and a target and a pick are one choice answered at different times

An effect names its subject by type: players, characters, or cards (objects in a hand, a deck or a
graveyard). Each subject has its own selector and verbs, and an effect is one selector and a list of
verbs. The selector is read once and every verb runs, in order, on the whole selection, so one
target or one random draw feeds several verbs. The verb list may be empty.

A choice is the options and how many to name: `Choice<O> { options, count, bound, different }`. A
selector is `Targeted` or `Picked` when a player answers a choice, and the two differ only in when.
A target is answered while the card is played and bound into the pending effect when it is pushed
(ADR 0017). A pick is answered when resolution reaches its effect, and pushing copies it unchanged,
so trigger text can pick. Options that are random or hidden from the chooser are picks only: card
targets reach the chooser's hand or a graveyard, and card picks also reach the chooser's deck
through a `DeckWindow` and can offer a random sample. A card pick finds its options by window, then
kind filter, then sample. No choice reaches another player's hand or deck, while `All` and `Random`
can.

`Effect<S: TargetForm>` keeps one type parameter. The trait's associated types say what each
subject's `Targeted` selector holds in each form of text: `Declared` holds choices, `Triggered`
holds `NoTarget` until trigger text can declare targets, and `Bound` holds objects.

## Considered options

- One object subject, with a load check that each verb gets objects it acts on. The types would
  allow a buff on a spell card, and load checks are what ADR 0017 moved away from.
- A back-reference from one effect to an earlier effect's selection, or a named result. Either
  brings back a reference that loading must check.
- Separate choice types for targets and picks. The count, bound and difference rule are the same
  for both, and only the options differ, which `Choice<O>` takes as a parameter.
- One type parameter per subject's target. All of them change together at the push, so one marker
  naming them all is enough.

## Consequences

- A verb's result can't be selected: "Summon a Squire, then give it Taunt" needs named results.
- A later effect can't reuse an earlier effect's selection.
- A new field on `Choice` changes every choice written in card data.
