# How Forge and XMage let one target be a player or a card (session 04c)

Researcher report, 2026-10-03. Quotes from current `master` (raw files on GitHub), cited by file and member, not pinned to a commit.

## Forge (github.com/Card-Forge/forge, `forge-game/src/main/java/forge/game/`)

- `GameEntity.java`: `public abstract class GameEntity implements GameObject, IIdentifiable {`. `GameObject.java`: `public interface GameObject {`. `player/Player.java`: `public class Player extends GameEntity implements Comparable<Player> {`. `card/Card.java`: `public class Card extends GameEntity implements Comparable<Card>, IHasSVars, ITranslatable {`.
- Spells and abilities are `GameObject`s but not `GameEntity`s: `CardTraitBase implements GameObject, IHasCardView, IHasSVars`, and `SpellAbility extends CardTraitBase`.
- `GameEntity` fields: `protected int id;`, `private String name = "";`, `protected CardCollection attachedCards`, `protected Multiset<CounterType> counters`, `protected List<Pair<Integer, Boolean>> damageReceivedThisTurn`. A player's poison counters and a card's +1/+1 counters share the same structure (`addCounter` is `final` on `GameEntity`).
- Damage is an abstract hook that each subclass implements: `public abstract int addDamageAfterPrevention(final int damage, final Card source, final SpellAbility cause, final boolean isCombat, GameEntityCounterTable counterTable);`. `staticDamagePrevention`, `receiveDamage` and `getAssignedDamage` are shared.
- `canBeTargetedBy` is a default on the `GameObject` interface (`default boolean canBeTargetedBy(final SpellAbility sa) { return false; }`), overridden as `final` in `Player` (checks `hasLost()`, `StaticAbilityCantTarget`) and `Card` (owner in game, phased out, `StaticAbilityCantTarget`).
- Player life is an int: `private int life = 20; private int startingLife = 20;`. `Player.addDamageAfterPrevention` turns infect damage into poison counters.
- Card damage is marked: `private Map<Integer, Integer> damage`, summed by `getDamage()`. `Card.addDamageAfterPrevention` returns 0 unless the card is a planeswalker, creature or battle ("120.1a Damage can't be dealt to an object that's neither a creature nor a planeswalker nor a battle.").
- `spellability/TargetChoices.java`: `public class TargetChoices extends ForwardingList<GameObject>`, holding `FCollection<GameObject> targets`. `add` accepts `Player`, `Card` or `SpellAbility`. Typed views: `getTargetCards()`, `getTargetPlayers()`, `getTargetSpells()`, `getTargetEntities()` (filters by `GameEntity.class`).
- Ids come from separate counters, so they overlap. `Game.java`: `public int nextCardId() { return ++cardIdCounter; }` (cards from 1); players get `plId++` (from 0). `GameEntity.equals` compares class as well as id: `return o.hashCode() == id && o.getClass().equals(getClass());`.

## XMage (github.com/magefree/mage, `Mage/src/main/java/mage/`)

- Targets are UUIDs. `target/Target.java`: `List<UUID> getTargets();`, `void addTarget(UUID id, Ability source, Game game);`, `boolean canTarget(UUID id, Ability source, Game game);`. `TargetImpl`: `protected final Map<UUID, Integer> targets = new LinkedHashMap<>();` (the value is the amount, for divided targets).
- Players and objects both get random UUIDs. `players/PlayerImpl.java`: `protected final UUID playerId;`, set from `UUID.randomUUID()`. `MageObjectImpl.java`: `protected UUID objectId;`, set from `UUID.randomUUID()`. Permanents inherit it (`PermanentImpl extends CardImpl`).
- No shared object type. `MageItem.java`: `public interface MageItem extends Serializable { UUID getId(); }`. `players/Player.java`: `public interface Player extends MageItem, Copyable<Player> {`. `MageObject.java`: `public interface MageObject extends MageItem, Serializable, Copyable<MageObject> {`. `game/permanent/Permanent.java`: `public interface Permanent extends Card, Controllable {`. `Player` and `Permanent` declare parallel `damage(...)` and `canBeTargetedBy(...)` methods. `PlayerImpl` has `protected int life;`, `PermanentImpl` has `protected int damage;`.
- `target/common/TargetAnyTarget.java`: `public class TargetAnyTarget extends TargetPermanentOrPlayer`, with `FilterAnyTarget`; docstring: "Warning, it's a target for damage effects only (ignore lands, artifacts and other non-damageable objects)".
- `target/common/TargetPermanentOrPlayer.java` `canTarget` looks the id up both ways: `Permanent permanent = game.getPermanent(id); Player player = game.getPlayer(id);` then matches the filter against whichever exists. `possibleTargets` builds one `Set<UUID>` from player ids and permanent ids.
- `abilities/effects/common/DamageTargetEffect.java` branches per target:

  ```java
  for (UUID targetId : this.getTargetPointer().getTargets(game, source)) {
      Permanent permanent = game.getPermanent(targetId);
      if (permanent != null) {
          permanent.damage(amount.calculate(game, source, this), source.getSourceId(), source, game, false, preventable);
      } else {
          Player player = game.getPlayer(targetId);
          if (player != null) {
              player.damage(amount.calculate(game, source, this), source.getSourceId(), source, game, false, preventable);
          }
      }
  }
  ```

  `PermanentImpl` then decides by type: creatures mark damage (or -1/-1 counters with wither or infect), planeswalkers lose loyalty counters, battles lose defense counters.

## Not checked

XMage `TargetCreatureOrPlayer` and `FilterAnyTarget`; the life-loss part of Forge `Player.addDamageAfterPrevention`.
