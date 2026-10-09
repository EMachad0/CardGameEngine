# Multi-target drafts: verified rules and engine facts (session 04b)

Researcher report, session 04b. Rules text comes from the hyperlinked Comprehensive Rules at
https://yawgatog.com/resources/magic-rules/, effective September 25, 2026 (hyperlink markup removed,
wording kept). Engine code comes from the `master` branches of Card-Forge/forge and magefree/mage.

## 1. Same object for several "target" words (CR 115.3, 601.2c)

CR 115.3:

> "The same target can't be chosen multiple times for any one instance of the word "target" on a
> spell or ability. If the spell or ability uses the word "target" in multiple places, the same
> object or player can be chosen once for each instance of the word "target" (as long as it fits
> the targeting criteria). This rule applies both when choosing targets for a spell or ability and
> when changing targets or choosing new targets for a spell or ability (see rule 115.7)."

CR 601.2c says the same and gives an example:

> "The same target can't be chosen multiple times for any one instance of the word "target" on the
> spell. However, if the spell uses the word "target" in multiple places, the same object or player
> can be chosen once for each instance of the word "target" (as long as it fits the targeting
> criteria)."
>
> "Example: If a spell says "Tap two target creatures," then the same creature can't be chosen
> twice; the spell requires two different legal targets. A spell that says "Destroy target artifact
> and target land," however, can target the same artifact land twice because it uses the word
> "target" in multiple places."

Across two separate "target" words, the same object can be chosen once for each. Twice for one
word, as in "two target creatures", it can't. Confidence: high.

## 2. "Another target" / "other target"

No CR rule defines how "another target" works. CR 115.4 only lists the phrase among damage
wordings: "Some spells and abilities that refer to damage require "any target," "another target,"
"two targets," or similar rather than "target [something]."" CR 115.7e shows "other" in an example:

> "Arc Trail is a sorcery that reads "Arc Trail deals 2 damage to any target and 1 damage to any
> other target." The current targets of Arc Trail are Runeclaw Bear and Llanowar Elves, in that
> order. You cast Redirect … targeting Arc Trail. You can change the first target to Llanowar Elves
> and change the second target to Runeclaw Bear."

