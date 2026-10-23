# Picks-as-choices: card text checked against primary sources

1. **Hearthstone Brawl is confirmed.** The wiki shows "Destroy all minions except one. *(chosen randomly)*" — https://hearthstone.wiki.gg/wiki/Brawl
   - The italic parenthetical has been in place since Patch 2.0.0.7234 (2014-12-04). The wording is unchanged; only the formatting changed.
   - This is a random pick, not a player choice.

2. **MTG Duneblast: your wording is wrong.** The current Oracle text reads "Choose up to one creature. Destroy the rest." Your version, "...Destroy all other creatures.", is not the current text. — https://api.scryfall.com/cards/named?exact=Duneblast

3. **Cards where the unchosen cards get a non-move effect** (both checked on Scryfall):
   - **Divine Reckoning:** "Each player chooses a creature they control. Destroy the rest.\nFlashback {5}{W}{W} (You may cast this card from your graveyard for its flashback cost. Then exile it.)" — https://api.scryfall.com/cards/named?exact=Divine%20Reckoning
   - **Tragic Arrogance:** "For each player, you choose from among the permanents that player controls an artifact, a creature, an enchantment, and a planeswalker. Then each player sacrifices all other nonland permanents they control." — https://api.scryfall.com/cards/named?exact=Tragic%20Arrogance
   - Both choose at resolution, and the unchosen cards are destroyed or sacrificed. They don't simply get sent to a zone.
   - Duneblast (item 2) also fits this pattern.
   - I didn't find or check a card where the unchosen cards get damaged or buffed.

4. **YGO Pot of Duality is confirmed** (YGOPRODeck API): "Excavate the top 3 cards of your Deck, add 1 of them to your hand, also, after that, shuffle the rest back into your Deck. You can only activate 1 "Pot of Duality" per turn. You cannot Special Summon during the turn you activate this card." — https://db.ygoprodeck.com/api/v7/cardinfo.php?name=Pot%20of%20Duality

**Risks**
- I didn't check the official Blizzard card library or the Konami DB. The sources used were the wiki, Scryfall and YGOPRODeck.
- Destroy and sacrifice both still end with the card in the graveyard. If "non-move" has to exclude that, item 3 needs a card that damages or buffs the unchosen ones, and I haven't found one.