# Reactions in the middle of a mutation, in Rust codebases

Researcher report, session 02f. How Rust projects handle code that mutates part of a large state
and, between its writes, runs reactions that read the whole state.

## Summary

Rust sources that deal with callbacks needing shared state fall into a few groups. Some route
events as data from a root that holds `&mut` state (Xilem). Some pass one context everywhere and
send every write through a single gate (rustc `TyCtxt`, salsa). Game engines mostly defer
reactions, or pass a broad `&mut` context, or check borrows at runtime. The six families already
cover most of this. The new items are: take-out-and-put-back, callback receives `&mut Self` as a
parameter, scoped-closure accessors, multi-borrow / `get2_mut`, handles instead of references, and
persistent-structure snapshots. Salsa narrows *recomputation* of derived reads. It does not narrow
*reaction*, because derived queries cannot write. That second point is the researcher's inference,
not something salsa states.

## Findings

### 1. Xilem / Druid (Raph Levien)

1. **Why callbacks are hard.** The "standard" approach is a callback that captures a reference to
   the state it mutates. That "requires *shared mutable* access to that state, which is clunky at
   best in Rust (it requires interior mutability)". Rust also has no getter/setter syntax, so
   firing a notification needs an explicit call.
   [Levien 2022](https://raphlinus.github.io/rust/gui/2022/05/07/ui-architecture.html)
2. **Druid's costs.** App data must satisfy a `Data` bound, which means clone plus equality, and
   "interior mutability is effectively forbidden". Lenses are "confusing".
3. **What Xilem does.**
   - An event is tagged with its widget's **id path**, for example `[1, 3]`, and "sent to the app
     logic for dispatching".
   - Dispatch starts at the root, and each `event` call "contain[s] a mutable borrow of the app
     state".
   - **Adapt** nodes hand a child "a different mutable state reference than the parent", passed
     through a "thunk" callback.
   - The widget tree never holds app state. The event is data, and the root holder lends `&mut`
     downward.
   - Mapped to the families: lend from the top combined with event-as-data, not reaction in the
     middle of a mutation.
4. **Stated costs.**
   - Elm-style explicit message types are "verbose" and factor into components poorly.
   - An Adapt node over `Arc<Parent>` calls `make_mut`, which copies the parent struct on each
     event into the child.
   - Equality checks on large aggregates are "a valid concern".
   - The current ARCHITECTURE.md says view trees are "always eagerly evaluated", which "can create
     performance issues", with `Memoize` as the fix. That section is itself marked "TODO, needs a
     rewrite".
   - [Levien](https://raphlinus.github.io/rust/gui/2022/05/07/ui-architecture.html),
     [xilem ARCHITECTURE.md](https://github.com/linebender/xilem/blob/main/xilem/ARCHITECTURE.md)

### 2. rustc `TyCtxt` and salsa

5. **`TyCtxt` is passed everywhere on purpose.**
   - It is `struct TyCtxt<'tcx> { gcx: &'tcx GlobalCtxt }`. It is `Copy`, with `Deref` and no
     `DerefMut`.
   - "In practice `TyCtxt` is passed around everywhere, and all operations are done via
     `TyCtxt`."
   - Writes are restricted. `TyCtxtFeed` is a narrow escape hatch ("Do not add new ways to create
     `TyCtxtFeed` without consulting with T-compiler"). Some results are `Steal` values that a
     later pass takes.
   - [TyCtxt docs](https://doc.rust-lang.org/stable/nightly-rustc/rustc_middle/ty/context/struct.TyCtxt.html)
6. **Salsa's write gate.**
   - Tracked functions take `&db`: "it is not possible to modify inputs during a tracked
     function!"
   - Setters take `&mut db` because they may invalidate derived data, and "mutation of inputs
     always happens outside", in a master loop.
   - [Salsa overview](https://salsa-rs.github.io/salsa/overview.html)
7. **How salsa narrows the reader.** It records which inputs each tracked function reads and
   reuses the memoized result when those inputs have not changed (the red-green algorithm).
   - Inference (unverified): this narrows recomputation automatically, without hand-written
     indices. But it is pull-based and read-only, so it cannot run a reaction that writes history
     or the effect stack. At most it could memoize "which triggers match" or the per-player views.
8. **Salsa costs (rust-analyzer).**
   - Applying a change "bumps this counter and waits until all other threads using salsa finish".
     Stale computations panic with `Cancelled`, so "rust-analyzer requires unwinding".
   - Non-deterministic inputs (proc macros) "conflict [with] how salsa works".
   - Getting good reuse needed hand-designed firewalls such as `ItemTree`, kept "stable over
     modifications to function bodies".
   - [r-a architecture](https://github.com/rust-lang/rust-analyzer/blob/master/docs/book/src/contributing/architecture.md)
   - Memory for memoized values is a real cost: one PR cut about 20 MB.
     [PR #16643](https://github.com/rust-analyzer/rust-analyzer/pull/16643)

### 3. Rust game and card engines

9. **bracket-lib roguelike tutorial: defer through a global queue.**
   - Effects can be added from anywhere, "so passing in the `World` isn't available. However,
     *spawning* effects will need full `World` access!"
   - The queue is `lazy_static! Mutex<VecDeque<EffectSpawner>>`, drained by
     `run_effects_queue(&mut World)`. Effects can enqueue further effects (chains).
   - Stated motivation: Specs "doesn't really play well with systems that read a ton of data
     storages at once".
   - [Chapter 63](https://bfnightly.bracketproductions.com/chapter_63.html)
10. **`tcg_core` (a TCG engine crate): defer plus whole state.**
    - `emit_trigger` pushes to a `TriggerBus`. `resolve_triggers` loops until the bus is empty and
      calls `handler(self, &effect)` with `&mut GameState`, active player's effects first.
    - The Knockout path emits during mutation and resolves later, in `apply_action`.
    - Where matching happens (in `emit` or in `pop_next`) is unverified.
    - [game.rs source](https://docs.rs/tcg_core/latest/src/tcg_core/game.rs.html)
11. **phase-rs (MTG engine).**
    - Describes itself as "pure reducers… immutable state with structural sharing".
      [README](https://github.com/phase-rs/phase/)
    - The engine crate depends on `im = "15"`.
      [Cargo.toml](https://raw.githubusercontent.com/phase-rs/phase/main/crates/engine/Cargo.toml)
    - How it times trigger detection is unverified.
12. **Fyrox.**
    - Scripts receive a `ScriptContext` holding `scene: &mut Scene`, the whole-state context.
      [Fyrox book, Scripts](https://fyrox-book.github.io/scripting/script.html)
    - The borrow-checker chapter offers two fixes:
      - multi-borrow: rules checked at runtime, "~1-4%" slower.
      - message passing: "processed in the same frame", "~1-7%" slower, "Do not use… for tiny
        changes".
    - [Fyrox book, Borrow Checker](https://fyrox-book.github.io/beginning/borrow_checker.html)
13. **Veloren: per-tick deferral.**
    - Systems push into an `Emitter` (a local `VecDeque`) backed by a `Mutex` `EventBus`.
      [Emitter](https://docs.veloren.net/veloren_common/event/struct.Emitter.html)
    - Each `ServerEvent` type has a batch `handle(events, SystemData)` that declares its own
      narrowed data access.
      [ServerEvent](https://docs.veloren.net/veloren_server/events/trait.ServerEvent.html)
14. **Others.**
    - hecs `CommandBuffer` is "useful when operations cannot be applied directly due to… borrow
      checking". [hecs](https://docs.rs/hecs/latest/hecs/struct.CommandBuffer.html)
    - The `hearthstone` crate uses `Rc`, `Weak` and `RefCell` back-pointers
      (`match_: Weak<Match>`).
      [source](https://docs.rs/crate/hearthstone/latest/source/src/lib.rs)

### 4. Persistent data structures

15. **What `im` reports.**
    - Clone cost: clone only bumps the root refcount. Nodes are copied on write and hold up to 64
      values.
    - Operation cost: about log64.
    - Unique-owner in-place updates are "about an order of magnitude faster than immutable
      operations".
    - Versus std: `Vec` wins below "several hundred elements". `HashMap` is "neck and neck" with
      std, and `OrdMap` is "2-3x slower".
    - `Arc` adds overhead. `im-rc` is "20-25%" faster but not `Send`.
    - Values must be `Clone`. Changing everything after a clone "eventually" costs a full clone.
    - [im docs](https://docs.rs/im/latest/im/)
    - Inference: this makes "snapshot at the moment, react later" cheap in memory. But a reaction
      would read a snapshot while writes go elsewhere, close to the excluded defer family unless
      the reaction runs immediately against that snapshot.
    - `rpds` offers similar structures. No stated benchmark numbers found (crates.io fetch failed,
      unverified). [rpds](https://docs.rs/rpds/latest/rpds/)

### 5. Community writing that names the problem

16. **users.rust-lang.org "Pass self to callback".**
    - "For the duration of the `&mut Channel`, all access… has to chain back to the single,
      exclusive `&mut`."
    - Fixes offered: give the handler a reborrow, `FnMut(&mut Self, String)`
      (callback-receives-context), or split the struct into state and dispatcher.
    - The poster's cost: "changes the design that I had in mind".
    - [thread](https://users.rust-lang.org/t/pass-self-to-callback/99688)
17. **Kyren's RustConf 2018 keynote.**
    - Describes a public `GameState` passed `&mut` to each system.
    - Cost: "the entire game state is visible and mutable to all systems", and every system
      "depends on *all* of the types".
    - Opaque accessors "*amplify* borrowing problems". A `RefCell` re-entry becomes "a RefCell
      panic".
    - Prefers indices over `Rc<RefCell>`; self-borrowing crates are a "last resort".
    - [kyren blog](https://kyren.github.io/2018/09/14/rustconf-talk.html)
18. **LogLog Games, "Leaving Rust gamedev".**
    - Lists: indirection/events, "moving something out, then doing the thing, then moving it
      back", command buffers, `get2_mut`, generational arenas, and splitting the context.
    - Cost of events: logic is "split into two part[s]", and "3 lines of code in C# suddenly
      becomes 30 lines of Rust". Games "care about inter-connected events, specific timings".
    - Defends `RefCell` over reshaping the context.
    - [LogLog](https://loglog.games/blog/leaving-rust-gamedev/)
19. **Scoped-closure accessor.** `fn access_field<F,R>(&self, f: F) where F: FnOnce(&mut A) -> R`
    keeps the `RefMut` from escaping.
    [facteer](https://blog.facteer.com/posts/bypass_the_borrow_checker.html)
20. **Stack Overflow.**
    - `try_borrow_mut` to skip the re-entered object; the answer itself is unsure of the
      consequences.
      [SO 68050092](https://stackoverflow.com/questions/68050092/nested-event-loop-with-with-rust-avoiding-borrowing-conflicts)
    - `RefCell` is "a single flag test… the main downside is… a panic at runtime".
      [SO 43552480](https://stackoverflow.com/questions/43552480/whats-the-correct-way-to-implement-the-equivalent-of-multiple-mutable-statical)

Solutions it lists as not in the families: take-out-and-put-back; callback receives
`&mut Self`/context as a parameter; scoped-closure accessors; multi-borrow / `get2_mut` (runtime
or index-checked); generational handles instead of references; `Weak` back-pointers;
`try_borrow_mut` skip; persistent-structure snapshots; Xilem's root-dispatch with Adapt narrowing;
per-event-type declared data access (Veloren `SystemData`).

## Gaps

- Exactly when phase-rs and tcg_core match triggers.
- rpds performance numbers.
- No benchmarks of im snapshots inside a rules engine.
- Levien's post-2022 posts were not reviewed beyond ARCHITECTURE.md.
- No Rust MTG or Hearthstone engine found that documents in-mutation matching under the borrow
  checker.
