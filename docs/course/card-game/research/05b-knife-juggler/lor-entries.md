# Legends of Runeterra: playing, summoning and summon triggers (research for Knife Juggler)

## Summary
In LoR, playing a unit from hand also summons it. A unit put onto the board by an effect (created, revived, or pulled from hand or deck) is summoned but not played, so its **Play** text doesn't fire and its **"When I'm summoned"** text does. A played unit's Play effect starts first, then the unit is summoned and its own "When I'm summoned" fires, then other "When you summon" triggers fire, on the board from left to right. Each summoned unit causes its own separate "When you summon" trigger. No rulings document how units summoned by one effect react to each other; player reports suggest they can.

## Findings
1. **Play means from hand only (Riot, official).** "To trigger a unit's Play trigger … it must be played from hand … creating a summoned card or reviving a card that has a Play effect won't trigger its Play effect … if you use Marai Warden to summon Crusty Codger, Crusty Codger's Play effect won't trigger." It also says: "Playing a card means dragging it from your hand to the board." Riot's page doesn't define Summon or "When I'm summoned" on their own. [Riot Support – Keywords and Game Vocab](https://support.riotgames.com/en-us/legends-of-runeterra/gameplay/keywords-and-game-vocab-legends-of-runeterra)
2. **Playing a unit also summons it (wiki).** "unlike 'When I'm summoned' effects, Play effects do not trigger if a card is summoned another way. For example, Dunekeeper's effect … will occur both when it is played and when it is summoned by a card effect; but Yordle Squire's effect only triggers when it is played." Also: "Simply summoning a unit does not trigger Play effects, but playing them does conversely trigger their summon effects." And: "A summoned unit can be created, or it can be taken from the hand or deck." [Wiki: Vocabs/Play](https://wiki.leagueoflegends.com/en-us/LoR:Vocabs/Play), [Fandom: Keywords](https://leagueoflegends.fandom.com/wiki/Keywords_(Legends_of_Runeterra))
   - Transforming a unit counts as neither a play nor a summon (Reddit consensus, not verified). [Reddit](https://www.reddit.com/r/LegendsOfRuneterra/comments/oc800r/does_transform_eg_shady_character_counts_as/)
3. **Trigger order (wiki).** "Triggers will only activate after whatever event met their condition has fully resolved … the 'When I'm summoned' effect of a unit will activate before any other triggers for 'When you summon a unit'." The order is:
   - champion level-up
   - Lab powers
   - Nexus or outside-of-play triggers, in the order they were given (e.g. Zoe's level 2)
   - triggers from cards in the deck (Lurk)
   - triggers from cards in hand, left to right
   - "Unit or landmark triggers on the board, going left to right."

   Also: "enemy triggers of the same hierarchy will always trigger after allied ones." Simultaneous effects such as The Ruination still produce triggers "sequentially." [Wiki: Keywords/Trigger](https://wiki.leagueoflegends.com/en-us/LoR:Keywords/Trigger)
   - For ties when choosing targets, Riot says officially: "units will be targeted in from left to right … This is the Tiebreaker Rule." [Riot: Order Resolution](https://support.riotgames.com/en-us/legends-of-runeterra/gameplay/order-resolution-mechanics-and-the-tiebreaker-rule)
4. **Play effect, then own summon effect, then others' summon triggers (community, not official).** A Reddit guide lists:
   1. Unit X's Play effect activates.
   2. X's summon window opens.
   3. X's own summon effect activates.
   4. The window closes and X's summon resolves.
   5. Other effects that react to X's summon activate.

   It adds: "Units are not treated as being on the board … until the Summon window resolves." [Reddit guide](https://www.reddit.com/r/LegendsOfRuneterra/comments/p0nvql/understanding_lor_mechanics_summon_effects_and/)

   Supporting evidence:
   - Petty Officer's Play summons a 1-cost unit, and that unit counts as summoned before Petty Officer itself. Navori Conspirator played onto a full board "doesn't obliterate anything." [Reddit](https://www.reddit.com/r/LegendsOfRuneterra/comments/itqnt7/petty_officer_summons_fleetfeather_tracker_w/)
   - A wiki user page gives the same order: "Play" effect, then the "unit … that played is summoned," then "'When X summoned' or 'When you summon' effect (repeated every time a unit is summoned)." [Wiki: User:Onfealive](https://wiki.leagueoflegends.com/en-us/User:Onfealive/LoR_Stuffs)
5. **Skills go on the stack; other triggers resolve immediately.** Riot: "Skill — A unit's spell-like effect that allows enemy reactions"; "When a trigger has a Skill icon, enemies can react." If a Play or summon effect is a Skill, it goes onto the stack and resolves later, so instant "When you summon" triggers can resolve before it. In a 2022 Reddit case, players say "Summon effect happens first … because the play effect put skill on stack" (unverified). [Reddit](https://www.reddit.com/r/LegendsOfRuneterra/comments/zwo5ot/what_happens_first/)
   - Targets for Play effects are chosen when the card is played. This is general game knowledge; I found no explicit quote for it.
6. **One effect summoning several units triggers once per unit.**
   - Wiki note: "repeated every time a unit is summoned."
   - Blighted Caretaker summons 2 Saplings, and Roiling Sands gave Vulnerable to both. [Reddit](https://www.reddit.com/r/LegendsOfRuneterra/comments/ly20k6/roiling_sands_can_trigger_vulnerable_on_multiple/)
   - Crimson Bloodletter triggered on both Braum and his Mighty Poro.
7. **Units summoned together can see each other's summons (community, not verified).** With Zoe at level 2, "When you summon an ally, grant its keywords to all allies": playing the Golden Sister made "the Golden Sister … give the Silver Sister lifesteal, and the Silver Sister … give the Golden Sister lifesteal." When Dunekeeper summons a Sand Soldier, the soldier counts as summoned first. [Reddit](https://www.reddit.com/r/LegendsOfRuneterra/comments/lzd64u/is_this_a_bug/)
   - I found nothing on two copies of the same "When you summon another ally" unit (e.g. Ember Monk) being summoned together.
8. **Recursive copies are stopped by design (Riot).** Silverwing Vanguard "summons exactly one other copy … it does not summon further copies as the text implies." In patch V4.2 its text changed from "When I'm summoned, summon an exact copy of me" to "Play: Summon an exact copy of me." [Wiki](https://wiki.leagueoflegends.com/en-us/LoR:Silverwing_Vanguard), [Reddit quoting Riot's known-issues page](https://www.reddit.com/r/LegendsOfRuneterra/comments/i0dm3m/why_is_this_text_written_like_that_shouldnt_it/)
9. **"Another ally" wording.** These cards say "another" explicitly:
   - Fleetfeather Tracker: "When you summon another ally, grant me Challenger." [RuneterraFire](https://www.runeterrafire.com/cards/fleetfeather-tracker)
   - Ember Monk: "When you summon another ally, grant me +1|+0." [Wiki](https://wiki.leagueoflegends.com/en-us/LoR:Ember_Monk)
   - Rockbear Shepherd uses the same wording.

   Zoe's Nexus effect says "When you summon an ally, grant its keywords …" and applies to every summoned ally. The wiki also says a unit "sees an action happen if it is on the board when it happens," and a unit isn't on the board until its summon resolves. So a unit most likely can't trigger its own "When you summon" text; that's an inference, not a ruling.

## Sources
- Kept:
  - Riot Support pages (Keywords and Game Vocab; Tiebreaker Rule): official.
  - wiki.leagueoflegends.com (Vocabs/Play, Keywords/Trigger, card pages, User:Onfealive): well maintained, though the user page is personal notes.
  - Reddit threads: grocktops' timing guide, Petty Officer, Roiling Sands, Zoe/Golden Sister, Silverwing (with Riot's known-issues quote).
- Dropped:
  - esportstalk deck lists and Liquipedia card lists: not about rules.
  - runeterrafire's keyword guide: duplicates the wiki.

## Gaps
- Riot has no official comprehensive rules for summon timing. Everything after point 2 comes from the wiki or from players' observations.
- Unverified:
  - How units summoned by the same effect see each other's summons (only one reported case, Zoe with the Golden/Silver Sisters).
  - Whether a Skill-type Play effect resolves after "When you summon" triggers.
  - Whether a unit's own summon can set off its "When you summon an ally" text.
  - The order between the copies when one effect summons several units (probably left to right, i.e. board slot order).
- Next step: confirm with replays or clips (e.g. Ember Monk ×2 via Spectral Matron/Splinter Soul, Haunted Relic + Soul Shepherd), or check Riot dev Q&A archives. I didn't search Mobalytics.