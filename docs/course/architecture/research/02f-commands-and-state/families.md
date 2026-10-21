# Commands that interleave narrow writes with whole-state reads

Researcher report, session 02f. Question: what established designs let a command that interleaves
narrow writes with whole-state reads (report, checkpoint) avoid holding the whole state, and what
each costs.

## Summary

Every design found handles "narrow writes interleaved with whole-state reads" in one of four ways:

1. **Make the borrow narrower** with view structs, free functions or splitting the struct. All of
   these are workarounds that Matsakis lists himself.
2. **Invert control** so the command yields requests and an interpreter that owns the whole state
   carries them out. This covers coroutines, sans-IO, effect handlers, Elm `Cmd` and the Decider.
3. **Check access at runtime** instead of compile time, either with per-component dynamic borrows
   (hecs) or with an exclusive whole-world handle that runs reactions immediately (Bevy
   observers).
4. **Accept the whole state.** Forge and XMage pass the whole `Game` and match triggers right
   away, and they keep last-known-information copies for "look back in time" rules.

## Findings

### 1. Matsakis

1. **Four workarounds, with their exact headings** from
   ["After NLL: Interprocedural conflicts" (2018)](https://smallcultfollowing.com/babysteps/blog/2018/11/01/after-nll-interprocedural-conflicts/):
   - **"Inlining as a possible fix."** Cost: breaks DRY, because two copies of the logic can
     drift apart.
   - **"Factoring as a possible fix."** Split out a sub-struct such as `EventSignal`. Cost: more
     verbose, more opaque, and it "doesn't always work" when the field sets overlap and don't fall
     into clean groups.
   - **"Free variables as a general, but extreme solution."** Free functions that take one
     `&`/`&mut` per field. Cost: "a significant ergonomic regression", and you have to track the
     transitive set of fields by hand. Upside: you can see exactly what is accessed.
   - **"View structs as a general, but extreme solution."** Structs whose fields are references.
     Cost: "fairly heavy weight", and you may need a separate view per callee. Upside: you can have
     several views at once, which suits double buffering.
   - He also names the cost of any language fix: exposing which fields a method uses becomes a
     semver commitment, plus annotation burden and extra borrow-checker complexity.
2. **The view types series:**
   - [2021](https://smallcultfollowing.com/babysteps/blog/2021/11/05/view-types/): "not an RFC or
     a proposal."
   - [Feb 2025, "View types redux and abstract fields"](https://smallcultfollowing.com/babysteps/blog/2025/02/25/view-types-redux/):
     syntax `&mut {f1, f2} self`, plus abstract fields so private field names stay hidden.
   - [21 Mar 2026, "Maximally minimal view types"](https://smallcultfollowing.com/babysteps/blog/2026/03/21/view-types-max-min/):
     syntax `&mut self {statistics}`, checked purely at compile time (one loan per listed field).
     It respects privacy, so in practice it targets private helper methods. Inference and abstract
     fields are post-MVP.
   - Status: a blog proposal, not an RFC. He says nothing blocks it "at least with a lang
     experiment" and that "we could ship it this year if we found a good contributor." A
     [follow-up post](https://smallcultfollowing.com/babysteps/blog/2026/03/22/max-min-view-types-followup/)
     floats `self.{statistics}` syntax.
   - Related academic work: [View Types in Rust, SPLASH Companion '25](https://dl.acm.org/doi/10.1145/3758316.3765481).
     Not read.

### 2. Inversion of control

3. **Coroutines.** `std::ops::Coroutine<R>` has `resume(self: Pin<&mut Self>, arg: R)`. The
   resume argument is how the interpreter would hand back answers. The trait is nightly-only
   (`coroutine_trait`, #43122) ([docs](https://doc.rust-lang.org/std/ops/trait.Coroutine.html)).
   - `gen` is a reserved keyword in edition 2024
     ([edition guide](https://doc.rust-lang.org/stable/edition-guide/rust-2024/gen-keyword.html)).
     `gen` blocks remain unstable behind `#![feature(gen_blocks)]`
     ([tracking issue #117078](https://github.com/rust-lang/rust/issues/117078)).
   - Per RFC 3513, `gen` blocks produce iterators, so they have **no resume argument**
     ([RFC](https://rust-lang.github.io/rfcs/3513-gen-blocks.html)). They could only yield
     requests, not receive replies.
   - On stable Rust you write the state machine by hand.
4. **Sans-IO.** The state machine owns the protocol state and "only expresses what should happen."
   It takes input through `handle_input`/`handle_timeout` and gives up requests through
   `poll_transmit`/`poll_timeout`. The event loop owns the sockets and timers and carries out the
   effects ([Firezone](https://www.firezone.dev/blog/sans-io);
   [definition](https://sans-io.readthedocs.io/how-to-sans-io.html)).
   - Stated costs: sequential workflows have to be modelled step by hand ("could become
     tedious"), and a hand-written event loop can hide subtle bugs such as busy-looping.
5. **Effect handlers.** An operation is `perform`ed, the nearest handler intercepts it, and the
   handler can *resume* the computation with a result
   ([Leijen, MSR-TR-2017-21](https://www.microsoft.com/en-us/research/wp-content/uploads/2017/05/asynceffects-msr-tr-2017-21.pdf);
   [Koka TR](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/08/algeff-tr-2016-v3.pdf)).
   - The handler would hold the whole state, and the command code holds only its locals.
   - Rust has no native effect handlers. Coroutines are the closest building block (the
     researcher's inference).
6. **Elm.** `update : Msg -> Model -> (Model, Cmd Msg)`. Commands are data handed to the runtime,
   which performs them and feeds results back as new `Msg`s
   ([Elm guide](https://guide.elm-lang.org/effects/)).
   - This is effectively "return requests, get a reply as a later message." Inside one `update`
     there is no immediate reply.
7. **Functional core, imperative shell.** Gary Bernhardt's
   [Boundaries (SCNA 2012)](https://www.destroyallsoftware.com/talks/boundaries) argues for simple
   values as the boundaries between components, with a pure core and a thin imperative shell
   around it.
   - Not verified: any statement from the talk about the costs of nested or multi-step workflows.

### 3. Bevy (docs.rs latest, bevy_ecs 0.20.0)

8. **`World::trigger` vs `Commands::trigger`.**
   - `World::trigger`: observers "are evaluated immediately, as are all commands they queue up."
   - `Commands::trigger`: observers run "at the next sync point in the ECS schedule, just like any
     other Command."
   - So an observer triggered from an ordinary system runs at the flush, not immediately.
   - Commands queued by observers are "currently not immediately applied" (#19569). Nested
     triggers are evaluated recursively. The order among observers of the same event is arbitrary
     (#14890). ([Observer docs](https://docs.rs/bevy/latest/bevy/ecs/observer/struct.Observer.html))
   - Unverified: that `World::trigger` needs `&mut World` (exclusive access). Inferred from the
     API, not quoted.
9. **`DeferredWorld`** is "a `World` reference that disallows structural ECS changes." It can
   mutate components and resources, trigger observers and queue commands, but it cannot spawn
   entities or add/remove components. Hooks receive it
   ([docs](https://docs.rs/bevy/latest/bevy/ecs/world/struct.DeferredWorld.html)).
   - So Bevy reconciles "react immediately" with safety by giving reactions a non-structural
     whole-world handle, and deferring structural edits.
10. **Hook and observer ordering, and flushing.** On add, hooks run before observers; on remove,
    observers run before hooks
    ([Observer docs](https://docs.rs/bevy/latest/bevy/ecs/observer/struct.Observer.html)).
    - Since 0.16, `EntityWorldMut` flushes the command queue right after every structural change.
      The guide notes a hook or observer may now despawn an entity that `EntityWorldMut` still
      refers to ([migration guide](https://bevy.org/learn/migration-guides/0-15-to-0-16/)).
    - Ordering edge cases are still open ([#20833](https://github.com/bevyengine/bevy/issues/20833)).
11. **Declared narrow access.** Systems declare their access through their parameter types.
    Conflicting parameters are rejected (error [B0001](https://bevy.org/learn/errors/b0001/)).
    - `ParamSet` allows up to 8 conflicting parameters, used one at a time through `p0()`…`p7()`
      ([docs](https://docs.rs/bevy/latest/bevy/ecs/system/struct.ParamSet.html)). This is the ECS
      version of a view struct that is borrowed in turns.

### 4. Runtime borrow checking

12. **hecs.** `World::query` uses "dynamic borrow checking" and panics if two simultaneously
    active queries would alias a mutable component. `query_mut` on `&mut World` is "faster because
    dynamic borrow checks can be skipped" ([hecs World](https://docs.rs/hecs/latest/hecs/struct.World.html)).
    - The trade: a conflict becomes a panic at runtime instead of a compile error, plus a small
      check cost on each borrow.
    - Unverified: specs/shred and legion using similar runtime-checked resource borrows.

### 5. Card engines

13. **XMage.** `GameImpl.fireEvent(event)` calls `state.handleEvent(event, this)`. That runs
    `watchers.watch`, then `delayed.checkTriggers`, then `triggers.checkTriggers`, immediately and
    with the whole `Game` passed in
    ([GameImpl](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/game/GameImpl.java),
    [GameState](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/game/GameState.java)).
    - Triggered abilities are copied and queued with `state.addTriggeredAbility`, so they go onto
      a pending list rather than resolving at once.
    - `addSimultaneousEvent` queues events, and `handleSimultaneousEvent` later batches them into,
      for example, a `ZoneChangeBatchEvent`.
    - LKI: `rememberLKI` stores a `copy()` of the object per zone and per zone-change counter;
      `resetLKI` clears it "if all effects are applied and stack is empty".
14. **Forge.** `TriggerHandler.runTrigger(mode, runParams, holdTrigger)` has three outcomes:
    - state triggers run immediately;
    - if the stack is frozen or `holdTrigger` is set, the trigger goes onto `waitingTriggers`;
    - everything else runs immediately.
    - `collectTriggerForWaiting` can pre-match waiting triggers so they are evaluated against the
      state at that moment.
    - For "look back in time" triggers it uses `looksBackInTime()` together with a
      `lastStateBattlefield` collection
      ([TriggerHandler](https://github.com/Card-Forge/forge/blob/master/forge-game/src/main/java/forge/game/trigger/TriggerHandler.java)).
    - In both engines, effects and triggers get the whole `Game`.
15. **The rules themselves.**
    - CR 603.10: triggers are normally checked against the state immediately after the event.
      Rule 603.10a makes leaves-the-battlefield and similar triggers an exception: they "look back
      in time" ([CR 603](https://friendswith.cards/en/rules/603)).
    - CR 608.2h: information from the game is determined once, when the effect is applied. If an
      object has left its expected public zone, the effect uses its last known information
      ([CR 608](https://magic-rulebook.vercel.app/rules/608)).
    - So the rules themselves call for "match against the state at the moment of the event" plus a
      snapshot of objects that just left. The mirrors are unofficial; check against the current
      Wizards PDF before quoting.

### 6. Decider

16. **The Decider pattern.** Chassaing defines `decide: Command -> State -> Event list` and
    `evolve: State -> Event -> State`. Several events are applied by folding `evolve` in order,
    i.e. `List.fold evolve state events`
    ([Decider post](https://thinkbeforecoding.com/post/2021/12/17/functional-event-sourcing-decider);
    [7-parameter gist](https://gist.github.com/thinkbeforecoding/026a1d90ea2f3ea86d151b1229cad932)).
    - The post does **not** cover reacting to each event against the state right after it. It
      mentions only "processes" and composition ("another story").
    - Researcher's inference: to match each event against the state right after it, the fold has
      to interleave `evolve` with a reaction step per event. That step needs the intermediate
      state, so the reaction logic sits in the interpreter, not in `decide`.
    - A further cost, also inference: `decide` produces its whole event list up front, against a
      single state. It cannot see a trigger's effect partway through unless the command is split.

## Gaps

- The exact text of Bernhardt's talk and whether he discusses nested workflows (video only).
- Whether specs/legion use runtime-checked borrows.
- The bodies of XMage's `TriggeredAbilities.checkTriggers` (not read).
- Chassaing's later posts on composing deciders and process managers.
- Whether the 2026 view types proposal has turned into a lang experiment or a project goal.
- The CR rules mirrors are unofficial.
