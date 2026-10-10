# Forge and XMage: how triggers are found, look back in time, replacements

Researcher report, session 05. Read from current `master` of Card-Forge/forge and magefree/mage.

## Summary

Forge keeps a cached list (`TriggerHandler.activeTriggers`), patched on zone changes and rebuilt in
full from every card's current traits at the end of `checkStateEffects`, plus a rebuild with LTB
triggers from LKI copies after a simultaneous zone change. XMage keeps a registry
(`GameState.triggers`, a `TriggeredAbilities` LinkedHashMap): a card's own triggers are added once
when the card is loaded and are not removed on zone change or ability loss. On each event,
`isInUseableZone` / `hasSourceObjectAbility` checks the current object, or its LKI, for the ability
and the zone.

## Forge

1. Listener source CONFIRMED (cached list, patched and rebuilt). `TriggerHandler.java`:
   `private final List<Trigger> activeTriggers`, `delayedTriggers`, `waitingTriggers`. Full
   rebuild: `resetActiveTriggers(...)` does `activeTriggers.clear(); game.forEachCardInGame(c -> {
   for (final Trigger t : c.getTriggers()) {... if (isTriggerActive(t)) activeTriggers.add(t);`
   (`isTriggerActive` checks `zonesCheck(game.getZoneOf(host))`, `isSuppressed()`, phase, and
   removes duplicates). Patches: `registerActiveTrigger(c, onlyExtrinsic)`, `registerOneTrigger(t)`,
   `clearActiveTriggers(c, zoneFrom)` (removes entries where `!c.getTriggers().contains(t) ||
   !t.zonesCheck(zoneFrom)`). Callers: `GameAction.changeZone` (`clearActiveTriggers(copied, null);
   registerActiveTrigger(copied, false);`), transform in Card.java, `checkStateEffects` after its
   SBA loop (`resetActiveTriggers();`), `CardZoneTable.triggerChangesZoneAll`
   (`resetActiveTriggers(false, lastStateBattlefield)`). Events queue in `waitingTriggers` while the
   stack is frozen; `collectTriggerForWaiting()` freezes the candidates per event
   (`wt.setTriggers(getActiveTrigger(wt.getMode(), wt.getParams()))`), and `runWaitingTrigger` uses
   `wasCollected ? wt.getTriggers() : activeTriggers`.
   https://github.com/Card-Forge/forge/blob/master/forge-game/src/main/java/forge/game/trigger/TriggerHandler.java
2. Losing abilities CONFIRMED (re-derived from current traits). `Card.getTriggers()` returns
   `currentState.getTriggers()`; `Card.updateTriggers` applies layer-6 changes
   (`ck.applyTrigger(list)`); `CardTraitChanges.applyTrigger` does `list.removeIf(remove)`;
   `RemoveAllAbilities$ True`. The next `resetActiveTriggers` yields nothing for that card, and
   `clearActiveTriggers` drops entries not in `c.getTriggers()`. (The `remove` predicate's builder
   not traced.)
3. Look back in time CONFIRMED. `Trigger.looksBackInTime()` is true for `Destroyed`,
   `Sacrificed`, `Exploited`, and `ChangesZone`/`ChangesZoneAll` with Battlefield in `Origin`.
   `CardZoneTable.triggerChangesZoneAll` runs `resetActiveTriggers(false, lastStateBattlefield)`
   (skips look-back triggers of cards in play now but not in the last state), then `for (Card lki :
   lastStateBattlefield) { registerActiveLTBTrigger(lki); }`, so a dying Blood Artist's trigger is
   live through its LKI copy. `TriggerChangesZone.performTest`: `// CR 603.10a leaves battlefield or
   GY look back in time ... moved = (Card) runParams.get(AbilityKey.CardLKI);`. Snapshot:
   `Game.copyLastState()` (`lastStateBattlefield.addAll(p.getZone(ZoneType.Battlefield)
   .getLKICopy(cachedMap))`); where it is called: NOT FOUND.
4. LKI on resolution PARTLY CONFIRMED. `Game.addChangeZoneLKIInfo(lki)` into
   `Table<Integer, Long, Card> changeZoneLKIInfo`; `getChangeZoneLKIInfo(c)` returns
   `Objects.requireNonNullElse(changeZoneLKIInfo.get(c.getId(), c.getGameTimestamp()), c)`.
   `AbilityUtils`: `// CR 608.2h ... c = game.getChangeZoneLKIInfo(c);`. `Defined$ Self` returns
   `hostCard` directly.
5. Trigger order CONFIRMED. `runSingleTriggerInternal` → `addSimultaneousStackEntry(wrapperAbility)`.
   `MagicStack.addAllTriggeredAbilitiesToStack` loops `game.getPlayersInTurnOrder(playerTurn)`
   (`// CR 603.3b`) → `chooseOrderOfSimultaneousStackEntry(p, ...)` →
   `activePlayer.getController().orderAndPlaySimultaneousSa(activePlayerSAs)`.
