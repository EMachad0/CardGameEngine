# Engines pausing mid-resolution for a player decision

Researcher report, session 06. Sources are current `master` source code of each engine, the
hearthstone.wiki.gg Discover page and Advanced rulebook, the yawgatog Comprehensive Rules (effective
2026-09-25) and OpenSpiel's headers.

## Summary

Three ways engines handle a choice needed partway through an effect:

- **Block the game thread.** Forge, XMage and Metastone call into a player controller
  synchronously during resolution. For a human, the game thread waits: Forge on a `CountDownLatch`
  or `FutureTask.get()`, XMage on `response.wait()`, Metastone polling with `Thread.sleep`. The AI
  answers inline on the same thread. The paused effect lives on the Java call stack, so a game copy
  cannot carry it.
- **Fiber or virtual thread.** Spellsource keeps the synchronous `requestAction` call style. Older
  versions marked `GameLogic` methods with Quasar's `@Suspendable`. The current server uses Vert.x
  `await(promise.future())`, which requires a virtual thread.
- **Pending choice stored as data.** SabberStone stores the choice in a `Choice` object on the
  `Controller`, and the work after the choice in `Choice.AfterChooseTask`. The game can be cloned
  while the choice is pending, and `Options()` returns only `ChooseTask`s until it is answered.
  Tasks listed after `DiscoverTask` in the same task list do not wait for the choice.
- **Replay.** Forge's simulation AI (for search) and Spellsource's `Trace` (for reproduction)
  rebuild a game by replaying recorded answers from a starting point, instead of storing a
  continuation.

Rules: MTG 608.2c and 608.2d make choices "while applying the effect", in written order, and 603.3
holds triggered abilities until "the next time a player would receive priority", so never during
resolution. The Hearthstone Advanced rulebook says triggers resolve immediately and depth-first,
and only Deaths wait for the outermost Phase to end. No documented rule says whether a trigger
raised earlier in a spell resolves before or after a Discover choice.

## 1. Forge (Card-Forge/forge)

- CONFIRMED: the human answers through a blocking latch on the game thread.
  `forge-gui/src/main/java/forge/gamemodes/match/input/InputSyncronizedBase.java` holds
  `private final CountDownLatch cdlDone;`. `showAndWait()` calls
  `getController().getInputQueue().setInput(this); awaitLatchRelease();`. `awaitLatchRelease()`
  asserts it is not on the GUI thread (`FThreads.assertExecutedByEdt(false)`) and calls
  `getController().getGui().awaitInput(cdlDone)`. `stop()` releases the latch with
  `cdlDone.countDown()`.
  https://github.com/Card-Forge/forge/blob/master/forge-gui/src/main/java/forge/gamemodes/match/input/InputSyncronizedBase.java
- CONFIRMED: `PlayerControllerHuman.chooseSingleEntityForEffect` and `chooseCardsForEffect` build
  `InputSelectEntitiesFromList` or `InputSelectCardsFromList`, call `input.showAndWait()`, then
  read `input.getSelected()`. `chooseCardsForEffect` delegates to `chooseSingleEntityForEffect`
  when `min == 1 && max == 1`. When the input can't be used, both fall back to a direct
  `getGui().chooseSingleEntityForEffect(...)` or `getGui().many(...)` call.
  https://github.com/Card-Forge/forge/blob/master/forge-gui/src/main/java/forge/player/PlayerControllerHuman.java
- CONFIRMED: the desktop dialog fallback also blocks. `forge-gui-desktop/src/main/java/forge/gui/GuiChoose.java`:
  `final FutureTask<List<T>> future = new FutureTask<>(showChoice); FThreads.invokeInEdtAndWait(future); ... return future.get();`.
- CONFIRMED: the AI answers inline. `forge-ai/src/main/java/forge/ai/PlayerControllerAi.java`:
  `chooseCardsForEffect` returns `brains.chooseCardsForEffect(...)`, and
  `chooseSingleEntityForEffect` returns `SpellApiToAi.Converter.get(sa).chooseSingleEntity(...)`.
