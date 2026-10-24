# Behaviour

Does the code do what it should on every input it accepts?

Read the review range, the code it calls and the code that calls it. Look for wrong results, panics
on input a reachable state can send, broken invariants, and behaviour the range breaks elsewhere.
`docs/conventions/file-structure.md` defines a reachable state.

You run in a throwaway worktree. Probe each behaviour the range adds or changes that no test pins
yet, as `docs/conventions/testing.md` says under "Reviewing an exercise". Run the tests near the
range too. Put the code of any test that proves a finding in its proof.

Not this lane: whether the spec asked for the change (Spec), how the code is organized (Locality,
Names and placement), test conventions (Tests), and comments (Text).
