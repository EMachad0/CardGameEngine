# SabberStone and Metastone: how triggers are found and run

Researcher report, session 05. Read from `master` of HearthSim/SabberStone and
demilich1/metastone. Spellsource not needed.

## Summary

SabberStone is push-based: on entering play a trigger adds a delegate to a C# `event` on
`TriggerManager`, and removes it on leaving; matching handlers enqueue tasks on an event-scoped
`TaskQueue`, drained later. Metastone keeps one flat `List<IGameEventListener>`, scans it on every
event, asks each listener whether it is interested, and runs matching spells synchronously (inline
recursion). In both, a deathrattle is read from the dead minion's data at death time, not a
registered trigger. Both check Divine Shield inside the damage function before HP changes. Only
Metastone has loop caps.

## SabberStone

1. Listener source: registry, subscribed on entering play. CONFIRMED. `TriggerManager.cs`: `public
   event TriggerHandler DeathTrigger;`, raised with `DeathTrigger?.Invoke(sender);`.
   `Trigger.Activate` clones per entity and subscribes (`source.Game.TriggerManager.DeathTrigger +=
   instance._processHandler;`; some on the entity: `m.TakeDamageTrigger += ...`,
   `((Minion)source).PreDamageTrigger += ...`), sets `source.ActivatedTrigger = instance`; sequence
   triggers also go to `Game.Triggers` for `ValidateTriggers`. `Trigger.Remove()` unsubscribes
   (`DeathTrigger -= _processHandler;`), sets `_owner.ActivatedTrigger = null`, `_removed = true`.
   Activation: `BoardZone.Add` → `ActivateAura(entity)` → `entity.Power?.Trigger?.Activate(entity);`.
   Removal: `Zone.Remove` does `entity.Zone = null; entity.ActivatedTrigger?.Remove();`. Filtering
   per handler in `Validate(source)` by `TriggerSource` (SELF, FRIENDLY, MINIONS_EXCEPT_SELF, ...)
   plus an optional `Condition`.
2. Silence CONFIRMED. `Minion.Silence()` clears keyword tags (`HasDeathrattle = false;`), then
   `OngoingEffect?.Remove(); ... ActivatedTrigger?.Remove();`, removes enchantments except
   aura-created ones, resets `AttackDamage = Card.ATK;`, sets `IsSilenced = true`. The deathrattle
   isn't subscribed, so clearing the tag disables it.
3. Deathrattles read from the card at death. CONFIRMED (mechanism, order); PARTLY CONFIRMED
   (last-known values). `ToBeDestroyed` adds to `Game.DeadMinions`. `Game.GraveYard()`:
   `DeadMinions.InsertionSort(GetOrderOfPlay);` then per minion `TriggerManager
   .OnDeathTrigger(minion); minion.LastBoardPosition = minion.ZonePosition;
   minion.Zone.Remove(minion); if (minion.HasDeathrattle)
   minion.ActivateTask(PowerActivation.DEATHRATTLE); minion.Controller.GraveyardZone.Add(minion);`.
   `ActivateTask` takes `Power.DeathrattleTask` and enqueues it. All run in the later
   `ProcessTasks()`. Board position kept via `LastBoardPosition`; owner is the dead object's
   current `Controller`; attack is not kept as on the board, because `GraveyardZone.Add` removes
   every applied enchantment before the queued deathrattle runs.
4. Queue by default, inline only with FastExecution. CONFIRMED. `ProcessInternal`: `if
   (FastExecution) Game.TaskQueue.Execute(...) else Game.TaskQueue.Enqueue(...)`; `FastExecution`
   only for `TURN_END` and `WORGEN_TRANSFORM`. `TaskQueue` (in TaskProcessor.cs) holds
   `Stack<Queue<TaskInstance>> _eventStack`; `StartEvent()` pushes, `EndEvent()` pops.
   `Game.ProcessTasks()`: `while (!queue.IsEmpty) { queue.Process() ... }`.
   `DeathProcessingAndAuraUpdate`: `AuraUpdate()`; summon triggers then `ProcessTasks()`;
   `TaskQueue.StartEvent(); do { GraveYard(); // Death Creation Step  ProcessTasks(); // Death
   Resolution Phase } while (DeadMinions.Count != 0); TaskQueue.EndEvent(); AuraUpdate();`. Damage
   triggers drain inside `TakeDamage` (`game.ProcessTasks(); game.TaskQueue.EndEvent();`).
