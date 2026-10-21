# Trigger architecture in card engines

Researcher report, session 02f. Per engine: how triggers are found per event (scan or registry),
what an effect may access, and how deaths are sequenced.

## Summary

All five engines give an effect the whole game: Sabberstone and the Metastone family pass it a
`Game`/`GameContext`, Fireplace reaches it through `source.game`, and XMage passes `Game`. None of
the code read gives a command or effect narrowed access to state. On finding triggers, the engines
split. Fireplace scans every entity on every event. Sabberstone keeps a per-event-type registry: a
trigger subscribes when its minion enters the board and unsubscribes on silence or replace.
Metastone and Spellsource keep one flat list that every event walks in full. XMage registers all
abilities of a card once, when the card enters the game, and filters by zone at check time. It also
drops and rebuilds granted triggers on every effects pass.

## Findings

### 1. Sabberstone (C#)

1. **Registry built from delegate subscriptions.** `Trigger.Activate` creates a per-entity
   instance and adds its handler to an event on `Game.TriggerManager` (`DeathTrigger +=`,
   `DamageTrigger +=` and so on). Some triggers subscribe to a per-minion event instead, such as
   `TakeDamageTrigger` on the minion itself. `Trigger.Remove()` reverses each subscription.
   [Trigger.cs](https://raw.githubusercontent.com/HearthSim/SabberStone/master/SabberStoneCore/src/Triggers/Trigger.cs)
2. **Activated when the minion enters the board.** `BoardZone.Add` sets
   `OrderOfPlay = Game.NextOop` and then calls `ActivateAura`, which runs
   `Power?.Trigger?.Activate(entity)`. `Replace` calls `oldEntity.ActivatedTrigger?.Remove()`.
   [BoardZone.cs](https://raw.githubusercontent.com/HearthSim/SabberStone/master/SabberStoneCore/src/Model/Zones/BoardZone.cs)
   - Silence also calls `ActivatedTrigger?.Remove()`.
     [Minion.cs](https://raw.githubusercontent.com/HearthSim/SabberStone/master/SabberStoneCore/src/Model/Entities/Minion.cs)
   - Unverified: where an ordinary death removes the trigger. `BoardZone.Remove` only removes
     auras.
3. **Filters run when the event fires.** `Validate` checks a `TriggerSource` (FRIENDLY, ENEMY,
   SELF, MINIONS_EXCEPT_SELF…) and a `Condition`. Triggers tied to a play sequence are
   pre-validated at the start of the sequence by `ValidateTriggers`. A matched trigger enqueues its
   `SingleTask` on `Game.TaskQueue`, or runs it at once when `FastExecution` is set.
4. **Tasks get the whole game.** The signature is `ISimpleTask.Process(in Game game, in Controller
   controller, in IEntity source, in IPlayable target, in TaskStack stack)`.
   [SimpleTask.cs](https://raw.githubusercontent.com/HearthSim/SabberStone/master/SabberStoneCore/src/Tasks/SimpleTask.cs)
5. **Deaths.** `DeathProcessingAndAuraUpdate` runs an aura update, then the summon triggers, then
   the loop `do { GraveYard(); ProcessTasks(); } while (DeadMinions.Count != 0)`, then another aura
   update. `GraveYard` first sorts `DeadMinions` by `OrderOfPlay`. For each dead minion it fires
   `OnDeathTrigger(minion)`, removes the minion from its zone, enqueues its deathrattle and moves
   it to the graveyard. Death events are created one per minion, but the tasks are processed
   together after the batch.
   [Game.cs](https://raw.githubusercontent.com/HearthSim/SabberStone/master/SabberStoneCore/src/Model/Game.cs)
6. **History.** The 2.0.0 changelog says: "Entirely new Trigger / Aura / Enchantment system, which
   uses events and the actual ENCHANTMENT entities". Unverified: what the earlier design was.
   [Changelog](https://github.com/HearthSim/SabberStone/wiki/Changelog)

### 2. Fireplace (Python): pure scan

1. `Action.broadcast` walks `game.entities`, then every hand, then every deck. For each entity,
   `_broadcast` loops over `entity.events` and checks `event.trigger.matches(...)`.
   [actions.py](https://raw.githubusercontent.com/jleclanche/fireplace/master/fireplace/actions.py)
2. `game.entities` is sorted by `play_counter`.
   [game.py](https://raw.githubusercontent.com/jleclanche/fireplace/master/fireplace/game.py)
3. **Triggers depend on the zone, computed when read.** `events` returns `scripts.Hand.events` in
   hand, `Deck.events` in deck, and otherwise `base_events + list(self._events)`. Dormant minions
   and secrets override it.
   [card.py](https://raw.githubusercontent.com/jleclanche/fireplace/master/fireplace/card.py)
4. Silence wipes `target._events = []`.
5. **Deaths.**
   - `process_deaths` triggers `Death(self.live_entities)`, and `action_end` calls
     `refresh_auras()` and `process_deaths()` when the action stack empties.
   - `Death._broadcast` queues the dying minion's `Deathrattle` once the broadcast loop passes an
     entity with a higher `play_counter`, citing issue #126. The researcher reads this as
     interleaving deathrattles with other on-death triggers in order of play.
6. **Access.** `do(self, source, ...)` reaches everything through `source.game`.

### 3. Metastone and Spellsource (Java): flat list, scanned on every event

1. **Metastone.** `TriggerManager` holds a `List<IGameEventListener>`. `fireGameEvent` walks the
   whole list, filters with `interestedIn(eventType)` and `canFire`, then calls `onGameEvent`
   synchronously. A code comment reads "double check here if the trigger still exists; after all,
   a previous trigger may have removed it".
   [TriggerManager.java](https://raw.githubusercontent.com/demilich1/metastone/master/game/src/main/java/net/demilich/metastone/game/spells/trigger/TriggerManager.java)
2. **Metastone registration.** `summon` and `equipWeapon` call `addGameEventListener` for each
   `SpellTrigger`. `silence` calls `removeSpellTriggers`.
   [GameLogic.java](https://raw.githubusercontent.com/demilich1/metastone/master/game/src/main/java/net/demilich/metastone/game/logic/GameLogic.java)
3. **Metastone deaths are batched.** `checkForDeadEntities` sorts by entity id ("lower id entered
   the game earlier") and calls `destroy`, then recurses, with a hard limit of 20 passes. `destroy`
   removes non-aura triggers, moves all the dead to the graveyard, fires `KillEvent` for each one
   and resolves its deathrattles, and removes auras last. Spells get the full context:
   `spell.cast(context, player, spellDesc, source, targets)`.
4. **Spellsource.** `fireGameEvent` takes "a snapshot of the triggers currently in play", queues
   those where `interestedIn(...) && queues(event)`, and pushes event data and the host reference
   onto context stacks. A trigger that is already processing goes to
   `getDeferredTriggersQueue()`. Recursion is capped at `triggerDepth > 96`.
   [GameLogic.java](https://raw.githubusercontent.com/hiddenswitch/Spellsource/develop/spellsource-game/src/main/java/net/demilich/metastone/game/logic/GameLogic.java)
   - `removeEnchantments` scans every trigger hosted by the entity and calls `expire`.
   - Silence calls `removeEnchantments(target, true, true, true)`.
   - Death removes non-aura enchantments first and auras at the end. Aftermath triggers activate
     only while their host is in the graveyard.

### 4. XMage: a registry that is filtered when checked

1. `GameState.addCard` loops over `card.getInitAbilities()` and calls `addAbility`. Triggered
   abilities go into `triggers`. This happens when the card enters the game, not when it changes
   zone.
   [GameState.java](https://raw.githubusercontent.com/magefree/mage/master/Mage/src/main/java/mage/game/GameState.java)
2. `TriggeredAbilities` is a `LinkedHashMap`; the code notes that ordering is required "to make
   same result for all game instances". `checkTriggers` copies `values()` and iterates every entry,
   calling `checkEventType`, then `isInUseableZone(game, object, event)`, then `checkTrigger`. For
   leaves-the-battlefield triggers it uses last known information. A registry of every printed
   ability, scanned in full, with zone filtering per check.
   [TriggeredAbilities.java](https://raw.githubusercontent.com/magefree/mage/master/Mage/src/main/java/mage/abilities/TriggeredAbilities.java)
3. **Granted triggers are rebuilt every pass.** Before `applyEffects`, `reset()` calls
   `triggers.removeAllGainedAbilities()` ("to prevent adding it multiple times"). Continuous
   effects then add them back with `add(ability, sourceId, attachedTo)`.
   `removeAbilitiesOfNonExistingSources` cleans up after tokens.
4. `handleEvent` runs `watchers.watch`, then `delayed.checkTriggers`, then
   `triggers.checkTriggers`.

### 5. Official and near-official sources

1. **MTG Arena (official Wizards article).** The GRE is C++ plus CLIPS. Per-card "CLIPS rules"
   match on a shared "whiteboard" while the core engine "naps", and "the GRE neither knows nor cares
   how many CLIPS rules came along and modified the contents".
   [Wizards](https://magic.wizards.com/en/news/mtg-arena/on-whiteboards-naps-and-living-breakthrough)
   - The same article describes a bug. A trigger stored the spell's ID, and a later lookup read the
     mana value the object had on the battlefield. The fix was to store the value itself.
   - Inference, unverified: CLIPS is a production-rule system, so this is pattern matching rather
     than listener registration.
2. **Hearthstone, community rulebook with developer citations.**
   - "A Queue is created and filled with all triggers that can respond, in order of play." It
     "becomes immutable once Hearthstone starts to resolve the first entry".
   - Most conditions are checked when the trigger resolves, not when it is queued.
   - Rule 4a: an aura update, then the Death Creation Step, which kills every mortally wounded
     entity simultaneously, then a second aura update.
   - The game is processed server-side and has a priority layer over order of play.
   - [Advanced rulebook](https://hearthstone.wiki.gg/wiki/Advanced_rulebook)
3. **Hearthstone protocol (HearthSim).** The game state is "a bucket of entities", each a store of
   integer tags. The server sends nested `BLOCK`s such as `TRIGGER` and `DEATHS`, and replays are
   built from this power history. [HearthSim](https://hearthsim.info/docs/gamestate-protocol/)
4. **Hearthstone dominant-player bug.** When both players' triggers respond to one event, they are
   ordered by player, then by order of play.
   [hs-bugs #29](https://github.com/HearthSim/hs-bugs/issues/29)
5. **Legends of Runeterra.** From a search snippet only: "Almost all code - both client and server
   - is written in C#". Article body not loaded.
   [Riot](https://www.riotgames.com/en/news/bringing-features-life-legends-runeterra)
6. **Marvel Snap.** Public material covers only the AWS serverless backend, not the rules engine.
   [GDC](https://www.gdcvault.com/play/1029205/Launching-MARVEL-SNAP-on-Mobile)

### 6. Keeping a registry consistent, and the bugs it causes

1. **XMage #8426.** `ApplyEffects` ran during mana activation and "recreate[d] triggers… and it
   broke original triggers iterator so it continue to random trigger" (Sasaya's Essence produced
   too much mana). [Issue](https://github.com/magefree/mage/issues/8426)
   - The code now carries opt-in integrity checks. Two are marked "impossible to fix due dynamic
     triggers gen": `MustKeepSameTriggersList` and `EventMustNotProcessNewTriggers`.
2. **Common defences against changing the list while iterating it:** XMage and Spellsource copy
   the list before walking it, and Spellsource also defers re-entrant triggers. Metastone re-checks
   `triggers.contains`. Sabberstone sets a `_removed` flag and returns early in `Process`.
3. **Death needs a split teardown in the registry engines.** Metastone and Spellsource remove
   non-aura triggers before deathrattles and auras after them, because auras must outlive the
   death step.
4. **Scan to registry.** Sabberstone moved to an event registry in 2.0.0; the earlier design is
   unverified. No engine found that moved from a registry back to a scan.

## Gaps

- Where Sabberstone removes a dead minion's trigger, and its design before 2.0.
- How Spellsource registers an enchantment when a card enters play.
- Anything first-party on the internals of Blizzard's, Riot's or Second Dinner's rules engines.
- The CLIPS/Rete reading of the GRE is the researcher's inference.
