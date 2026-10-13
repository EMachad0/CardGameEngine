# Pausing a computation and resuming it from plain data

Researcher report, session 06. Context: a deterministic Rust rules core whose `apply` must return
when a decision is needed mid-effect, with `Game: Clone + PartialEq`.

## Summary

Every system checked keeps "where am I" as runtime data: an enum, a trait object or an explicit
stack. None uses typestate for a stored suspended state. Only some of that data can be cloned,
compared and serialized. Ink does all three: its call stack is saved to JSON as content paths,
indices and variables. Rust's own suspension tools don't fit a `Clone + PartialEq` `Game`: futures
from `async` blocks are not `Clone`, `coroutine_clone` is unstable and incomplete, and nothing gives
coroutines `PartialEq` or serde support. The defunctionalized continuation (Reynolds, Danvy and
Nielsen, the CEK machine) fits: a data type for "the rest of the work" plus an `apply` function.

## 1. Sans-IO state machines

1. CONFIRMED (sans-io.readthedocs.io): "a sans-IO protocol implementation is one that is defined
   entirely in terms of synchronous functions returning synchronous results, and that does not
   block or wait for any form of I/O." Testability: "there is no risk of non-determinism in the
   tests." Related designs: "Functional Core and Imperative Shell". "The major abstraction used by
   most of the sans-IO protocol stacks is to translate the bytes received from the network into
   'events'." https://sans-io.readthedocs.io/how-to-sans-io.html
2. CONFIRMED, quinn-proto: `Connection` holds `state: State,` with
   `#[derive(Clone)] pub enum State { Handshake(state::Handshake), Established, Closed(state::Closed), Draining, Drained }`.
   The struct also holds `crypto: Box<dyn crypto::Session>` and `rng: StdRng`, so only the phase
   enum is `Clone`. API: "Handlers for incoming events from the network or system, named
   `handle_*`" and "Polling functions for outgoing events or actions for the caller to take, named
   `poll_*`." https://docs.rs/quinn-proto/latest/src/quinn_proto/connection/mod.rs.html,
   https://docs.rs/quinn-proto/latest/quinn_proto/struct.Connection.html
3. CONFIRMED, rustls: `pub(crate) struct ConnectionCore<Data> { pub(crate) state: Result<Box<dyn State<Data>>, Error>, ... }`,
   and `process_msg` consumes the boxed state and returns the next one. Neither `Clone` nor
   `PartialEq`. No stated rationale found. https://docs.rs/rustls/latest/src/rustls/conn.rs.html
4. CONFIRMED, str0m: "it has no internal threads or async tasks. All operations are happening from
   the calls of the public API." "Every mutation of an `Rtc` instance must be followed by a complete
   drain of `poll_output` until it returns `Output::Timeout`, before the next mutation." "str0m
   never calls `Instant::now()` itself. All time is external input."
   https://github.com/algesten/str0m. Its internal state representation is UNVERIFIED.

## 2. Typestate

5. CONFIRMED, Cliffle: the pattern has operations "only available when the object is in certain
   states", "a way of encoding these states at the type level, such that attempts to use the
   operations in the wrong state fail to compile", and "state transition operations ... that
   *change* the type-level state of objects". It "can enforce run-time *order of operations* at
   compile-time." https://cliffle.com/blog/rust-typestate/ No passage found on runtime-dependent
   transitions.
6. CONFIRMED, Hobden: "Since the size of the state is variable we end up needing to wrap this in an
   `enum` as above for it to be usable where the state machine is simply one component of a more
   complex system." On plain enums: "you can't control which transitions are actually permitted
   ... our errors would still only be found at runtime!"
   https://hoverbear.org/blog/rust-state-machine-pattern/ That a data-dependent `step` must return
   the wrapper enum is an inference from Hobden's `BottleFillingMachineWrapper::step`.

## 3. Resumable interpreters

7. CONFIRMED, Lua 5.4 manual §4.5 "Handling Yields in C": "Internally, Lua uses the C `longjmp`
   facility to yield a coroutine. Therefore, if a C function `foo` calls an API function and this
   API function yields ..., Lua cannot return to `foo` any more, because the `longjmp` removes its
   frame from the C stack. ... All those functions receive a *continuation function* (as a
   parameter named `k`) to continue execution after a yield." "Lua does not use this context value;
   it only passes this value from the original function to the continuation function."
   `typedef int (*lua_KFunction) (lua_State *L, int status, lua_KContext ctx);`
   https://www.lua.org/manual/5.4/manual.html#4.5
