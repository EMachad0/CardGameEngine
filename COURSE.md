# Card game systems course

Read this at the start of every session. Update it at the end of every session.

## Goal

Understand card game systems well enough to design MTG, Hearthstone, or Yu-Gi-Oh style games from scratch, plus original ones. Focus on code quality, extensibility, card data, card effects, tooling, and automated testing. The rules core is engine-agnostic Rust. Bevy is the main front end.

## How sessions work

- Teaching follows `.pi/skills/teach/SKILL.md`: motivate, establish, connect, quiz-check, one node at a time.
- One pi session per course session. Start with `/new`, `/name NN-topic`, `/md-log course/sessions/NN-topic.md` (the file must exist first), then "continue the course".
- Every session opens with a short retrieval quiz on the previous session's nodes, and adjusts the knowledge map below if something didn't stick.
- Roles in exercises. First we agree on the design in discussion. The learner writes the design-bearing code (types, traits, key function signatures and bodies). The agent writes scaffolding and tests against that API, then reviews. Tests come before the implementation.
- End at a node boundary, not mid-node. Commit once per node.
- Core track (A to G, T, K) is concept-first with one small exercise per session. Application track (H, I, J) gets one design session each. Implementation there is optional.

## Knowledge map (from the 2025-09-28 probe)

Solid:
- Card definition (never changes) vs instance with its own ID and modifiers.
- Non-commuting modifiers ("set to 1" vs "+2") need an ordering rule. Noted himself that 1 vs 3 is a design choice.
- Replacement effects ("if X would happen, do Z instead") vs triggers ("whenever X happens"). A listener can't undo an event.
- Effects referring to cards by ID, not `Rc<RefCell>` or `&mut`. Game owns everything.
- Stable card ID vs object ID that changes on zone change (MTG 400.7).
- Expression problem: enum makes new operations cheap, new variants touch every `match`. Knows `_` arms hide the checklist.
- Core returns a pending choice instead of blocking.

Partial:
- Effects as data. Got it once shown in concrete Rust (inspect, serialize, highlight targets). First guess was "enum is faster than dyn dispatch". Did not reach for inspectability unprompted.
- Pending choice: has the idea, hasn't yet seen that the half-finished effect must be stored as data, not on the call stack.

Gaps (teach into these):
- Timing, i.e. *when* things run. Put the death check in a health setter ("every change goes through the setter"). Dislodged by the "can't die this turn" expiry case, but still needs a proper node. Chose the trigger queue for decoupling reasons, not for re-entrancy and timing reasons.
- Core to shell output. Unsure whether the core should return events or push them.
- Determinism sources. Knows the core must be deterministic, didn't know std `HashMap` iteration order is randomized per process.
- Property-based testing. Sees crashes and rejected legal moves as fuzz findings. Missed invariants you assert yourself (card in two zones, replay divergence).

Rust: knows traits, generics, lifetimes, but they don't come naturally when designing. Explain *why* each trait, generic, or ownership choice is the one to make.

## Decisions so far

- Build a tiny Hearthstone-like game first.
- Final exam (K): add a Yu-Gi-Oh style chain without rewriting the core. Learner knows YGO best.
- Effects: `enum` by default (open to a code escape hatch later).
- Two IDs: stable card ID and per-zone object ID.
- Tooling wanted: card data files with validation, generated rules text, test tooling (scenario DSL, replays, fuzzer), headless CLI with a machine-readable protocol so bots and LLM agents can playtest, a visual editor, hot reload.

## Open threads

- Where does a half-finished effect live between `apply` calls? (node G)
- Return vs push for events. (node D)

## Dependency map

```mermaid
graph TD
  R1["Game = start state + sequence of decisions"]
  R2["Printed card data never changes"]
  R3["Code can inspect data, not closures"]
  R4["The core can't wait: it must return"]
  R5["Replayable only if every input is known"]

  A["Core/shell split: legal_actions + apply"]
  B["Definitions, objects, IDs"]
  C["Derived stats + state checks"]
  D["Events out: log + trigger input"]
  E["Effects as data"]
  F["Hooks: triggers, replacements, statics"]
  G["Resolution loop + pending choices"]
  H["Card files, validation, text gen, editor"]
  T["Test tooling: scenarios, replays, fuzz"]
  I["Headless CLI + agent protocol"]
  J["Bevy front end + hot reload"]
  K["Final exam: YGO-style chain"]
  GOAL["Design any card game, engine-agnostic"]

  R1-->A
  R2-->B
  A-->B
  B-->C
  A-->D
  C-->D
  R3-->E
  C-->F
  D-->F
  E-->F
  R4-->G
  E-->G
  F-->G
  E-->H
  R5-->T
  A-->T
  D-->T
  A-->I
  G-->I
  D-->J
  G-->J
  G-->K
  H-->GOAL
  T-->GOAL
  I-->GOAL
  J-->GOAL
  K-->GOAL
```

## Sessions

| # | Nodes | Topic | Status |
|---|-------|-------|--------|
| 00 | probe | Knowledge probe and plan | done |
| 01 | R1, A | Game as a state machine, `legal_actions` + `apply` | next |
| 02 | R2, B, C | Definitions, objects, IDs; derived stats and state checks | |
| 03 | D | Events out; return vs push | |
| 04 | R3, E | Effects as data | |
| 05 | F | Triggers, replacements, statics | |
| 06 | R4, G | Resolution loop, pending choices, stored half-finished effects | |
| 07 | R5, T | Scenario DSL, replays, determinism trap, invariant fuzzing | |
| 08 | K | YGO chain: spell speed, right to act, LIFO, SEGOC, missing the timing | |
| app | H | Card files, validation, text generation, editor (design session) | |
| app | I | Headless CLI + agent protocol (design session) | |
| app | J | Bevy front end + hot reload (design session) | |

## Verified facts to reuse

- Hearthstone applies enchantments in the order granted, auras after. Deaths are processed after the outermost phase ends, all at once. Source: hearthstone.wiki.gg Advanced rulebook, Set attribute page.
- MTG Arena's rules engine (GRE) is C++ plus CLIPS. Much of the card behavior code is machine-generated from card text. Source: magic.wizards.com "On Whiteboards, Naps, and Living Breakthrough".
- Forge card scripts separate abilities (`A:`), triggers (`T:`), statics (`S:`), and replacement effects (`R:`). Source: Card-Forge wiki, Card scripting API.
- Spellsource/Metastone cards are JSON `CardDesc` with `SpellDesc` trees and value providers. Source: playspellsource.com javadoc.
- YGO SEGOC order: turn player mandatory, non-turn player mandatory, turn player optional, non-turn player optional. Same category, the owner picks order. Source: Yugipedia "Simultaneous Effects", YGOrganization part 3.
- YGO spell speed: respond only with equal or higher speed. Spell Speed 1 can't respond. Source: Yugipedia "Spell Speed".
- YGO: optional "when... you can" triggers miss the timing unless their condition was the last thing to happen. "If" and mandatory triggers don't miss it. Source: Yugipedia "If... You Can VS When... You Can".
- Toolchain on this machine: rustc/cargo 1.92. Check crate versions (bevy, rand, ron, proptest, insta) when adding them. Bevy was at 0.20 RC in the index at probe time.
