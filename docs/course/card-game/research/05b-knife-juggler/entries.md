# Entering play and "after another enters" triggers (research for Knife Juggler)

## Summary

In Hearthstone each summoned minion gets its own After Summon Phase. When one effect summons
several minions, Knife Juggler fires once for each one, in the order they are summoned. For a
minion played from hand, Knife Juggler fires after that minion's Battlecry. "Whenever you summon"
triggers fire before the Battlecry, or at the end of the outer Phase. MTG checks all permanents
after an event, newcomers included, so creatures that enter together see each other. Yu-Gi-Oh!
works the same way: monsters Summoned at the same time can trigger on each other. Fireplace and
SabberStone both fire "after summon" triggers right after each minion, not once after all of them.

## Findings

### 1. Hearthstone ([Advanced rulebook](https://hearthstone.wiki.gg/wiki/Advanced_rulebook))

**1a. Several minions from one effect: Knife Juggler fires once per minion, immediately.**

- Rule 3 example: *"You play Knife Juggler then Dr. Boom. When Dr. Boom's Battlecry (a Phase)
  begins, Boom Bots are summoned, and each one triggers the Knife Juggler to throw knives at enemy
  minions. Even though minion summoning is a Sequence of multiple Phases, we are already inside of
  a Phase, so the entire Battlecry remains inside of one Phase, rather than starting and stopping.
  Finally when the Battlecry resolves, Deaths are processed and the Deathrattles go off."*
- Summon Resolution Step section: *"You play a Starving Buzzard, two Knife Jugglers, a second
  Starving Buzzard then Unleash the Hounds. Each hound summoning triggers the two Knife Jugglers in
  order of play, then finally when the Spell Text Phase ends, for each hound summoned, the two
  Starving Buzzards trigger in order of play."* It adds: *"Snake Trap works the same way."* and
  *"Using Savannah Highmane's Deathrattle has a similar result."*
- Same section: *"You play two Undertakers, a Knife Juggler and Dr. Boom. For each Boom Bot
  summoned, the Knife Juggler triggers, then at the end of the Battlecry Phase, the Undertakers
  trigger in order of play for each Boom Bot during the 'Summon Resolution Step'. Finally, the
  Knife Juggler throws a knife for Dr. Boom's summon."* So the Boom Bots fire Knife Juggler during
  Dr. Boom's Battlecry, before Dr. Boom's own After Summon trigger.
- Footnote 388: *"Played Starving Buzzard then Knife Juggler; opponent then activated Snake Trap.
  Knife Juggler triggered from each Snake, then Buzzard drew three times once all had been
  summoned."*
- A minion summoned but not played: *"If a minion is merely summoned rather than played, it is
  inside of a larger Phase, so we are currently not processing Deaths or checking for
  win/loss/draw. The Sequence is now far simpler: 1. The card is generated … and enters Play,
  creating a Summon Event and becoming interactible. 2. After Summon Phase"*
- Rule 5: *"If you play two Knife Jugglers and then Unleash the Hounds, no enemy will be reduced to
  lower than 0 Health by a knife, because Knife Juggler is a damaging trigger."* Minions are not
  removed mid-Phase, but knives skip targets already at 0 Health or less ("mortally wounded").

