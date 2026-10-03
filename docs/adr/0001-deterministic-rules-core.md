---
status: accepted
---

# The rules core is deterministic

A game is a function of its seed, its decks and the actions applied to it. A replay needs only those
three, and a game cloned for search plays out exactly like the original. So all randomness comes
from a seeded `Rng` held in `Game`, time reaches the core only as an action, and nothing depends on
the iteration order of a std `HashMap` or `HashSet`, which differs between instances. The crate's
`clippy.toml` bans those two types and the clock functions, and the workspace lints turn any use
into a clippy error.
