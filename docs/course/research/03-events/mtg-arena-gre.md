# MTG Arena GRE: what the rules engine sends the client

Researcher report, session 03 (node D). Agent 8ea357f9. The protocol is undocumented; labels:
- [PROTO] riQQ/MtgaProto `messages.proto`, decompiled from `Wizards.MDN.GreProtobuf.Unity.dll`
  (https://github.com/riQQ/MtgaProto, raw:
  https://raw.githubusercontent.com/riQQ/MtgaProto/master/messages.proto). Third-party extraction
  of the client's own schema.
- [WOTC-LOG] real `output_log.txt` from GitHub account WotC-Charlie, Jan 2019, client 990.673967
  (https://gist.github.com/WotC-Charlie/c5e7d1fc0b3e296a9a1b0374142c5b07). Old.
- [WOTC-ART] Wizards developer articles on magic.wizards.com. Primary.
- [MANASIGHT] manasight-parser, corpus, blog. Third-party, against 2026 logs.
- [17LANDS] rconroy293/mtga-log-client. Third-party.

## Summary

Protobuf messages (seen as JSON in logs). The client keeps a running state: one
`GameStateType_Full` snapshot, then `GameStateType_Diff` messages applied on top. Each message
carries `annotations` saying what happened (ZoneTransfer with a `category`, ObjectIdChanged,
DamageDealt, ResolutionStart/Complete, ...). Each seat gets its own redacted view; zones have
`Visibility_Public/Private/Hidden` and `viewers`. Opponent hand and library ids appear in zone
lists but no game object (no `grpId`) is sent for them. No source on why ids change or how the
client paces animations.

## 1. Output format (CONFIRMED)

- [PROTO] `message GameStateMessage { GameStateType type = 1; uint32 gameStateId = 2; GameInfo
  gameInfo = 3; repeated TeamInfo teams = 4; repeated PlayerInfo players = 5; TurnInfo turnInfo = 6;
  repeated ZoneInfo zones = 7; repeated GameObjectInfo gameObjects = 8; repeated AnnotationInfo
  annotations = 9; repeated uint32 diffDeletedInstanceIds = 10; uint32 pendingMessageCount = 11;
  uint32 prevGameStateId = 12; ... GameStateUpdate update = 14; repeated ActionInfo actions = 15;
  repeated AnnotationInfo persistentAnnotations = 16; repeated uint32
  diffDeletedPersistentAnnotationIds = 17; }`. `enum GameStateType { None_acfa = 0; Full = 1; Diff
  = 2; Binary = 3; }`.
- Full then Diffs chained by id. [WOTC-LOG]: `"gameStateId": 1, "gameStateMessage": { "type":
  "GameStateType_Full", ...`. [MANASIGHT] fixture: `"type": "GameStateType_Diff", "gameStateId": 3,
  ... "prevGameStateId": 2`
  (https://docs.rs/crate/manasight-parser/latest/source/tests/fixtures/diff_gsm_prev_id_corpus_slice.log).
  The manasight blog wrongly says "There's no explicit flag"; their issue #182 shows it exists.
- Merge and delete. [MANASIGHT] blog: "`diffDeletedInstanceIds`: Instance IDs that should be purged
  from your local game state… Your parser needs to merge incoming fields into a running game state
  and remove anything listed in `diffDeletedInstanceIds`." (https://blog.manasight.gg/arena-log-format-guide/)
- Annotations. [PROTO] `message AnnotationInfo { uint32 id; uint32 affectorId; repeated uint32
  affectedIds; repeated AnnotationType type; repeated KeyValuePairInfo details; bool
  redactAffector; repeated uint32 excludedSeatIds; bool redactAffected; repeated uint32
  temporarilyExcludedSeatIds; }`. [MANASIGHT] `annotations.rs`: "Persistent annotations accumulate
  across game state updates (unlike ephemeral `annotations` which appear only in the diff). They
  include targeting (`TargetSpec`), trigger attribution (`TriggeringObject`)…"
- `enum AnnotationType` 0..117, e.g. `ZoneTransfer=1`, `LossOfGame=2`, `DamageDealt=3`,
  `TappedUntappedPermanent=4`, `PhaseOrStepModified=8`, `ModifiedLife=10`, `ObjectIdChanged=13`,
  `CounterAdded=16`, `LayeredEffectCreated=18`, `TargetSpec=26`, `TriggeringObject=32`,
  `ManaPaid=34`, `TokenCreated=35`, `ResolutionStart=43`, `ResolutionComplete=44`,
  `CardRevealed=47`, `DisqualifiedEffect=50`, `Shuffle=56`, `EnteredZoneThisTurn=63`,
  `InstanceRevealedToOpponent=75`, `GamewideHistoryCount=81`, `DamagedThisTurn=90`,
  `HiddenInformation=105`.
- Details [WOTC-LOG]: ZoneTransfer has `zone_src`, `zone_dest`, `category` (seen: `"Draw"`,
  `"PlayLand"`, `"CastSpell"`, `"Resolve"`, `"Countered"`; 2026: `SBA_Damage`, "Raw GRE shows
  `DamageDealt` (id 143) precedes `ZoneTransfer SBA_Damage` (id 152)",
  https://github.com/manasight/manasight-corpus/pull/24). DamageDealt: `"affectorId": 295,
  "affectedIds": [ 1 ], "type": [ "AnnotationType_DamageDealt" ], "details": [ { "key": "damage",
  ... [ 1 ] }, { "key": "type", ...`. ResolutionStart: `"details": [ { "key": "grpid", ... [ 67750 ]
  } ]`. `type` is always an array; `details` is typed key/value.

## 2. Per-player view (CONFIRMED)

- [PROTO] `message GREToClientMessage { GREMessageType type = 1; repeated uint32 systemSeatIds = 2;
  uint32 msgId = 3; uint32 gameStateId = 4; ...}`. `message GameStateRedactorConfiguration { bool
  enableRedaction = 1; bool enableForceDiff = 2; bool enableZoneRedaction = 3; }`.
- [PROTO] `enum Visibility { None_a643 = 0; Public = 1; Private = 2; Hidden = 3; Deceptive = 4; }`.
  `message ZoneInfo { uint32 zoneId; ZoneType type; Visibility visibility; uint32 ownerSeatId;
  repeated uint32 objectInstanceIds; repeated uint32 viewers; }`. `GameObjectInfo` has `Visibility
  visibility = 6`, `repeated uint32 viewers = 26`.
- [WOTC-LOG] as seen by seat 1: `{ "zoneId": 35, "type": "ZoneType_Hand", "visibility":
  "Visibility_Private", "ownerSeatId": 2, "objectInstanceIds": [ 229, 228, 227, 226, 225, 224, 223 ],
  "viewers": [ 2 ] }, { "zoneId": 36, "type": "ZoneType_Library", "visibility": "Visibility_Hidden",
  "ownerSeatId": 2, ...`
- Hidden cards are ids only (researcher's own search of the 2019 log): no `"instanceId": 229` or
  `284` object anywhere. Own cards get full objects (`"instanceId": 283, "grpId": 67582, ...`). An
  opponent land becomes a new public object with a grpId (`"instanceId": 285, "grpId": 68212`).
  [17LANDS] `self.cards_in_hand[owner] = [player_objects.get(instance_id) for instance_id in
  hand_card_ids ...]` yields None for unknown ids. Not checked against 2026 logs.

## 3. Identity (CONFIRMED mechanics, reason NOT FOUND)

- [WOTC-LOG] `"affectedIds": [ 108 ], "type": [ "AnnotationType_ObjectIdChanged" ], "details": [ {
  "key": "orig_id", ... [ 108 ] }, { "key": "new_id", ... [ 283 ] } ]`, right after
  `PerformActionResp` with `"InstanceId": 108` and a `PlayLand` ZoneTransfer. Old ids appear in
  `ZoneType_Limbo`: `"objectInstanceIds": [ 230, 108 ]`.
- Draws re-id: opponent library id 230 gets ObjectIdChanged after the draw step; the hand then
  shows 284. [MANASIGHT]: `AnnotationType_Shuffle` "(reissues every library instanceId)" with
  `OldIds`/`NewIds`.
- Why (400.7 vs hiding info): NOT FOUND. Facts fit both. [WOTC-ART]: for a resolving permanent
  spell, "it's still… 'the same object'… We tracked that by having that object keep its ID number",
  so the engine's internal id may differ from the wire `instanceId`.

## 4. Pacing (NOT FOUND)

- [PROTO] `enum GameStateUpdate { Send = 1; SendAndRecord = 2; SendHiFi = 3; Undo = 4; Restore = 5;
  }`; logs show `"update": "GameStateUpdate_SendHiFi"` often; meaning undocumented. Also
  `pendingMessageCount`, `GREMessageType_QueuedGameStateMessage` ([MANASIGHT]: "a deferred game
  state update"). Over half of GRE events bundle 2+ GameStateMessages.
- [WOTC-ART] Zurgo dev diary: "a 'disqualified pulse,' a little animation that plays when some
  object in the game stops something from happening… the creature would just be spamming
  disqualified pulses" (https://magic.wizards.com/en/news/mtg-arena/dev-diary-zurgo-thunders-decree).
  `AnnotationType_DisqualifiedEffect=50` exists; no source links them.
- [PROTO] `message UIMessage { repeated uint32 seatIds; oneof { OnSelect; OnHover; OnShuffle;
  OnChat; OnGenericEvent } }` (emotes, hovers).
- Acknowledgement: requests are answered by echoing the id (`ActionsAvailableReq ... "msgId": 17,
  "gameStateId": 6` answered by `"GameStateId": 6, "RespId": 17, "PerformActionResp"`). No evidence
  of acks for plain GameStateMessages. `FailureReason` has `Expired`, `ReqRespMismatch`,
  `UnexpectedMessageId`.

## 5. Engine design ([WOTC-ART])

Source: https://magic.wizards.com/en/news/mtg-arena/on-whiteboards-naps-and-living-breakthrough
- "It's written in a combination of C++ and a language called CLIPS, which is a variant of LISP."
  (CLIPS is actually a rule-based expert-system language; that's the article's wording.)
- "the Game Rules Parser (GRP). This program (written in Python) takes raw English rules text of
  Magic cards and converts them into one or more CLIPS rules. It's what allows 80% or so of newly
  written Magic cards to just work." The GRE "does not know… what any of the thousands of
  individual Magic cards do." CLIPS rules edit a "whiteboard" while the GRE "naps". Zurgo article:
  rules grouped into "agendas… abilities triggering, replacement effects, or state-based actions."
- History: "We have a nice system for finding last-known-information about spells that resolved in
  the past." Living Breakthrough posted a note ("Player 1 cast spell #278") and later had to store
  the mana value itself, because looking up the object returned its current value.
- "It sends a message asking the client to make a choice, and it's the responsibility of the DS
  [Duel Scene] team to display that choice." X-value request gained "a list of values that are
  specifically not allowed" (matches `disallowedValues` in `NumericInputReq`). Rules tested
  "without a graphical client at all, just little scripted text games" ("nearly 5,000" regression
  tests per the Zurgo article).

## Gaps

Why ids are reissued. Meaning of `GameStateUpdate` values, `QueuedGameStateMessage`,
`pendingMessageCount`; animation pacing. Whether 2026 opponent hands still send no objects. Full
ZoneTransfer category list.
