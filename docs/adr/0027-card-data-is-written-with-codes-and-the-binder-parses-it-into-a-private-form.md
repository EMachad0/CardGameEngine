---
status: accepted
---

# Card data is written with codes, and the binder parses it into a private form

Other crates write card data in public types that name a card by its code, a `DefCode`: a set
prefix, a name and a version, each a `&'static str` or a number, so the code is `Copy`. A deck names
its hero and its cards the same way. The binder parses written cards into private types that the
rules use everywhere inside, where a card is named by a `DefId` only the binder builds. `Game::new`
parses the decks against its own binder. A code that names no card fails the parse, which is the
load check ADR 0021 keeps, and a `DefId` can't come from another binder. Views and events report
codes, so `DefId` stays private and can change shape, such as into an index.

The public and private types are split only where they differ, from the card definition down to the
verbs that name a card. Types that name no card, such as filters and amounts, are shared until the
private side needs a different shape. Each conversion matches with no wildcard, so a new variant
fails the build until the parse handles it.

## Considered options

- One set of types generic over how a card is named, `CardDef<C = DefId>`, as GHC's Trees That Grow
  indexes one tree by phase. It is the least code, but every derive bounds the parameter, defaults
  apply only in type position, and the parsed shape stays public.
- A load check that walks card data for codes and leaves them as codes. A field the walk doesn't
  visit passes in silence.
- A code as one string, `"base.bolt.v0"`. The binder would have to split it and check its format.
- Every card data type split from the start. Most leaves hold no card and would be copies.
