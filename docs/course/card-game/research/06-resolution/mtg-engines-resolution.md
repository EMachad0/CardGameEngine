# MTG engines: how a multi-part resolution is structured, and where choices sit

Researcher report, session 06. Sources: `master`/`main` source of each engine at fetch time.

## Summary

Magarena, Forge and XMage run a spell's parts as nested Java calls. Magarena never pauses inside a
part: a `MagicEvent`'s choices are fixed in an `Object[]` before its action runs, and a choice that
depends on something revealed during resolution becomes a follow-up `MagicEvent` the action
queues. The event queue is copied with the game, so its MCTS AI copies the game at every pending
choice, mid-resolution included. Forge and XMage ask inside the effect code and block the game
thread for a human; their search AIs copy only at priority or stack boundaries (XMage says so in a
TODO). Two newer engines store the in-progress resolution as data: Argentum (Kotlin) with a
`continuationStack` of `ContinuationFrame`s, and phase-rs (Rust) with a serializable
`ResolutionStack` of `ResolutionFrame`s.

## 1. Magarena

1. CONFIRMED, `src/magic/model/MagicGame.java`: `executeNextEvent(final Object[] choiceResults) { doAction(new ExecuteFirstEventAction(choiceResults)); }`
   and the loop
   `advanceToNextEventWithChoice() { while (!isFinished()) { if (!hasNextEvent()) { executePhase(); } else if (!getNextEvent().hasChoice()) { executeNextEvent(); } else { return true; } } ...`.
   `ExecuteFirstEventAction` does `firstEvent = game.getEvents().removeFirst(); game.executeEvent(firstEvent, choiceResults);`
   and its undo re-adds the event.
2. CONFIRMED, `src/magic/model/event/MagicEvent.java`:
   `public final void executeEvent(final MagicGame game,final Object[] choiceResults) { chosen = choiceResults; chosenTarget = getLegalTarget(game); ... payManaCost(game); action.executeEvent(game,this); }`.
   Accessors read the stored `chosen` (`getCardChoice() { return (MagicCardChoiceResult)chosen[0]; }`).
   The action never calls out for input.
