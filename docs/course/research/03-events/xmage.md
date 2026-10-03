# XMage (magefree/mage, Java MTG engine, client/server): what happened

Researcher report, session 03 (node D). Agent e459153f. Raw source on `master`
(https://github.com/magefree/mage/blob/master/<path>). Pin SHAs before citing.

## Summary

Three separate ways. Rules: Watchers listen to every `GameEvent` and keep per-turn records that
cards query. Triggers: events go through replacement effects first, then are broadcast to watchers
and triggers; fired triggers are queued and put on the stack the next time a player would get
priority. Client: full per-player `GameView` snapshots plus log text, no discrete events. Replays
stored whole `GameState` copies (feature marked outdated). AI simulations run on copies with a
`simulation` flag that turns off client notifications; watchers and triggers still run.

## 1. Rules record: Watchers (CONFIRMED)

- `Mage/src/main/java/mage/watchers/Watcher.java`: "watches for certain game events to occur and
  flags condition". `public abstract void watch(GameEvent event, Game game);`, default `public void
  reset() { condition = false; }`. Scope `GAME` / `PLAYER` / `CARD` for `getKey()`.
- `Watchers extends HashMap<String, Watcher>`; `watch(event, game)` calls every watcher; `reset()`
  does `this.values().forEach(Watcher::reset)`.
- `GameState.handleEvent`: `watchers.watch(event, game); delayed.checkTriggers(event, game);
  triggers.checkTriggers(event, game);` (watchers before triggers).
- `GameImpl.endOfTurn()` calls `state.resetWatchers();` (`this.watchers.reset()`). Also once after
  the mulligan: "watcher objects from cards are reused during match so reset all card watchers
  already added".
- Each watcher defines its reset:
  - `SpellsCastWatcher`: `Map<UUID, List<Spell>> spellsCast`, stores `spell.copy()` ("copy needed
    because attributes like color could be changed later"); `reset()` clears.
  - `CreaturesDiedWatcher`: filters `ZONE_CHANGE` with `isDiesEvent()`, counts by controller and
    owner.
  - `PlayerLostLifeWatcher`: watches `LOST_LIFE`; `reset()` copies this turn into
    `amountOfLifeLostLastTurn` before clearing; "automatically started in gameImpl.init for each
    game".

## 2. Trigger input: events, replacement, trigger queue (CONFIRMED)

- `GameImpl`: `fireEvent(GameEvent event) { state.handleEvent(event, this); }`,
  `replaceEvent(GameEvent event) { return state.replaceEvent(event, this); }`.
  `GameState.replaceEvent` checks `effects.preventedByRuleModification(...)` then returns
  `effects.replaceEvent(event, game)`.
- `PlayerImpl.gainLife`: `new GameEvent(GameEvent.EventType.GAIN_LIFE, ...)`, then `if
  (!game.replaceEvent(event)) { ... this.life = CardUtil.overflowInc(this.life, event.getAmount());
  ... game.fireEvent(GameEvent.getEvent(GameEvent.EventType.GAINED_LIFE, ...)); }`. Pattern:
  replaceable pre-event, mutation, fired past-tense event.
- `TriggeredAbilities.checkTriggers`: `if (ability.checkEventType(event, game)) {
  checkTrigger(ability, event, game); }`; on `ability.checkTrigger(event, game)` builds a
  replaceable `NumberOfTriggersEvent` (trigger doublers), calls `ability.trigger(...)` per result.
- `TriggeredAbilityImpl.trigger`: `checkInterveningIfClause(game) && checkTriggerCondition(game)`
  then `game.addTriggeredAbility(this, triggeringEvent)`. `GameState`: `List<TriggeredAbility>
  triggered` ("raised triggers, waiting to resolve"), `this.triggered.add(ability)`.
- `playPriority`: "603.3. Once an ability has triggered, its controller puts it on the stack ... the
  next time a player would receive priority", then `checkStateAndTriggered();`: loops `if
  (!checkStateBasedActions())` then `state.handleSimultaneousEvent(this)` and `checkTriggered()`.
  `checkTriggered()` walks APNAP order from `state.getPlayerList(state.getActivePlayerId())`, reads
  `state.getTriggered(playerId)`, runs non-stack triggers first.
- `GameState`: `List<GameEvent> simultaneousEvents`; `addSimultaneousEvent` appends;
  `handleSimultaneousEvent` builds `createEventGroups(...)` (incl. `new
  ZoneChangeGroupEvent(movedCards, movedTokens, ...)`), clears, passes each to `handleEvent`. Batch
  helpers for damage, mill, life loss, sacrifice, tap/untap (e.g. `addSimultaneousLifeLossToBatch`).
- `GameEvent.java` itself not fetched (`mage.game.events.GameEvent`, nested `EventType`, from
  imports).

## 3. Output to client: snapshots, redaction, log (CONFIRMED)

- `Mage.Common/src/main/java/mage/view/GameView.java`: `GameView(GameState state, Game game, UUID
  createdForPlayerId, UUID watcherUserId)` builds a `PlayerView` per player, stack, exile,
  revealed, combat, ...
- `Mage.Server/src/main/java/mage/server/game/GameSessionWatcher.java` `update()`: `new
  ClientCallback(ClientCallbackMethod.GAME_UPDATE, game.getId(), getGameView())`.
- Every prompt carries a fresh view: `new GameClientMessage(getGameView(), options, question)` in
  `GameSessionPlayer.ask`. TODO: "implement RepeatedGameView to send back ref number instead full
  game view on non-changeable".
- Redaction at view build: `if (player.getId().equals(createdForPlayerId)) { ...
  this.myHand.putAll(...) }`; `PlayerView` gives others only `handCount`, `libraryCount`.
  `opponentHands` only for controlled players (`processControlledPlayers`); `watchedHands` only with
  `player.hasUserPermissionToSeeHand(userId)`; `state.getLookedAt(this.myPlayerId)`. `CardView`:
  "opponent cards: face down status, face down image", `showHiddenFaceDownData = showAsControlled ||
  game.hasEnded()`, name blanked otherwise.
- What happened reaches the client as text: `GameImpl.informPlayers(String)` calls
  `tableEventSource.fireTableEvent(EventType.INFO, message, this)`; `GameController` `case INFO:`
  routes to `chatManager().broadcast(chatId, ...)`. `fireUpdatePlayersEvent()` fires
  `EventType.UPDATE`, then `updateGame()`, `gameSession.update()` for every player and watcher
  (also `clearLookedAt()`, `clearRevealed()`). Engine writes log text by hand, e.g.
  `game.informPlayers(this.getLogName() + " gains " + event.getAmount() + " life" ...)`.
- `ClientCallbackMethod`: `GAME_UPDATE("gameUpdate")`, `GAME_UPDATE_AND_INFORM("gameInform")`,
  `GAME_INFORM_PERSONAL`, dialogs (`GAME_TARGET`, `GAME_ASK`, ...), `GAME_OVER`.
- Animation-level events: NOT FOUND (only `GAME_REDRAW_GUI`). Client-side diffing not checked.

## 4. Replays: state snapshots, outdated (CONFIRMED)

- `Mage/src/main/java/mage/game/GameStates.java`: `save(GameState gameState) {
  states.add(gameState.copy()); }`. `GameImpl.saveState` only when `!simulation && gameStates !=
  null` and `bookmark || saveGame`. Same list backs rollback (`GameStates.rollback(int index)`).
- `GameController.saveGame()`: `output.writeObject(game); output.writeObject(game.getGameStates());`
  into `"saved/" + gameId + ".game"` (gzipped), enabled by `isSaveGameActivated()`.
- `GameReplay.next()` returns `savedGame.get(stateIndex++)`; `ReplaySession` sends `new
  GameView(state, game, null, null)` as `REPLAY_UPDATE`; `ReplayManagerImpl` start/stop/next/
  previous/skipForward.
- `GameReplay` javadoc: "Replay system, outdated and not used. TODO: delete";
  `ClientCallbackMethod`: `// replay (unsupported)`. No input recording on this path.

## 5. AI cost: simulation flag (CONFIRMED)

- `createSimulationForAI()`: `Game res = this.copy(); ((GameImpl) res).simulation = true;
  ((GameImpl) res).aiGame = true;`. `createSimulationForPlayableCalc()` also sets
  `checkPlayableState`. `isSimulation()` returns `simulation`.
- `tableEventSource`, `playerQueryEventSource` are `transient`; copy constructor:
  `//this.tableEventSource = game.tableEventSource; // client-server part, not need on
  copy/simulations`.
- `if (simulation) { return; }` in `fireUpdatePlayersEvent`, `fireStatusEvent`, `fireAskPlayerEvent`,
  `fireGetChoiceEvent`, `fireChoosePileEvent`, ... `informPlayers` returns early after
  `DataCollectorServices.getInstance().onGameLog(this, message)`. `saveState` skips simulations.
  Callers guard string building: `if (!game.isSimulation()) { game.informPlayers(...) }`.
- Rules bookkeeping not skipped: `GameState.handleEvent` (watchers, triggers) has no simulation
  check.
- AI: `Mage.Server.Plugins/Mage.Player.AI.MAD/src/mage/player/ai/ComputerPlayer6.java`,
  `simulatePriority`: `Game sim = game.createSimulationForAI();` then `sim.checkStateAndTriggered()`.
  Minimax with alpha-beta, `MAX_SIMULATED_NODES_PER_CALC = 5000`.

## Gaps

`GameEvent.java` not fetched. `ComputerPlayer7` body not read. Client rendering and any view
diffing in `Mage.Client` not investigated. `initGameDefaultWatchers()` not read.
