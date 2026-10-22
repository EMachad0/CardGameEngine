# Player input holds decisions that change apart

Status: ready-for-agent
Category: enhancement
Type: task
Blocked by: 12
Branch: `player-input`, new, from `main` after ticket 12 merges. New draft PR with base `main`.

## Why

As of ticket 12 (PR #62), `game/player_input.rs` is one module around `PlayerInput`. Its data,
and the code that reads and writes each part:

- **What the game awaits from each player.** `targetings` and `pickings`, each sorted by player,
  with "No player is in both". The `Awaiting` read, its shared-borrow proofs and owned tokens, and
  the commands `start_targeting`, `target`, `cancel`, `commit_targeting`, `start_picking` and `pick`.
  `legal_actions`, `apply` and `play` read and write it. `commit_targeting` closes a targeting and
  hands it to `play`, which binds what was targeted into the card's pending effect (ADR 0017).
- **Where revealed cards wait.** A revealed card is in no zone. It sits in a picking's options
  until picked, then in `picks`, until `ResolvePick` in `game/resolve/effect.rs` moves it. `pick`
  closes a picking and stores its picks in one command.
- **How a targeting is built.** `Targeting` holds a card and the `(object, target)` pairs targeted
  so far, with `with_targeted` for the walk. `game/targeting_walk.rs`, `game/view.rs` and `play`'s
  binding read it, and `PlayerInput::target` pushes onto its private field.

Plausible changes reach these apart:

- A rule for building a targeting, such as "up to N targets" or a `Back` that undoes the last
  `Target`, changes `Targeting` and leaves what the game awaits alone.
- Where a pick's answer is kept, by source today, changes `picks` and leaves how a targeting is
  built alone.
- Several players picking at once changes what the game awaits, and leaves how a targeting is built
  alone.

Some of the data is tied: "No player is in both" ties `targetings` to `pickings`, and a revealed
card must be in a picking's options or in `picks` until `ResolvePick` moves it.

The doc's rules: "A module keeps the knowledge of its decision local, because all of its code
serves that decision. A boundary that holds many decisions that change independently doesn't. Its
code has to be sorted by decision before a change." "Decisions that change together count as one."
"Data tied by an invariant shares one owner." "Merge owners only for an invariant the domain
needs."

## Leave alone

- What each command does, the panics `apply` can't reach, and the order of each list
  `legal_actions` returns.
- The tokens' rules from ticket 11: each names its target by identity and isn't `Clone` or `Copy`,
  and each command looks its target up again.
- Stored picks are never cleared. See the spec's known bugs.

## Done when

- Each part above that changes apart from the others is its own module, and data tied by an
  invariant shares one owner.
- The checks pass, and the PR justifies every behavior change.
