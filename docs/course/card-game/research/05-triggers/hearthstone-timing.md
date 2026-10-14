# Hearthstone: trigger, deathrattle and damage-prevention timing

Researcher report, session 05. Primary source: hearthstone.wiki.gg Advanced rulebook
(https://hearthstone.wiki.gg/wiki/Advanced_rulebook), plus card and mechanic pages. Many rulebook
examples were tested between 2015 and 2017; the rulebook's foreword warns they may be dated.

## Summary

Triggers and deaths resolve in order of play (oldest entity first), each event depth-first. Deaths
are processed after the outermost Phase: all dying entities leave play at once, then a Death Phase
resolves each death's triggers in order of play. Mortally wounded minions stay in play until then,
so their on-damage triggers fire. Divine Shield acts while the Damage Event is created (damage set
to 0, and damage that changes nothing raises no triggers). Not documented: silence while mortally
wounded, which Attack a deathrattle reads, any cap on trigger chains.

## Findings

1. Trigger queue order CONFIRMED. Rule 2a: "When we start to resolve a Phase or Event, A Queue is
   created and filled with all triggers that can respond, in _order of play_." "_Order of play_
   means the order the Entities each Event/trigger is associated with entered play, from oldest to
   newest." Rule 2c: "A Queue becomes immutable once _Hearthstone_ starts to resolve the first entry
   in it." Depth-first: "The moment a new trigger/Event is raised, _Hearthstone_ begins to resolve
   it and immediately begins exploring its consequences, before resolving those remaining for the
   current trigger/Event." Exception: "Some triggers have a **Priority** that supersedes _order of
   play_… [Redemption] always Queues last."
2. Deathrattle order CONFIRMED; dying minions leave before any deathrattle. Rule 4a: "After the
   outermost Phase ends, _Hearthstone_ does an Aura Update (Health/Attack), then does _the Death
   Creation Step_ (Looks for all _mortally wounded_… /_pending destroy_… Entities and _kills_ them,
   removing them from play simultaneously)… Entities that have been _removed from play_ cannot
   trigger, be triggered, or emit auras, and do not take up space." Rule 7: "a new Phase (called a
   "Death Phase") begins, where Deaths are Queued in _order of play_. For each Death, all Death
   Event triggers (Deathrattles, on-Death Secrets and on-Death triggered effects) are Queued and
   _resolved_ in _order of play_." Rule 8: "A Death Phase can have yet another Death Phase after
   it."
3. Simultaneous deaths.
   - Cult Master CONFIRMED (does not draw): "If Cult Master is killed at the same time as another
     minion… its draw effect will not activate from that minion's death (regardless of order of
     play)." (https://hearthstone.wiki.gg/wiki/Cult_Master) Rulebook: "The Cult Master cannot
     trigger on the Bloodfen Raptor's Death because it is _removed from play_ at the same time." A
     death in an earlier Death Phase does trigger it (Explosive Sheep example).
   - Flesheating Ghoul CONFIRMED (does not gain). Footnote 22: "Using both Cult Master and
     Flesheating Ghoul, neither minion triggers from the simultaneous Death of other minions,
     regardless of order of play." Card note: "Flesheating Ghoul cannot trigger off the same death
     that summons it."
   - Scavenging Hyena PARTLY CONFIRMED (no card notes; follows from 4a). Triggered effect page:
     "Death-triggered effects activate only after the death has fully resolved, and the minion or
     weapon removed from play." (https://hearthstone.wiki.gg/wiki/Triggered_effect)
4. Mortally wounded minions keep triggering CONFIRMED. Rulebook: "the Acolyte of Pain is hit,
   triggered and draws three cards, despite being _mortally wounded_ after the first Deathrattle."
   "the Armorsmiths will be mortally wounded but remain in play, triggering on each Damage Event."
   Armorsmith notes: "This includes damage that mortally wounds a minion, damage dealt to the
   Armorsmith herself." Triggered effect page: "Damage-triggered effects activate immediately after
   damage is received, but before any deaths are resolved."
5. "Whenever" vs "After" CONFIRMED. Spell Sequence: "On Play Phase: All "whenever" triggers on
   playing a card/casting a spell… such as… Questing Adventurer, Mana Wyrm… and Violet Teacher."
   "Spell Text Phase: … the entirety of the spell text takes place in one Phase." "After Spell
   Phase: All "after" triggers… Queue and _resolve_ here… these triggers can only work if they're
   valid when the Sequence starts." Game terms: "when"/"whenever" is "at the time the indicated
   action starts to take place"; "after" is "after the time the indicated action has completed."
   October 2017 update: "in order for a card's trigger to activate during a sequence of triggers,
   the card needs to be present and valid before the sequence of triggers begins."
6. Divine Shield and Predamage CONFIRMED. DH2a: "If the target is Immune or the Damage Event's
   proposed damage is 0, the Damage Event is prevented. Then if the target has Divine Shield, the
   Divine Shield is removed and the Damage Event's proposed damage is reduced to 0. Then queue and
   resolve Predamage triggers (see Rule DH7). Then the Character's current Health+Armor is
   reduced…" DH2b: "If it was prevented, or if it did not change the Character's current
   Health+Armor total (including due to Divine Shield…), it will not run any triggers." DH7: "Just
   before a Damage Event is created, Predamage triggers will queue and resolve in play order. They
   will stop triggering if the Damage Event is prevented… [examples:] Bolf Ramshield, Ice Block,
   Animated Armor." DH7b: "Bolf Ramshield's effect is to prevent the Damage Event, then create a
   Damage Event whose source is still the original source but target is Bolf Ramshield." DH7d: Ice
   Block prevents the event and grants Immune. Flag: DH2a and DH7 disagree slightly on where
   Predamage sits.
7. Silence PARTLY CONFIRMED. Removes "Deathrattles… Triggered effects, including triggered effect
   keywords like Overkill and Honorable Kill." (https://hearthstone.wiki.gg/wiki/Silence) Silence
   while mortally wounded: NOT FOUND; inferred from 4a and 7 that the deathrattle would not fire.
8. Which Attack a deathrattle reads: NOT FOUND. Only related text: "minions moved to the Graveyard
   are reset to default attributes" (a Vol'jin example).
9. Cap on trigger chains: NOT FOUND. Rule 8: the Death Phase process "repeats forever until no new
   Deaths occur." "a trigger cannot be nested inside itself. Instead it works the skipped times as a
   compensation after all other consequences of it are resolved." Documented limits outside a
   sequence: 89 turns, then a draw (https://hearthstone.wiki.gg/wiki/Turn); 75-second turn timer.

## Gaps

- Items 7, 8, 9 undocumented. Next steps: rulebook talk page, HearthSim hs-bugs, simulator source.
