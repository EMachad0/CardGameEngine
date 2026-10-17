# Audit 1: orders between steps in `game/`

Against `file-structure.md` "Orders between steps" (line 95), the coordinator bullet (line 86) and
"Rules no owner guards" (line 126). Read with ADRs 0006, 0012, 0014, 0015 and the card-game plan.
Findings 1 to 32 of the architecture course are not repeated, only extended. No code changed.

Visibility used throughout: `Game`'s fields are private to `game`, so every child (`condition`,
`lookup`, `player`, `resolve`, `state_check`, `triggering`, `view`, `zone_move`) writes them.
Private items of `game/mod.rs` (`end_turn`, `start_turn`, `setup`) are open to the same children.
`emit`, `check_state`, `play`, `summon`, `kill`, `damage`, `draw`, `apply_effect` are
`pub(crate)`; `apply_action` is `pub(super)`; `drain_effect_queue` and
`continue_resolving_effect` are private to `resolve.rs`. `Event` is public, so `emit` can report
any variant from anywhere in the crate (finding 30). `Object.order` is a `pub` field and
`ObjectBag::update_order` is `pub(crate)` (finding 4).

## Order rules and preconditions

### O1. Play effects pushed before `Played` is emitted

- Steps: push, `zone_move.rs:66-71`; `emit(Played)`, `zone_move.rs:73-77`.
- Swap: the Played reactions go below the card's body, so the body resolves first. Case: Blast
  with a Drillmaster on board. Today the Squire enters, then Blast hits it; swapped, Blast resolves
  first and the Squire enters unhit. Every step keeps its contract: order rule (ADR 0015).
- Keeper: `play`. Rule: if the reactions to a card's `Played` are pushed, its play effects are
  already below them. A: emitting `Played`.
- Who else: any crate code can call `emit` with `Event::Played`; any `game` child can push to
  `effect_stack`. Extends finding 30 with this rule.
- Test: `triggers.rs` `the_spell_that_made_drillmaster_summon_hits_the_squire`;
  `events.rs` `drillmasters_squire_enters_in_its_own_step_between_the_play_and_the_spell`.
- Confidence: high.

### O2. A dying object's own death triggers pushed before any `Died`

- Steps: the push loop, `zone_move.rs:98-105`; the destroy and emit loop, `zone_move.rs:107-113`.
- Swap, two effects. (a) Push order: rattles land above the reactions, so a Bomber's deathrattle
  resolves before the Mourner's draw, against ADR 0014. (b) Read time: rattles read after removal.
  With the base set this changes nothing, since Rally Banner grants only `Damaged(Itself)`
  (`base.rs:428-429`). With data the validator accepts (a granted deathrattle, as
  `validity.rs:363-377` builds), an aura source dying in the same pass as the minion it grants to
  takes the grant with it: the minion's deathrattle is lost, against ADR 0012. Auras reach every
  zone (`lookup.rs:158`), so only the source's removal matters. Order rule for both.
- Keeper: `kill`. Rules: (a) if `Died` for an object of the pass is emitted, every rattle of the
  pass is pushed; (b) if any of the pass leaves the board, every rattle of the pass was read. A:
  (a) emitting `Died`, (b) `destroy` and `Board::remove`.
- Who else: (a) `emit` crate-wide (finding 30). (b) `destroy` is `pub(crate)`; `Board::remove` is
  `pub(crate)` and `Player.zones` is `pub(super)`, so every `game` child (finding 28).
- Test: (a) `triggers.rs` `the_triggers_reacting_to_a_death_resolve_before_the_dead_minions_own`,
  `resolution.rs` `a_reaction_to_a_death_resolves_before_the_deathrattle_that_pauses`. (b)
  nothing; no base card reaches it. Extends finding 28 with the case.
- Confidence: high for (a), medium for (b) (depends on G3).

The per-object destroy then emit inside the second loop is finding 2 (the Mourner issue); not
repeated.

### O3. State check before the next pending effect is popped