**Can a Juggler fire for other minions summoned by the same effect?** No rulebook example has two
Jugglers from one effect (N'Zoth, resurrect). Two rules let you infer the answer, **not verified
in-game**:

- *"Minions … summoned while resolving an Event cannot respond to Events that are in the middle of
  resolution, because their associated Queues are now immutable. However, they can Queue on any
  later Phases and Events created."*
- *"After Patch 9.2.0.21517, triggers in a Sequence that read 'After ...' can only work if they're
  valid when the Sequence starts."*

Each summon is its own Sequence. Juggler #1 is already in play when Juggler #2's Sequence starts,
so #1 should fire for #2 and every later minion, and #2 should not fire for #1. A
[2017 Reddit report](https://www.reddit.com/r/hearthstone/comments/6j59t9/til_a_knife_juggler_will_trigger_its_effect_when/)
says a Juggler summoned by Barnes's Battlecry fired for Barnes. It predates Patch 9.2.0.21517.
**Unverified.**

**1b. Order when a minion is played from hand** ("Playing/summoning a minion"):

1. *"The card is removed from your hand and its Mana cost is paid. The card enters Play as a
   minion, creating a Summon Event and becoming interactible…"*
2. *"On Play Phase: All 'whenever' triggers on playing a card/playing a minion, such as Hobgoblin,
   … Queue and resolve here."*
3. *"(Because of the Summon Resolution Step, all 'whenever' triggers on summoning a minion, such as
   Starving Buzzard, One-eyed Cheat, Undertaker and more Queue and resolve for the played minion
   here.)"*
4. *"Battlecry Phase: … the on-play keyword, such as Battlecry/Combo/Choose One/Manathirst is fully
   resolved here."*
5. *"After Play & After Summon Phase: First all 'after' triggers on playing a card/playing a
   minion, such as Mirror Entity, … Queue and resolve. Then all 'after' triggers on summoning a
   minion, such as Ship's Cannon, Knife Juggler, and Sword of Justice Queue and resolve."*
6. *"Hearthstone checks for win/loss/draw."*

The sequence *"ends early if the minion leaves play (such as due to death)"*.

The [Knife Juggler notes template](https://hearthstone.wiki.gg/wiki/Template:Knife_Juggler_notes)
says Knife Juggler activates after *"Any 'whenever' triggers, such as Hobgoblin."*, *"Any
Battlecry of the summoned minion has fully resolved, including all deaths (and any resulting
consequences, including Deathrattles) caused by the Battlecry."* and *"Any Secrets triggered in
response to a minion being played, such as Mirror Entity or Snipe."* It also says: *"If the minion
being summoned dies during any earlier phase in the summoning process, then the summoning process
is cut short and thus the Knife Juggler does not activate."*

**Does Knife Juggler fire on its own summon?** No. No wiki sentence says so outright. It follows
from the Patch 9.2 rule and from Fireplace's explicit guard (section 4). Well established but
inferred.

**"Whenever you summon" vs "After you summon":** "Whenever" summon triggers run in the Summon
Resolution Step: *"Whenever a minion enters play … a 'Summon Event' is created, but not resolved.
Instead, during the Summon Resolution Step, in order of play, we resolve each Summon Event"*. That
step runs at the end of the outermost Phase, *"after the first Aura Update (Health/Attack) but
before Death Creation Step."* For a played minion that is before its Battlecry; for minions
summoned inside an effect, after all of them have arrived. "After" triggers fire inside each
minion's own After Summon Phase. The [Triggered effect](https://hearthstone.wiki.gg/wiki/Triggered_effect)
page quotes Ben Brode (2014): *"If the summoned minion has a Battlecry, this will take place prior
to the activation of any triggered effects or Secrets."*

### 2. MTG Comprehensive Rules ([yawgatog](https://yawgatog.com/resources/magic-rules/), effective September 25, 2026 as printed)

- **603.6a:** *"Enters-the-battlefield abilities trigger when a permanent enters the battlefield.
  These are written, 'When [this object] enters, . . .' or 'Whenever a [type] enters, . . .' Each
  time an event puts one or more permanents onto the battlefield, all permanents on the
  battlefield (including the newcomers) are checked for any enters-the-battlefield triggers that
  match the event."*
- **603.6d:** *"Some permanents have text that reads '[This permanent] enters with . . . ,' 'As
  [this permanent] enters . . . ,' '[This permanent] enters as . . . ,' or '[This permanent]
  enters tapped.' Such text is a static ability--not a triggered ability--whose effect occurs as
  part of the event that puts the permanent onto the battlefield."*
- **603.10:** *"Normally, objects that exist immediately after an event are checked to see if the
  event matched any trigger conditions…"*
- **603.3:** a triggered ability goes on the stack *"the next time a player would receive
  priority."*
- **608.3a:** a permanent spell with no targets *"becomes a permanent and enters the battlefield
  under the control of the spell's controller."* So a creature spell's ETB trigger goes on the
  stack after the spell has resolved.
- **"Another":** Soul Warden's Oracle text is *"Whenever another creature enters, you gain 1
  life."* WotC ruling (2025-01-24): *"If this creature enters at the same time as one or more other
  creatures, its ability will trigger for each of those other creatures."*
  ([Scryfall rulings](https://api.scryfall.com/cards/d96266b3-a7cb-40ce-a328-ac13719fe5f0/rulings))

### 3. Yu-Gi-Oh!

- Monsters Summoned at the same time see each other.
  [Catoblepas and the Witch of Fate rulings](https://yugipedia.com/wiki/Card_Rulings:Catoblepas_and_the_Witch_of_Fate):
  *"Q: If 'Catoblepas and the Witch of Fate' and an opponent's Spellcaster monster are Special
  Summoned at the same time by 'Magician's Circle', can you activate the effect…? A: Even in this
  case, you can activate the effect… The opponent's Spellcaster monster that was Special Summoned
  at the same time as 'Catoblepas and the Witch of Fate' is destroyed."*
- A trigger fires once per summon event, not once per monster.
  [Summoning Curse rulings](https://yugipedia.com/wiki/Card_Rulings%3ASummoning_Curse): *"If more
  than 1 monster is Summoned at the same time, this effect only triggers once…"*

### 4. Simulators

**Fireplace** ([actions.py](https://github.com/jleclanche/fireplace/blob/master/fireplace/actions.py)):

- `Summon._broadcast`: `# Prevent cards from triggering off their own summon` /
  `if entity is args[1]: return`
- `Summon.do` loops `for card in cards:`, sets `card.zone = Zone.PLAY`, then queues the ON
  broadcast (`self.queue_broadcast(...)`) and fires the AFTER broadcast at once
  (`self.broadcast(source, EventListener.AFTER, target, card)`), per card.
- The queued ON broadcasts run in `TargetedAction.trigger` after all repetitions
  (`self.resolve_broadcasts()`), matching the Summon Resolution Step.
- `Play.do` resolves the Summon ON broadcast before queuing the `Battlecry`, and calls
  `summon_action.broadcast(player, EventListener.AFTER, player, played_card)` after the
  Battlecry.
- A Juggler summoned first in a multi-summon is already in PLAY and receives the AFTER broadcast
  for the later cards.

**SabberStone** ([Summon.cs](https://github.com/HearthSim/SabberStone/blob/master/SabberStoneCore/src/Actions/Summon.cs)):

- `SummonBlock` = `SummonPhase.Invoke(...)` (board add, `g.AuraUpdate()`,
  `g.SummonedMinions.Add(minion)`), then `AfterSummonTrigger.Invoke(...)`, which calls
  `g.TriggerManager.OnAfterSummonTrigger(minion); g.ProcessTasks();`
- [SummonTask.cs](https://github.com/HearthSim/SabberStone/blob/master/SabberStoneCore/src/Tasks/SimpleTasks/SummonTask.cs)
  runs `SummonBlock` once per minion, so "after summon" triggers run fully for each minion in turn.
- [Game.cs](https://github.com/HearthSim/SabberStone/blob/master/SabberStoneCore/src/Model/Game.cs)
  `DeathProcessingAndAuraUpdate`: `// Summon Resolution Step` iterates `SummonedMinions` calling
  `TriggerManager.OnSummonTrigger(...)`, then the Death Creation Step.
- How SabberStone stops a Juggler triggering on itself was not verified (trigger source 404).

## Gaps

- No source covers one effect summoning two or more Knife Jugglers. "The earlier one fires for
  later ones, not the other way round" is inferred from the rulebook and Patch 9.2.0.21517, and
  matches Fireplace.
- "Doesn't fire on itself" has no explicit wiki sentence; inferred, plus Fireplace code.
- The Barnes-into-Juggler result predates Patch 9.2.0.21517.
