# Audit: PR #55 (`legal-actions`) against the rewritten sections

General-purpose agent, session 02h. Branch `origin/legal-actions` at `cb3ddae`, checked against
`file-structure.md` "Parse, don't validate" and "Checks before writes" (General and Rust) as
committed in `f1633f2`. Read-only.

Summary: the owned tokens are the right tool for a check that lists many actions. The findings are
about what they hold (positions, not identities), how the checks at the write are phrased and
named.

## Findings

1. `HandSlot` (`zones.rs:179`) and `InHand::slot` (`zones.rs:168`). Owned token holding
   `player_id`, `idx`, `object_id`; `slot` copies the hand index out past the borrow. Names its
   target by position, against "A fact a write can change, such as where the target is, stays out
   of the reference's type. Parse it where it's used." `Hand::take` (`zones.rs:61-67`) checks
   `hand[idx] == object_id`, not the check's question ("is this card in this hand?"), and fails if
   another card left the hand first though the card is still there. The name is a noun, not a
   past-tense fact.
2. `Payment` (`mana.rs:13`) and `Mana::pay`'s assert (`mana.rs:31-34`). Holds a cost
   `legal_actions.rs:132` computed through `stats::cost` (modifiers, history), a fact a write can
   change. The assert asks only whether the stale cost still fits the mana and never recomputes the
   cost, so a changed cost would be charged at the old value without failing. Name and message
   break the naming and "expect as precondition" rules. `Mana` can't compute costs, so the check
   at the write would sit in `play` (`play.rs:79`).
3. `TargetingSlot` (`player_input.rs:64`). Holds the index into `targetings` plus the card's id;
   `targeting_idx` (`:353-360`) checks the entry at that index. Position rule again; the identity
   is the player, who has at most one targeting. Message "a targeting slot is used before what the
   game awaits changes" has no "should".
4. `PickOption` (`player_input.rs:78`). Holds `pick_idx` and `option_idx`, both positions; `pick`'s
   assert (`:315-318`) compares by position. Identity would be the player plus the object id.
5. `NothingAwaited` (`player_input.rs:92-93`) and `start_targeting`'s assert (`:242-245`). The name
   reads present tense. The assert re-asks "is nothing awaited from anyone" (`any_awaiting`), while
   the write's contract is narrower: this player is not in `pending_picks` ("No player is in both"),
   plus the sort order `push` assumes (`:246`). "Nobody else has a pending pick" only decides
   whether the input is allowed, so asserting it in the step goes against "establishes no
   precondition, so no step takes a value from it or checks it again." `NothingAwaited` is `Copy`.
6. `Playable` (`play.rs:21-25`) and `Playable::with_targeting` (`play.rs:36-45`). Bundles three
   tokens with positions (1 to 3). Pairing card and targeting by object id is a relation between
   identities, the right owner and shape. Assert message "a card is played from its own targeting"
   states a rule, not "should".
7. `LegalAction` (`legal_actions.rs:15-29`). `pub(super)` enum with open variants: any child of
   `game` can build `EndTurn`, or `StartTargeting` from a copied `NothingAwaited`, or the others by
   minting owners' tokens through `pub(super)` methods. "The code that runs the steps for that
   input takes a value only the check can build" is informal today, as the PR body says.
8. Which steps rely on the check. `end_turn` (`turn_change.rs:9`, called at `apply.rs:50`) takes
   nothing: complies. `PlayerInput::target` (`player_input.rs:249`) relies on this player having a
   targeting open, since without it there is no entry to push onto; it doesn't rely on the pair
   being a candidate, which travels as plain ids. Only its token's position is wrong (3). `cancel`
   (`:259`), `commit_targeting` (`:266`), `pick` (`:310`) and `play`'s take and pay rely on the
   check. `start_targeting` relies partly (5).
9. New asserts: `mana.rs:31`, `zones.rs:62`, `player_input.rs:242`, `:315`, `:355`, `play.rs:37`.
   All panic on the spot; all are worded as rules, not "X should ...". The four tied to tokens check
   positions instead of asking again (1, 3, 4). `ask_pick`'s panic at `:294` has the same style but
   predates the branch.
10. `Option` questions on the action path: `Game::legal_action` (`state.rs:62`), `Mana::payment`
    (`mana.rs:26`), `nothing_awaited` (`player_input.rs:205`), `awaiting`, `find_in_hand`
    (`legal_actions.rs:119`): callers don't know the answer, fine. `targetings.get(idx)` (`:354`)
    and `pending_picks.get(..)` (`:313`) feed asserts and become `entry_idx(..).expect(..)` with
    identity tokens.

## Already complies

`apply` (`apply.rs:30`) takes a plain `Action` and parses on arrival; `LegalAction` never leaves
`game`. `apply_action` (`apply.rs:41`) and `LegalAction::action` (`legal_actions.rs:34`) match
exhaustively. The stored `Targeting` names its card by `ObjectId`, and in-hand and payable are
checked again at `Play` (`legal_actions.rs:116-123`). `InHand`, `AwaitedPick`, `AwaitedTargeting`
are shared-borrow proofs named for the fact. Each owner mints its own tokens through private
fields; only `NothingAwaited` is `Clone`. Splitting `targetings` from `pending_picks` makes "cancel
can't drop a pick" a matter of shape. The old `expect`s and `unreachable!`s are gone; failures
panic. `Target`'s candidacy is not a token, and `end_turn` takes nothing.

## Where the doc is ambiguous or silent

- A token holding identity and position: the position ban is written for "a stored reference".
- How exact the question asked again must be: a position check or a superset fact.
- Whether owned step tokens are wanted at all, with the runner-level `LegalAction` formal.
- Naming tokens that carry a thing or a place, not a fact (`Payment`, the slots).
- `Copy` on an owned token.
- Whether "fail on the spot" covers checks in a constructor that guards a relation.