- CONFIRMED by code reading: `forge-ai/src/main/java/forge/ai/simulation/GameCopier.java` copies
  players, zones, combat and remembered objects. `copyStack` runs only
  `if (GameSimulator.COPY_STACK)`, and `GameSimulator` declares
  `public static boolean COPY_STACK = false;`. Nothing captures a pending choice or a partly
  executed `SpellAbility`; that state exists only on the blocked thread's call stack. That a copy
  taken mid-choice cannot resume the choice is an inference: there is no code for it.
- CONFIRMED: replay with recorded answers, for AI search. `SpellAbilityPicker.evaluateSa` loops
  `do { MyRandom.setRandom(new Random(randomSeedToUse)); GameSimulator simulator = new GameSimulator(controller, game, player, phase); simulator.setInterceptor(choicesIterator); lastScore = simulator.simulateSpellAbility(sa); ... } while (choicesIterator.advance(lastScore));`.
  Its comment explains the fixed seed: "as otherwise random effects may result in a different
  number of choices each iteration". Resolution-time choices such as
  `chooseCardToHiddenOriginChangeZone` and `chooseModeForAbility` return the interceptor's answer.
  `SpellAbilityChoicesIterator` stores `ChoicePoint { numChoices, nextChoice, ... }` and steps
  through combinations like an odometer. Each candidate set of answers is replayed from a fresh
  copy of the game.
- UNVERIFIED: how network play (`NetGuiGame`, `sendAndWait`) returns a forced choice.

## 2. XMage (magefree/mage)

- CONFIRMED: the human blocks the game thread on `response.wait()`.
  `Mage.Server.Plugins/Mage.Player.Human/src/mage/player/human/HumanPlayer.java`, header comment:
  "server executing a game's code by single game thread ... GAME thread: open response for income
  command and wait (go to sleep by response.wait) ... CALL thread: on opened response - save answer
  to player's response object and notify GAME thread about it by response.notifyAll".
  `waitForResponse(Game game)` calls `ThreadUtils.ensureRunInGameThread()`, then
  `response.wait()` inside `synchronized (response)`.
- CONFIRMED: `choose` and `chooseUse` run `prepareForResponse(game); game.fire...Event(...); waitForResponse(game);`
  inside `while (canRespond())`.
- CONFIRMED: no blocking in simulated games:
  `private boolean canCallFeedback(Game game) { return !gameInCheckPlayableState(game) && !game.isSimulation(); }`.
- CONFIRMED: the AI answers inline. `Mage.Server.Plugins/Mage.Player.AI/src/mage/player/ai/ComputerPlayer.java`:
  `choose(Outcome, Choice, Game)` uses heuristics then `choice.setRandomChoice()`;
  `choose(Outcome, Cards, TargetCard, Ability, Game)` returns `makeChoice(...)`.
- UNVERIFIED: whether XMage copies a `Game` while a human choice is pending.

## 3. SabberStone (HearthSim/SabberStone)

- CONFIRMED: `SabberStoneCore/src/Model/Entities/Controller.cs` declares
  `public Choice Choice { get; set; }`, "The last choice set proposed to this player."
- CONFIRMED: `SabberStoneCore/src/Tasks/SimpleTasks/DiscoverTask.cs` `Process` calls
  `Generic.CreateChoiceCards.Invoke(controller, source, null, ChoiceType.GENERAL, choiceAction, result, _taskTodo);`
  and returns `TaskState.COMPLETE`. `_taskTodo` is the constructor's `ISimpleTask afterDiscoverTask`.
  Repeated discovers chain `Choice.NextChoice`.
- CONFIRMED: `SabberStoneCore/src/Model/Choice.cs` has `public ISimpleTask AfterChooseTask`,
  `public Choice NextChoice`, `internal List<int> EntityStack`, `ChoiceAction` (HAND, SUMMON,
  CAST, ...) and `SourceId`.
- CONFIRMED: `SabberStoneCore/src/Tasks/PlayerTasks/ChooseTask.cs` (case `GENERAL`) calls
  `Generic.ChoicePick.Invoke(...)`, then `Game.ProcessTasks(); Game.DeathProcessingAndAuraUpdate();`.
  `Generic.ChoicePick` (`SabberStoneCore/src/Actions/Choose.cs`) applies the `ChoiceAction`, runs
  `c.Choice.AfterChooseTask.Process(...)` with a `TaskStack` built from `EntityStack`, then sets
  `c.Choice` to `nextChoice` or `null`.
