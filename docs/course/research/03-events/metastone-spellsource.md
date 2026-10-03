# Metastone and Spellsource: what happened

Researcher report, session 03 (node D). Agent 30e8e178. Metastone at `master`, Spellsource at
`HEAD` (Spellsource-Server URLs resolve to `hiddenswitch/Spellsource`). Pin SHAs before citing.

## Summary

Three channels in both. Rules read counters (per-player `GameStatistics`, `Attribute` values on
entities, the graveyard). Triggers get `GameEvent` objects dispatched synchronously when each
event fires. The UI gets the same event objects: Metastone buffers them in a `GameContext`
subclass; Spellsource streams them through `Notification` hooks, each message bundled with a full
per-player redacted state snapshot. Spellsource replays store inputs only (`Trace`: seed, decks,
mulligans, chosen action indices); viewable replays are rebuilt by re-simulation. AI clones skip
UI delivery because `clone()` builds a plain base `GameContext`.

## 1. Rules record

Metastone:
- CONFIRMED: no `SPELLS_CAST` attribute; has `DIED_ON_TURN`, `LAST_HIT`.
  https://github.com/demilich1/metastone/blob/master/game/src/main/java/net/demilich/metastone/game/Attribute.java
- CONFIRMED: per-player `GameStatistics`. `Player` holds `private final GameStatistics statistics`;
  `GameStatistics` stores `EnumMap<Statistic,Object> stats` and `cardsPlayed` as `Map<String,
  Map<Integer,Integer>>` (cardId, turn, count). `cardPlayed()` runs `add(Statistic.SPELLS_CAST, 1)`.
  https://github.com/demilich1/metastone/blob/master/game/src/main/java/net/demilich/metastone/game/statistics/GameStatistics.java
- CONFIRMED: `GameLogic` writes the record right before firing: `player.getStatistics().cardPlayed(card,
  context.getTurn()); CardPlayedEvent cardPlayedEvent = new CardPlayedEvent(...);
  context.fireGameEvent(cardPlayedEvent);`. `destroyMinion` fires `KillEvent` first, then
  `minion.setAttribute(Attribute.DIED_ON_TURN, context.getTurn())`.
  https://github.com/demilich1/metastone/blob/master/game/src/main/java/net/demilich/metastone/game/logic/GameLogic.java
- CONFIRMED: value providers read it: `CardsPlayedValueProvider` uses
  `player.getStatistics().getCardsPlayed()`; `DeadMinionsThisTurn` calls
  `SpellUtils.howManyMinionsDiedThisTurn`, which scans `player.getGraveyard()` for
  `getAttributeValue(Attribute.DIED_ON_TURN) == currentTurn`.

