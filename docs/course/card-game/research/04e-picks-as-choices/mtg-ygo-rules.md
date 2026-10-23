## Summary
Both games' rules support your design: one selection declaration that differs only in when the player answers. In MTG, targets are chosen while the spell is cast (601.2c). Other choices are made while it resolves (608.2d, 115.10). A choice is a target only if the text says "target" (115.10a), so hexproof doesn't stop "choose a creature". In Yu-Gi-Oh, targets are picked at activation (before the ";"), "choose" selections are made at resolution, and the official rulebook says excavated cards "are still treated as being in the Deck" until moved.

**Sources and their weight:**
- **MTG rule text** is quoted from the official Comprehensive Rules TXT, effective September 25, 2026. I checked the numbering against the yawgatog hyperlinked copy. Rulings come from the Scryfall API, which mirrors Wizards' official rulings.
- **Yu-Gi-Oh text** comes from Konami's Official Rulebook v10 and Kevin Tewart's 2011 PSCT articles on yugioh-card.com. Yugipedia blocks direct fetches (HTTP 403), so its wording comes from search-index excerpts. Those quotes are second-hand.

## Findings

### Magic: The Gathering
CR TXT: https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt · Hyperlinked: https://yawgatog.com/resources/magic-rules/

**Q1. Targets vs non-targeted choices; hexproof**
- **115.1** — "Some spells and abilities require their controller to choose one or more targets for them. The targets are object(s) and/or player(s) the spell or ability will affect. These targets are declared as part of the process of putting the spell or ability on the stack. The targets can't be changed except by another spell or ability that explicitly says it can do so."
- **601.2c** — "The player announces their choice of an appropriate object or player for each target the spell requires. A spell may require some targets only if an alternative or additional cost (such as a kicker cost) or a particular mode was chosen for it; otherwise, the spell is cast as though it did not require those targets. … If the spell has a variable number of targets, the player announces how many targets they will choose before they announce those targets. … Once the number of targets the spell has is determined, that number doesn't change…"
- **608.2d** — "If an effect of a spell or ability offers any choices other than choices already made as part of casting the spell, activating the ability, or otherwise putting the spell or ability on the stack, the player announces these while applying the effect. The player can't choose an option that's illegal or impossible, with the exception that having a library with no cards in it doesn't make drawing a card an impossible action (see rule 121.3)."
- **115.10** — "Spells and abilities can affect objects and players they don't target. In general, those objects and players aren't chosen until the spell or ability resolves. See rule 608, 'Resolving Spells and Abilities.'"
- **115.10a** — "Just because an object or player is being affected by a spell or ability doesn't make that object or player a target of that spell or ability. Unless that object or player is identified by the word 'target' in the text of that spell or ability, or the rule for that keyword ability, it's not a target."
- **115.10b** — "In particular, the word 'you' in an object's text doesn't indicate a target."
- **702.11b** — "'Hexproof' on a permanent means 'This permanent can't be the target of spells or abilities your opponents control.'"
- **Effect on hexproof:** this follows from 702.11b plus 115.10a. Hexproof only blocks targeting, so a "choose a creature" or "each/all" effect can affect a hexproof permanent. There is no single rule sentence that says this; it is a direct reading of the two rules. Third-party judge material says the same: https://cantrip.ru/en/mtg-judge/targets.shtml.
- **101.4** (simultaneous choices, APNAP order) — "If multiple players would make choices and/or take actions at the same time, the active player … makes any choices required, then the next player in turn order … Then the actions happen simultaneously." Its example: "Each player sacrifices a creature" — each player chooses in turn, then all chosen creatures are sacrificed at once.

