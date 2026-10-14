# Hearthstone simulators: where the rest of a card lives, and mid-resolution choices

Researcher report, session 06. Sources: `master` of each repo, read as raw files. Nothing was
executed; points from reading the code are marked INFERENCE.

## Summary

None of the three suspends a card mid-resolution and resumes it as one general mechanism.
Fireplace lets the Python call stack unwind when a choice opens and parks follow-up work on the
choice action: `.then()` callbacks in `_callback`, closures in `choice_callback`; `choose()` replays
them. SabberStone keeps the rest of a card's task list on the C# call stack (`StateTaskList`'s
loop) and doesn't stop at a Discover; work that must wait for the pick is data in
`Choice.AfterChooseTask`, and its `TaskQueue` is a stack of FIFO queues, one per event. Metastone
blocks the thread inside the Discover; everything lives on the Java call stack.

## 1. Fireplace (jleclanche/fireplace)

1. CONFIRMED, `fireplace/game.py`: `queue_actions` runs the actions at once
   (`trigger_actions`: `for action in actions: ... ret.append(action.trigger(source))`).
2. CONFIRMED: an action block is a depth counter, deaths are processed when it returns to 0, and
   PLAY blocks are not counted:
   ```python
   def action_end(self, type, source):
       ...
       if type != BlockType.PLAY:
           self._action_stack -= 1
       if not self._action_stack:
           self.log("Empty stack, refreshing auras and processing deaths")
           self.refresh_auras()
           self.process_deaths()
   ```
   INFERENCE: the first non-PLAY block inside a play processes deaths when it closes, while
   `Play.do` is still running.
3. CONFIRMED, `actions.py` `Action.broadcast` opens a TRIGGER block and walks every entity;
   `entity.trigger_event` ends in `source.game.trigger(self, actions, args)`, a nested block.
4. CONFIRMED, `Discover.do`: `player.choice = self; self._callback = self.callback; self.callback = []`.
5. CONFIRMED, `Discover.choose`: clears `player.choice`, runs each stored callback in a new
   `game.trigger(...)`, restores `self.callback`, then `self.trigger_choice_callback()`.
6. CONFIRMED, `TargetedAction._trigger`:
   ```python
   if source.controller.choice:
       self.choice_callback.append(lambda: self._trigger(i, source))
       return []
   ```
   Added in commit c36ac87 ("fix choice mulitimes bug", 2023-12-05), tested by `test_volcanosaur`.
   https://github.com/jleclanche/fireplace/commit/c36ac87410466c840ee8e1357bb07567edf67e5d.diff
7. INFERENCE: in a list `[Discover(...), Buff(...)]`, the Buff's closure goes on the Buff's own
   `choice_callback`, which nothing drains; non-targeted actions in the list run at once. The
   supported pattern is `.then(...)` on the choice action. `Play.do` also keeps going after the
   battlecry and runs the AFTER play and summon broadcasts before the player has chosen.
8. CONFIRMED: `Player.__init__` has `self.choice = None`, one slot per player; `is_playable` and
   `can_attack` return `False` while `self.controller.choice` is set (`card.py`).
9. CONFIRMED, `utils.py` `play_turn`: `while player.choice: ... player.choice.choose(choice)`.
10. No `deepcopy` in the core files searched. Wiki "AI notes": "Deep-copying of the game state
    performance is very slow. We need a fast copy mechanism". INFERENCE: a mid-choice deepcopy
    would keep closures that point at the original game, since Python's `deepcopy` returns
    functions unchanged (https://docs.python.org/3/library/copy.html).

## 2. SabberStone (HearthSim/SabberStone)

11. CONFIRMED, `Model/TaskProcessor.cs` (the live class; `Model/TaskQueue.cs` is commented out):
    ```csharp
    private readonly Stack<Queue<TaskInstance>> _eventStack;
    ...
    public void StartEvent() { _eventStack.Push(CurrentQueue); CurrentQueue = null; }
    public void EndEvent()   { CurrentQueue = _eventStack.Pop(); }
    ```
12. CONFIRMED: `Process` peeks, runs, dequeues, and saves and restores `CurrentTask`, so it nests.
13. CONFIRMED, `Game.ProcessTasks` drains only the current event's queue.
14. CONFIRMED: a card's power is one queued task (`Playable.ActivateTask` ends with
    `Game.TaskQueue.Enqueue(...)`), and its sub-tasks run in `StateTaskList.Process`'s `for` loop,
    which breaks on any result other than `COMPLETE`. `TaskStack` is a scratch register
    (`Playables`, `Flag`, `Number..Number4`) passed between sibling tasks, not a continuation.
