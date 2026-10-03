---
status: accepted
---

# `legal_actions` is the one definition of legality

The core's interface is `legal_actions(player)` and `apply(player, action)`. `apply` accepts an
action exactly when `legal_actions` lists it for that player, and a rejected action leaves the game
unchanged. A UI or a bot that offers only listed actions can't disagree with the core. A pure
`applied` returns the next game and leaves the original untouched, for search.

Every call names its player, and there is no `current_player()`. The game waits on whoever has a
non-empty list, so a decision that isn't the active player's needs no new interface.

## Considered options

- A `current_player()` that every call implies, as in OpenSpiel. It assumes one decider at a time.
- A legality check inside `apply`, written apart from `legal_actions`. Two definitions drift apart.
