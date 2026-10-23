# Code against the rules the untangling review adds to `file-structure.md`

Status: needs-triage
Category: enhancement

The review of the untangling PRs adds rules to `docs/conventions/file-structure.md`. Code on main
that breaks them is listed here, one section per rule. The untangling's tickets 11 to 13 covered
some of it, and those cases are left out.

## A command fails on input only a bug can send

The rule, at the end of "Parse, don't validate": "Where no type can rule out a wrong input, the
command that would act on it checks it. If some reachable state sends that input, the command
refuses it in its return type, so every caller has to handle it. If only a bug in a caller can
send it, the command fails on the spot. That keeps the check in one place, instead of an
assumption in every caller. Doing nothing hides the bug."

- `EffectStack::start_next` in `game/effect_stack.rs` does nothing while an effect is resolving.
  Its one caller, `drain_effect_queue` in `game/state.rs`, calls it only when
  `!is_resolving()`, so only a bug reaches that branch. It should fail there.
- `Zones::leave_board` in `zones.rs` returns `None` when the object isn't on the board. Its one
  caller, `destroy` in `game/resolve/aftermath/death.rs`, `expect`s it, since the state check
  found the object on the board. `leave_board` should fail itself and return no `Option`.

Both branches are unreachable, so observers see no change. Left to the tickets: what relies on
`apply`'s legality check (ticket 11, PR #55: `take_from_hand`, `PlayerInput::target`,
`commit_targeting`, `pick` and `Mana::pay`) and what relies on the validator (ticket 12, PR #62:
`Binder::get`, the `Targeted` selector, and `CardDef::attack` and `health`).
