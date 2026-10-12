# Pending decisions in production card game protocols

Researcher report, session 06.

## Summary

All three production protocols use a typed decision request: the server asks one player one
question, and the client answers with a matching typed response. MTG Arena has about 30 `*Req`
kinds in one `GREToClientMessage` oneof, addressed by `systemSeatIds`, with `prompt`,
`allowCancel` and `allowUndo` on the envelope, and the decider visible to everyone as
`turnInfo.decisionPlayer`. Hearthstone has two request kinds: `AllOptions` (the turn's legal
actions with sub-options and targets) and `EntityChoices` (`choice_type`, `count_min`/`count_max`,
`entities`, `source`, `player_id`); its example replay shows both players' mulligan choices open at
once. OpenSpiel puts simultaneity in the model: `CurrentPlayer() == kSimultaneousPlayerId (-2)`
marks a joint node taking one action per player through `ApplyActions`.

## 1. MTG Arena GRE protocol

Source: riQQ/MtgaProto `messages.proto`, extracted from `Wizards.MDN.GreProtobuf.Unity.dll`
(https://raw.githubusercontent.com/riQQ/MtgaProto/master/messages.proto). The extractor adds hash
suffixes (`None_a526`) and lowercases some letters (`SelectNreq`); real logs use
`GREMessageType_MulliganReq`.

1. CONFIRMED, envelope:
   ```
   message GREToClientMessage {
   	GREMessageType type = 1;
   	repeated uint32 systemSeatIds = 2;
   	uint32 msgId = 3;
   	uint32 gameStateId = 4;
   	oneof message { GameStateMessage gameStateMessage = 5; ... ActionsAvailableReq actionsAvailableReq = 9; OrderReq orderReq = 10; GroupReq groupReq = 11; SelectNReq selectNReq = 12; MulliganReq mulliganReq = 14; ... SelectTargetsReq selectTargetsReq = 32; PayCostsReq payCostsReq = 34; ... NumericInputReq numericInputReq = 42; SearchReq searchReq = 43; CastingTimeOptionsReq castingTimeOptionsReq = 44; ... }
   	Prompt prompt = 6;
   	AllowCancel allowCancel = 50;
   	bool allowUndo = 56;
   }
   ```
   `enum AllowCancel { None_a526 = 0; Continue = 1; Abort = 2; No_a526 = 3; }`.
   `message Prompt { uint32 promptId = 1; repeated PromptParameter parameters = 2; }` is a
   localization id plus parameters.
2. CONFIRMED, `GREMessageType` includes `ActionsAvailableReq`, `ChooseStartingPlayerReq`,
   `GroupReq`, `IllegalRequest`, `MulliganReq`, `OrderReq`, `PromptReq`, `SelectNreq`,
   `DeclareAttackersReq`, `DeclareBlockersReq`, `AssignDamageReq`, `SelectTargetsReq`,
   `PayCostsReq`, `SelectReplacementReq`, `DistributionReq`, `NumericInputReq`, `SearchReq`,
   `OptionalActionMessage`, `CastingTimeOptionsReq`, `SelectFromGroupsReq`, `GatherReq`,
   `TimeoutMessage`, `TimerStateMessage`, `StringInputReq`, `SelectCountersReq`.
3. CONFIRMED, responses: `ClientMessageType` includes `CancelActionReq`, `ConcedeReq`,
   `GroupResp`, `MulliganResp`, `OrderResp`, `PerformActionResp`, `SelectNresp`, `UndoReq`,
   `OptionalActionResp`, `DeclareAttackersResp`, `SubmitAttackersReq`, `SelectTargetsResp`,
   `SubmitTargetsReq`, `NumericInputResp`, `SearchResp`, `CastingTimeOptionsResp`, ... Envelope
   `message ClientToGREMessage { ClientMessageType type = 1; uint32 systemSeatId = 2; uint32 gameStateId = 3; uint32 respId = 4; oneof message {...} }`.
   Log: `ClientMessageType_SelectNResp` with `"selectNResp": { "ids": [ 280 ] }`
   (https://github.com/manasight/manasight-parser/issues/206). UNVERIFIED: that `respId` echoes
   `msgId`.
4. CONFIRMED: combat and targeting have separate toggle and submit messages
   (`DeclareAttackersResp`/`SubmitAttackersReq`, `SelectTargetsResp`/`SubmitTargetsReq`), with
   `enum SelectAction { None_a1ad = 0; Select_a1ad = 1; Unselect = 2; }`. UNVERIFIED: the flow
   (toggle one at a time, updated Req after each, Submit closes it), inferred from names.
5. CONFIRMED payloads:
   `SelectNReq { int32 minSel = 1; uint32 maxSel = 2; ... repeated uint32 ids = 7; ... uint32 sourceId = 15; ... }`;
   `SelectTargetsReq { repeated TargetSelection targets = 1; uint32 sourceId = 2; uint32 abilityGrpId = 3; }`
   with `TargetSelection { uint32 targetIdx = 1; repeated Target targets = 2; uint32 minTargets = 3; uint32 maxTargets = 4; uint32 selectedTargets = 5; ... }`;
   `NumericInputReq { uint32 minValue = 2; uint32 maxValue = 3; ... }`;
   `PayCostsReq { ... ActionsAvailableReq paymentActions = 2; SelectNReq paymentSelection = 3; ... }`
   (requests nest); `GroupReq` with `enum GroupingContext { None_a0f6 = 0; Scry_a0f6 = 1; Surveil = 2; LondonMulligan = 3; }`.
6. CONFIRMED: `CastingTimeOptionsReq { repeated CastingTimeOptionReq castingTimeOptionReq = 1; }`,
   each with `uint32 playerIdToPrompt = 6; bool isRequired = 7; ... oneof message { NumericInputReq ...; ModalReq ...; SelectNReq ...; }`.
7. CONFIRMED, who decides is in state:
   `TurnInfo { ... uint32 activePlayer = 4; uint32 priorityPlayer = 5; uint32 decisionPlayer = 6; ...}`
   and `PlayerInfo { ... ClientMessageType pendingMessageType = 16; ...}`. Real log:
   `"turnInfo": { "activePlayer": 2, "decisionPlayer": 1 }`
   (https://docs.rs/crate/manasight-parser/latest/source/tests/fixtures/diff_gsm_prev_id_corpus_slice.log).
8. CONFIRMED, addressing in a real log: a `MulliganReq` to `"systemSeatIds": [ 1 ]`; an
   informational `PromptReq` to `[ 1, 2 ]`.
9. UNVERIFIED as policy: one outstanding request at a time. Evidence: one `decisionPlayer`, and
   `FailureReason` values `Expired`, `OutOfTurn`, `ReqRespMismatch`, `UnexpectedMessage`,
   `InvalidCancelState`, `InvalidUndoState`, `UnexpectedMessageId`. `pendingMessageType` is per
   player, so the schema could hold one per seat.
10. UNVERIFIED: whether both seats get `MulliganReq` at once.
11. CONFIRMED fields: `TimerStateMessage`, `TimeoutMessage`, `enum TimerType { ... Decision = 1; ... }`,
    `enum TimerEvent { ... OnRequestSent = 1; OnResponseReceived = 2; OnExpiration = 3; ...}`.
    No dedicated "waiting" message found; the waiting player likely reads `decisionPlayer`.
12. CONFIRMED: `UndoReq` and `CancelActionReq` from the client; the server sets `allowUndo` and
    `allowCancel` per request.

## 2. Hearthstone (HearthSim)

1. CONFIRMED, python-hearthstone `enums.py`:
   `class ChoiceType(IntEnum): """CHOICE_TYPE""" INVALID = 0 MULLIGAN = 1 GENERAL = 2 TARGET = 3`
   and `class OptionType(IntEnum): PASS = 1 END_TURN = 2 POWER = 3`.
   https://raw.githubusercontent.com/HearthSim/python-hearthstone/master/hearthstone/enums.py
2. CONFIRMED, hs-proto `pegasusgame.proto` (client 13.0.9.28556):
   `message AllOptions { int32 id = 1; repeated Option options = 2; }`,
   `message Option { Type type = 1; SubOption main_option = 2; repeated SubOption sub_options = 3; }`,
   `message SubOption { int32 id = 1; repeated TargetOption targets = 3; int32 play_error = 4; ... }`,
   `message ChooseOption { int32 id = 1; int32 index = 2; int32 target = 3; int32 sub_option = 4; int32 position = 5; }`,
   `message EntityChoices { int32 id = 1; int32 choice_type = 2; int32 count_min = 4; int32 count_max = 5; repeated int32 entities = 6; int32 source = 7; int32 player_id = 8; bool hide_chosen = 9; }`,
   `message ChooseEntities { int32 id = 1; repeated int32 entities = 2; }`,
   `message NAckOption { int32 id = 1; }`.
   https://raw.githubusercontent.com/HearthSim/hs-proto/master/pegasus/pegasusgame/pegasusgame.proto
3. CONFIRMED, HearthSim Gamestate Protocol: "the server will hand off input to a player by giving
   that player 'Options'. This blocks further state changes from happening until that player
   executes an option, or the server says otherwise (eg. a rope timeout or a disconnect)." "Options
   are not part of what we call the game state. They have their own 'Option State'."
   https://hearthsim.info/docs/gamestate-protocol/
4. CONFIRMED, HSReplay DTD: `Choices` "entity: The entity ID of the player the choices are
   presented to ... type: A ChoiceType enum value representing the type of choice (MULLIGAN vs.
   GENERAL)"; `SendChoices` "May contain no choice as not sending a choice is valid, eg. in
   Mulligan"; "Power.log does not output ChosenEntities for the player who sent the choices."
   https://raw.githubusercontent.com/HearthSim/hsreplay-xml/master/hsreplay.dtd
5. PARTLY CONFIRMED: Tracking and Kazakus log `ChoiceType=GENERAL CountMin=1 CountMax=1`, with the
   candidates in `SETASIDE` (https://gist.github.com/Jetz72/706f59414a5f3ede4a47e2dc18012591).
   UNVERIFIED: a log of a Discover-keyword card.
6. CONFIRMED, the official example replay (https://hearthsim.info/hsreplay/): `<Choices entity="2" id="1" ... type="1" ...>`
   is followed, before any answer, by `<Choices entity="3" id="2" ... type="1" ...>`; player 3
   answers first, and player 2's `SendChoices id="1"` comes 16 seconds later. Two choices are
   pending at once, one per player. The waiting state shows in the per-player tag
   `MULLIGAN_STATE` (`INPUT = 1 DEALING = 2 WAITING = 3 DONE = 4`).

## 3. OpenSpiel

1. CONFIRMED, `spiel_globals.h`: `kSimultaneousPlayerId = -2`.
2. CONFIRMED, `spiel.h`: `bool IsSimultaneousNode() const { return CurrentPlayer() == kSimultaneousPlayerId; }`;
   `IsPlayerActing(player)` returns `CurrentPlayer() == player || IsSimultaneousNode()`;
   `Dynamics { kSimultaneous, // In some or all nodes every player acts. kSequential, // Turn-based games. ...}`.
3. CONFIRMED, `ApplyActions`: "one per player, for simultaneous action games. ... Element i of the
   vector is the action for player i. Every player must submit a action. If some of the players
   have no legal actions at this node, then 0 should be passed instead." `ApplyAction` covers
   "turn-based games or in non-simultaneous nodes of simultaneous move games".
4. CONFIRMED that a transform exists (`turn_based_simultaneous_game.h`): "This wrapper turns any
   n-player simultaneous move game into an equivalent turn-based game where simultaneous move nodes
   are encoded as n turns." UNVERIFIED that games must use it.

## 4. Other engines

1. CONFIRMED, Manabrew protocol (https://docs.manabrew.app/protocol/): a prompt is "An
   `AgentPrompt` - a call to action. The engine has paused and needs a decision from a specific
   player." `interface AgentPrompt { promptId: number; decidingPlayerId: string; sourceCard?: CardDto; input: PromptInput; }`.
   Response `{ kind: "response"; promptId: number; action: PromptOutput }`, "Strictly paired: one
   response per prompt". Errors: `"stalePrompt" | "wrongPlayer" | "wrongPromptType" | "unknownActionId" | "invalidShape"`.
   "Prompts deliberately carry no `gameView`." Catalogue includes chooseNumber, chooseCards,
   chooseBoolean, chooseFromSelection, scry, reorder, chooseAction, payManaCost, mulligan,
   chooseAttackers, chooseBlockers, chooseBoardTargets.
2. CONFIRMED, Forge `ProtocolMethod.java`: server-to-client calls with return values (`getChoices`,
   `confirm`, `order`, `chooseSingleEntityForEffect`, ...); client-to-server calls "should all
   return void, to avoid awkward situations in which client and server wait for one another's
   response".
3. CONFIRMED, XMage `ClientCallbackMethod.java`: `GAME_TARGET`, `GAME_CHOOSE_ABILITY`,
   `GAME_CHOOSE_CHOICE`, `GAME_ASK`, `GAME_SELECT`, `GAME_GET_AMOUNT`, ... as `DIALOG` callbacks;
   waiting players get `GAME_UPDATE_AND_INFORM`.
4. CONFIRMED: Legends of Runeterra's public client API is read-only (decklist, card positions,
   game result). https://developer.riotgames.com/docs/lor

## Gaps

- MTGA: whether `respId` echoes `msgId`, whether two seats' requests are ever open at once, and
  the toggle-then-submit flow.
- Hearthstone: what `ChoiceType.TARGET` is for; hs-proto is from client 13.0.
- Shadowverse and Marvel Snap: no public protocol found.
- MTG CR 103.5 (mulligan order) not fetched.
