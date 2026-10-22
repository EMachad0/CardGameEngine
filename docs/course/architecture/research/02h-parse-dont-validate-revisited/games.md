# Games: stale references and rechecking at apply time

Researcher report, session 02h. Question: when an engine lists legal actions and later applies the
one picked, or holds a reference to an entity across changes, how does it keep the earlier check or
reference from going stale?

## Summary

No source trusts the earlier check or the old reference. Either the handle carries a generation or
counter compared again at lookup (West, `slotmap`, Bevy `Entity`, XMage `MageObjectReference`), or
the apply path runs the legality check again. Fireplace, Forge and XMage recheck on apply and fail
softly or loudly. SabberStone is the exception: options built by `Options()` skip the recheck by
default.

## 1. Catherine West (kyren), RustConf 2018

https://kyren.github.io/2018/09/14/rustconf-talk.html

- "if they kept raw pointers, they would be continually invalidated and this tends to lead to hard
  to solve ephemeral bugs and so more or less nobody does this (least, nobody keeps raw pointers /
  references around for "very long")."
- "this is done so that an EntityId is never or very rarely re-used. This way, if an entity goes
  away, generally anything monitoring it will notice it gone rather than some other entity
  immediately taking its place."
- "it's possible that the very next allocated entity will use the same index. This is fine if we
  make sure that there are no outstanding "index references" to this entity before deletion, but
  what if we mess that up? We'll get a "random other entity" in its place without being able to
  tell that it was in fact removed out from under us!"
- "If you delete that index, it will go into a pool of free indexes, so next time you allocate an
  index you might get back another generational index with real index 0, but crucially the
  generation will now be 1. Generational indexes are never re-used because the generation will
  always increment."
- `// Gets the value for some generational index, the generation must match.`
  `pub fn get(&self, index: GenerationalIndex) -> Option<&T>`

## 2. slotmap

https://docs.rs/slotmap/latest/slotmap/

- `sm.remove(key); assert_eq!(sm.get(key), None);`
  (https://docs.rs/slotmap/latest/slotmap/struct.SlotMap.html)
- "once a key is removed, it stays removed, even if the physical storage inside the slotmap is
  reused for new elements." "Only when the stored version and version in a key match is a key
  valid. This allows us to reuse space in the vector after deletion without letting removed keys
  point to spurious new elements."
- "After 2^31 deletions and insertions to the same underlying slot the version wraps around and
  such a spurious reference could potentially occur. It is incredibly unlikely however, and in all
  circumstances is the behavior safe." (Exponent read from a flattened "231".)

## 3. Bevy (docs.rs 0.20.0)

- `Entity`: "The generation is incremented each time an entity with a given index is despawned.
  This serves as a "count" of the number of times a given index has been reused (index, generation)
  pairs uniquely identify a given Entity."
  https://docs.rs/bevy/latest/bevy/ecs/entity/struct.Entity.html. Also: "it is possible for a later
  entity to be spawned at the exact same id! If this happens, which is rare but very possible, it
  will be logged. Aliasing can happen without warning. ... it is generally best practice to stop
  holding an `Entity` ... as soon as you know it has been despawned."
- `QueryEntityError`: `QueryDoesNotMatch(Entity, ArchetypeId)`, `NotSpawned(EntityNotSpawnedError)`,
  `AliasedMutability(Entity)`. "The given `Entity` is not spawned."
  https://docs.rs/bevy/latest/bevy/ecs/query/enum.QueryEntityError.html
- `EntityCommands`: "All deferred entity commands will check whether the entity exists at the time
  of execution and will return an error if it doesn't."
  https://docs.rs/bevy/latest/bevy/prelude/struct.EntityCommands.html
- `Commands::entity` "does not guarantee that commands queued by the returned `EntityCommands` will
  be successful, since the entity could be despawned before they are executed."
  https://docs.rs/bevy/latest/bevy/prelude/struct.Commands.html
- `try_insert`: "If the entity does not exist when this command is executed, the resulting error
  will be ignored." `despawn`: "This will emit a warning if the entity does not exist."
  `clone_with_opt_out`: "The command will panic when applied if the target entity does not exist."
- 0.16 release notes: "Bevy systems, observers, and commands now support returning `Result`".
  `GLOBAL_ERROR_HANDLER` "defaults to panicking, but can be set to anything (log, panic, custom
  logic, etc)." https://bevy.org/news/bevy-0-16/
- Current default handler is inconsistent in the docs: the `Commands` pages say "The fallback error
  handler panics"; `FallbackErrorHandler` "Defaults to `match_severity()`", which "defers to an
  error's `Severity`". https://docs.rs/bevy/latest/bevy/ecs/error/struct.FallbackErrorHandler.html

## 4. Card game engines

Permalinks pin the fetched HEAD commits; line anchors not verified.

### Fireplace (Python)

`fireplace/card.py` @
https://github.com/jleclanche/fireplace/blob/47a2572a000db66645bb74a425a090d51f1004fa/fireplace/card.py

