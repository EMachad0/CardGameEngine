# Marvel Snap, Shadowverse, Gwent: simultaneous destruction and trigger timing

Researcher report, session 05. Fandom wikis (Snap, Shadowverse, Gwent) returned 403; only their
search snippets were used. Shadowverse sources are a guide site and a community wiki; Gwent sources
are player-written.

## Summary

Marvel Snap: one destroy effect locks its targets up front, then destroys them one at a time in play
order, and each card's "when destroyed" resolves before the next is destroyed (Second Dinner
developer). Shadowverse: a follower destroyed by the same sweep doesn't trigger its own "whenever
another follower is destroyed"; Last Words resolve turn player first, then oldest first, before "when
destroyed" triggers, without interruption (guide and community wiki). Gwent: Deathwish fires on
moving from board to graveyard; simultaneous effects resolve melee before ranged, left to right, turn
player first (player-documented, disputed for Deathwish); no source on a watcher dying in the same
sweep.

## Marvel Snap

1. Watcher destroyed in the same sweep NOT FOUND. Firehair: "When one of your cards is destroyed,
   this uses that card's On Reveal." Dev: "she just uses each On Reveal effect she sees on a card
   that dies" (https://marvelsnapzone.com/developer-update-for-the-week-of-march-15-2025/). PARTLY
   CONFIRMED: "Destroy triggers in our game only resolve if the card with the trigger is revealed"
   (glenncj3, https://marvelsnapzone.com/answers/wolverinex-23-destroy-interactions/).
2. Own "when destroyed" resolves, in play order, before the next card is destroyed CONFIRMED.
   Nimrod: "When Destroyed: Add a copy of this to each other location". Dev: "Destroyer blows things
   up in the order in which they were played… Caiera was played before the other 1 and 6-Cost cards,
   so she was already destroyed when it was their turn"
   (https://marvelsnapzone.com/cards/destroyer/). Reddit: "destroy effects from Carnage, Venom,
   Deathlock, etc, destroy cards in the order that they arrived at the location".
3. Targets locked once, destroyed one by one, board changes between CONFIRMED. "her ability looks
   for every valid target on resolution and then locks them in. Everything in the game is coded that
   way; for example, Carnage won't eat Angel if the first card he eats summons Angel into play"
   (https://marvelsnapzone.com/answers/lady-deathstrike-interaction/). "Destroyer doesn't recheck… If
   he did, he would kill Bucky, then kill Winter Soldier. He doesn't." No separate death check;
   destruction is an explicit effect, not damage.
4. Order of simultaneous triggers PARTLY CONFIRMED. "Card abilities will activate before location
   abilities, in the order they're revealed" (Help Center); dev: "locations happen before cards when
   their triggers are the same"; locations "should activate from left to right"; "The player who is
   currently winning will reveal their cards first. If it is currently a tie, the reveal order is
   random" (https://marvelsnap.helpshift.com/hc/en/3-marvel-snap/faq/38-whose-cards-are-revealed-first-during-a-turn/).

## Shadowverse

1. "Whenever another allied follower is destroyed" dying in a wipe does not trigger PARTLY CONFIRMED
   (community). Reddit: "when your pair of Wights gets aoe'd, their ability doesn't trigger"; "they
   still wouldnt trigger. This isnt MtG". atwiki: 「ラストワード・「場を離れる時」以外の効果は、解決前に効果を持ったカードが場を離れた場合解決されない」
   (effects other than Last Words and "when this leaves play" don't resolve if their card left play
   before resolution) (https://w.atwiki.jp/svkoukasyori/pages/16.html, 2022, original Shadowverse).
2. Last Words resolve CONFIRMED (GameWith, not Cygames). Glossary: "Effects activate when the card
   is destroyed. They will not activate if the card is banished or transformed". Order: "the current
   turn player's Last Words will activate first… processed in order, starting from the card that was
   summoned to the field the earliest" (https://gamewith.net/shadowverse-wb/69084). Summary order: 1.
   turn player's Last Words, 2. non-turn player's, 3. "when destroyed" effects, 4. newly added Last
   Words.
3. Destruction at 0 deferred PARTLY CONFIRMED (atwiki): 「体力0フォロワーの破壊処理は優先度が高いが体力0になった瞬間ではない。全体ダメージによる破壊の場合、「場にある間有効」効果が後ろのフォロワーにかかった状態でダメージが入る」
   (destroying a 0-defense follower is high priority but not instant; with area damage, "while on
   the field" auras still apply to later followers when hit). Multi-target handled 「1ターゲットずつ」.
   Random multi-destroy is sequential. Grimnir: "all four instances resolved first before any other
   effects".
4. Triggers queued, no interruption CONFIRMED (guide, community wiki). "while Last Words is being
   activated… no other abilities will activate until the current chain of Last Words activations is
   complete". atwiki: 「効果は誘発した順に解決される。ひとつの効果の解決中に他の誘発効果は割り込まない」.
   Order: 「ターンプレイヤー→非ターンプレイヤー… 盤面の左(先に出したカード)から順に」. Last Words before "when
   destroyed": 「[先]ラストワード後破壊された時」.

## Gwent

1. Watcher dying in the same sweep NOT FOUND. Arachas Queen: "Whenever an allied unit is destroyed
   during your turn, Spawn an Arachas Drone".
2. Deathwish resolves CONFIRMED: "Deathwish: Trigger this ability when destroyed and moved from the
   battlefield to the graveyard. Note: Banished units are not sent to the graveyard"
   (https://gwent.one/en/card/202531). Order PARTLY CONFIRMED and disputed: "They trigger in left to
   right, melee to siege priority afaik"; a reported game contradicts it ("I think it might be
   random").
3. Removal timing NOT FOUND as a rule. CDPR forum (Treamayne, quoted): "a unit dying before using
   it's ability doesn't get to use that ability".
4. Order PARTLY CONFIRMED (player guide claiming a dev explanation): "Turn player's Melee Row from
   left to right ↓ Opposing player's Melee Row… ↓ Turn player's Ranged Row… ↓ Opposing player's
   Ranged Row"; conditional triggers "will always be applied immediately after their trigger is fully
   resolved". Unofficial ruleset 109.2: "Abilities that should resolve at the same time are resolved
   from left to right, melee row to ranged row".

## Gaps

- No first-party Cygames or CDPR timing text; Snap watcher-in-sweep (Firehair) untested; Gwent
  Deathwish mid-effect vs after-effect undocumented; atwiki predates Worlds Beyond (agrees with
  GameWith on Last Words order).
