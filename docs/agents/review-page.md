# Review page

How the `review-page` skill builds a page for this repo.

## Structure

Judge the Structure section by `docs/conventions/file-structure.md`. Show each new rule with the
module that owns or keeps it, and whether the build holds it or only a test or a review does. Show
each new or moved module, each refactor as before and after, and each rename.

## Tooltips

- Take a term's definition from the glossary, `docs/CONTEXT.md`, and a decision's from its record
  in `docs/adr/`.
- A card's name shows the card: its name, mana cost, kind, attack and health for a minion, and its
  text. The definitions in `crates/rules/src/cards/binder/sets/` hold all of it but the text. Take
  the text from the ticket that added the card, or write it from the definition when no ticket
  gives it.