- Steps: `check_state`, `resolve.rs:231`; `effect_stack.pop`, `resolve.rs:233`. A second
  `check_state` runs at `resolve.rs:265` when the resolving effect finishes.
- Swap of 231 and 233 alone: nothing reachable changes, because 265 already ran the check before
  the loop returns. The only path where 231 alone precedes a pop is the first iteration after an
  action, and no action today both kills and pushes (fatigue's `FatigueDamaged` matches no
  pattern). Swap with 265 also removed: a deathrattle's kill is checked after an older waiting
  trigger is popped, so the older one resolves first. Case: Twin Shot kills two Bombers, the
  first rattle kills the third, and the second rattle resolves before the third's. Order rule
  (ADR 0012 "after each trigger", ADR 0014 consequences first).
- Keeper: today two sites in `resolve.rs`, `drain_effect_queue` and `continue_resolving_effect`.
  Rule: if a pending effect is popped, the state check ran since the last step finished. A: the
  pop.
- Who else: `effect_stack` is open to every `game` child; `resolve.rs:233` is the only pop today.
- Test: `triggers.rs` `a_deathrattles_consequences_resolve_before_triggers_already_waiting` and
  `a_minion_killed_by_one_deathrattle_is_gone_before_the_next` pin it only jointly; either site
  alone can move. Extends findings 25 and 27.
- Confidence: high.

### O4. A played minion on the board before its play effects resolve

- Steps: `summon` inside `play`, `zone_move.rs:80-83`; the drain, `resolve.rs:52`.
- Swap (resolve, then summon): Drummer's "+1 Health to friendly minions" misses Drummer. Order
  rule (ADR 0015 consequence, card-game thread "Enter effects").
- Keeper: `apply`. Rule: if a card's play effects resolve, a card with board presence is on the
  board. A: starting the drain.
- Who else: nobody; the drain and its loop are private to `resolve.rs`, and `apply` is their only
  caller (finding 26 notes this breaks if the loop moves out).
- Test: `resolution.rs` `a_minion_is_on_the_board_when_its_play_effects_resolve`.
- Confidence: high.

### O5. Cost read and paid while the card is in hand

- Steps: `mana_cost` and the payment, `resolve.rs:98-101`; `play`, `resolve.rs:102`, whose first
  step is `hand.remove` (`zone_move.rs:60-64`).
- Swap: `mana_cost` returns `None` off the hand (`lookup.rs:41-47`), so the `expect` at
  `resolve.rs:100` panics on every play. The payment step gets input its contract rules out:
  precondition (see G4). Paying after `play` but reading before would instead show unpaid mana at
  the checkpoint `zone_move.rs:79`.
- Keeper: the `Play` arm of `apply_action`. Rule: if a card leaves the hand by play, its cost was
  read and paid first. A: `play`, or `Hand::remove`.
- Who else: `play` is `pub(crate)`, called only at `resolve.rs:102`; `Hand::remove` is
  `pub(crate)` and the hand is open to every `game` child.
- Test: every play test panics on the swap; `drafts.rs`
  `playing_a_drafted_card_pays_its_cost_and_takes_it_from_the_hand` pins the payment. The
  checkpoint variant: nothing.
- Confidence: high (panic), medium (checkpoint).

### O6. Deaths of the pass reported before `GameEnded`

- Steps: `remove_dead`, `state_check.rs:11`; `decide_outcome`, `state_check.rs:12`.
- Swap: state is identical. `decide_outcome` reads hero health and `playing`, and `remove_dead`
  changes neither hero health nor which heroes are above 0. Only the stream changes: `GameEnded`
  and its checkpoint come before the pass's `Died` events. Case: Blast drops a Recruit and its
  caster's hero together. Order rule if event order counts as behavior (G1), free if not.
- Keeper: `check_state`. Rule: if `GameEnded` is reported, every death of the pass was reported.
  A: emitting `GameEnded`, and writing `outcome`.
- Who else: `emit` crate-wide (finding 30); `outcome` open to every `game` child (finding 29).
- Test: `events.rs` `a_hero_brought_to_zero_is_reported_dead_once_before_the_game_ends`,
  `heroes_brought_to_zero_together_are_each_reported_dead_before_the_draw` (both assert
  `GameEnded` last).
