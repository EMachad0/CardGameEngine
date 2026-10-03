---
status: accepted
---

# `apply` reports to an observer, and `view` decides what each player sees

`apply` takes an observer, `&mut impl Observer`, and calls it while the action resolves: `event`
for each thing that happens, in order, and `checkpoint` at set moments with a handle whose only
method returns one player's view. Events name objects by id only, never by card definition, so
`Game::view` is the one place that decides what a player may see.

`Game` keeps no event log and holds no observer. The program around the core owns the log and its
readers, and `()` is the observer that ignores everything. In a networked game only the server runs
the core, and each client gets its own views.

## Considered options

- An outbox of events stored in `Game`. Two games in the same state would differ by their pending
  events, which breaks `==` and `clone`.
- An observer kept in a field of `Game` for the length of one call. `Game` would carry a mutable
  field that means nothing between calls.
- Events that carry a card's definition. A hidden card would leak through its event.

## Consequences

Every function that can report takes the observer as a parameter. The parameter is generic, so `()`
adds no cost.
