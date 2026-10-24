# A choice no verb uses

Status: needs-triage
Category: enhancement
Type: task
Blocked by: 02, and card-game session 04g

## Why

ADR 0019 lets an effect have no verbs. Ticket 02 (PR #74) builds it: an effect with no verbs
changes nothing, and a `Random` one reads no selection, so it takes nothing from the stream. A
choice is different, because the player answers it whatever the verbs:

- A `Targeted` selector with no verbs still declares its target. The player targets an object
  nothing acts on, and with no candidate the card can't be played. Ticket 03 adds card targets,
  which can do the same.
- A `Picked` selector with no verbs and no `others`, from tickets 04 and 05, would open a pick
  whose answer nothing uses.

No card does either. An empty verb list alone is fine: ticket 05's "Choose a minion. Give the
other minions +1 Attack." picks with no verbs, and its `others` act. So the rule has to read
`others` too, and a verb list that can't be empty is ruled out.

## Options to weigh

- A load check that rejects a choice no verb uses.
- The choice isn't asked: targeting and the pick skip it.
- It stays allowed, like an empty filter.

## Done when

Session 04g settles the design and writes it here, with the tests, before anyone builds it.
