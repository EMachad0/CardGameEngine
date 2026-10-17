# Audit 1: rules no owner guards, and restricting callers

Branch `architecture-02`. For every finding: `Game`'s fields are private to `game` and read by
every child of `game`, so each raw write below is open to all of `game/`, and `emit` is
`pub(crate)`. "Visibility only" means this boundary and nothing tighter.

## Findings

1. History and the report, the reverse direction (extends finding 13).
   1. If a history entry exists, its event was reported. A: pushing a history entry.
      (The known direction, report then entry, is kept by `emit`.)
   2. Keeper: `emit`, through `history_from_event` (game/triggering.rs:14, :80).
   3. One write site, game/triggering.rs:112. None outside the keeper today, but `History.entries`
      is `pub` and `Game.history` is open to every child of `game`.
   4. Visibility only. `giant_costs_one_less_per_spell_its_holder_has_cast` (tests/spec/derived.rs:213)
      pins Played then entry. Nothing pins the reverse.
   5. Nesting. The history owner drawn in 02a has one rightful writer, `emit`. Put the history
      leaf and `emit`'s leaf under one parent and make `record` `pub(super)`, while `count` stays
      open. This changes the 02a thread's "`record` is called only by `emit`" from a convention to
      an enforced rule. Caveat: that thread snapshots stats "before the dead leave the board",
      but `emit` reports `Died` after removal (ADR 0012), and `health` returns `None` off the
      board (game/lookup.rs:68). The `Died` entry then can't be written at report time, so either
      the snapshot travels with the call or `kill` becomes a second writer and the nesting breaks.
   6. High for the rule and the tool, medium for the snapshot conflict.

