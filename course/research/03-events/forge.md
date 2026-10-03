# Forge (Card-Forge/forge, Java MTG engine): what happened

Researcher report, session 03 (node D). Agent 576133dd. Source read on `master` (not pinned to a
commit; pin SHAs before citing).

## Summary

Four separate channels:
1. Rules record: plain "this turn" fields on `Player`, `Game` and `MagicStack`, queried by card
   rules and reset in cleanup.
2. Trigger input: `TriggerHandler.runTrigger(TriggerType, Map<AbilityKey,Object>, holdTrigger)`,
   queued and moved to the stack when a player would receive priority.
3. UI output: `game.fireEvent(...)` to a Guava `EventBus`, separate from triggers, plus
   `TrackableObject` views written by the engine. The views hold the hidden-information rules.
4. AI simulation runs on a full copied `Game` with no GUI subscribers.
No full-game replay file. Macros record user inputs.

## Q1. Rules record: CONFIRMED (no single history log)

- `Player` fields (`forge-game/src/main/java/forge/game/player/Player.java`):
  `private int lifeLostThisTurn; private int lifeLostLastTurn; private int lifeGainedThisTurn; ...
  private int numDrawnThisTurn; ... private int landsPlayedThisTurn; ... private List<Card>
  discardedThisTurn ...; private List<Card> sacrificedThisTurn ...`. `getCreaturesAttackedThisTurn()`
  reads an `attackedThisTurn` map.
  - Updated at the mutation: `boolean firstLost = lifeLostThisTurn == 0; lifeLostThisTurn += toLose;`
    then `runTrigger(TriggerType.LifeLost, ...)`.
  - Reset in `Player.onCleanupPhase()`: `setLifeLostLastTurn(getLifeLostThisTurn());
    setLifeLostThisTurn(0); ... resetNumDrawnThisTurn(); ... resetSacrificedThisTurn();`
  - Queried, e.g. `getOpponentLostLifeThisTurn()` sums `opp.getLifeLostThisTurn()`.
  - https://github.com/Card-Forge/forge/blob/master/forge-game/src/main/java/forge/game/player/Player.java
- Spells cast this turn (`forge-game/src/main/java/forge/game/zone/MagicStack.java`):
  `private final List<SpellAbility> thisTurnCast`, `lastTurnCast`, `thisTurnActivated`.
  On cast: `thisTurnCast.add(sp.equals(source.getCastSA()) ? source.getCastSA() : sp.copy(lki, true));`
  Storm: `runParams.put(AbilityKey.CurrentStormCount, thisTurnCast.size());`
  `Player.getSpellsCastThisTurn()` filters `getGame().getStack().getSpellsCastThisTurn()`.
  `MagicStack.onNextTurn()` moves this turn's list to `lastTurnCast` and clears it.
  https://github.com/Card-Forge/forge/blob/master/forge-game/src/main/java/forge/game/zone/MagicStack.java
- Died / left the battlefield this turn (`forge-game/src/main/java/forge/game/Game.java`):
  `private List<Card> leftBattlefieldThisTurn`, `leftGraveyardThisTurn`, `countersAddedThisTurn`
  (`Table<CounterType, Player, ...>`), `countersRemovedThisTurn`, `globalDamageHistory`
  (`FCollection<CardDamageHistory>`), `damageThisTurnLKI`. Stores LKI copies:
  `public void addLeftBattlefieldThisTurn(Card lki)`. Cleared in `Game.onCleanupPhase()`:
  `clearLeftBattlefieldThisTurn(); clearLeftGraveyardThisTurn(); clearCountersThisTurn();
  clearGlobalDamageHistory();`. Revolt: `Player.hasRevolt()` uses
  `getGame().getLeftBattlefieldThisTurn().stream()...`.
  https://github.com/Card-Forge/forge/blob/master/forge-game/src/main/java/forge/game/Game.java
- `GameLog` (`forge-game/src/main/java/forge/game/GameLog.java`): `List<GameLogEntry> log`,
  `extends Observable`. Filled from events: the `Game` constructor calls
  `subscribeToEvents(gameLog.getEventVisitor());` (a `GameLogFormatter`). No rules code reading
  `GameLog` found (search, not a full audit).

