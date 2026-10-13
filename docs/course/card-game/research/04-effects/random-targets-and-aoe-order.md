# Random targets, empty target sets and AoE order: verified engine facts (session 04a)

Researcher report, session 04a. Sources: hearthstone.wiki.gg, HearthstoneJSON build 25770, and the
`master` branches of demilich1/metastone and HearthSim/SabberStone.

## 1. Hearthstone: random effect with no valid target, and Deadly Shot

- A random battlecry with no targets does nothing. Bomb Lobber ("Battlecry: Deal 4 damage to a
  random enemy minion"), Notes: "If the Bomb Lobber is played when the opponent has no minions on
  the board, his Battlecry will have no effect." https://hearthstone.wiki.gg/wiki/Bomb_Lobber
- A random trigger with no targets does nothing either. Advanced rulebook: "Knife Juggler will hit
  no one as there is no valid target, preventing you from being killed." The same page lists random
  damage effects such as Knife Juggler and Arcane Missiles under "Ignores mortally wounded/pending
  destroy", so a minion already at 0 health is not a valid target.
  https://hearthstone.wiki.gg/wiki/Advanced_rulebook
- Deadly Shot has the play requirement. HearthstoneJSON build 25770, EX1_617:
  `"id":"EX1_617","name":"Deadly Shot","playRequirements":{"REQ_MINIMUM_ENEMY_MINIONS":1},...,"text":"Destroy a random enemy minion.","type":"SPELL"`
  https://api.hearthstonejson.com/v1/25770/enUS/cards.collectible.json
  - In the same build, Mad Bomber (EX1_082) and Arcane Missiles (EX1_277) have no
    `playRequirements` field.
- The wiki pages Random, Deadly Shot and Deadly Shot (Classic) don't state the requirement.
- No single wiki sentence states a general "random effect fizzles" rule. The rule comes from
  per-card examples.

Verdict: verified. A random effect with no valid target does nothing. Deadly Shot carries
`REQ_MINIMUM_ENEMY_MINIONS: 1`, so it can't be played while the enemy board is empty. Battlecry
minions like Bomb Lobber can still be played, and the effect does nothing.

## 2. Metastone

EntityReference constants, from
https://github.com/demilich1/metastone/blob/master/game/src/main/java/net/demilich/metastone/game/targeting/EntityReference.java:

- `NONE(-1)`, `ENEMY_CHARACTERS(-2)`, `ENEMY_MINIONS(-3)`, `ENEMY_HERO(-4)`, `FRIENDLY_CHARACTERS(-5)`, `FRIENDLY_MINIONS(-6)`
- `OTHER_FRIENDLY_MINIONS(-7)`, `ADJACENT_MINIONS(-8)`, `FRIENDLY_HERO(-9)`, `ALL_MINIONS(-10)`, `ALL_CHARACTERS(-11)`, `ALL_OTHER_CHARACTERS(-12)`
- `ALL_OTHER_MINIONS(-13)`, `FRIENDLY_WEAPON(-14)`, `ENEMY_WEAPON(-15)`, `FRIENDLY_HAND(-16)`, `ENEMY_HAND(-17)`, `OPPOSITE_MINIONS(-18)`
- `LEFTMOST_FRIENDLY_MINION(-19)`, `LEFTMOST_ENEMY_MINION(-20)`, `FRIENDLY_PLAYER(-21)`, `ENEMY_PLAYER(-22)`, `MINIONS_TO_LEFT(-23)`, `MINIONS_TO_RIGHT(-24)`
- `TARGET(-30)`, `SPELL_TARGET(-31)`
- `EVENT_TARGET(-40)`, `SELF(-41)`, `KILLED_MINION(-42)`, `ATTACKER_REFERENCE(-43)`, `PENDING_CARD(-44)`, `EVENT_CARD(-45)`

Any negative key is a group: `public boolean isTargetGroup() { return key < 0; }`. Real entities
are referenced by their positive id.

"A random X" is a boolean on top of a group reference.

- `SpellArg.java` contains `RANDOM_TARGET,`. In card JSON it is written `"randomTarget"`.
- `Spell.java`, `cast(...)`:

  ```java
  List<Entity> validTargets = SpellUtils.getValidTargets(context, player, targets, targetFilter);
  // there is at least one valid target and the RANDOM_TARGET flag is set,
  // pick one randomly
  if (validTargets.size() > 0 && desc.getBool(SpellArg.RANDOM_TARGET)) {
      Entity target = SpellUtils.getRandomTarget(validTargets);
      castForPlayer(context, player, desc, source, target);
  } else {
      ...
      for (Entity target : validTargets) { ... castForPlayer(context, player, desc, source, target); ... }
  }
  ```

- `SpellUtils.java`:

  ```java
  public static <T> T getRandomTarget(List<T> targets) {
      int randomIndex = ThreadLocalRandom.current().nextInt(targets.size());
      return targets.get(randomIndex);
  }
  ```

Card JSON, `cards/classic/hunter/spell_deadly_shot.json`:

```json
"targetSelection": "NONE",
"spell": { "class": "DestroySpell", "target": "ENEMY_MINIONS", "randomTarget": true },
"condition": { "class": "MinionCountCondition", "targetPlayer": "OPPONENT", "operation": "GREATER", "value": 0 }
```

Card JSON, `cards/classic/neutral/minion_mad_bomber.json`:

```json
"spell": { "class": "MissilesSpell", "target": "ALL_OTHER_CHARACTERS", "value": 1, "howMany": 3, "randomTarget": true }
```

Empty target list: nothing is cast. With an empty list the `size() > 0` guard sends execution to
the `else` loop, which runs over an empty list, so `onCast` is never called. `getRandomTarget`,
whose `nextInt(0)` would throw, is never reached. If `targets == null` (no target given at all),
the spell is cast once with a null target. Deadly Shot's playability gate is the card-level
`condition`: `SpellCard.java` `canBeCast` ends with
`if (condition != null) { return condition.isFulfilled(context, player, null, null); }`.

Verdict: verified. A group `EntityReference` plus `"randomTarget": true` expresses "a random X".
An empty list casts nothing. Metastone models the play requirement as a card `condition`.

## 3. SabberStone

Bomb Lobber, `SabberStoneCore/src/CardSets/GvgCardsGen.cs`:

```csharp
cards.Add("GVG_099", new CardDef(new Power {
    PowerTask = ComplexTask.Create(
        new RandomTask(1, EntityType.OP_MINIONS),
        new DamageTask(4, EntityType.STACK))
}));
```

Deadly Shot, `SabberStoneCore/src/CardSets/Standard/Expert1CardsGen.cs`:

```csharp
// PlayReq:
// - REQ_MINIMUM_ENEMY_MINIONS = 1
cards.Add("EX1_617", new CardDef(new Dictionary<PlayReq, int>() {{PlayReq.REQ_MINIMUM_ENEMY_MINIONS,1}}, new Power {
    PowerTask = ComplexTask.DestroyRandomTargets(1, EntityType.OP_MINIONS) }));
```

`SabberStoneCore/src/Tasks/ComplexTasks.cs`:

```csharp
public static ISimpleTask DestroyRandomTargets(int targets, EntityType type) => Create(
    new IncludeTask(type),
    new FilterStackTask(SelfCondition.IsNotDead),
    new RandomTask(targets, EntityType.STACK),
    new DestroyTask(EntityType.STACK));
public static ISimpleTask Create(params ISimpleTask[] list) { return StateTaskList.Chain(list); }
```

`SabberStoneCore/src/Tasks/SimpleTasks/RandomTask.cs`:

```csharp
IList<IPlayable> entities = IncludeTask.GetEntities(in _type, in controller, source, target, stack?.Playables);
if (entities.Count == 0)
    return TaskState.STOP;
if (entities.Count <= _amount) { stack.Playables = entities; return TaskState.COMPLETE; }
```

`SabberStoneCore/src/Tasks/StateTaskList.cs`:

```csharp
for (int i = 0; i < _tasks.Length; ++i)
    if (_tasks[i].Process(...) != TaskState.COMPLETE)
        break;
State = TaskState.COMPLETE;
return TaskState.COMPLETE;
```

STOP breaks the chain, so the following `DamageTask` or `DestroyTask` never runs, and the chain
still reports COMPLETE.

Verdict: verified. "Random enemy minion" is `RandomTask(1, EntityType.OP_MINIONS)` followed by a
task on `EntityType.STACK`. On an empty set `RandomTask` returns `STOP`, which skips the rest of
the chain. Deadly Shot is also gated by `PlayReq.REQ_MINIMUM_ENEMY_MINIONS`.

## 4. Hearthstone AoE order

Advanced rulebook, https://hearthstone.wiki.gg/wiki/Advanced_rulebook:

- "Rule DH1: When an area of effect Damage or Healing effect happens, all of the Damage/Healing
  Events are created in play order, then all of the Damage/Healing Events are resolved (queuing and
  resolving triggers) in play order."
- "Order of play means the order the Entities each Event/trigger is associated with entered play,
  from oldest to newest."
- "(Your Heroes are older than every minion. The Hero of the Dominant Player is older than the Hero
  of the Secondary Player.)"
- Rule DH3 exception: "Some Damage effects sound like they should be area of effect ... but
  actually resolve each Damage Event as soon as it is created in play order. Some examples of these
  are: Lightning Storm, Elemental Destruction, Dark Iron Skulker, Lightbomb."

Verdict: verified for play order (oldest entity first), with all damage events created before any
resolves. "By entity id" is not verified: the rulebook defines the order as order of entering play.
Hellfire isn't named in the quoted text. It falls under DH1 as a standard AoE and isn't in the DH3
list.

## Gaps

- No single wiki sentence states a general "random effect with no valid target fizzles" rule.
- Not checked whether today's client still enforces `REQ_MINIMUM_ENEMY_MINIONS` on Deadly Shot.
  Newer HearthstoneJSON builds dropped `playRequirements`.