8. CONFIRMED, wasmi: `pub enum ResumableCall { Finished, HostTrap(ResumableCallHostTrap), OutOfFuel(ResumableCallOutOfFuel) }`.
   `ResumableCallHostTrap` is "State required to resume a `Func` invocation after a host trap."
   Source comment: "We need to keep the stack around since the user might want to resume the
   execution." The handle derives only `Debug`, is tied to an `Engine`, and is not `Clone`.
   https://docs.rs/wasmi/latest/wasmi/enum.ResumableCall.html,
   https://docs.rs/wasmi/latest/src/wasmi/engine/resumable.rs.html

## 4. Defunctionalization and the CEK machine

9. CONFIRMED via Wikipedia (secondary): defunctionalization "eliminates higher-order functions,
   replacing them by a single first-order *apply* function ... first described by John C. Reynolds
   in his 1972 paper, 'Definitional Interpreters for Higher-Order Programming Languages'." A program
   "contains only finitely many function abstractions, so that each can be assigned and replaced by
   a unique identifier. Every function application ... is then replaced by a call to the *apply*
   function." https://en.wikipedia.org/wiki/Defunctionalization
10. CONFIRMED, Danvy and Nielsen 2001 abstract: "Reynolds's defunctionalization technique ... is a
    whole-program transformation from higher-order to first-order functional programs."
    https://www.brics.dk/RS/01/23/index.html
11. CONFIRMED via Wikipedia, citing Felleisen and Friedman 1987: "A state in a CEK machine includes
    a control statement, environment and continuation ... the continuation stores another state, or
    a special halt case." The continuation is a data type (`halt`, `ar(e, E, K)`, `ap(f, E, K)`).
    https://en.wikipedia.org/wiki/CEK_Machine

## 5. Rust async and coroutines

12. CONFIRMED, unstable book: "Coroutines are an extra-unstable feature in the compiler right now
    ... A further RFC will be required to stabilize coroutines." "Note that coroutines do not
    implement traits like `Copy` or `Clone` automatically." "coroutines are currently compiled as
    state machines. Each `yield` expression will correspond to a different state that stores all
    live variables over that suspension point."
    https://doc.rust-lang.org/beta/unstable-book/language-features/coroutines.html
13. CONFIRMED, rust-lang/rust#95360 (`coroutine_clone`, open, `S-tracking-impl-incomplete`): it
    "does not apply to anonymous futures created using `async` blocks/functions (since these are
    desugared to `static` coroutines)." "Cloning a coroutine that has been polled before is akin to
    that coroutine returning from its `yield` multiple times. This is a non-trivial extension of
    coroutine semantics." https://github.com/rust-lang/rust/issues/95360
14. UNVERIFIED: no source either way on serializing or comparing futures. They are anonymous
    compiler-generated types, so derives can't name them.

## 6. Game and narrative VMs

15. CONFIRMED, Ink: "To save the state of your story within your game, call:
    `string savedJson = _inkStory.state.ToJson();` ...and then to load it again:
    `_inkStory.state.LoadJson(savedJson);`"
    https://github.com/inkle/ink/blob/master/Documentation/RunningYourInk.md. `StoryState.WriteJson`
    writes `flows`, `variablesState`, `evalStack`, `visitCounts`, `turnIndices`, `storySeed`,
    `previousRandom` and a save version. `Flow.cs` holds `CallStack callStack` and `currentChoices`
    and writes `callstack`. `CallStack.cs` writes each element as `cPath` (container path), `idx`,
    `exp`, `type` and `temp` (temporary variables), and loading resolves paths with
    `storyContext.ContentAtPath(...)`, warning "Has the story changed since this save data was
    created?" when one is missing. Frames hold a stable address into immutable compiled content plus
    local variables, never a host closure. Ink also pauses at choices by returning to the host,
    which reads `currentChoices` and calls `ChooseChoiceIndex(index)`.
    https://github.com/inkle/ink/blob/master/ink-engine-runtime/CallStack.cs

## Gaps

- Reynolds 1972 and Danvy and Nielsen: no quotes from the paper bodies.
- rustls: no stated reason for `Box<dyn State>`.
- str0m: internal state not inspected.
- Serializing futures or coroutines: inference only.
- Yarn Spinner VM and the Z-machine Quetzal save format: not researched.