## Q2. Trigger input: CONFIRMED

File: `forge-game/src/main/java/forge/game/trigger/TriggerHandler.java`
(https://github.com/Card-Forge/forge/blob/master/forge-game/src/main/java/forge/game/trigger/TriggerHandler.java)

- Entry: `public final void runTrigger(final TriggerType mode, final Map<AbilityKey, Object>
  runParams, boolean holdTrigger)`. Callers build `AbilityKey` maps, e.g.
  `runParams.put(AbilityKey.LifeAmount, toLose); ... runTrigger(TriggerType.LifeLost, runParams, false)`.
  No `runTriggerOld` on current `master`.
- Waiting list: `private final List<TriggerWaiting> waitingTriggers`. Queue condition:
  `else if (canWait && (game.getStack().isFrozen() || holdTrigger) && mode != TriggerType.TapsForMana
  && mode != TriggerType.ManaAdded) { waitingTriggers.add(new TriggerWaiting(mode, runParams)); }
  else { runWaitingTrigger(...) }`. `TriggerType.Always` (state triggers) goes to
  `runStateTrigger`. `collectTriggerForWaiting()` snapshots matching triggers at event time
  (`wt.setTriggers(getActiveTrigger(...))`). When the stack unfreezes, `MagicStack` ("Add all
  waiting triggers onto the stack") calls `runWaitingTriggers()`.
- Two stages to the stack: `runSingleTriggerInternal` wraps (`new WrappedAbility(regtrig, sa,
  decider)`) and calls `game.getStack().addSimultaneousStackEntry(wrapperAbility)` ("the trigger
  will be ordered later in MagicStack"). Then `PhaseHandler`
  (`forge-game/src/main/java/forge/game/phase/PhaseHandler.java`): `// CR 704.3 Whenever a player
  would get priority, the game checks ... for state-based actions` and
  `while (game.getStack().addAllTriggeredAbilitiesToStack());`.
- Separate from GameEvent: `TriggerHandler` does not reference `fireEvent` or `EventBus`. Engine
  code calls both side by side: in `Player` life loss, `game.fireEvent(new
  GameEventPlayerLivesChanged(...))` then `game.getTriggerHandler().runTrigger(TriggerType.LifeLost,
  ...)`. Trigger handler supports suppression (`suppressMode`, `setSuppressAllTriggers`); the event
  bus has nothing comparable.

## Q3. Output to UI

- Guava EventBus (CONFIRMED). `Game.java`: `import com.google.common.eventbus.EventBus;`,
  `private final EventBus events = new EventBus("game events");`,
  `public void fireEvent(final Event event) { events.post(event); }`,
  `public void subscribeToEvents(final Object subscriber) { events.register(subscriber); }`.
  Javadoc on `fireEvent`: "Fire only the events after they became real for gamestate and won't get
  replaced. The events are sent to UI, log and sound system."
- Visitor (CONFIRMED). `forge-game/src/main/java/forge/game/event/GameEvent.java`:
  `public interface GameEvent extends Event, Serializable { <T> T visit(IGameEventVisitor<T> visitor); }`.
  `IGameEventVisitor.java` has one method per event type (`GameEventCardDamaged`,
  `GameEventCardChangeZone`, `GameEventSpellResolved`, `GameEventSpellAbilityCast`,
  `GameEventPlayerPriority`, `GameEventAddLog`, ...) and a no-op `class Base<T>`.
  `MagicStack` fires `game.fireEvent(new GameEventSpellResolved(sa, thisHasFizzled));`.
- Subscribers (CONFIRMED). `forge-gui/src/main/java/forge/gui/control/FControlGameEventHandler.java`:
  `extends IGameEventVisitor.Base<Void>`, `@Subscribe public void receiveGameEvent(final GameEvent
  ev) { ev.visit(this); SoundSystem.instance.receiveEvent(ev); }`. Batches dirty
  `CardView`/`PlayerView` sets and flushes with `invokeInEdtLater`.
  `forge-gui/src/main/java/forge/gamemodes/match/HostedMatch.java` registers: local GUI
  `game.subscribeToEvents(new FControlGameEventHandler(humanController));`, network GUI a
  `GameEventForwarder`, match level `match.subscribeToEvents(SoundSystem.instance)` and
  `HapticEngine.instance`, hosting `NetworkGameEventListener`, quest mode `QuestController`, games
  with no human `FControlGamePlayback`. No `EventVisualizer` class found.
- Views alongside events (CONFIRMED). `forge-game/src/main/java/forge/trackable/TrackableObject.java`
  Javadoc: "Base for objects that mirror engine state into a serialized view consumed by GUI(s)...
  the engine writes via set and consumers (GUIs) read via get." Per-consumer dirty bits for
  network delta sync: "Offline games never register consumers, so set does no tracking work."
  Frozen `Tracker` queues changes. Engine updates views directly (`view.updateLife(this)`,
  `game.updateStackForView()`, `getView()` returns `GameView`).
- Per-player hidden info in views (CONFIRMED). `forge-game/src/main/java/forge/game/card/CardView.java`:
  `canBeShownTo(PlayerView viewer)` with per-zone rules, e.g. `case Hand: if
  (controller.equals(viewer)) return true;`, "Library... hidden to all unless they specify
  otherwise". Exceptions via `mayPlayerLook(viewer)` (`PlayerMayLook`) and a mind-control check.
  Also `canFaceDownBeShownTo`, `getImageKey(viewers)` (returns `HIDDEN_CARD`), `getHiddenId()`
  ("Face-down card (H…)").

## Q4. Replays

- No full-game replay file: NOT FOUND. 2014 forum post: "not a simple feature to add (it involves
  rewriting many thousands of lines of code...)".
  https://slightlymagic.net/forum/viewtopic.php?f=52&t=15545
- Macros record user inputs (CONFIRMED from docs only): "Macros let advanced users record and
  replay a short sequence of match actions... selecting cards or players, activating abilities,
  passing priority...", "Macro replay is prompt-aware... If the game state diverges from the
  recording, playback stops." https://github.com/Card-Forge/forge/wiki/User-Guide,
  PR https://github.com/Card-Forge/forge/pull/10930
- On network reconnect, the host replays its log entries as `GameEventAddLog` events (client
  catch-up, not replay). https://github.com/Card-Forge/forge/commit/340eee0221093d5bddc1b5046749d79f8da1971d
- `FControlGamePlayback` slows and pauses a live AI-vs-AI game by sleeping on events
  (`pauseForEvent(castDelay)`, a `CyclicBarrier`). Not a replay.

## Q5. AI cost

- Separate copy (CONFIRMED). `forge-ai/src/main/java/forge/ai/simulation/GameCopier.java`,
  `makeCopy`: `Match newMatch = new Match(...); Game newGame = new Game(newPlayers, currentRules,
  newMatch); newGame.setNoGUIUser();`. Disables triggers during the copy:
  `newGame.getTriggerHandler().suppressMode(TriggerType.ChangesZone);`, `setTriggers(false)` on
  battlefield zones, then re-enables. Alternative path `EXPERIMENTAL_RESTORE_SNAPSHOT` with
  `GameSnapshot`. `GameSimulator.java`: `copier = new GameCopier(origGame); simGame =
  copier.makeCopy(...)`, resolves with `GameSimulator.resolveStack`.
- Events suppressed? PARTLY CONFIRMED. `fireEvent` has no simulation check; events still post to
  the copy's own `EventBus`. No `subscribeToEvents` in `GameCopier`/`GameSimulator`; only the
  constructor's `GameLog` formatter subscribes. "No UI listeners" rests on absence of
  registration. `isNoGUIUser()` effects not traced. The copy still creates a `GameView`/`Tracker`;
  with no consumers, `TrackableObject.set` does no dirty tracking.

## Gaps

`master` changes often (pin SHAs). Not found: `runTriggerOld`, `EventVisualizer`, a simulation
suppression flag for GameEvents. Not verified: `Game.isNoGUIUser()`, the macro recorder,
`GameSnapshot`.
