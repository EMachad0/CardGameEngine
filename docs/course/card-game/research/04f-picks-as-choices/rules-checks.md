## Summary
I confirmed Q1, Q2 and Q4 from primary or near-primary sources. The CR text comes from the official WotC TXT, effective Sept 19, 2025, and the card text from Scryfall's API. Q3 is only partly confirmed: hearthstone.wiki.gg says Discover drops duplicate options and that a choice is mandatory. Nothing on wiki.gg or from Blizzard says whether a single option is still shown or picked automatically, or what an empty deck does, so those parts are UNVERIFIED.

## Findings
1. **CR 609.3 (VERIFIED, exact):** "609.3. If an effect attempts to do something impossible, it does only as much as possible. Example: If a player is holding only one card, an effect that reads "Discard two cards" causes them to discard only that card. If an effect moves cards out of the library (as opposed to drawing), it moves as many as possible." [WotC CR TXT](https://media.wizards.com/2025/downloads/MagicCompRules%2020250919.txt)
2. **CR 608.2d, on choices made during resolution (VERIFIED):** "...the player announces these while applying the effect. The player can't choose an option that's illegal or impossible, with the exception that having a library with no cards in it doesn't make drawing a card an impossible action (see rule 121.3)." [same](https://media.wizards.com/2025/downloads/MagicCompRules%2020250919.txt)
3. **CR 701.9b, who chooses for discard (VERIFIED):** "By default, effects that cause a player to discard a card allow the affected player to choose which card to discard. Some effects, however, require a random discard or allow another player to choose which card is discarded." [same]
4. **Augur of Bolas WotC ruling, 2012-07-01 (VERIFIED):** "If there are fewer than three cards in your library when Augur of Bolas's ability resolves, you'll look at the remaining cards. This won't cause you to lose the game as you haven't drawn from an empty library." [Scryfall rulings](https://api.scryfall.com/cards/d19fbfe6-69bb-452a-be3c-b9c625e23193/rulings)
   - Oracle text for the other two cards (VERIFIED): Ancestral Memories reads "Look at the top seven cards of your library. Put two of them into your hand and the rest into your graveyard." Impulse reads "...Put one of them into your hand and the rest on the bottom of your library in any order." [Scryfall](https://api.scryfall.com/cards/named?exact=Ancestral%20Memories&format=text)
   - Neither card has an official ruling about having fewer cards than asked for; Impulse's only rulings are "This is not a draw." and the shuffle errata.
   - **Is a forced choice still a "choice"? UNVERIFIED.** No CR rule says it explicitly. 609.3 plus 608.2d imply that you put in as many as possible, and that with one candidate the only legal option is that one.
5. **Q2 Oracle text (VERIFIED, Scryfall API):**
   - Diabolic Edict: "Target player sacrifices a creature of their choice." The targeted player chooses.
   - Innocent Blood: "Each player sacrifices a creature of their choice." Each player chooses their own creature.
   - Duress: "Target opponent reveals their hand. You choose a noncreature, nonland card from it. That player discards that card." The caster chooses.
   - Sources: [Edict](https://api.scryfall.com/cards/named?exact=Diabolic%20Edict&format=text), [Blood](https://api.scryfall.com/cards/named?exact=Innocent%20Blood&format=text), [Duress](https://api.scryfall.com/cards/named?exact=Duress&format=text)
   - CR 701.21a: "To sacrifice a permanent, its controller moves it from the battlefield directly to its owner's graveyard."
6. **HS Discover with fewer options (VERIFIED):**
   - "Discover effects will not present duplicate options... If there aren't enough possible options, such as using Shadow Visions when the deck only has two of the same spell, this still applies and only one copy is shown."
   - "Players cannot choose not to take one of the cards offered by a Discover effect. They must choose one of the options. If the turn timer expires before one of the Discover options is picked, a random option will be chosen."
   - [wiki.gg Discover](https://hearthstone.wiki.gg/wiki/Discover)
   - "Only one copy is shown" means a 1-card choice screen appears in that case. That is how the wiki text reads, not an explicit Blizzard statement.
7. **HS Tracking and Dredge (PARTIAL):**
   - Current Tracking text is "Discover a card from your deck." [Blizzard card library](https://hearthstone.blizzard.com/en-us/cards/69545-tracking/)
   - A Blizzard forum reply (2025) says a card discovered from your deck is also drawn; that is from a player, not staff.
   - Dredge on wiki.gg: "Look at the bottom 3 cards of your deck. Put one on top." Also: "In the Choosing One scene, all cards are sorted by their order in your deck." [wiki.gg Dredge](https://hearthstone.wiki.gg/wiki/Dredge)
   - The Tracking and Dredge wiki pages have no notes on fewer than 3 cards, a single card, or an empty deck: **UNVERIFIED**.
   - Community sources only (old Tracking, 2014) say that with 1 card left you just get it, and with 0 cards nothing happens and there is no fatigue. [StackExchange](https://gaming.stackexchange.com/questions/175174/using-tracking-with-less-than-three-cards)
8. **YGO, Metaphys Horus (36898537), VERIFIED via the YGOPRODeck API mirror of the TCG text:** "● Pendulum Monster: Your opponent chooses 1 monster they control and gives control of it to you, but it cannot attack this turn." [YGOPRODeck API](https://db.ygoprodeck.com/api/v7/cardinfo.php?name=Metaphys%20Horus)
   - Yugipedia returned 403. A search snippet of [Yugipedia](https://yugipedia.com/wiki/Metaphys_Horus) and yugiohcardguide show the same wording.
   - I did not check the Konami DB directly.

## Sources
- Kept: WotC CR TXT 20250919 (primary); Scryfall API Oracle text and WotC rulings; hearthstone.wiki.gg Discover/Dredge/Tracking (raw wikitext checked); Blizzard card library; YGOPRODeck API.
- Dropped: Reddit and cardmaker.net threads (custom or anecdotal cards); the hearthstone.fandom old wiki (outdated).

## Gaps
- Whether Hearthstone auto-picks with exactly one option, or shows a 1-card screen, for Tracking/Dredge. Next step: test in game or check Blizzard dev posts.
- Hearthstone empty-deck behavior for Discover-from-deck and Dredge.
- Konami DB text for Metaphys Horus.
- No CR rule explicitly says a forced choice still counts as "choosing".