2. Every pending effect from a trigger comes from a reported event (ADR 0011).
   1. If a matched trigger is on the effect stack, its event was reported and recorded.
      A: pushing a matched trigger.
   2. Keeper: `emit` (game/triggering.rs:14).
   3. `enqueue_triggers` (game/triggering.rs:27, push at :39) is `pub(super)`, so every child of
      `game` can call it. Its only caller is game/triggering.rs:23.
   4. Visibility only, `pub(super)`.
   5. Nesting, narrowest form: make `enqueue_triggers` private to `triggering.rs`. The raw
      `effect_stack.push` stays open (finding 5, #17).
   6. High.

3. Own death triggers are pushed before the death is reported, and only the state check kills
   (ADR 0014, ADR 0005). Extends the known `kill` entry.
   1. If `Died` for X is reported, X's own death triggers are already pending, and X was found
      dead by a state check pass. A: reporting `Died`; and calling `kill`.
   2. Keeper: `kill` (game/zone_move.rs:96) for the order, `remove_dead`
      (game/state_check.rs:30) for who may kill.
   3. `Event::Died` is built only at game/zone_move.rs:112. `kill` is `pub(crate)`, called only at
      game/state_check.rs:49. Own death triggers are pushed only at game/zone_move.rs:100.
   4. `Died`: nothing. `kill`: visibility only. `the_triggers_reacting_to_a_death_resolve_before_the_dead_minions_own`
      (tests/spec/triggers.rs:270) pins the order.
   5. Nesting for `kill`, `destroy` and `entomb`: a death pass leaf under a `state_check`
      directory, the three `pub(super)`. One rightful caller, no reason to live in `zone_move.rs`.
      For `Died` no listed tool fits (gap 2). A Clippy ban on `emit` with 13 `#[expect]`s still
      lets a rightful site emit the wrong variant.
   6. High for nesting `kill`, low for any `Died` restriction.

4. The dead leave before their deaths are matched, and are read before they leave (ADR 0012).
   1. Two rules. (a) If `Died` for X is matched, every object of X's pass has left the board.
      A: reporting `Died`. (b) If a dying object leaves the board, every object of its pass has
      had its own death triggers read. A: removing a dying object from the board.
   2. Keeper: `kill` (game/zone_move.rs:96). (a) is broken there, finding 2, issue
      `dying-mourner-hears-older-death.md`. (b) holds: the read loop (:98) ends before the first
      `destroy` (:110).
   3. Board removal: `Board::remove` only at game/zone_move.rs:119, inside `destroy`, which is
      `pub(crate)` with one caller (:110). `Board::remove` is `pub(crate)` and `Zones.board` is
      `pub`, so any child of `game` can remove a minion.
   4. Visibility only. `a_mourner_dying_in_the_same_pass_draws_nothing`
      (tests/spec/triggers.rs:97) covers (a) in one order only. No test covers (b): no card yet
      grants an own death trigger through an aura whose source dies in the same pass.
   5. Nesting, as in finding 3, which changes finding 2's fix: reorder `kill`'s loop and also make
      `destroy` `pub(super)` inside the death pass subtree. The raw `Board::remove` stays open
      until the location owner (02c) owns the board; then its move command takes this place.
      Dead heroes never leave (finding 3, and the card-game thread on dead heroes).
   6. High for the rules, medium for the tool, since 02c moves the board.

5. Play effects are pending before `Played` is reported (ADR 0015).
   1. If `Played` for a card is reported, its play effects are on the effect stack, and its
      `BoardEntered` comes after. A: reporting `Played`.
   2. Keeper: `play` (game/zone_move.rs:54); push at :67, report at :77, `summon` at :81.
   3. `Event::Played` is built only at game/zone_move.rs:73. `play` is `pub(crate)`, called only
      at game/resolve.rs:102, after mana is paid at :101.
   4. Nothing for `Played`. Pinned by
      `drillmasters_squire_enters_in_its_own_step_between_the_play_and_the_spell`
      (tests/spec/events.rs:566) and `a_minion_is_on_the_board_when_its_play_effects_resolve`
      (tests/spec/resolution.rs:177).
   5. For `Played`, the same gap as `Died`. For `play`, nesting with `apply_action`'s play arm
      and the payment, which finding 8 already wants in one place. The rule is planned to move:
      the card-game thread "Enter effects" decides the push order with the first enter trigger,
      and 06b adds frames. That makes a single keeper worth enforcing now.
   6. Medium.

6. The state check runs between resolving effects (ADR 0012, ADR 0015). A rule a pass restores.
   1. If a pending effect becomes the resolving effect, or `apply` returns with no pick pending,
      the state check has run since the last step. A: setting `resolving_effect` from the stack.
   2. Keeper: `drain_effect_queue` (game/resolve.rs:225), the one module that decides when the
      pass runs.
   3. `resolving_effect = Some` only at game/resolve.rs:234, `= None` at :264.
      `check_state` is `pub(crate)` and called at :231 and :265 (open thread "`check_state` is
      `pub(crate)`"). Nothing outside the keeper today.
   4. Visibility only. Pinned by `a_minion_at_0_health_stays_on_the_board_while_the_card_that_hit_it_waits`
      (tests/spec/resolution.rs:62) and `a_minion_killed_by_one_deathrattle_is_gone_before_the_next`
      (tests/spec/triggers.rs:236).
   5. Nesting: the loop and the state check under one parent, `check_state` `pub(super)`, and the
      `resolving_effect` field moved into the loop's own type. This answers the thread's "how is a
      pass's call enforced". It meets the 02b question on the effect stack owner: if that owner
      only decides which pending effect comes next, the loop stays its only caller and nests with
      it.
   6. Medium.

7. Nothing is reported after the outcome.
   1. If `GameEnded` is reported, the outcome is set and no later event is reported.
      A: reporting `GameEnded`; reporting any event once the outcome is set.
   2. Keeper: `decide_outcome` (game/state_check.rs:64) for the first, the loop's guard
      (game/resolve.rs:226) and `legal_actions` (game/mod.rs:116) for the second.
   3. `Event::GameEnded` only at game/state_check.rs:82, `outcome` written only at :80. The second
      holds since only the state check sets the outcome, between resolving effects.
   4. Nothing for the event. Pinned by `only_the_apply_that_ends_the_game_reports_its_outcome`
      (tests/spec/events.rs:738).
   5. The `GameEnded` gap as in finding 3. The second rule rests on finding 6's nesting: as long
      as only the loop runs the state check, no step runs after the outcome. No extra tool.
   6. Medium.

8. Order with nothing depending on it: `Revealed` after `PendingPick` (game/zone_move.rs:173,
   :178), `Picked` after `pick_targets` (:204, :214), `Drew` before its order stamp (:37, :38). No
   trigger pattern or history entry reads these, and observers read views only at checkpoints.
   No keeper, no tool. Low confidence that no rule is intended.

## Doc gaps

1. Raw writes, not commands. "So a keeper that must be the only caller of a command picks its
   tool by where the command lives." Every restricted write above is also reachable as a field
   write on `Game`, which every child of `game` reads ("a child reads its parent's private
   items"). None of the three tools restricts a field write. The doc doesn't say whether the
   keeper's rule then needs the field moved into an owner type first.
2. Restricting by argument. "The keeper must be the only code that can make the writes the rule is
   about." Reporting `Died`, `Played` or `GameEnded` goes through one shared command, `emit`.
   Nesting, a token and a Clippy ban all restrict who calls a command, not which value it gets.
   A token inside `Event::Died` would be a public field of a public enum. The doc doesn't say what
   to do when A is one variant of a shared command.
3. Order rules. "A rule has no owner when its data can't share one, such as data outside the
   program's state, or when the domain lets it be false for a while and a pass restores it at
   known points." "Read own death triggers before removal" (finding 4b) ties the board and the
   effect stack, both in `Game`, and no stored state shows afterwards whether it held. It is
   neither an invariant ("holds whenever code outside its module can see it") nor a pass rule,
   and "Give a rule an owner whenever its data can share one" doesn't say if an order counts.
4. A pass rule's A. "A write that makes A true without B breaks it, so restrict A and leave B
   open." For finding 6, A is a point in the procedure (taking the next pending effect, returning
   from `apply`), not a write to data. The doc doesn't say what to restrict when A is a moment.
5. Chained keepers. "Its module and the keeper's share a parent whose subtree serves only the
   keeper's decision." The loop, the state check and `kill` each keep a rule, and nesting them
   puts three decisions in one subtree. The doc doesn't say whether that passes.
6. Nesting or a ban. "A Clippy ban fits when neither the tree nor a signature should carry the
   rule." Both work for finding 1, and "should" has no criterion.