**Q2. Looking at library cards; Reveal**
- **401.2** — "Each library must be kept in a single face-down pile. Players can't look at or change the order of cards in a library." Effects that say "look at" override this through the Golden Rule (101.1).
- **400.2 (part)** — "Hidden zones are zones in which not all players can be expected to see the cards' faces. Library and hand are hidden zones, even if all the cards in one such zone happen to be revealed."
- **701.20a (part)** — "To reveal a card, show that card to all players for a brief time. If an effect causes a card to be revealed, it remains revealed for as long as necessary to complete the parts of the effect that card is relevant to."
- **701.20b** — "Revealing a card doesn't cause it to leave the zone it's in."
- **401.5 (part)** — "Some effects tell a player to play with the top card of their library revealed, or say that a player may look at the top card of their library…"
- **"Look at" has no Comprehensive Rules entry.** It is not a keyword action; I searched the TXT and only found ordinary-language uses.
- **No official rule or ruling found** that says "looked-at cards remain in the library" — **UNVERIFIED**.
  - Impulse (https://api.scryfall.com/cards/3ec50625-5e14-4fd0-9a96-86f195342695/rulings) has only "This is not a draw." and "Due to errata, you no longer shuffle your library."
  - Anticipate has no rulings at all.
  - What does support it: only a "put"/"move" instruction changes zones, and "look" isn't one. Community answers state it outright ("If you're just looking at the top X cards of your library, they're still in your library" — MTG Salvation; the same on TappedOut). Those are not official.
- **Augur of Bolas ruling (2012-07-01, WotC)** — "If there are fewer than three cards in your library when Augur of Bolas's ability resolves, you'll look at the remaining cards. This won't cause you to lose the game as you haven't drawn from an empty library." This is consistent with looking not being a zone change, but it doesn't say so directly. https://api.scryfall.com/cards/mm3/30/rulings

**Q3. "Put one into your hand and the rest on the bottom of your library in any order"**
- Oracle text — Impulse: "Look at the top four cards of your library. Put one of them into your hand and the rest on the bottom of your library in any order." Anticipate is the same with "three".
- **401.4** — "If an effect puts two or more cards in a specific position in a library at the same time, the owner of those cards may arrange them in any order. That library's owner doesn't reveal the order in which the cards go into the library."
- **Augur of Bolas ruling (2012-07-01)** — "You don't reveal the other two cards, nor do you reveal their order on the bottom of the library."
- **Who orders them:** the owner of the cards chooses the order, both from the card's "in any order" and from 401.4. They are placed at the same time and the order stays hidden.
- **What "the rest" means:** the looked-at cards not put into the hand. With a short library it is whatever was actually looked at (the Augur ruling). No ruling defines the word itself; this is a plain-English reading under 608.2c — **UNVERIFIED** as an explicit ruling.
- **Related:** 103.5 (mulligan) uses the same wording: "puts … those cards … on the bottom of their library in any order."

**Q4. Modes chosen at casting**
- **601.2b (part)** — "If the spell is modal, the player announces the mode choice (see rule 700.2). …"
- **700.2a** — "The controller of a modal spell or activated ability chooses the mode(s) as part of casting that spell or activating that ability. If one of the modes would be illegal (due to an inability to choose legal targets, for example), that mode can't be chosen. (See rule 601.2b.)"
- **700.2b** — triggered abilities choose modes "as part of putting that ability on the stack… If no mode is chosen, the ability is removed from the stack. (See rule 603.3c.)"
- **700.2c (part)** — "If a spell or ability targets one or more targets only if a particular mode is chosen for it, its controller will need to choose those targets only if they chose that mode."
- **Order inside casting:** 601.2b (modes) comes before 601.2c (targets), so the mode decides which targets are needed.

**Q5. One selection used by several instructions; re-check timing**
- **608.2** — "…The steps described in rules 608.2a and 608.2b are followed first. The steps described in rules 608.2c–m are then followed as appropriate, in no specific order…" So the target check happens once, at the start of resolution.
- **608.2b (part)** — "If the spell or ability specifies targets, it checks whether the targets are still legal. A target that's no longer in the zone it was in when it was targeted is illegal. … If all its targets, for every instance of the word 'target,' are now illegal, the spell or ability doesn't resolve. … Illegal targets, if any, won't be affected by parts of a resolving spell's effect for which they're illegal. Other parts of the effect for which those targets are not illegal may still affect them. … If part of the effect requires information about an illegal target, it fails to determine any such information. Any part of the effect that requires that information won't happen."
- **608.2c** — "The controller of the spell or ability follows its instructions in the order written. However, replacement effects may modify these actions. In some cases, later text on the card may modify the meaning of earlier text (for example, 'Destroy target creature. It can't be regenerated' or 'Counter target spell. If that spell is countered this way, put it on top of its owner's library instead of into its owner's graveyard.') Don't just apply effects step by step without thinking in these cases—read the whole text and apply the rules of English to the text."
- **608.2h (part)** — "If an effect requires information from the game (such as the number of creatures on the battlefield), the answer is determined only once, when the effect is applied. If the effect requires information from a specific object… the effect uses the current information of that object if it's in the public zone it was expected to be in; if it's no longer in that zone, or if the effect has moved it from a public zone to a hidden zone, the effect uses the object's last known information."
- **608.2k (part)** — "If an ability's effect refers to a specific untargeted object that has been previously referred to by that ability's cost or trigger condition, it still affects that object even if the object has changed characteristics."
- **"They"/"it" refer back to the same set; nothing is re-selected.** This is my reading, not rule text. 608.2c's "It can't be regenerated" example and its "rules of English" instruction cover it. Applying the same logic to Wrath of God ("Destroy all creatures. They can't be regenerated.") is an inference — **UNVERIFIED** as a quoted ruling.

**Q6. Targeting cards in hidden zones**
- **115.2** — "Only permanents are legal targets for spells and abilities, unless a spell or ability (a) specifies that it can target an object in another zone or a player, or (b) targets an object that can't exist on the battlefield, such as a spell or ability. See also rule 115.4."
- **What the rules say:** by default nothing outside the battlefield (plus spells and abilities) can be targeted. A card can only target into another zone by saying so. No rule outright forbids "target card in a hand".
- **What I couldn't confirm:** that no printed card targets a card in a hand or library — **UNVERIFIED**.
- **Hidden-zone selections are non-targeted:** 101.4a — "If an effect has each player choose a card in a hidden zone, such as their hand or library, those cards may remain face down as they're chosen. However, each player must clearly indicate which face-down card they are choosing."

### Yu-Gi-Oh

**Q7. PSCT: activation-time targets vs resolution-time choices**
Source: Konami PSCT Part 3, Kevin Tewart, June 1, 2011 — https://www.yugioh-card.com/en/play/psct/psct-3/
- On the colour coding: "Anything that happens WHEN YOU ACTIVATE a card will be written in red… This includes things like costs and targeting." and "Anything that happens when you resolve a card will be written in blue…"
- "**Conditions (green text) are now always followed by a colon (:).** **Activation text (red) is separated from the card effects (blue) by a semicolon (;).** **The basic structure is CONDITIONS : ACTIVATION ; RESOLUTION.**"
- "BIG TIP: If there's a colon or semicolon in the text, that always means an effect that starts a chain."
- The Creature Swap example (a choice made at resolution): *"Each player chooses 1 monster they control and switches control of those monsters…"* — "its text is exactly the same as before because everything on Creature Swap happens when the effect resolves."

Source: Konami PSCT Part 4 — https://www.yugioh-card.com/en/play/psct/psct-4/
- "For any card that targets, if the effect text (after the semicolon) still refers to a 'target', you have to double-check that any targeting requirements are still met."
- "Otherwise, if the card refers to the target using any other word (like 'it') in the effect text, the targeting requirements only had to be met when the target was originally targeted."
- "…if the effect text refers to 'both' … If even one of the targets is no longer eligible, then the whole effect disappears."
- Magical Dimension example: "The semicolon tells us the only thing you do at activation is target a monster."
- **Design note:** this is relevant to your shared declaration. In YGO the back-reference word ("that target" vs "it") decides whether conditions are re-checked at resolution.

**What the Konami articles don't say:** they never state a general rule that "non-targeting selections are made at resolution". That rule comes from Yugipedia, which I could only read through search excerpts (UNVERIFIED wording):
- Yugipedia "Target" (https://yugipedia.com/wiki/Target):
  - "If a card or effect targets, its targets must be chosen when it is activated."
  - "In Problem-Solving Card Text, cards and effects that target always explicitly state that they target, except Equip Spell Cards because they always target."
  - On cards that don't target: "Cards that 'choose' a card(s) to be affected (e.g. 'Forbidden Droplet')"
  - "Cards that affect a card(s) at effect resolution, without having to target at activation (e.g. 'Fissure')"
  - "The player does not get to change their target or select another target at resolution."
- Yugipedia "Card Rulings:Creature Swap" OCG ruling: "The effect does not target. The monster whose control will be switched are chosen at resolution."
- **Targeting into hidden zones (parallel to Q6):** Yugipedia "Target" — "Cards can be targeted if they are on the field, in the Graveyard, or banished. Cards in the hand, Main Deck, Extra Deck, or attached as Xyz Material cannot be targeted." (UNVERIFIED wording, from search excerpt)

**Q8. Excavate: are the cards still in the Deck?**
- **Official Rulebook v10, glossary "Excavate"** (page 54 per a Reddit citation) — "When a card says to 'excavate' cards from your Deck, you reveal those cards to both players. Then, before you do anything else, apply the instructions from the card effect that excavated them. Cards that are being excavated are still treated as being in the Deck until sent elsewhere by the card instructions." https://img.yugioh-card.com/en/downloads/rulebook/SD_RuleBook_EN_10.pdf
  - This came from a search excerpt of the PDF and matches two Reddit transcriptions (rulebook v8 from 2013, and v10). I did not open the PDF myself.
- **Same rulebook, "Reveal"** — "When an effect says to reveal a card, you show it to both players… Normally, revealed cards are returned to their original position after being looked at by both players."
- **Yugipedia "Excavate"** (excerpt, UNVERIFIED wording) — "Excavated cards remain in the Deck even after being picked up, until they are explicitly moved to another location. Since the cards are revealed, they are shown to both players."
- **Example of your pattern — Adamancipator Researcher** (ygoprodeck API): "During your Main Phase: You can excavate the top 5 cards of your Deck, and if you do, you can Special Summon 1 excavated Level 4 or lower non-Tuner Rock monster, also place the rest on the bottom of your Deck in any order."
  - All 5 cards stay in the Deck while excavated.
  - The single pick is a resolution-time, non-targeting choice, because it comes after the ";" and doesn't use the word "target".
- **Close variant — Pot of Duality:** "Excavate the top 3 cards of your Deck, add 1 of them to your hand, also, after that, shuffle the rest back into your Deck."
- **I didn't find an exact "excavate 5, add 1 to hand, rest on bottom" card** — **UNVERIFIED** that one exists.

## Sources
- **Kept:**
  - MTG Comprehensive Rules TXT, effective 2026-09-25 (https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt) — primary source for all MTG quotes.
  - Yawgatog hyperlinked CR (https://yawgatog.com/resources/magic-rules/) — confirmed rule numbering.
  - Scryfall API rulings for Augur of Bolas and Impulse — official WotC rulings text.
  - Konami PSCT Parts 2–4 (yugioh-card.com) — primary PSCT source.
  - Konami Official Rulebook v10 PDF — definitions of Excavate and Reveal.
  - Yugipedia "Target" and "Card Rulings:Creature Swap" (via search excerpts) — targeting timing and the choose-at-resolution ruling.
  - ygoprodeck API — current card texts.
- **Dropped:**
  - Reddit, MTG Salvation, TappedOut threads — not official; used only to corroborate.
  - Yu-Gi-Oh Fandom wiki — duplicates the rulebook text.
  - Yugipedia "Forum: 90% of Answers" — outdated pre-PSCT opinions.
  - Gutless Plunderer / See the Truth Gatherer pages — not relevant enough.

## Gaps
- I found no official MTG ruling saying "looked-at cards stay in the library". A Gatherer search of other dig spells (e.g. Dig Through Time, Fact or Fiction) could turn one up.
- The Yugipedia pages and the rulebook PDF weren't opened directly. Before citing them as verbatim, open them in a browser (Yugipedia blocks automated fetches).
- The Wrath of God "They" reading and the "no card targets cards in hand/library" claim are inferences, not quoted rulings.
- Konami's own database FAQ (db.yugioh-card.com) for Creature Swap and Adamancipator Researcher wasn't checked.