15. CONFIRMED for damage: `Character.TakeDamage` wraps its triggers in
    `StartEvent`/`ProcessTasks`/`EndEvent` ("Death phase and aura update are not emerge here"), so
    they resolve inside the power before its next sub-task. Heal, after-summon and discard do the
    same. A plain `Trigger.ProcessInternal` enqueues into the current queue behind the running
    power, so it runs after the whole power. Which trigger types use which path is UNVERIFIED.
16. CONFIRMED: death processing only at phase boundaries. `DeathProcessingAndAuraUpdate` loops
    `do { GraveYard(); ProcessTasks(); } while (DeadMinions.Count != 0);` inside an event; damage
    only sets `ToBeDestroyed`. `PlayMinion` and `PlaySpell` call it after the power's
    `ProcessTasks`/`EndEvent`.
17. CONFIRMED: `DiscoverTask.Process` creates the choice and returns `COMPLETE`, so the list keeps
    running; the post-pick work is `Choice.AfterChooseTask` ("the task that must be done after the
    entity is chosen"), chained by `NextChoice` when a choice is already pending.
18. CONFIRMED: `ChooseTask.Process` runs `Generic.ChoicePick` (which applies the `ChoiceAction`
    and runs `AfterChooseTask`), then `Game.ProcessTasks(); Game.DeathProcessingAndAuraUpdate();`.
19. CONFIRMED: `Game`'s copy constructor ends with `TaskQueue = new TaskQueue(this);` (empty), and
    `Controller` copies `Choice = controller.Choice?.Clone(this);`, sharing `AfterChooseTask`.
    INFERENCE: cloning is safe while waiting for a choice, not in the middle of `ProcessTasks`.

## 3. Metastone (demilich1/metastone)

20. CONFIRMED: `castChildSpell` is a recursive `castSpell(..., true)`; `MetaSpell.onCast` loops
    over its child spells.
21. CONFIRMED: `castSpell` checks deaths only for a top-level spell card
    (`if (sourceCard != null && ... isCardType(CardType.SPELL) && !childSpell) { ... checkForDeadEntities(); ... }`);
    `performGameAction` checks after any action but a battlecry.
22. CONFIRMED: `checkForDeadEntities(int i)` destroys the dead sorted by id, resolves deathrattles
    inside `destroy`, and recurses with `i + 1`, throwing "Infinite death checking loop" above 20.
23. CONFIRMED: triggers fire inline in `fireGameEvent`; `SpellTrigger.onFire` casts as a child
    spell, so no death check runs inside a trigger.
24. CONFIRMED: Discover blocks inside the spell (`HumanBehaviour.requestAction` polls with
    `Thread.sleep`).
25. CONFIRMED that `GameContext.clone()` exists. INFERENCE: it can't capture an in-flight spell.

## 4. Where "the rest of the current card" lives

- Fireplace: the host call stack, and at a choice the `.then()` list plus closures on the action
  objects. Later siblings in the same list are not reliably continued.
- SabberStone: between whole tasks, an explicit `Stack<Queue<TaskInstance>>`; inside a power, the
  host stack; after a choice, `Choice.AfterChooseTask` as data, which survives `Game.Clone`.
- Metastone: the host call stack only.

## Gaps

- Fireplace: no repo-wide `deepcopy` search, no run of a `[Discover, Buff]` list.
- SabberStone: which `TriggerManager.On*` sites nest and which enqueue behind the power.
- Metastone: whether an AI clones from inside `requestAction`.
- Quotes from `master`, not pinned to commits.