- CONFIRMED: `Controller.Options()` begins
  `if (Choice != null) { switch (Choice.ChoiceType) { case ChoiceType.GENERAL: return Choice.Choices.Select(p => (PlayerTask)ChooseTask.Pick(this, p)).ToList(); ...`,
  and returns an empty list when `this != Game.CurrentPlayer`.
- CONFIRMED: the `Controller` copy constructor has `Choice = controller.Choice?.Clone(this);`. The
  `Choice` copy constructor copies `Choices`, `EntityStack` and `NextChoice`, and shares
  `AfterChooseTask` by reference.
- CONFIRMED by code reading: `SabberStoneCore/src/Tasks/StateTaskList.cs` runs
  `for (...) if (_tasks[i].Process(...) != TaskState.COMPLETE) break;` and never checks `Choice`.
  `DiscoverTask` returns `COMPLETE` once it creates the choice, so later tasks in the list run
  before the pick. Only `AfterChooseTask` runs after it.

## 4. Metastone and Spellsource

- CONFIRMED: Metastone `game/src/main/java/net/demilich/metastone/game/spells/SpellUtils.java`
  `getDiscover`: `return (DiscoverAction) player.getBehaviour().requestAction(context, player, discoverActions);`.
  `DiscoverCardSpell.onCast` then calls `SpellUtils.castChildSpell(context, player, SpellUtils.getDiscover(...).getSpell(), source, target);`.
  https://github.com/demilich1/metastone/blob/master/game/src/main/java/net/demilich/metastone/game/spells/SpellUtils.java
- CONFIRMED: Metastone `game/src/main/java/net/demilich/metastone/game/behaviour/human/HumanBehaviour.java`
  `requestAction` sets `waitingForInput = true`, sends `GameNotification.HUMAN_PROMPT_FOR_ACTION`,
  and loops `while (waitingForInput) { Thread.sleep(BuildConfig.DEFAULT_SLEEP_DELAY); ... }`.
  `onActionSelected` sets `waitingForInput = false`.
- CONFIRMED: the tarter-ian/Spellsource-Server fork's `GameLogic.java` imports
  `co.paralleluniverse.fibers.Suspendable` and has
  `@Suspendable public GameAction requestAction(Player player, List<GameAction> actions)` and
  `@Suspendable public void performGameAction(...)`. https://github.com/tarter-ian/Spellsource-Server
- CONFIRMED: current hiddenswitch/Spellsource
  `spellsource-server/src/main/java/com/hiddenswitch/framework/impl/UnityClientBehaviour.java`:
  `requestAction` creates `Promise.<GameAction>promise()`, adds a `GameplayRequest` with callback
  `promise::tryComplete` to `requests`, calls `onRequestAction(...)`, then
  `return await(promise.future());` (`import static io.vertx.await.Async.await`).
  `processActionForElapsedTurn` throws `"should not process elapsed turn outside of fiber"` unless
  `Thread.currentThread().isVirtual()`. The current `GameLogic.java` has no `@Suspendable`.
- CONFIRMED: Spellsource `SpellUtils.postDiscover`:
  `DiscoverAction discoverAction = (DiscoverAction) context.getLogic().requestAction(player, discoverActions); // We do not perform the game action here`.
  `GameLogic.requestAction`:
  `action = context.getBehaviours().get(player.getId()).requestAction(context, player, actions); context.getTrace().addAction(action);`.
- CONFIRMED: `spellsource-game/.../logic/Trace.java` javadoc: "Given the seed, starting conditions
  and index of each action in the available actions that a player chose, the game will
  reproduce." `replayContext` installs `TraceBehaviour` for both players and runs
  `gameContext.init(); gameContext.resume();`. Discover answers pass through
  `GameLogic.requestAction`, so they are among the recorded indices.

## 5. Hearthstone rules (hearthstone.wiki.gg)