```python
if not self.is_playable():
    raise InvalidAction("%r isn't playable." % (self))
if card.requires_target():
    if not target:
        raise InvalidAction("%r requires a target to play." % (self))
    elif target not in self.play_targets:
        raise InvalidAction("%r is not a valid target for %r." % (target, self))
```

```python
def attack(self, target):
    if not self.can_attack(target):
        raise InvalidAction("%r can't attack %r." % (self, target))
    self.game.attack(self, target)
```

### SabberStone (C#)

@ https://github.com/HearthSim/SabberStone/tree/1db6e2711a13b9061a5de2cd6fc4a00827203d6b (last
commit 2021)

- `PlayCardTask.Process` returns the bool from `Generic.PlayCard(Controller, Source, Target,
  ZonePosition, ChooseOne, SkipPrePhase)`.
- `PlayCardBlock` (`Actions/PlayCard.cs`): `if (!skipPrePhase) if (!PrePlayPhase.Invoke(...))
  return false;`
- `PrePlayPhase`: `if (!source.IsPlayableByPlayer || !subSource.IsPlayableByCardReq ||
  !subSource.IsValidPlayTarget(target)) { return false; }`
- `Controller.Options`: `/// <param name="skipPrePhase">Doesn't check validity of the options
  generated from this when it is processed.</param>`, signature `public List<PlayerTask>
  Options(bool skipPrePhase = true)`. Targeted minion plays pass a literal `true`.
- Attack (`Actions/Attack.cs`): `PreAttackPhase` returns `false` on `!source.CanAttack ||
  !source.IsValidAttackTarget(target)`. `OnAttackTrigger` returns `false` if either side
  `ToBeDestroyed` or has left `Zone.PLAY`, a mid-action staleness check.

### Forge (Java)

@ https://github.com/Card-Forge/forge/tree/ddb4b95510aed6e48e6b534e255f60a8d9961793

- `PlayerControllerHuman.playChosenSpellAbility` calls `PlaySpellAbility.playSpellAbility(this,
  player, chosenSa)` (moved from `HumanPlay` in March 2026,
  https://github.com/Card-Forge/forge/commit/0baf67308574c10dcbfcb9131eb03e71eda9692f).
- `// extra play check` `if (sa.isSpell() && !sa.canPlay()) { ... return false; }`
- `ability.checkRestrictions(player) && (!mayChooseTargets || ability.setupTargets()) &&
  ability.canCastTiming(player) && ability.isLegalAfterStack()`
- On failure: `GameActionUtil.rollbackAbility(ability, fromZone, zonePosition, payment, c)`, then
  `return false`.

### XMage (Java)

`Mage/src/main/java/mage/players/PlayerImpl.java` @
https://github.com/magefree/mage/blob/4d60b894c188644b56c3e5c2026e52aa1393e9ff/Mage/src/main/java/mage/players/PlayerImpl.java

- `activateAbility`: `ActivationStatus activationStatus = ability.canActivate(this.playerId,
  game); if (!activationStatus.canActivate()) { return false; }`
- `cast` recomputes `originalAbility.spellCanBeActivatedNow(getId(), game)`, copies the ability "to
  avoid problems with targets and costs on recast", takes `int bookmark = game.bookmarkState();`
  and on failure `restoreState(bookmark, ability.getRule(), game);` then `return false`.
- `playCard` on failure: `game.informPlayer(this, "You can't play " + card.getIdName() + '.')`.
- `MageObjectReference`, "A object reference that takes zone changes into account": `getPermanent`
  returns the permanent only `if (permanent != null && permanent.getZoneChangeCounter(game) ==
  zoneChangeCounter)`, else `null`.
  https://github.com/magefree/mage/blob/4d60b894c188644b56c3e5c2026e52aa1393e9ff/Mage/src/main/java/mage/MageObjectReference.java

### Rust

- cardinal-kernel 0.1.1 (3 stars): `apply` returns `Result<Vec<Event>, CardinalError>`, checks the
  priority player for `PassPriority`, but `PlayCard` moves the card with no legality check.
  https://docs.rs/crate/cardinal-kernel/latest/source/src/engine/reducer.rs
- phase-rs/phase (216 stars): "pure reducers ... and immutable state"; apply path not found.
  [UNVERIFIED]

## 5. Nystrom, Game Programming Patterns, "Command"

Nothing on validating at execution versus creation. "valid", "invalid" and "legal" don't appear.
Closest: commands "represent a thing that can be done at a specific point in time", and "commands
are one-use-only".
https://raw.githubusercontent.com/munificent/game-programming-patterns/master/book/command.markdown

## Could not confirm

- Line anchors for any permalink.
- Bevy's current default error handler, and the `Severity` of an entity-not-spawned command error.
- phase-rs's apply path; Fireplace internals beyond `card.py`; whether Fireplace's driver calls
  `play()` only after `is_playable()`.
- slotmap's current version number.