6. Replacement effects CONFIRMED. Consulted before the change: in `GameAction.changeZone`,
   `ReplacementResult repres = game.getReplacementHandler().run(ReplacementType.Moved, repParams);`
   comes before `zoneTo.add(...)`. `ReplacementHandler.run`: decider is `(Player) affected` or
   `((Card) affected).getController()`; loops `for (ReplacementLayer layer :
   ReplacementLayer.values())`; `chosenRE = decider.getController()
   .chooseSingleReplacementEffect(possibleReplacers)`; applies once via `chosenRE.setHasRun(true);
   hasRun.add(chosenRE);` (excluded from `getReplacementList` after); on `Updated` it calls
   `run(event, params)` again on the modified event; flags reset afterwards. Damage has its own path
   (`replaceDamageList`, `executedDamageMap`).

## XMage

1. Listener source CONFIRMED (persistent registry, filtered at check time). `GameState.java`:
   `private TriggeredAbilities triggers; // all normal triggers`, `DelayedTriggeredAbilities
   delayed`. On card load, `addCard` → `for (Ability ability : card.getInitAbilities()) {
   addAbility(ability, null, card); }` → `addTrigger` → `triggers.add(ability, attachedTo)`, for
   every zone. Dispatch: `handleEvent` runs `watchers.watch(event, game); delayed.checkTriggers(event,
   game); triggers.checkTriggers(event, game);`. `TriggeredAbilities.checkTriggers` iterates all
   values: `checkEventType`, then `ability.isInUseableZone(game, object, event)` and
   `ability.checkTrigger(event, game)`; a hit calls `ability.trigger(...)` →
   `state.addTriggeredAbility` (the `triggered` list). Removal: `removeTriggersOfSourceId` ("used if
   a token leaves the battlefield"), `removeAbilitiesOfNonExistingSources`, and `GameState.reset()`
   (every `applyEffects`) does `triggers.removeAllGainedAbilities()`, so gained triggers are wiped
   and re-added each time effects apply.
2. Losing abilities CONFIRMED (registry not pruned; checked at event time).
   `PermanentImpl.removeAllAbilities` only does `abilities.clear();`, with `// TODO: what about
   triggered abilities? See addAbility above -- triggers adds to GameState`.
   `AbilityImpl.isInUseableZone` starts with `if (!this.hasSourceObjectAbility(game, sourceObject,
   event)) return false;`, and `hasSourceObjectAbility` does `if (!object.hasAbility(this, game)) {
   return false; }`.
3. Look back in time CONFIRMED. LTB triggers call `setLeavesTheBattlefieldTrigger(true)`. Blood
   Artist uses `DiesThisOrAnotherTriggeredAbility` → `isInUseableZoneDiesTrigger(...)`. In
   `TriggeredAbilityImpl`: `if (game.checkShortLivingLKI(affectedSourceId, Zone.BATTLEFIELD)) {
   affectedObject = game.getLastKnownInformation(affectedSourceId, Zone.BATTLEFIELD); }`, then `if
   (!sourceAbility.hasSourceObjectAbility(game, affectedObject, event)) return false;`. Generic
   `isInUseableZone` does the same for ZONE_CHANGE, DESTROYED_PERMANENT, SACRIFICED_PERMANENT,
   EXPLOITED_CREATURE when `isLeavesTheBattlefieldTrigger()`. `TriggeredAbilities.checkTrigger` sets
   the controller from the LKI permanent for LTB triggers. `GameImpl.rememberLKI` stores a copy and
   adds the id to `lkiShortLiving`; `resetShortLivingLKI()` clears it at the end of resolution.
4. LKI on resolution CONFIRMED. `AbilityImpl.getSourcePermanentOrLKI(game)`:
   `getSourcePermanentIfItStillExists`, else `game.getLastKnownInformation(getSourceId(),
   Zone.BATTLEFIELD, getStackMomentSourceZCC())`. `GameImpl.getPermanentOrLKIBattlefield`.
   `getLastKnownInformation(id, zone[, zcc])` reads the `lki` map or the per-zone-change-counter
   `lkiExtended`; `resetLKI()` clears them "if all effects are applied and stack is empty".
5. Trigger order CONFIRMED. `GameImpl.checkTriggered()`: `for (UUID playerId :
   state.getPlayerList(state.getActivePlayerId()))` (APNAP); each player repeatedly
   `player.chooseTriggeredAbility(abilities, this)` and `player.triggerAbility(ability, this)`.
6. Replacement effects CONFIRMED. Consulted before the change: `if (!tapped &&
   !replaceEvent(EventType.TAP, game)) { this.tapped = true; ...}`. `GameState.replaceEvent` checks
   `preventedByRuleModification`, then `ContinuousEffects.replaceEvent` loops: recompute
   `getApplicableReplacementEffects` (skipping effects in `event.getAppliedEffects()`, `// Effect
   already applied to this event, ignore it`), drop `consumed`, if more than one remains
   `player.chooseReplacementEffect(...)` (`//20100716 - 616.1c`), apply, add to
   `event.getAppliedEffects()`, stop if `caught`.

## Gaps

- Forge: where `copyLastState()` is called; how a resolving trigger reads its controller after the
  source left (`WrappedAbility`).
- XMage: where `rememberLKI` is called in a zone move (`ZonesHandler`/`moveCards`);
  `HumanPlayer.chooseTriggeredAbility` auto-ordering.