3. CONFIRMED, `src/magic/model/choice/MagicChoice.java`: `getArtificialChoiceResults` ("Gets the
   choice results for AI"), `getPlayerChoiceResults(final IUIGameController controller, ...)`,
   `getSimulationChoiceResult(...)`. `MagicChoice.NONE` is not valid, so `hasChoice()` is false.
4. CONFIRMED, `src/magic/model/stack/MagicItemOnStack.java`:
   `public void resolve(final MagicGame game) { final MagicEvent resolveEvent = getEvent(); if (resolveEvent.isValid(game, choiceResults)) { game.executeEvent(resolveEvent, choiceResults); } }`.
5. CONFIRMED by example, `src/magic/model/trigger/CascadeTrigger.java`: the action exiles cards,
   then `game.addEvent(new MagicEvent(event.getSource(), event.getPlayer(), new MagicMayChoice("Cast " + nonland + " without paying its mana cost?"), exiled, CAST_ACTION, ...))`;
   `CAST_ACTION` chains a third event (`RESTORE_CARDS`). Data passes through the event's `ref`.
   `AddEventAction` appends to the event queue (`addLast`) and undoes with `removeLast()`.
6. CONFIRMED: the copy constructor copies the queue
   (`events=new MagicEventQueue(copyMap, game.events); stack=new MagicStack(copyMap, game.stack); pendingStack=...`),
   and `MCTSAI.findNextEventChoiceResults` copies the game and asks
   `event.getArtificialChoiceResults(aiGame)` for the next event.
7. CONFIRMED, `SwingGameController.java`: the game loop runs on its own thread; a human choice is
   `event.getChoice().getPlayerChoiceResults(this,game,event)`, which blocks in
   `waitForInputOrUndo() { ... return input.take(); }`. It blocks before
   `ExecuteFirstEventAction`, so the state is always at an event boundary while waiting.
8. CONFIRMED: an immediate trigger whose event has a choice is ignored:
   `if (immediate) { if (event.hasChoice()) { // ignore } else if (trigger.usesStack()) { ... } else { executeEvent(event, MagicEvent.NO_CHOICE_RESULTS); } }`.

## 2. Forge

9. CONFIRMED, `AbilityUtils.java`: sub-abilities chain by recursion
   (`resolveApiAbility` then `resolveSubAbilities`, which calls
   `game.getAction().checkStaticAbilities();` then `resolveApiAbility(abSub, game);`).
   `AbilitySub.resolve()` is `effect.resolve(this);`.
10. CONFIRMED, `DigEffect.java`: choices are synchronous calls inside the effect
    (`chooser.getController().chooseSingleEntityForEffect(...)`, `chooseEntitiesForEffect(...)`).
11. CONFIRMED: `MagicStack.resolveStack()` freezes the stack ("freeze the stack while we're in the
    middle of resolving"), and `TriggerHandler.runTrigger` adds to `waitingTriggers` while
    `game.getStack().isFrozen()`.
12. CONFIRMED: `MagicStack.finishResolving` unfreezes and runs the waiting triggers into
    simultaneous stack entries; `PhaseHandler.checkStateBasedEffects()` loops
    `do { ... checkStateEffects ... } while (game.getStack().addAllTriggeredAbilitiesToStack());`
    before priority, APNAP ("CR 603.3b").
13. CONFIRMED: `GameCopier` copies board state; the stack only when `GameSimulator.COPY_STACK`, and
    then only spells.

## 3. XMage

14. CONFIRMED, `AbilityImpl.resolveMode`: `for (Effect effect : getEffects()) { ... effect.apply(game, this) ... game.processAction(); }`.
    `processAction()`'s javadoc: "All restrained trigger events are fired now".
15. CONFIRMED: `HumanPlayer.waitForResponse` blocks the game thread on `response.wait()`.
16. CONFIRMED: the AI (`ComputerPlayer7` / `ComputerPlayer6`, module `Mage.Player.AI.MAD`) copies at
    priority (`createSimulation`) and resolves a stack object as one step. `ComputerPlayer6`:
    `// resolve ability can call choose dialogs and do not use simulation, TODO: implement it here?`
    and `// TODO: implement choices simulation?`.

## 4. Engines storing in-progress resolution as data

17. CONFIRMED, Argentum (`wingedsheep/argentum-engine`, Kotlin): `Continuation.kt`:
    `/** Stored execution stack entries. A [Suspension] consumes a player's answer; [AutomaticContinuation] resumes work by its position in the stack. */ @Serializable sealed interface ContinuationFrame`.
    `Suspension.kt`: `data class Suspension internal constructor(val question: PendingDecision, val answer: AnswerContinuation) : ContinuationFrame`.
    `EffectContinuationRunner.kt` pushes the rest before running effect i:
    `val remainingContinuation = EffectContinuation(remainingEffects = stillRemaining, effectContext = currentContext); currentState.pushContinuation(remainingContinuation)`,
    and threads the context (`storedCollections`, `chosenValues`). `ContinuationHandler.resume`
    checks `suspension.question.id != response.decisionId`, pops, and drains automatic frames.
    gym-trainer README: "`TrainerContext` — the state, the acting player, and the engine's
    `PendingDecision` (non-null when mid-decision)", MCTS via `GameEnvironment.fork()`.
18. CONFIRMED, phase-rs (`phase-rs/phase`, Rust), `crates/engine/src/types/resolution.rs`:
    `//! Typed, serializable suspension frames for ability resolution.`
    `pub enum ResolutionFrame { AbilityContinuation(AbilityContinuationFrame), RepeatFor(PendingRepeatIteration), ..., OptionalEffect(OptionalEffectFrame), CoinFlip(PendingCoinFlip), DieRoll(Box<PendingDieRoll>), ..., SpellResolution(PendingSpellResolution), ... }`.
    `PendingDieRoll` has `pub next_index: usize`, "the index into `results` at which the per-die
    results-table loop resumes ... instead of restarting at zero". Gates:
    `FrameGate::DirectChoice(...)` for frames that own a prompt, `FrameGate::AfterChild` for parents
    waiting on a child. `game_state.rs` imports `ResolutionStack` and `WaitingFor`. Migration is
    partial: "Migrated families use [`ResolutionStack`] as their runtime authority; unmigrated
    families remain in their legacy `GameState` slots".
19. CONFIRMED from a docs.rs search snippet only: manabrew-engine (a Rust port of Forge) has
    `pub fn resolve_stack(&mut self, game: &mut GameState, agents: &mut [Box<dyn PlayerAgent>])`.

## Gaps

- Forge's human blocking path in this report (covered by the first session 06 report).
- Magarena's `MagicEventQueue` copy constructor not read; scripted "reveal and choose" cards not checked.
- Where XMage holds triggers raised during `resolveMode`.
- phase-rs's resume driver and `PendingContinuation` payload not read.