- Confidence: high on the facts, the class depends on G1.

### O7. Event-only orders inside one command

Same shape as O6: state unchanged, only the stream order differs. Keeper is the command; A is the
later emit, unrestrictable (finding 30).

- `TurnEnded` before `TurnStarted` before `Drew` (`resolve.rs:109-110`, `mod.rs:255-259`). Pinned
  by `events.rs` `ending_the_turn_reports_the_end_then_the_next_turn_start_and_its_draw`.
- `Played` before `BoardEntered` (`zone_move.rs:77`, `:81`). The played minion would also be in
  `characters_in_play` at its own `Played`, but no minion's `Played` pattern matches minions
  today (Drillmaster filters spells). Pinned by `events.rs`
  `a_minion_is_reported_entering_the_board_after_its_play`.
- `ResolvePick` moves the picked cards before the others (`resolve.rs:180-185`). With Forage the
  zones differ (hand, deck bottom); with one zone for both, zone order changes too. Pinned by
  `events.rs` `a_pick_reports_the_picked_card_then_buries_the_rest`.
- Confidence: high.

### O8. Setup shuffles player 0's deck before player 1's

- Steps: iterations of `mod.rs:101-105`, each drawing from the shared `rng`.
- Swap: both decks change for the same seed, so a replay diverges. Order rule (R5 determinism).
- Keeper: `setup`. A: any draw from `rng` during setup. `rng` is open to every `game` child;
  `Deck::shuffle` is `pub(crate)`.
- Test: nothing pins the player order; `setup.rs` `new_shuffles_the_decks` checks only that
  decks differ across seeds.
- Confidence: medium (an order between iterations, G6).

## Free orders

Each swap was traced and changes nothing reachable today. "Becomes" names the roadmap item that
would turn it into an order rule.

- `emit`: `history_from_event`, `obs.event`, enqueue (`triggering.rs:15`, `:20`, `:21-24`). No
  matcher reads History, and the observer sees only `&Event`. Becomes: a trigger whose match reads
  a History count that must include the current event.
- `damage`: `damage +=` before `emit(Damaged)` (`zone_move.rs:270-277`); fatigue the same
  (`zone_move.rs:42-49`). Matchers read kind and side only (`condition.rs:63-72`). Becomes: a
  matcher or a History snapshot that reads health ("if it survives"), or the replacement work in
  the "one damage function" thread. No test pins it;
  `a_minion_brought_to_zero_health_still_draws_from_rally_banner` pins matching at 0, not the
  order.
- `summon`: `board.add` before `character_entered` (`zone_move.rs:92-93`), and `update_order`
  before `emit(BoardEntered)` (`zone_move.rs:255-260`). No pattern matches `BoardEntered`.
  Becomes: the first enter trigger (card-game thread "Enter effects"). The first decides whether a
  minion hears its own entry; the second decides where its matched triggers sit in
  `characters_in_order_of_play` (`mod.rs:219-223`), since a stale draw stamp would rank it older.
- `draw`, `add_to_hand`, `bury`: zone write, `update_order`, emit (`zone_move.rs:32-38`,
  `:223-230`, `:239-246`). Hand and deck stamps are read by nothing until board entry restamps.
- `reveal` and `pick`: interaction state or `pick_targets` written before the emit
  (`zone_move.rs:172-178`, `:203-214`). No matcher reads either.
- `play`: `hand.remove` before the push and the emit (`zone_move.rs:60-77`). The `Played`
  matcher reads kind and side, not zone.
- `apply_action` `Play`: `choice_targets` written before `play` (`resolve.rs:84-95`). Read only
  during resolution (`resolve.rs:368-376`), which starts after.
- `remove_dead`: `kill` before `playing = false` (`state_check.rs:49-54`). `playing` is read only
  by the next pass and `decide_outcome`.
- `continue_resolving_effect`: `advance` before `apply_effect` (`resolve.rs:255-258`).
  `apply_effect` never reads the cursor; the borrow forces the clone (finding 21).
