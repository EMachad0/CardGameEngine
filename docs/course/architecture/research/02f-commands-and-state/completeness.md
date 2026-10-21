# Is the taxonomy complete?

Researcher report, session 02f. It checks the six families (accept the whole state, lend, yield,
runtime check, defer, narrow the reader) against the general literature for a seventh.

## Summary

None of the sources checked shows a seventh family that fits the constraint (react synchronously,
to the state as it is at that moment). Each candidate turns out to be a variant of one of the six,
a mix of two, or a way of dropping the constraint. Two framings are missing from the list and are
worth adding. One is "forbid the reader", which Oracle uses. The other is "separate permission from
data", which qcell uses; it is arguably its own family, but by the researcher's reading it reduces
to Lend. Maier et al. propose two remedies, and they fall into two of the families: reactors are
Yield, and signals are Narrow.

## Findings

1. **Nystrom, Observer: synchronous dispatch is treated as a cost to know about, and the chapter
   text searched never mentions re-entrancy.** He writes: "the Observer pattern *is* synchronous.
   The subject invokes its observers directly, which means it doesn't resume its own work until all
   of the observers have returned." On locks: "If an observer tries to grab a lock that the subject
   has, you can deadlock the game. In a highly threaded engine, you may be better off with
   asynchronous communication using an Event Queue." He never names re-entrancy, meaning an
   observer writing to the state the subject is in the middle of changing.
   [Source](https://raw.githubusercontent.com/munificent/game-programming-patterns/master/book/observer.markdown)
   (Unverified: only keyword searches on this chapter, so a passage may be missed.
   gameprogrammingpatterns.com returned HTTP 400, so the book's source repo was used.)

2. **Nystrom, Event Queue: the "state may have changed" problem, in his words.** The heading is
   "The state of the world can change under you". The key passages: "When you receive an event, you
   have to be careful not to assume the *current* state of the world reflects how the world was
   *when the event was raised*. This means queued events tend to be more data heavy than events in
   synchronous systems. With the latter, the notification can say 'something happened' and the
   receiver can look around for the details. With a queue, those ephemeral details must be captured
   when the event is sent." On cycles: "When your messaging system is *synchronous*, you find cycles
   quickly -- they overflow the stack… A common rule… is to avoid *sending* events from within code
   that's *handling* one." He also notes: "A central event queue is a global variable." This is the
   Defer family, and the "snapshot at the moment" variant is exactly his capture-at-send.
   [Source](https://raw.githubusercontent.com/munificent/game-programming-patterns/master/book/event-queue.markdown)

3. **Nystrom, Double Buffer: the reader sees the state as of the last swap, not mid-command.** The
   chapter's intent is "Cause a series of sequential operations to appear instantaneous or
   simultaneous." It applies when "That same state may be accessed in the middle of modification"
   and "We want to prevent the code that's accessing the state from seeing the work in progress."
   In the slapstick example, "an actor will only see a slap in the frame *after* it was actually
   slapped." The cost he states: "The swap itself takes time." Classification: a variant of
   Defer/snapshot. The constraint rules it out, because readers deliberately see stale state. If
   the swap happens at every report point, the swap becomes a commit, and the scheme reduces to
   Lend or Yield (researcher's inference).
   [Source](https://raw.githubusercontent.com/munificent/game-programming-patterns/master/book/double-buffer.markdown)

4. **Maier, Rompf, Odersky (2010).** They say the observer pattern violates nine principles:
   side-effects, encapsulation, composability, separation of concerns, scalability, uniformity,
   abstraction, resource management and semantic distance. On scalability they write "there is no
   guarantee for data consistency": an observer of both a path and its bounds may draw "a frame on
   the screen that has the wrong size (a glitch)". On semantic distance: "the control flow is
   inverted." [Source](https://infoscience.epfl.ch/handle/20.500.14299/49107)
   - What they propose instead: Scala.React.
     - **Reactors**, "observers without inversion of control", are built on a
       continuation-passing transform, with `await` and `pause`. That is the Yield family.
     - **Signals** form a dependency graph that propagates "in turns… impossible to start a turn
       before the previous one has finished… to ensure the absence of glitches." That is the
       Narrow family (incremental computation).
     - [Source](https://infoscience.epfl.ch/handle/20.500.14299/80098)

5. **Fowler (2017): event-carried state transfer (ECST) is Narrow, not a separate family.** In
   ECST, recipients "update its own copy of customer data… so that it never needs to talk to the
   main customer system." The costs he names: "lots of data schlepped around and lots of copies" and
   "more complexity on the receiver." The reader keeps a stored replica, a variant of Narrow. Not to
   be confused with Nystrom's capture-at-send, which is Defer/snapshot.
   - Event notification is a plain observer. Its cost: the flow is "not explicit in any program
     text."
   - CQRS means "separate data structures for reading and writing", also a variant of Narrow (a
     stored read model).
   - Event sourcing is orthogonal: it is about where truth is stored, not who holds what. Fowler:
     "no need for event processing to be asynchronous."
   - [Source](https://martinfowler.com/articles/201701-event-driven.html)

6. **Incremental computation is the Narrow family, and its costs are memory and dependency
   tracking.**
   - **Salsa** records "what other tracked functions it depends on, and the revisions" and
     re-executes when inputs change. In Rust, setting an input takes `&mut db` and reading takes
     `&db`, so reads and writes happen at different times; this is Accept-whole-state at the
     database boundary. [Source](https://salsa-rs.github.io/salsa/reference/algorithm.html)
   - **Adapton** is demand-driven. It criticizes earlier incremental computation because "if a
     program input changes, all dependencies will be recomputed, even if an observer no longer
     requires certain outputs." [Source](http://matthewhammer.org/adapton/)
   - **Acar's line of work** names the cost directly. Chen and Acar present "a type system for
     precise dependency tracking that minimizes the time and space for storing dependency
     metadata." [Source](https://doi.org/10.1145/2628136.2628150)

7. **Transactions, STM and persistent state are not a new family.** STM provides composable atomic
   blocks that detect conflicts while the program runs and re-execute on conflict, a variant of
   Runtime check, aimed at concurrency.
   [Source](https://www.microsoft.com/en-us/research/publication/composable-memory-transactions/)
   - A "commit point" mid-command is the same thing as a report point. Publishing a persistent
     root needs either the whole root (Accept whole state) or a shared cell (Runtime check).
     Researcher's inference; no source says it.
   - **Oracle row triggers are the database version of the exact problem.** A "mutating table" is
     "a table that is being modified by a DML statement", and the restriction "prevents the trigger
     from querying or modifying the table that the triggering statement is modifying" (ORA-04091).
     Oracle's fixes are compound triggers, or a temporary table plus a statement-level trigger, both
     Defer. Missing framing: **forbid the reader**. It drops the requirement instead of solving it.
     [Source](https://docs.oracle.com/en/database/oracle/oracle-database/23/lnpls/trigger-restrictions.html)

8. **Capabilities, context object, mediator and dependency injection restate the goal; they don't
   answer it differently.**
   - **Capabilities.** The object-capability rule is that authority comes only from held
     references ("Only Connectivity Begets Connectivity"). In Rust, `&mut` already is such a
     capability, so this is the goal stated in other words, not a mechanism.
     [Source](http://erights.org/elib/capability/ode/ode-capabilities.html)
   - **Context object** is a variant of Accept whole state, or of Lend if the context is split.
   - **Mediator** is the reporter in Lend, or the driver in Yield.
   - **Dependency injection** is a variant of context object.
   - These classifications are the researcher's analysis; no source maps them.
   - **Permission tokens separated from data (qcell) are arguably new, but reduce to Lend.** The
     cell contents "can only be accessed by making borrowing calls on that owner", and "the
     borrowing-owner and dropping-owner are separated." A command could hold only the owner token
     for its own part, then lend `&token` to the reader at a report point. A variant of Lend where
     only tokens move and the data stays shared. Cost, from the docs: "restrictions are much
     stricter than they really need to be." [Source](https://docs.rs/qcell/latest/qcell/)
   - **Matsakis's view types are a language-level form of Lend or split borrows.** An unaccepted
     idea: "This is not a proposal (yet?)". The cost he notes is "premature commitment" and semver
     hazards. [Source](https://smallcultfollowing.com/babysteps/blog/2021/11/05/view-types/)

9. **Other candidates, classified:**
   - **ECS command buffers (Bevy `Commands`) are Defer.** "all queued commands are automatically
     applied in sequence when the `ApplyDeferred` system runs."
     [Source](https://docs.rs/bevy/latest/bevy/prelude/struct.Commands.html)
   - **Bevy observers are Accept whole state plus deferred structural writes.** `World::trigger`
     takes `&mut self` and runs observers "immediately". Commands that observers queue are
     "currently not immediately applied."
     [Source](https://docs.rs/bevy/latest/bevy/ecs/observer/struct.Observer.html)
     [Source](https://docs.rs/bevy/latest/bevy/ecs/world/struct.World.html) Hooks get a
     `DeferredWorld` that "disallows structural ECS changes."
     [Source](https://docs.rs/bevy/latest/bevy/ecs/world/struct.DeferredWorld.html)
   - **Actor model is Yield with partitioned owners.** Nystrom: state machines with mailboxes are
     "the actor model." A reader that needs the busy actor's part either waits for its next message
     (Yield) or receives a copy (Defer/snapshot). Researcher's inference.
     [Source](https://raw.githubusercontent.com/munificent/game-programming-patterns/master/book/event-queue.markdown)
   - **Data-oriented design is Accept whole state.** West's "giant nested tree of entirely public
     structs", where Rust lets you "split borrows for different fields of a public struct."
     [Source](https://kyren.github.io/2018/09/14/rustconf-talk.html)

## Gaps

- Forge and XMage were not checked here. Their attribution to Accept-whole-state comes from
  `families.md`.
- Nystrom on re-entrancy: unverified that the Observer chapter says nothing about it (keyword
  searches only). Read its "Destroying subjects and observers" section in full.
- Costs of the Maier/Odersky 2012 propagation turns (runtime overhead, figures) not extracted.
- Effect handlers and sans-IO have no sources here; see `families.md`.
