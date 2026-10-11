# Yu-Gi-Oh!: simultaneous destruction and when triggers activate

Researcher report, session 05. Yugipedia returned 403 to direct fetches, so its quotes are
search-result excerpts, some cut off.

## Summary

1. Since the April 2020 Master Rule revision (later adopted by the TCG), a trigger effect that
   activates on the field can't activate if its card was destroyed in the same event (official OCG
   DB, Heavymetalfoes Electrumite).
2. "If this card is destroyed/sent to the GY" triggers activate in the GY, so simultaneous
   destruction doesn't stop them. They go into a new chain after the current one, in SEGOC order.
3. Effect destruction happens as the chain link resolves. Battle destruction is decided at damage
   calculation; the monster goes to the GY at the end of the Damage Step.
4. Optional "when" effects miss the timing unless their trigger was one of the last things to
   happen.
5. No general MTG-style state check. Closest: continuous self-destroy effects, applied immediately,
   even mid-chain. Triggers are collected once, after the whole chain.

## Findings

1. Electrumite + Icarus Attack CONFIRMED (official OCG DB). Electrumite ③: "If a card in your
   Pendulum Zone leaves the field: Draw 1 card." Icarus Attack destroys Electrumite and Bismugear
   (Pendulum Zone) together. Answer: 「『自分はデッキから１枚ドローする』モンスター効果はフィールドにて発動する誘発効果です。…同時に「ヘビーメタルフォーゼ・エレクトラム」自身が破壊されフィールドに存在しなくなっている場合には…モンスター効果は発動しません。」
   (a trigger effect that activates on the field; destroyed at the same time and no longer on the
   field, it does not activate).
   https://www.db.yugioh-card.com/yugiohdb/faq_search.action?fid=21602&keyword=&ope=5&request_locale=ja&tag=-1
2. General rule, April 2020 revision CONFIRMED (Konami notice via YGOrganization): "if it is an
   effect that activates when the monster itself is Normal Summoned, it activates on the field, if
   it is a Trap Card whose effect is activated if it is sent to the GY, it activates in the GY.) If
   a card moves location from its activation location to a different location before its effect
   activated, its effect cannot be activated (including mandatory effects)." Sangan banished by
   D.D. Crow mid-chain "does not activate its effect after the Chain is done resolving."
   (https://ygorganization.com/whyarentyoubuyingddcrowsrightnow) Twin Twisters' Hongu destroyed by
   its own ① can't activate its field ② (db fid=23055).
3. TCG adoption PARTLY CONFIRMED: "Konami officially adopted the OCG April 2020 Master Rules
   revision for the TCG." (Cardmarket snippet)
4. Pre-2020 rule, historical only: a trigger could activate if the card was "public knowledge when
   it is time to activate its triggered effect" (Satellarknight Deneb); NA judges differed.
   Superseded. (https://ygorganization.com/advanced-rulings-location-matters/)
5. GY-activating triggers PARTLY CONFIRMED. Supervise: "If the equipped monster and 'Supervise' are
   destroyed at the same time, you can use the effect of 'Supervise' to Special Summon the monster
   it was equipped to."
6. Self "sent to the GY" triggers survive simultaneous destruction CONFIRMED. Demystifying Part 3:
   "Resolve: Dark Hole destroys Dante and Beast, and Cir is also sent to the Graveyard, all of this
   at the same time… Chain Link 1 and Chain Link 2: Dante and Cir (either order) Chain Link 3:
   Beast." (https://ygorganization.com/learnrulingspart3/) Part 4: "Dark Hole destroys Bixi and Card
   Trooper at the same time… both effects can activate."
   (https://ygorganization.com/learnrulingspart4/)
7. Last known information as a general rule NOT FOUND. Card text only (Mask Change: "the same
   Attribute that the sent monster had when it was on the field"). Sun Dragon Inti's effect "does
   not activate in any particular location".
8. Effect destruction at chain-link resolution CONFIRMED (worked examples): "Chain Link 2 resolves:
   Raigeki Break destroys Fire Hand Chain Link 1 resolves: MST destroys its target." Rulebook v10:
   "A card is destroyed when it is sent to the Graveyard due to battle between monsters or by an
   effect that destroys a card."
   (https://img.yugioh-card.com/en/downloads/rulebook/SD_RuleBook_EN_10.pdf)
9. Battle destruction CONFIRMED (Konami Damage Step Rules): "Determine whether either monster will
   be destroyed by battle or not, but do not send it to the Graveyard yet." … "If a monster was
   determined to be destroyed by the battle during damage calculation, send it to the Graveyard
   first. Then effects that activate when a monster is destroyed by battle and sent to the
   Graveyard, activate now." (https://www.yugioh-card.com/eu/play/damage-step-rules/)
10. 0 ATK CONFIRMED: "Monsters with 0 ATK cannot destroy anything by battle." No rule destroys a
    monster for 0 ATK or DEF.
11. Triggers during a chain wait for the whole chain, then a new chain in SEGOC CONFIRMED. "Trigger
    effects always activate after the current chain (if any) resolves." "1: TP mandatory 2: NTP
    mandatory 3: TP optional 4: NTP optional". "After all trigger effects go on the Chain, only then
    can the players Chain with fast effects." Rulebook v10: "If effects of Spell Speed 1 cards (like
    Trigger Effects) are activated at the same time, they will be resolved in a special Chain. This
    Chain is made starting with the turn player's mandatory effects."
12. Missing the timing CONFIRMED. "A 'when' optional effect can only be activated when its
    activation condition was 'one of the last things to happen'… this mechanic does not apply to
    mandatory effects, or effects that use 'if' instead of 'when'." Things destroyed together are
    all "last" (Bixi and Card Trooper). A negated chain link doesn't count as a thing that happened.
13. No general state check between chain links PARTLY CONFIRMED (inferred). Continuous self-destroy
    effects apply at once: Burning Abyss "'If you control a monster that is not a "Burning Abyss"
    monster, destroy this card.' is a Continuous Effect"; Graff summons Cir next to a non-BA monster:
    Cir "will be destroyed immediately by its own effect, but you would not be able to activate its
    effect"; Call of the Haunted destroyed mid-chain "immediately destroys" its monster.

## Gaps

- Yugipedia 403 (Bihamut answer cut off); no official general LKI statement; Konami's TCG
  announcement of the 2020 revision not read; rulebook v10 silent on missing the timing.