- `resolving_effect = None` before `check_state` (`resolve.rs:264-265`). The check doesn't read it.
- `start_turn`: mana refill before `draw` (`mod.rs:256-259`). `draw` doesn't read mana.
- `setup`: the deal before `start_turn` (`mod.rs:107-111`). Player 0 draws the same top four in
  the same hand order either way; only hand-card stamps differ, which nothing reads.

Not obviously forced, but by data through `Game`'s fields (G2): `apply_action` before the drain
(`resolve.rs:51-52`), `end_turn` before `start_turn` (`resolve.rs:109-110`, through
`turn_order`), legality before `apply_action` (`resolve.rs:46-51`, a precondition already in
02b audit 2, item 8). `objects.insert` before `summon` (`resolve.rs:218-219`) is forced by the
returned id.

Roadmap note on `EndTurn`: no drain runs between `end_turn` and `start_turn`, so reactions to
`TurnEnded` would resolve after the next turn's refill and draw. No `TurnEnded` pattern exists;
the first end-of-turn trigger makes this an order rule kept by the `EndTurn` arm.

## Checkpoints relative to events

Only the ones the test classes cleanly; placement as a whole is a later session's topic.

- `play`: checkpoint after `Played`, before `summon` (`zone_move.rs:79`). Swap moves
  `BoardEntered` into the first step. Order rule; pinned by `events.rs`
  `a_checkpoint_follows_a_card_leaving_the_hand` and the Drillmaster step test above.
- `remove_dead`: one checkpoint after each pass's deaths (`state_check.rs:56-57`). Pinned by
  `events.rs` `one_state_check_pass_reports_its_deaths_in_one_step`.
- Writes before the `zone_move.rs:79` checkpoint: mana payment (O5) and the draft taken
  (`resolve.rs:73`). Swapping either after `play` shows unpaid mana or a live draft in that step.
  Nothing pins the draft case; `views.rs` `a_played_draft_no_longer_shows` checks only the end.

## Doc gaps

- G1. Is event order behavior? ADR 0006 makes the stream an output, so the swap test calls every
  pair of emits in one command an order rule (O6, O7) even when the state is identical. The
  section doesn't say whether "observable" covers output to the shell or only state. If it does,
  most keepers in this report restrict "emit variant X", which finding 30 says the tools can't.
- G2. Data handed through stored fields. The section's "forced by data" is "the swap doesn't
  compile". `apply_action` then the drain, and `end_turn` then `start_turn`, hand data through
  `effect_stack` and `turn_order`, so the swap compiles and the test files them as order rules.
  By intent they are data dependencies. Two answers.
- G3. "Every state the program can reach": with today's cards or with any card data the validator
  accepts? O2(b) is free for the base set and an order rule for valid data. The same question
  decides every "becomes" in Free orders.
- G4. Precondition of which step? In O5 the panic comes from the caller's `expect` on an `Option`
  whose contract allows `None`. Read as the callee's contract, it is an order rule; read as the
  payment step's, a precondition. In O3, a popped but unstarted resolving effect: finding 25's
  pass rule ("no resolving effect part-way through") may or may not cover idx 0, giving
  precondition or order rule.
- G5. Two sites keeping one order. O3's swap at one site is free because the other masks it. The
  section reasons per pair of steps and doesn't say how to find an order whose knowledge sits at
  two call sites.
- G6. Orders between iterations. O8, `emit`'s push loop (`triggering.rs:21-24`), `kill`'s loops
  and `ResolvePick`'s two `for_each` are orders between instances of one step, not two steps. The
  section applies, but its wording names two steps.
- G7. Which side of an order rule is A. "X before Y" reads as "if Y ran, X ran" (restrict Y) or "if
  X runs, Y hasn't" (restrict X). For O1 and O2(a) the first gives "restrict the emit", which is
  unrestrictable, and the second gives "restrict the push to `effect_stack`", which is a field
  every `game` child writes. The implication rule doesn't say which reading an order takes, or
  whether an order always runs both ways.