Spellsource:
- CONFIRMED: no plain `SPELLS_CAST` (only `SPELLS_CAST_TWICE`/`THRICE` effect flags). Counters as
  Attributes: `MINIONS_SUMMONED_THIS_TURN` ("this counter is incremented on the summoning player"),
  `TOTAL_DAMAGE_DEALT_THIS_GAME`, `DAMAGE_THIS_TURN`, `DIED_ON_TURN` ("records which turn a Minion
  was marked as DESTROYED"), `PLAYED_FROM_HAND_OR_DECK`.
  https://github.com/hiddenswitch/Spellsource-Server/blob/HEAD/spellsource-game/src/main/java/net/demilich/metastone/game/cards/Attribute.java
- CONFIRMED: inline updates, end-of-turn reset: `target.modifyAttribute(Attribute.DAMAGE_THIS_TURN,
  damageDealt)`, `sourceOwner.modifyAttribute(Attribute.TOTAL_DAMAGE_DEALT_THIS_GAME, ...)`, then
  `fireGameEvent(damageEvent)`. End of turn: `eachPlayer.setAttribute(Attribute.MINIONS_SUMMONED_THIS_TURN,
  0)`, `context.setLastSpellPlayedThisTurn(playerId, null)`.
- CONFIRMED: `GameStatistics` kept too; `PlayedThisTurnValueProvider` reads
  `getCardsPlayed()` with `getOrDefault(context.getTurn(), 0)`.

## 2. Trigger input

Metastone:
- CONFIRMED: synchronous, no queue. `GameContext.fireGameEvent`: `if (ignoreEvents()) return;` then
  `triggerManager.fireGameEvent(gameEvent)`.
- CONFIRMED: `TriggerManager.fireGameEvent` loops a flat `List<IGameEventListener>`, filters with
  `trigger.interestedIn(event.getEventType())` and `trigger.canFire(event)`, then `canFireCondition`,
  `countDown`, `onGameEvent(event)`. `SpellTrigger.onGameEvent` pushes
  `event.getEventTarget().getReference()` on `getEventTargetStack()`, casts via `castSpell`, pops.
  That stack holds entity references, not events.
- CONFIRMED `GameEventType`: `DAMAGE`, `KILL`, `SUMMON`, `SPELL_CASTED`, `PLAY_CARD`, `TURN_END`,
  `ALL`, ...

Spellsource:
- CONFIRMED: synchronous, two phases (queue then process), deferred queue for re-entrant triggers.
  `GameLogic.fireGameEvent`: checks `getIgnoreEvents()`, throws past `triggerDepth > 96` ("infinite
  recursion"), calls `context.onNotificationWillFire(event)` and `pushEventData(event)`, snapshots
  `new ArrayList<>(context.getTriggers())`, collects `interestedIn(...) && trigger.queues(event)`
  into `thisQueuedTriggers` (or `context.getDeferredTriggersQueue()` as `QueuedTrigger(event,
  trigger)` if already processing), runs `processTrigger`, drains deferred when
  `getProcessingTriggers().isEmpty()`, ends with `context.onNotificationDidFire(event)` in
  `finally`. Comment: "Queueing gives a trigger an opportunity to look at the state of the board
  BEFORE all the other trigger's effects have been evaluated."
- CONFIRMED stacks on `GameContext`: `getEventTargetStack()`, an event-value stack,
  `getTriggerHostStack()`, `Deque<GameAction> actionStack`, `deferredTriggersQueue`. No
  `getEventStack()`.
- CONFIRMED `Enchantment` matches via `EventTrigger`: `interestedIn(eventType)` loops `getTriggers()`
  (`trigger.interestedIn() == eventType || ... == GameEventType.ALL`); `onGameEvent` calls
  `event.getGameContext().onEnchantmentFired(this)` (except BOARD_CHANGED, WILL_END_SEQUENCE, ALL)
  then `process(...)`. `GameEventType` is a protobuf enum
  (`Spellsource.GameEventTypeMessage.GameEventType`).

## 3. Output to UI

Metastone:
- CONFIRMED: same `GameEvent` objects buffered by a subclass. `GameContextVisualizable.fireGameEvent`:
  `super.fireGameEvent(gameEvent); getGameEvents().add(gameEvent);`. `onGameStateChanged()` sends
  `NotificationProxy.sendNotification(GameNotification.GAME_STATE_UPDATE, this)` (the whole
  context), then blocks on `blockedByAnimation`. `EventVisualizerDispatcher.visualize` maps `DAMAGE`,
  `HEAL`, `PLAY_CARD`, `JOUST`, `REVEAL_CARD` to visualizers, then `getGameEvents().clear()`. Base
  `GameContext.onGameStateChanged()` is empty.
  https://github.com/demilich1/metastone/blob/master/app/src/main/java/net/demilich/metastone/gui/playmode/GameContextVisualizable.java
- `addTempCard` is unrelated (registers non-catalogue cards).

Spellsource:
- CONFIRMED: `GameEvent implements Notification`. Notification docs: "Unlike a GameEvent, a
  notification does not say anything about having side effects or triggering other rules". Has
  `isPowerHistory()` and `default boolean isClientInterested() { return false; }`. `BasicGameEvent`
  takes an `isClientInterested` flag; `AbstractDamageEvent` passes `true` (inferred position).
- CONFIRMED: server forwards the same objects with a full snapshot. `ServerGameContext.onNotificationWillFire`:
  `if (event.isClientInterested()) { var gameStateCopy = getGameStateCopy(); for (var client :
  getClients()) client.sendNotification(event, gameStateCopy); }`. `onNotificationDidFire` flushes
  via `client.lastEvent()` when a nesting counter hits 0. Game actions (`onWillPerformGameAction`)
  and `TriggerFired` (`onEnchantmentFired`) sent the same way; a `TriggerFired` hosted in a
  private zone goes only to the owner.
  https://github.com/hiddenswitch/Spellsource-Server/blob/HEAD/spellsource-server/src/main/java/com/hiddenswitch/framework/impl/ServerGameContext.java
- CONFIRMED: each message bundles event plus redacted per-player state.
  `UnityClientBehaviour.sendNotification` builds `ServerToClientMessage` with
  `MessageType.ON_GAME_EVENT`, `.setChanges(visibleEntities(gameState))`,
  `.setGameState(getClientGameState(playerId, gameState))`. Events converted by
  `ModelConversions.getClientEvent(event, playerId)`. Buffered in `messageBuffer`. `powerHistory`
  capped at `MAX_POWER_HISTORY_SIZE = 10`.
- CONFIRMED: redaction in `ModelConversions` (no `isHidden`/`getHiddenValue`).
  `getGameState(ctx, local, opponent)`: "Censor the opponent hand and deck entities", "This view does
  not leak secure information". `getCensoredCard(...)` sets `setCardId("hidden")` (opponent secrets
  unless `SecretRevealedEvent`). `Attribute.UNCENSORED` opts out. `getClientGameState` calls
  `simulatedContext.setIgnoreEvents(true)` while converting. Equivalent of `toClientEntity` is
  `ModelConversions.getEntity(...)`.

## 4. Replays

- CONFIRMED (Spellsource): `Trace` doc: "Given the seed, starting conditions and index of each
  action in the available actions that a player chose, the game will reproduce." Fields `seed`,
  `heroClasses`, `deckCardIds`, `mulligans`, `List<Integer> actions` (`actions.add(action.getId())`).
  `replayContext` re-runs with a `TraceBehaviour` per player. `GameContext.startTrace()`:
  `trace.setSeed(getLogic().getSeed())`.
  https://github.com/hiddenswitch/Spellsource-Server/blob/HEAD/spellsource-game/src/main/java/net/demilich/metastone/game/logic/Trace.java
- CONFIRMED: persisted and used to resume. `removeGameAndRecordReplay` stores
  `gameContext.getTrace().toJson()`; `checkpoint()` writes `Tables.GAMES.TRACE` for resumable bot
  matches; `restoreFromTrace` calls `trace.replayContext(...)` then
  `setGameState(replayed.getGameStateCopy())`.
- CONFIRMED: viewable `Replay` rebuilt by re-simulation:
  `ModelConversions.replayFromGameContext` calls `originalCtx.getTrace().replayContext(false,
  augmentReplayWithCtx)`, recording `ReplayGameStates` from both points of view plus forward and
  backward `ReplayDeltas` built with `visibleEntities`.
- NOT FOUND (Metastone): no replay or trace class in files read.

## 5. AI cost

- CONFIRMED (Metastone): `GameContext.clone()` builds the base class; `GameContextVisualizable` does
  not override `clone`, so clones neither buffer nor notify. Triggers still copied
  (`clone.triggerManager = triggerManager.clone()`). Explicit flag:
  `clone.getLogic().setLoggingEnabled(false)`. `setIgnoreEvents(boolean)` exists but `clone()`
  doesn't set it. AI: `GreedyOptimizeMove` calls `simulateAction(context.clone(), ...)`.
- PARTLY CONFIRMED (Spellsource): `clone()` Javadoc "used by AI functions to evaluate a game state",
  returns `new GameContext(this)`; `ServerGameContext` has no `clone` override; base hooks empty
  (`public void onNotificationWillFire(Notification event) { }`). `GameStateValueBehaviour.requestAction`:
  `// Isolate this context  context = context.clone();`. Copy constructor still clones behaviours
  and trace (`setTrace(fromContext.getTrace().clone())`), unmeasured cost.

## Gaps

Metastone replays only NOT FOUND. `ValueEvent` constructor not opened. `processTrigger`,
`Enchantment.queues`/`fires` bodies not read. Cloned `UnityClientBehaviour` sending during
simulation unverified. Trace growth in clones unconfirmed.