5. Divine Shield in `Character.TakeDamage` before damage. CONFIRMED. `if (minion != null &&
   minion.HasDivineShield) { ... minion.HasDivineShield = false; return 0; }`, after a guard for
   minions not in play, before PreDamage triggers, `IsImmune`, armor and `Damage += amount;`.
6. Loop cap NOT FOUND in Game.cs, TaskProcessor.cs, Trigger.cs, Character.cs.

## Metastone

1. Listener source: flat list scanned on every event. CONFIRMED. `TriggerManager`: `private final
   List<IGameEventListener> triggers`; `addTrigger` warns past 100. `fireGameEvent`: `for
   (IGameEventListener trigger : triggers) { ... if (!trigger.interestedIn(event.getEventType()))
   continue; if (triggers.contains(trigger) && trigger.canFire(event)) eventTriggers.add(trigger);
   }`, then fires the collected ones after re-checking `triggers.contains(trigger)`. Registration:
   `GameLogic.addGameEventListener` (`setHost`, `onAdd`, `context.addTrigger`); `summon` registers
   each `getSpellTriggers()`; weapons, hand (`PASSIVE_TRIGGER`) and deck (`DECK_TRIGGER`) too.
   Unregistration by host via `removeSpellTriggers` → `removeTriggersAssociatedWith(ref,
   removeAuras)`. `SpellTrigger.canFire` checks `primaryTrigger`/`secondaryTrigger.fires(event,
   host)`.
2. Silence CONFIRMED. Fires `SilenceEvent`, strips every attribute outside `immuneToSilence`, then
   `removeSpellTriggers(target);`. Deathrattles live in `Attribute.DEATHRATTLES`, so they go.
3. Deathrattles read from the actor at death, sorted by entity id, resolved inline. CONFIRMED.
   `checkForDeadEntities` sorts `destroyList` by `getId()` (comment: lower ids "entered the game
   earlier"). `destroy(Actor... targets)`: remove the dead actors' triggers but keep auras; add to
   graveyard and record board positions; remove all from the board; per target `destroyMinion`
   (fires `KillEvent` inline) then `resolveDeathrattles(owner, target, boardPositions[i])`; finally
   remove auras. Owner from `target.getOwner()`; attributes not reset, so attack stays readable;
   auras of the dying minions stay active while deathrattles run (PARTLY CONFIRMED for a spell
   reading the source's attack).
4. Inline, no queue. CONFIRMED. `GameContext.fireGameEvent` → `triggerManager.fireGameEvent`
   directly; `SpellTrigger.onFire` casts immediately as a child spell. `checkForDeadEntities()` runs
   after `performGameAction`, after a top-level cast (`!childSpell`), after a battlecry, and at
   `startTurn`, `endTurn`, `equipWeapon`.
5. Divine Shield in `damageMinion`, after PreDamage. CONFIRMED. `damage()`: spell damage, double
   damage, `PreDamageEvent`, then `damageMinion`: `if (minion.hasAttribute(Attribute.DIVINE_SHIELD))
   { removeAttribute(...); return 0; }`, then Immune, then `setHp`. `DamageEvent` only if
   `damageDealt > 0`.
6. Loop caps CONFIRMED. `if (i > 20) { panicDump(); throw new RuntimeException("Infinite death
   checking loop"); }`; `// sanity check to prevent StackOverFlowError with Mistress of Pain +
   Auchenai Soulpriest  if (target.getHp() < -100) { return 0; }`; trigger list: warning only.

## Gaps

- SabberStone loop cap: full-repo search not done.
- Which attack a deathrattle reads: no deathrattle task traced in either engine.
- Metastone id order vs true order of play: unverified beyond one `summon.setId(...)` path.
