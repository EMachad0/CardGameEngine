# A game is built from a binder of sets, and the rules crate holds no cards

Status: ready-for-agent
Category: enhancement

## Why

`set_up` builds its own binder with `Binder::built_in()`, from the base set in
`cards/binder/sets/base.rs`. A test can't bring the card that pins a behaviour. It has to add the
card to the base set, where every game loads it, as Dud's empty verb list shows. A branch no base
card reaches stays unpinned. `setup` also names `HERO` for every player.

ADR 0026: the rules crate holds no cards, a game is started with a binder, and a binder is built
from sets whose prefix it composes into each card's code. ADR 0027: card data is written in public
types that name cards by `DefCode`, and the binder parses it into private types that name them by a
`DefId` only it builds. ADR 0028: loading rejects a code that names no card, a code used twice, and
a deck whose hero names a card that is not a hero.

## What

### Codes and sets

- `DefCode { set: &'static str, name: &'static str, version: u16 }`, public and `Copy`. Its
  `Display` writes `set.name.vN`, as in `test.bolt.v0`.
- A public set type holds a prefix and its cards. Each card carries its name, its version and its
  written definition, with no prefix. The binder composes each card's code from the set's prefix,
  so no card can be written under another set's prefix.

### Written and parsed cards

- The public card data types become the written form. Every type that can reach a card reference
  names it by `DefCode`. Today the reference is `PlayerVerb::Summon`. It is reached from
  `CardDef` through `play_effects`, `triggers`, `modifier_effects` and the minion auras, and from
  `CharacterVerb::Modify` and `AddFriendlyAura` through `ModifierEffect::GrantTrigger`. Follow
  the types to find every one: `CardDef`, `CardKindDef`, `MinionCardDef`, `TriggerDef`,
  `ModifierEffect`, `Effect`, `PlayerVerb` and `CharacterVerb`, at least.
- Each of those types gets a private counterpart that names the card by `DefId`, and the rules use
  only the private types. Types that can't reach a reference stay shared: filters, selectors,
  amounts, `HeroCardDef`, `SpellCardDef`, `CardVerb` and the rest.
- Each conversion from written to private matches with no `_` and destructures with no `..`, so a
  new variant or field fails the build until the parse handles it.
- `DefId` leaves the prelude. Only the binder builds one. Its representation is the agent's call.
  Keep it `Copy`.

### The binder

- `Binder` becomes public. Its one constructor takes the sets and returns `Result`, with a public
  error type:
  - two cards with one code: `Duplicate(DefCode)`
  - a reference that names no card in any of the binder's sets: `NotFound(DefCode)`
- The constructor parses every card into its private form. `Binder::get` keeps its panic, which
  only a bug can reach.
- `ALL_DEF_ID`, `def_ids!`, `static_card_definition` and `Binder::built_in` go, with the
  `clippy.toml` entry for `built_in`.

### Starting a game

- `Deck { hero: DefCode, cards: Vec<DefCode> }`, public.
- `Game::new(seed, binder, decks: [Deck; 2])` and `Game::with_deck_order` with the same inputs.
  Both parse the decks against `binder` and return `Result`, with an error for a code that names
  no card in it and one for a hero that names a card that is not a hero. They check every deck
  before the first write.
- `setup` makes each player's hero from its deck's `hero`, and no longer names a card.
- `Game::def_id` and the views' `def_id` fields report the `DefCode`. Rename them to match.

### Where the cards go

- Every card in `base.rs` (49 at this writing, the hero included) moves to a set in `tests/spec/`,
  shared by the crate tests, with the prefix `test`. Dud and every other card come along unchanged
  apart from their codes. The constants the tests import, such as `BOLT`, become `DefCode`
  constants there.
- A crate test may build its binder from the shared set plus a set of its own, defined in the test.
- `benches/game.rs` builds its own set with only the cards its decks use.
- Unit tests in `src/` can't reach the crate tests' set. Each builds the cards it needs inline.
- `src/` holds no card definitions afterwards.

## Leave alone

- Every existing crate test passes with the same events, the same random draws and the same
  `ObjectId`s, apart from the edits the new inputs force at its call sites.
- The bench measures the same games.

## Tests

Binder:

- A card in a set with prefix `p`, name `n` and version `3` is found by `DefCode { set: "p", name:
  "n", version: 3 }`.
- Two sets holding one code fail with `Duplicate` naming it.
- Two sets with different prefixes may hold cards with the same name and version.
- A `Summon` that names no card fails with `NotFound` naming the code.
- A `Summon` inside a trigger granted by a minion's aura that names no card fails with `NotFound`.
  The parse reaches nested references.
- A `Summon` that names a card in another set of the same binder parses, and the summoned minion
  has that card's code.

Starting a game:

- A deck card that names no card in the binder fails `Game::new` with an error naming it.
- A hero that names no card fails `Game::new`.
- A hero that names a minion fails `Game::new` with the not-a-hero error, and so does one that names
  a spell.
- A deck built for one binder fails against a binder that lacks its codes.
- Two decks with different heroes give each player the hero its deck named.
- The view of a card in hand reports the code its deck named.

Test cards:

- A crate test whose binder adds a set of its own plays a card only that set defines.

## Names

Name the set type, the set's card entry, the private types, the error types and the renamed
`def_id` accessors by the rules in `docs/conventions/file-structure.md`, from the glossary (Code,
Binder, Set). List each name with a line on why it passes in the pull request body.

## Done when

- `src/` holds no card definitions, and the prelude exports neither `DefId` nor
  `static_card_definition`.
- The checks pass.
