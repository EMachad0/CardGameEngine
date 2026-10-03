# Bevy messages, event sourcing vs command sourcing, lockstep replays

Researcher report, session 03 (node D). Agent 5763b880. Saved verbatim in substance.

## 1. Bevy 0.17: Events vs Messages split (CONFIRMED)

- Migration guide 0.16 to 0.17 (https://bevy.org/learn/migration-guides/0-16-to-0-17/):
  "'Event' as a concept (and the `Event` trait) are now used solely for 'observable events'.
  'Buffered events' are now known as 'messages' and use the `Message` trait. `EventWriter`,
  `EventReader`, and `Events<E>`, are now known as `MessageWriter`, `MessageReader`, and
  `Messages<M>`."
- "`world.trigger_targets` has been removed in favor of a single `world.trigger` API".
  Observers use `On<E>` instead of `Trigger<E>`; entity-targeted events derive `EntityEvent`.
- Release notes (https://bevy.org/news/bevy-0-17/, Sept 30, 2025): "In Bevy 0.17, `Event` is now
  exclusively the name/trait for the concept of something that is 'triggered' and 'observed'.
  `Message` is the name / trait for something that is 'buffered': it is 'written' via a
  `MessageWriter` and 'read' via a `MessageReader`." A type can derive both.
- docs.rs bevy 0.17.3 `Commands`: "Triggers the given `Event`, which will run any `Observer`s
  watching for it." Old send method: "Deprecated since 0.17.0: Use `Commands::write_message`
  instead."
- Versions (crates.io API): latest stable 0.19.1 (`max_stable_version`), newest pre-release
  0.20.0-rc.2 (2026-09-28). bevy.org/news: Bevy 0.19 written June 19, 2026; 0.18 on
  January 13, 2026. Not verified: whether 0.18/0.19 changed the Message/Event API again.

## 2. Fowler, "Event Sourcing" (CONFIRMED)

Source: https://martinfowler.com/eaaDev/EventSourcing.html (12 Dec 2005, marked draft).
- Definition: "Capture all changes to an application state as a sequence of events."
- External Updates: "One of the tricky elements to Event Sourcing is how to deal with external
  systems that don't follow this approach (and most don't)." Fix: gateways disabled on replay.
- External Queries: "The primary problem with external queries is that the data that they
  return has an effect on the results on handling an event. If I ask for an exchange rate on
  December 5th and replay that event on December 20th, I will need the exchange rate on Dec 5
  not the later one."
- Code Changes: "Events handle changes to data, what about changes to code? We can think as
  three broad kinds of code changes here: new features, defect fixes, and temporal logic."
  Bug fixes: "all you need to do is make the fix and reprocess the events." Temporal logic:
  "The domain model should be able to run events at any time with the correct rules for the
  event processing" (e.g. `chargingRules.get(aDate).process(anEvent)`).

## 3. "Command sourcing" (CONFIRMED as a term)

- Akka Classic Persistence docs (https://doc.akka.io/libraries/akka-core/current/persistence.html):
  "In order to implement the pattern known as '*command sourcing*' call `persistAsync(cmd)(...)`
  right away on all incoming messages". `PersistentActor` "can be used to implement both
  *command* as well as *event sourced* actors."
- thinkbeforecoding, "Event Sourcing vs Command Sourcing" (2013,
  https://thinkbeforecoding.com/post/2013/07/28/Event-Sourcing-vs-Command-Sourcing): argues
  Fowler's article describes command sourcing. "A Command is a request made to the system to do
  something... It can fail, it can be influenced by external state.. An event is something that
  happen and that cannot be changed." Event sourcing: `Decide: Command -> State -> Event list`,
  replay uses only `ApplyStateChange: State -> Event -> State`, so replay never reruns decisions.
- axelsvensson.com/command-sourcing: "With CS you persist the commands, the actual input to the
  application. With Event Sourcing you persist the events, which are closer to the output."
- Standard contrast: CS stores inputs and reruns decision logic on replay (depends on code and
  external state). ES stores decided outcomes and replay only applies them.

## 4. Lockstep replays (PARTLY CONFIRMED)

- Gaffer On Games, Deterministic Lockstep (https://gafferongames.com/post/deterministic_lockstep/):
  networks a simulation "by sending only the *inputs* that control that system, rather than the
  *state*." "Determinism means that given the same initial condition and the same set of inputs
  your simulation gives exactly the same result." Warns determinism may break across compilers,
  OS, architectures, debug vs release.
- 1500 Archers on a 28.8 (Game Developer): "run the exact same simulation on each machine,
  passing each an identical set of commands that were issued by the users." AoE2 recordings
  "guaranteed to play out the exact same way every time." Does not say recordings are
  commands only, nor that patches break them.
- Blizzard, SC2 Patch 2.0.10 replay update
  (https://news.blizzard.com/en-us/article/10495761/replay-feature-update-with-patch-2-0-10):
  "replays from previous versions are incompatible with StarCraft II 2.0.10 and must be loaded in
  earlier versions of the game." "StarCraft II has always automatically loaded replays in the
  appropriate version of the game behind the scenes." No stated cause.
- Gap: no single primary source says "input-only replays, so a patch breaks them".

## Lost

The first engine researcher (agent 44b5d92f, HS/SabberStone/Forge/Arena/Metastone in one
prompt) completed, but its report was never delivered. Replaced by one researcher per engine.
