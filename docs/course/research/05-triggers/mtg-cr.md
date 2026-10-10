# MTG Comprehensive Rules: triggers, replacements, statics, last known information

Researcher report, session 05. Source: the official CR TXT, "These rules are effective as of
September 25, 2026" (https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt, linked
from https://magic.wizards.com/en/rules), spot-checked against yawgatog
(https://yawgatog.com/resources/magic-rules/, anchors `#R6032` style).

## Summary

Rule numbers mostly as expected. Corrections: 616.1 runs a to f (plus g), not a to e; 603.3b is now
a two-part ordering process; no rule in 604 or 611 literally says static abilities "don't use the
stack" (closest is 117.2b).

## Findings

1. 603.2 CONFIRMED. "Whenever a game event or game state matches a triggered ability's trigger
   event, that ability automatically triggers. The ability doesn't do anything at this point."
   603.2g: "An ability triggers only if its trigger event actually occurs. An event that's prevented
   or replaced won't trigger anything."
2. 603.3 CONFIRMED. "Once an ability has triggered, its controller puts it on the stack as an object
   that's not a card the next time a player would receive priority." 603.3b (changed, two-part):
   "First, each player, in APNAP order, puts each triggered ability they control with a trigger
   condition that isn't another ability triggering on the stack in any order they choose. …
   Second, each player, in APNAP order, puts all remaining triggered abilities they control on the
   stack in any order they choose. Then the game once again checks for and performs state-based
   actions until none are performed, then abilities that triggered during this process go on the
   stack. This process repeats until no new state-based actions are performed and no abilities
   trigger."
3. 603.4 CONFIRMED. "When the trigger event occurs, the ability checks whether the stated condition
   is true. The ability triggers only if it is… If the ability triggers, it checks the stated
   condition again as it resolves. If the condition isn't true at that time, the ability is removed
   from the stack and does nothing." (Felidar Sovereign example.)
4. 603.6 / 603.10 / 603.10a CONFIRMED. 603.6: "During resolution, these abilities look for the
   object in the zone that it moved to. If the object is unable to be found in the zone it went to,
   the part of the ability attempting to do something to the object will fail to do anything."
   603.10: "objects that exist immediately after an event are checked to see if the event matched
   any trigger conditions, and continuous effects that exist at that time are used to determine what
   the trigger conditions are and what the objects involved in the event look like." Exceptions:
   "the game 'looks back in time' … using the existence of those abilities and the appearance of
   objects immediately prior to the event." 603.10a: "leaves-the-battlefield abilities, abilities
   that trigger when a player sacrifices a permanent, abilities that trigger when a card leaves a
   graveyard, and abilities that trigger when an object that all players can see is put into a hand
   or library." 603.10b to g: phasing out, becoming unattached, losing control, a spell being
   countered, a player losing the game, planeswalking away. Example: "Two creatures are on the
   battlefield along with an artifact that has the ability 'Whenever a creature dies, you gain 1
   life.' Someone casts a spell that destroys all artifacts, creatures, and enchantments. The
   artifact's ability triggers twice, even though the artifact goes to its owner's graveyard at the
   same time as the creatures." 603.6c: "Whenever [something] is put into a graveyard from the
   battlefield" is a leaves-the-battlefield ability.
5. 608.2h CONFIRMED. "the effect uses the current information of that object if it's in the public
   zone it was expected to be in; if it's no longer in that zone, or if the effect has moved it from
   a public zone to a hidden zone, the effect uses the object's last known information." Continues:
   "If an ability states that an object does something, it's the object as it exists—or as it most
   recently existed—that does it, not the ability." 604.7: "static abilities can't use an object's
   last known information."
6. 614.1 / 614.5 / 614.6 CONFIRMED. 614.1: "Such effects watch for a particular event that would
   happen and completely or partially replace that event with a different event." 614.1a: "Effects
   that use the word 'instead' are replacement effects." 614.1b: "skip" effects too. 614.5: "A
   replacement effect doesn't invoke itself repeatedly; it gets only one opportunity to affect an
   event or any modified events that may replace that event." (Exceptions 717.6, 903.9b.) 614.6:
   "If an event is replaced, it never happens. A modified event occurs instead, which may in turn
   trigger abilities."
7. 616.1 CONFIRMED, steps a to f plus g. "the affected object's controller (or its owner if it has
   no controller) or the affected player chooses one to apply… If two or more players have to make
   these choices at the same time, choices are made in APNAP order." a: self-replacement effects;
   b: under whose control an object enters; c: enters as a copy; d: enters back face up (each "one
   of them must be chosen"). e: "Any of the applicable replacement and/or prevention effects may be
   chosen." f: "Once the chosen effect has been applied, this process is repeated (taking into
   account only replacement or prevention effects that would now be applicable) until there are no
   more left to apply." g: nested events, "the second effect can't be chosen until after the first
   effect has been chosen."
8. 615 PARTLY CONFIRMED. Prevention is a sibling of replacement, not a subtype. 615.1: "Some
   continuous effects are prevention effects. Like replacement effects (see rule 614), prevention
   effects apply continuously as events happen… Such effects watch for a damage event that would
   happen and completely or partially prevent the damage." 615.1a: "Effects that use the word
   'prevent' are prevention effects." 616 orders both kinds together.
9. 604.1 / 611.3 PARTLY CONFIRMED. 604.1: "Static abilities do something all the time rather than
   being activated or triggered. They are written as statements, and they're simply true." 604.2:
   "These effects are active as long as the permanent with the ability remains on the battlefield
   and has the ability, or as long as the object with the ability remains in the appropriate zone."
   611.3a: "A continuous effect generated by a static ability isn't 'locked in'; it applies at any
   given moment to whatever its text indicates." 611.3b: "The effect applies at all times that the
   permanent generating it is on the battlefield or the object generating it is in the appropriate
   zone." 117.2b: "Static abilities continuously affect the game. Priority doesn't apply to them."
10. 704.3 / 117.5 CONFIRMED. 704.3: "the game checks for any of the listed conditions for
    state-based actions, then performs all applicable state-based actions simultaneously as a
    single event. If any state-based actions are performed as a result of a check, the check is
    repeated; otherwise all triggered abilities that are waiting to be put on the stack are put on
    the stack, then the check is repeated. Once no more state-based actions have been performed as
    the result of a check and no triggered abilities are waiting to be put on the stack, the
    appropriate player gets priority." 117.5: "the game first performs all applicable state-based
    actions as a single event… then repeats this process until no state-based actions are
    performed. Then triggered abilities are put on the stack… These steps repeat in order until no
    further state-based actions are performed and no abilities trigger. Then the player who would
    have received priority does so."

113.7a CONFIRMED. "Once activated or triggered, an ability exists on the stack independently of its
source. Destruction or removal of the source after that time won't affect the ability." Continues:
"…if the source is no longer in the zone it's expected to be in at that time, its last known
information is used."

## Gaps

- "Static abilities don't use the stack" is not literal text in 604 or 611; 113.3d or 405 not
  searched.
- 603.3b's two-part ordering is recent wording; older summaries describe one pass.