- CONFIRMED (Discover page): "The card chosen by the Discover effect is not placed in your hand
  until the end of the current Sequence…". Example with Raven Idol, Wild Pyromancer and Acolyte of
  Pain on a full hand: "the card chosen will float in the air, Acolyte of Pain triggering off of
  Wild Pyromancer's damage will fill your hand, and finally the card chosen will be discarded."
  https://hearthstone.wiki.gg/wiki/Discover
- CONFIRMED (Discover page): "If the turn timer expires before one of the Discover options is
  picked, a random option will be chosen."
- CONFIRMED (Advanced rulebook): "The moment a new trigger/Event is raised, Hearthstone begins to
  resolve it and immediately begins exploring its consequences… When a trigger/Event is resolved,
  Hearthstone picks up where it left off." Rule 3: "only the outermost Phase ending initiates the
  Death Creation Step." Glossary: "Death is the only Event that is not resolved in the middle of a
  Phase." Spell Text Phase: "the entirety of the spell text takes place in one Phase. All triggers
  to its events are fully resolved." Rule DC3: draw-then-act effects "wait for the card draw to
  resolve before continuing." https://hearthstone.wiki.gg/wiki/Advanced_rulebook
- UNVERIFIED: whether the rest of a spell's text waits for a Discover choice, and whether a trigger
  raised earlier in the spell resolves before or after the pick. Neither page says.

## 6. MTG Comprehensive Rules (yawgatog, effective 2026-09-25)

All CONFIRMED, verbatim from https://yawgatog.com/resources/magic-rules/:

- 608.2c: "The controller of the spell or ability follows its instructions in the order written.
  However, replacement effects may modify these actions. …"
- 608.2d: "If an effect of a spell or ability offers any choices other than choices already made as
  part of casting the spell, activating the ability, or otherwise putting the spell or ability on
  the stack, the player announces these while applying the effect. The player can't choose an
  option that's illegal or impossible…"
- 608.2e: "…the choices for the first action are made in APNAP order, and then the first action is
  processed simultaneously. Then the choices for the second action are made in APNAP order…"
- 603.3: "Once an ability has triggered, its controller puts it on the stack as an object that's
  not a card the next time a player would receive priority."
- 603.3b: "If multiple abilities have triggered since the last time a player received priority, the
  abilities are placed on the stack in a two-part process…" (APNAP).
- 101.4: "If multiple players would make choices and/or take actions at the same time, the active
  player … makes any choices required, then the next player in turn order … Then the actions happen
  simultaneously. … 'Active Player, Nonactive Player (APNAP) order'." 101.4b: "A player knows the
  choices made by the previous players when making their choice…"
- 117.3b: "The active player receives priority after a spell or ability (other than a mana ability)
  resolves."
- 117.5: "Each time a player would get priority, the game first performs all applicable
  state-based actions … Then triggered abilities are put on the stack … Then the player who would
  have received priority does so."

## 7. OpenSpiel (google-deepmind/open_spiel)

All CONFIRMED.

- `spiel_globals.h`: `enum class PlayerId : int { kDefaultPlayerId = 0, kChancePlayerId = -1, kSimultaneousPlayerId = -2, kInvalidPlayer = -3, kTerminalPlayerId = -4, kMeanFieldPlayerId = -5 };`
  https://github.com/google-deepmind/open_spiel/blob/master/open_spiel/spiel_globals.h
- `spiel.h`, `CurrentPlayer()`: "Returns current player. Player numbers start from 0. Negative
  numbers are for chance (-1) or simultaneous (-2). kTerminalPlayerId should be returned on a
  TerminalNode()."
- `LegalActions()`: "returns the actions for the current player (including at chance nodes)."
  `LegalActions(Player)`: "returning an empty list for players who don't act at this state."
- `ApplyActions`: "one per player, for simultaneous action games." `IsSimultaneousNode()`:
  `return CurrentPlayer() == kSimultaneousPlayerId;`. `Clone()`: "Return a copy of this state."
  https://github.com/google-deepmind/open_spiel/blob/master/open_spiel/spiel.h

## Gaps

- XMage: whether `Game.copy()` or AI simulation runs while a human choice is open.
- Forge: the network-play choice path.
- Hearthstone: how a trigger is ordered against an open Discover choice. A client test or HearthSim
  power logs would settle it.
