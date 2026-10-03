---
status: accepted
---

# Card behavior is data that the core interprets

What a card does is stored as enum values, such as `Effect`, and code in the core interprets them.
A card holds no closures or trait objects. Every reader works from the same data: resolution, the
search for legal targets, generated rules text and AI. A card built from existing pieces needs no
new code. A new kind of effect adds one match arm to each interpreter.

If a card needs behavior the vocabulary can't express, the escape hatch is a `Custom` variant
holding an id that the interpreter maps to a function.

## Considered options

- A closure on the card. It can't be inspected, compared, printed or loaded from a file, and a
  second closure written for another reader, such as targeting, drifts from the first.
- A function pointer on the card. It compares unreliably and prints as an address, so card data
  that holds one loses `PartialEq` under `-D warnings` and can't be loaded from a file.
