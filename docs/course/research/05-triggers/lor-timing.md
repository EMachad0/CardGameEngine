# Legends of Runeterra: simultaneous deaths and trigger timing

Researcher report, session 05. LoR has no published comprehensive rules; Riot's support pages cover
keyword definitions and the left-to-right tiebreaker only. Timing comes from the community wiki
(wiki.leagueoflegends.com; the fandom wiki returned 403) and player-tested reports, which agree.

## Summary

A board-wide effect kills its units at once. Each Last Breath fires after the whole effect
resolves. "When an ally dies" triggers on the board fire only if their unit survives. Other
triggers fire one at a time, left to right. Deaths are processed between sequential triggered
effects and between stack items. Inside one multi-strike spell, damaged units stay targetable until
the spell finishes.

## Findings

1. A "when an ally dies" unit dying in the same wipe does not trigger for the others. PARTLY
   CONFIRMED (wiki rule, player reports, no Riot ruling). Wiki, Keywords/Trigger, "Simultaneous
   Effects": "Though effects such as Judgment or The Ruination happen simultaneously, triggers that
   would be activated by them happen sequentially… For other triggers of units or landmarks on the
   board, they'll only activate if they survive the effect, taking into account any changes from
   previous triggers." (https://wiki.leagueoflegends.com/en-us/LoR:Keywords/Trigger) Reddit
   (Neverglade Collector, "When another ally dies, drain 1"): "if the opponent has six Neverglade
   Collectors and you cast Intimidating Roar with a level 2 Yasuo, you will take no damage, as all
   the Collectors die simultaneously and non[e trigger]" (snippet). Exception: Tortured Prodigy
   counts its own death ("Tortured prodigy is an exception…"; "This was added a few months ago").
   Viego does not count his own death. Kalista, Glimpse Beyond, Hecarim, Ledros: NOT FOUND.
2. A unit's own Last Breath fires after the whole effect resolves, after the other triggers it
   caused. CONFIRMED (wiki). "…with the exception of Last Breath, which triggers after the effect
   has resolved." Judgment example: step 4 "The Spiderling from Hapless Aristocrat's Last Breath
   will be summoned." after Fury buffs and Shyvana's level-up (steps 1 to 3), before step 5 "Thresh
   will gain 2 level progress from the Hapless Aristocrat and Maokai dying." Strike tooltip: "The
   strike itself (and any effects that are triggered on the strike, including Nexus Strike on
   Overwhelm and Lifesteal) will resolve BEFORE any of the resulting effects (often "kill" or death
   effects such as Last Breath)." Riot glossary: "Last Breath: When the unit dies, do the effect."
   Obliterate "Doesn't cause Last Breath."
   (https://support.riotgames.com/en-us/legends-of-runeterra/gameplay/keywords-and-game-vocab-legends-of-runeterra)
   Order among several Last Breaths: no source; inferred from "Unit or landmark triggers on the
   board, going left to right" and "enemy triggers of the same hierarchy will always trigger after
   allied ones, with the ally being whichever player's card started the triggers."
3. A unit at 0 health stays on the board until the spell finishes. PARTLY CONFIRMED (player tests).
   Death tooltip: "When a unit is reduced to 0 Health or is otherwise killed, it is removed from
   play." Concerted Strike: "Units can go into negative health and both strike happen
   simultaneously", "Strike effects work even when they're concerted striking a dead unit", "the
   unit is still technically "alive"". Multi-strike spells: "these "multistrikes" don't calculate
   the damage simultaneously (Yasuo leveling mid-Roar, etc.), the post-spell effects will." Wiki
   Terminology: "Health cannot go below 0".
4. Skills go on the stack in board order and resolve last-in-first-out; deaths are processed between
   stack items and between sequential triggers. CONFIRMED (wiki, Riot); death detail PARTLY
   CONFIRMED. "Triggers will only activate after whatever event met their condition has fully
   resolved… triggers do not activate until after the spell resolves on the stack." Activation
   order: champion level-up, Lab powers, Nexus or outside-play triggers, deck, hand (left to right),
   board (left to right). Skill: "Goes on the Spell Stack like a Fast or Slow spell." Stack: "the
   spells all cast, in the order of the last spell played occurring first." "Spells and Skills on
   the stack that require a condition to cast that is no longer fulfilled or all of the targets of
   are no longer valid at the time they are reached on it, will Fizzle."
   (https://wiki.leagueoflegends.com/en-us/LoR:Spell) Attack skills: "The unit on the left puts
   their skill on the stack first, but since the stack resolves the last thing first, then the unit
   on the right will end up resolving their skill first." "Skills almost never fizzle, only when
   they're related to a units combat position (Taliyah, Ziggs, Vladimir)." Round Start, patch 2.12:
   "the Ravine ends up killing the Maiden and Slotbot before their effects resolve, so their
   abilities won't trigger, because they will be dead." "The effect must be on your board when the
   Roundstart begins and also at the time of its resolution."
   (https://lor.cardsrealm.com/en-us/articles/how-roundstart-works-in-legends-of-runeterra)
5. The only documented state check is a loss check at the end of a round phase. PARTLY CONFIRMED.
   "we have a small checking moment where it is checked if a player lost for any reason during all
   the resolutions from Roundstart… in case both players have lost the game ends in a Draw, no
   matter which player lost first." Wiki: "if both players should lose during this phase, it will be
   considered a draw regardless of who would lose first." Tiebreaker (targeting only): "If a tie
   cannot be resolved in this way, units will be targeted in from left to right. This is the
   Tiebreaker Rule."
   (https://support-legendsofruneterra.riotgames.com/hc/en-us/articles/360043734914-Order-Resolution-Mechanics-and-the-Tiebreaker-Rule)

## Gaps

- No Riot statement on the order of several Last Breaths; no documented sweep case for Kalista,
  Hecarim's phantoms, Glimpse Beyond, Ledros; nothing on a two-part spell targeting a unit at 0.
  Reddit quotes are search snippets (direct fetch blocked).