Leeching Bite, Oracle text via the Scryfall API
(https://api.scryfall.com/cards/search?q=o%3A%22another+target+creature+gets+-%22): "Target
creature gets +1/+1 until end of turn. Another target creature gets -1/-1 until end of turn." The
same search returns Steal Strength, Rites of Reaping, Consume Strength, Schismotivate and Rookie
Mistake.

Inference, not a quoted rule: under 115.3 two "target" words could pick the same object, and the
word "another" is card text that overrides it, through CR 101.1: "Whenever a card's text directly
contradicts these rules, the card takes precedence." Confidence: high that no dedicated rule exists;
medium on the 101.1 reasoning.

## 3. Unable to finish choosing targets (CR 601.2, 601.2c, 601.2e, 733.1)

CR 601.2:

> "To cast a spell is to take it from where it is (usually the hand), put it on the stack, and pay
> its costs, so that it will eventually resolve and have its effect. Casting a spell includes
> proposal of the spell (rules 601.2a-d) and determination and payment of costs (rules 601.2f-h).
> To cast a spell, a player follows the steps listed below, in order. A player must be legally
> allowed to cast the spell to begin this process (see rule 601.3). If a player is unable to comply
> with the requirements of a step listed below while performing that step, the casting of the spell
> is illegal; the game returns to the moment before the casting of that spell was proposed (see rule
> 733, "Handling Illegal Actions")."

CR 601.2c, opening:

> "The player announces their choice of an appropriate object or player for each target the spell
> requires. A spell may require some targets only if an alternative or additional cost (such as a
> kicker cost) or a particular mode was chosen for it; otherwise, the spell is cast as though it did
> not require those targets. … If the spell has a variable number of targets, the player announces
> how many targets they will choose before they announce those targets. … Once the number of
> targets the spell has is determined, that number doesn't change, even if the information used to
> determine the number of targets does."

CR 601.2e: "The game checks to see if the proposed spell can legally be cast. If the proposed spell
is illegal, the game returns to the moment before the casting of that spell was proposed (see rule
733, "Handling Illegal Actions")."

CR 733.1: "If a player takes an illegal action or starts to take an action but can't legally
complete it, the entire action is reversed and any payments already made are canceled. No abilities
trigger and no effects apply as a result of an undone action. If the action was casting a spell,
the spell returns to the zone it came from. Each player may also reverse any legal mana abilities…"

601.2c has no rewind clause of its own. The rewind comes from 601.2: failing a step makes the cast
illegal, and the whole cast is undone. Confidence: high.

## 4. Forge card scripts (`forge-gui/res/cardsfolder/`)

(a) "Two target …" is one ability with `TargetMin$ 2 | TargetMax$ 2`. `u/undo.txt`
(https://raw.githubusercontent.com/Card-Forge/forge/master/forge-gui/res/cardsfolder/u/undo.txt):

```
A:SP$ ChangeZone | TargetMin$ 2 | TargetMax$ 2 | ValidTgts$ Creature | Origin$ Battlefield | Destination$ Hand | SpellDescription$ Return two target creatures to their owners' hands.
```

`p/plow_under.txt` follows the same pattern with `ValidTgts$ Land`.

(b) "Target creature … another target creature" is a SubAbility with its own `ValidTgts$` plus
`TargetUnique$ True`. `l/leeching_bite.txt`:

```
A:SP$ Pump | ValidTgts$ Creature | TgtPrompt$ Select target creature to get +1/+1 | TargetUnique$ True | NumAtt$ +1 | NumDef$ +1 | SubAbility$ DBPumpNeg | SpellDescription$ Target creature gets +1/+1 until end of turn. Another target creature gets -1/-1 until end of turn.
SVar:DBPumpNeg:DB$ Pump | ValidTgts$ Creature | TgtPrompt$ Select another target creature to get -1/-1 | TargetUnique$ True | NumAtt$ -1 | NumDef$ -1 | IsCurse$ True
```

`a/arc_trail.txt` uses the same pattern: its `DBDealDamage` sub-ability has
`ValidTgts$ Any | TgtPrompt$ Select any other target (1 damage) | TargetUnique$ True`.

Enforcement, `forge-game/src/main/java/forge/game/spellability/SpellAbility.java`:

```java
if (tr.isUniqueTargets() && getUniqueTargets().contains(entity)) return false;
...
public final List<GameObject> getUniqueTargets() {
    final List<GameObject> targets = Lists.newArrayList();
    SpellAbility child = getParent();
    while (child != null) {
        if (child.usesTargeting()) { targets.addAll(child.getTargets()); }
        child = child.getParent();
    }
    return targets;
}
```

`TargetRestrictions.java`: `if (mapParams.containsKey("TargetUnique")) { setUniqueTargets(true); }`.
The check only looks at targets of parent abilities, so on the root ability `TargetUnique$ True`
does nothing.

Blood Feud's Oracle text says "another target creature", yet Forge scripts it as one ability with
two targets, `b/blood_feud.txt`:

```
A:SP$ Fight | ValidTgts$ Creature | TargetMin$ 2 | TargetMax$ 2 | SpellDescription$ Target creature fights another target creature. ...
```

Mixed kinds: `d/decimate.txt` chains Pump sub-abilities with `ValidTgts$ Artifact`, then
`Creature`, `Enchantment` and `Land`, and ends in `DB$ Destroy | Defined$ Targeted`.

Confidence: high.

## 5. XMage (magefree/mage)

Targets of different kinds are several `addTarget` calls on one ability.
`Mage.Sets/src/mage/cards/d/Decimate.java`:

```java
this.getSpellAbility().addEffect(new DestroyTargetEffect().setTargetPointer(new EachTargetPointer()));
this.getSpellAbility().addTarget(new TargetArtifactPermanent());
this.getSpellAbility().addTarget(new TargetCreaturePermanent());
this.getSpellAbility().addTarget(new TargetEnchantmentPermanent());
this.getSpellAbility().addTarget(new TargetLandPermanent());
```

"Two target creatures" is `TargetCreaturePermanent(2)`. `Mage.Sets/src/mage/cards/u/Undo.java`:

```java
this.getSpellAbility().addTarget(new TargetCreaturePermanent(2));
```

In `Mage/src/main/java/mage/target/common/TargetCreaturePermanent.java`,
`TargetCreaturePermanent(int numTargets)` calls `this(numTargets, numTargets)`, setting the minimum
and the maximum.

"Another target" is `setTargetTag` plus `AnotherTargetPredicate`.
`Mage.Sets/src/mage/cards/l/LeechingBite.java`:

```java
this.getSpellAbility().addEffect(new BoostTargetEffect(1, 1));
this.getSpellAbility().addEffect(new BoostTargetEffect(-1, -1).setTargetPointer(new SecondTargetPointer())...);
this.getSpellAbility().addTarget(new TargetCreaturePermanent().withChooseHint("+1/+1").setTargetTag(1));
this.getSpellAbility().addTarget(new TargetPermanent(StaticFilters.FILTER_ANOTHER_CREATURE_TARGET_2).withChooseHint("-1/-1").setTargetTag(2));
```

`StaticFilters.java`:

```java
FILTER_ANOTHER_CREATURE_TARGET_2 = new FilterCreaturePermanent("another target creature");
FILTER_ANOTHER_CREATURE_TARGET_2.add(new AnotherTargetPredicate(2));
```

Javadoc of `Mage/src/main/java/mage/filter/predicate/other/AnotherTargetPredicate.java`: "All
targets that are already selected in other target definitions of the source are omitted To use this
predicate you have to set the targetTag of all targets involved in the card constructor to a unique
value (e.g. using 1,2,3 for three targets)". `ArcTrail.java` and `BloodFeud.java` use the same
tag-1/tag-2 pattern.

Confidence: high.

## 6. Hearthstone: two chosen targets on one spell

The general rule, https://hearthstone.wiki.gg/wiki/Target: "Players can only choose one target per
action, although that may cause additional effects which apply to characters beyond the target."

Barbed Nets (Voyage to the Sunken City, hunter spell, 1 mana) contradicts it.
https://hearthstone.wiki.gg/wiki/Damage/Wild_format: "Deal 2 damage to an enemy. If you played a
Naga while holding this, choose a second target." Notes on
https://hearthstone.wiki.gg/wiki/Barbed_Nets:

> "You cannot undo nor cancel the first chosen target while assigning the second target."
>
> "You cannot choose the same target twice, e.g. if your opponent has no minions, you can't hit the
> enemy hero twice."

Unconfirmed on the wiki: Earthen Roar ("Set an enemy minion's Health to 1. If you're holding a
Dragon, pick another.", from hs.cardsrealm.com only) and Misfire ("Quickdraw: Choose the targets.",
from a wiki search snippet only). Other such cards may exist. Confidence: high for Barbed Nets, low
for the other two.

## 7. MTG Arena: does clicking a selected target deselect it?

Unconfirmed. No Wizards help article, patch note or official forum post found. The only related
note concerns blocking: "When deselecting a block in a stack, other blockers will remain stacked"
(1.01.00 patch notes, seen only through a mirror at mtgazone.com/1-01-00-patch-notes/).

## Gaps

- Item 7 needs an official source or a direct test in the Arena client.
- Item 6: not every Hearthstone card with two chosen targets was checked.
- How Forge's and XMage's code rewinds a target step that can't be completed was not checked
  (Forge `HumanPlaySpellAbility`, XMage `PlayerImpl` cast flow).
