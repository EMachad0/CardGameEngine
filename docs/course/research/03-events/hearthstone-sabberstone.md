# Hearthstone protocol and SabberStone: what happened

Researcher report, session 03 (node D). Agent bcbeb33a. SabberStone read on `master` (no line
numbers; locate by quote). The Blizzard server is closed: every Hearthstone server-side claim is
inferred from what reaches the client (Power.log, HearthSim docs) or from the community Advanced
Rulebook.

## Summary

- Hearthstone: per-turn and per-game counters are integer tags on entities (e.g.
  `NUM_MINIONS_PLAYED_THIS_TURN = 317` on the Player entity), sent like any other tag. The client
  gets a flat stream of state changes (`FULL_ENTITY`, `TAG_CHANGE`, ...) grouped in nested
  `BLOCK_START`/`BLOCK_END` blocks recording what caused what. HSReplay saves that stream as XML;
  Joust plays it back by applying tag diffs.
- SabberStone: same tag counters plus a per-player `PlayHistory` list. Triggers subscribe as C#
  delegates to events on a `TriggerManager` and enqueue tasks on a `TaskQueue`. Death handling
  follows the wiki's Death Creation Step. `Game.PowerHistory` exists only when
  `GameConfig.History` is true, and `Game.Clone()` turns it off by default.

## Q1. Rules record

Hearthstone:
- CONFIRMED: counters as GameTags (python-hearthstone `enums.py`): `NUM_CARDS_PLAYED_THIS_TURN =
  269`, `NUM_MINIONS_PLAYED_THIS_TURN = 317`, `NUM_MINIONS_KILLED_THIS_TURN = 369`,
  `NUM_FRIENDLY_MINIONS_THAT_DIED_THIS_TURN = 398`, `NUM_CARDS_DRAWN_THIS_TURN = 399`,
  `NUM_FRIENDLY_MINIONS_THAT_DIED_THIS_GAME = 412`, `NUM_RESOURCES_SPENT_THIS_GAME = 418`,
  `NUM_TIMES_HERO_POWER_USED_THIS_GAME = 394`, `NUM_SPELLS_PLAYED_THIS_GAME = 1780`.
  https://raw.githubusercontent.com/HearthSim/python-hearthstone/master/hearthstone/enums.py
- CONFIRMED: on the Player entity, sent as `TAG_CHANGE`, reset each turn. Power.log:
  `TAG_CHANGE Entity=Patashu tag=NUM_MINIONS_PLAYED_THIS_TURN value=1`, next turn
  `TAG_CHANGE Entity=The Innkeeper tag=NUM_MINIONS_PLAYED_THIS_TURN value=0`.
  https://gist.github.com/Patashu/f62502927732c17058cd . HSReplay example inside a PLAY block
  (`type="7"`): `<TagChange entity="2" tag="269" value="1"/> <TagChange entity="2" tag="317"
  value="1"/>` and `tag="418"`. https://hearthsim.info/hsreplay/
- CONFIRMED: "There are also plenty of tags which are not sent down to the client at all because
  the client does not need them, or should not know about them."
  https://hearthsim.info/docs/gamestate-protocol/
- NOT FOUND: how server card scripts query counters, or whether a hidden event list exists.

SabberStone:
- CONFIRMED: tag-backed properties on `Controller`
  (`SabberStoneCore/src/Model/Entities/Controller.cs`): `public int NumMinionsPlayedThisTurn { get
  { _data.TryGetValue(GameTag.NUM_MINIONS_PLAYED_THIS_TURN, ...`, `public int
  NumSpellsPlayedThisGame { ... GameTag.NUM_SPELLS_PLAYED_THIS_GAME ...`,
  `NumFriendlyMinionsThatDiedThisTurn`.
- CONFIRMED: list records: `public readonly List<Card> CardsPlayedThisTurn; public readonly
  List<PlayHistoryEntry> PlayHistory;`. `GraveyardZone`: "The zone containing all entities which
  were once in play, but now destroyed." `PlayHistoryEntry` is a `readonly struct`
  (`SabberStoneCore/src/Model/PlayHistory.cs`) with `SourceController, TargetController,
  SourceCard, TargetCard, SubOption, SourceId`.
- CONFIRMED: written in `SabberStoneCore/src/Actions/PlayCard.cs`: `c.NumCardsPlayedThisTurn++;`
  ... `// record played cards for effect of cards like Obsidian Shard and Lynessa Sunsorrow`
  `c.CardsPlayedThisTurn.Add(source.Card); c.PlayHistory.Add(new PlayHistoryEntry(...));`, plus
  `c.NumMinionsPlayedThisTurn++` in `PlayMinion`, `c.NumSpellsPlayedThisGame++` in `PlaySpell`.
- CONFIRMED: reset in `Game.MainReady()` (`c.NumCardsPlayedThisTurn = 0; c.NumMinionsPlayedThisTurn
  = 0; ... c.NumFriendlyMinionsThatDiedThisTurn = 0;`) and `MainEnd()`
  (`CurrentPlayer.CardsPlayedThisTurn.Clear();`).
- PARTLY CONFIRMED: card code reads them (comment names Obsidian Shard and Lynessa; cards not opened).

## Q2. Trigger input and timing

Hearthstone (community rulebook, https://hearthstone.wiki.gg/wiki/Advanced_rulebook):
- "Rule 2a: When we start to resolve a Phase or Event, A Queue is created and filled with all
  triggers that can respond, in order of play."
- "Rule 1c: Death is not checked during the standard resolution of Events." "Rule 4a: After the
  outermost Phase ends, Hearthstone does an Aura Update (Health/Attack), then does the Death
  Creation Step (Looks for all mortally wounded … Entities and kills them, removing them from play
  simultaneously), then does an Aura Update (Other)." Rule 7: a Death Phase follows if any deaths.
- "There are actually 5 … Aura Update (Health/Attack) Step, Summon Resolution Step, Aura Update
  (Health/Attack) Step, Death Creation Step, Aura Update (Other) Step, 'Next Phase'". "Whenever a
  minion is summoned, a Sequence immediately begins with an 'On Summon' and ends with an 'After
  Summon' Phase."
- NOT FOUND: the server's actual mechanism.

SabberStone:
- CONFIRMED: `TriggerManager` (`SabberStoneCore/src/Model/TriggerManager.cs`) is C# delegate
  events: `public delegate void TriggerHandler(IEntity sender); public event TriggerHandler
  DeathTrigger; ...`, `internal void OnDeathTrigger(IEntity sender) { DeathTrigger?.Invoke(sender);
  }`, `AddTrigger(TriggerType type, TriggerHandler method)`.
- CONFIRMED: `TriggerType` (`SabberStoneCore/src/Enums/Triggers.cs`): `TURN_END, TURN_START, DEATH,
  DEAL_DAMAGE, TAKE_DAMAGE, SUMMON, AFTER_SUMMON, PLAY_CARD, CAST_SPELL, AFTER_CAST, ...`. Side
  note: `case TriggerType.AFTER_PLAY_CARD:` adds to `AfterPlayMinionTrigger`.
- CONFIRMED: `SabberStoneCore/src/Triggers/Trigger.cs`: "During activation, the instance's
  Process(IEntity) subscribes to the events in TriggerManager." e.g.
  `source.Game.TriggerManager.DealDamageTrigger += instance._processHandler;`. `ProcessInternal`:
  `Game.TaskQueue.Enqueue(SingleTask, _owner.Controller, _owner, ...)`, or executes immediately
  (`Game.TaskQueue.Execute(...)`) with `FastExecution` (set for `TURN_END`, `WORGEN_TRANSFORM`).
  `ValidateTriggers(...)`: "Checks triggers related to the current Sequence at once before the
  Sequence starts."
- CONFIRMED: phases bracketed in action code, e.g. `PlayMinion`: `game.TaskQueue.StartEvent();
  game.TriggerManager.OnSummonTrigger(minion); game.ProcessTasks(); game.TaskQueue.EndEvent();`
  then `game.DeathProcessingAndAuraUpdate();`. Comments use wiki phase names.
- CONFIRMED: `Game.cs` `GraveYard()`: "Death Creation Step (Death event is created but not resolved
  here)", calls `TriggerManager.OnDeathTrigger(minion);` per dead minion sorted by `OrderOfPlay`.
  `DeathProcessingAndAuraUpdate()`: `do { GraveYard(); // Death Creation Step  ProcessTasks(); //
  Death Resolution Phase } while (DeadMinions.Count != 0); ... AuraUpdate();`.

## Q3. Output to client

Hearthstone:
- CONFIRMED PowerType: `FULL_ENTITY=1, SHOW_ENTITY=2, HIDE_ENTITY=3, TAG_CHANGE=4, BLOCK_START=5,
  BLOCK_END=6, CREATE_GAME=7, META_DATA=8, CHANGE_ENTITY=9, RESET_GAME=10, SUB_SPELL_START=11,
  SUB_SPELL_END=12, VO_SPELL=13, CACHED_TAG_FOR_DORMANT_CHANGE=14, SHUFFLE_DECK=15, VO_BANTER=16,
  TAG_LIST_CHANGE=17` ("Renamed in 12574 ACTION_START = BLOCK_START").
- CONFIRMED BlockType: `INVALID=0, ATTACK=1, JOUST=2, POWER=3, TRIGGER=5, DEATHS=6, PLAY=7,
  FATIGUE=8, REVEAL_CARD=10, GAME_RESET=11, MOVE_MINION=12, DECK_ACTION=13`; removed `SCRIPT=4,
  RITUAL=9, ACTION=99`; `TRADE = DECK_ACTION`.
- CONFIRMED (https://hearthsim.info/docs/gamestate-protocol/): "Each of those packets mutates the
  global game state in some way… the `TAG_CHANGE` packet mutates a card's tags (one at a time)."
  "Blocks may be nested, in cases where actions trigger other actions… This block system allows
  clients to keep track of what causes what, which is important in order to ensure animations play
  in the correct order." `META_DATA` is "a non-mutative packet whose only purpose is to inform
  clients on the actual targets of an action and/or its damage/heal… more to guide animations."
- CONFIRMED `CHANGE_ENTITY`: "Represents an entity's Card ID being changed. Carries a list of tags
  like ShowEntity. The previous tags are reset." (5.0.0.12574).
  https://raw.githubusercontent.com/HearthSim/hsreplay-xml/master/hsreplay.dtd
- Derived ATK/HEALTH as TAG_CHANGE: CONFIRMED for enchantments (Abusive Sergeant: `TAG_CHANGE
  Entity=[name=Silver Hand Recruit id=84 ...] tag=ATK value=3`; Crazed Alchemist `tag=HEALTH
  value=3` … `tag=ATK value=1`). PARTLY CONFIRMED for auras (search snippet: `TAG_CHANGE
  Entity=[name=Loot Hoarder id=9 ...] tag=ATK value=4`, cause not confirmed).
- CONFIRMED hidden until revealed: "When such a card is drawn, it is revealed to a player (but,
  usually, not to the other one!). This is where `SHOW_ENTITY` comes in." Opponent draw:
  `TAG_CHANGE Entity=[id=41 cardId= type=INVALID zone=DECK zonePos=0 player=2] tag=ZONE
  value=HAND`. HSReplay example: `<FullEntity id="5">` with no `cardID`, later
  `<ShowEntity entity="7" cardID="EX1_506">`.
- CONFIRMED per-player view (HearthSim): "Somewhere between the server's simulation and the
  player's client is a dispatcher which knows to hold back and/or change some packets for each
  player (and spectator…)… asymmetry between the server's game state (global game state) and a
  client's knowledge of it (local known state)."

SabberStone:
- CONFIRMED `SabberStoneCore/src/Kettle/PowerHistory.cs`: `public List<IPowerHistoryEntry> Full {
  get; } ... public List<IPowerHistoryEntry> Last { get; }`, `Add` appends to both. Entries:
  `PowerHistoryCreateGame`, `PowerHistoryBlockStart`, `PowerHistoryBlockEnd`,
  `PowerHistoryTagChange`, `PowerHistoryFullEntity`, `PowerHistoryShowEntity`,
  `PowerHistoryHideEntity`; factory `PowerHistoryBuilder`; comments mirror the protobuf.
- CONFIRMED `Game.PowerHistory` docstring: "This object facilitates building POWER blocks to send
  to the hearthstone client." `Game.Process(PlayerTask)` starts with `// clear last power history
  PowerHistory?.Last.Clear();`. So `Last` = delta for one action, `Full` = whole game. HearthSim:
  "Sabberstone ships a builtin Kettle server" (https://hearthsim.info/simulators/); server code
  not traced.
- CONFIRMED tag diffs from the entity indexer (`SabberStoneCore/src/Model/Entities/Entity.cs`):
  `if (_history && (int)t < 1000) if (value + (AuraEffects?[t] ?? 0) != this[t])
  Game.PowerHistory.Add(PowerHistoryBuilder.TagChange(Id, t, value));`. `FullEntity` computes
  derived values: `tags[GameTag.ATK] = c.AttackDamage; tags[GameTag.HEALTH] = c.Health;`. Deck cards
  get `PowerHistoryFullEntity` with `Name = ""`.
- NOT FOUND: per-player redaction in SabberStoneCore.
- Caveat: `GraveYard()`, `MainEnd()`, `MainCleanUp()` build `BlockStart(BlockType.DEATHS, ...)`
  under `if (History)` without `PowerHistory.Add(...)`; `PlayCardBlock` does add its PLAY block.

## Q4. Replays

- CONFIRMED: HSReplay is XML mirroring Power.log packets one to one ("Corresponds to TAG_CHANGE in
  Power.log", ..., "Block … (from BLOCK_START to corresponding BLOCK_END) … Previously known as
  ACTION_START / ACTION_END."). HearthSim: "the Hearthstone Game State replay, stored as an XML
  file. It's easier to parse than Power.log files".
- PARTLY CONFIRMED: state-change recording with inputs annotated. DTD has `Options`, `SendOption`
  ("Sends a chosen option"), `Choices`, `SendChoices`, `ChosenEntities`. Example: `<SendOption
  option="1" ... />` then `<Block type="7">` full of TagChanges.
- PARTLY CONFIRMED: Joust plays back by applying tag diffs: `ts/state/mutators/`
  (`TagChangeMutator.ts`, `ShowEntityMutator.ts`, `HideEntityMutator.ts`,
  `ReplaceEntityMutator.ts`, `AddEntityMutator.ts`). `TagChangeMutator.applyTo`: `const newEntity =
  oldEntity.setTag(this.tag, this.value);`.
  https://raw.githubusercontent.com/HearthSim/Joust/main/ts/state/mutators/TagChangeMutator.ts
  No explicit statement that it never simulates.
- NOT FOUND: explicit claim that replays survive patches. DTD records `build`. Card names and art
  still need a card database by `cardID`. Patch independence is inference.
- SabberStone: no replay format found.

## Q5. AI cost (SabberStone)

- CONFIRMED `SabberStoneCore/src/Config/GameConfig.cs`: `public bool Logging { get; set; } = true;`,
  `/// ... whether Game should store POWER history entries. ... public bool History { get; set; } =
  true;`. Builder `.History(bool)`, `.Logging(bool)`.
- CONFIRMED: flag off means no history object: `if (history) { _history = true; ... PowerHistory =
  new PowerHistory(); }`. Emit sites guarded (`if (History) PowerHistory.Add(...)`, `if (_history
  && ...)`).
- CONFIRMED: `public Game Clone(bool logging = false, bool resetRandomSeed = true, bool history =
  false)`. Copy constructor: `_gameConfig = game._gameConfig.Clone(); _gameConfig.Logging =
  logging; _gameConfig.History = history;`, "Logs are not cloned." Parent history never copied.
- CONFIRMED: BasicAI `OptionNode.cs`: `_game = game.Clone(); // create clone`. `Program.cs` configs
  `Logging = false, History = false`.
- CONFIRMED: rules record copied on clone: `Controller` copy constructor `PlayHistory = new
  List<PlayHistoryEntry>(controller.PlayHistory); ... CardsPlayedThisTurn = new
  List<Card>(controller.CardsPlayedThisTurn);`. Tag counters live in cloned entity `_data`. Rules
  history is state; power history is optional output.

## Gaps

Server trigger mechanism unknowable. Aura-caused `TAG_CHANGE ATK` only from a snippet. No
explicit statement that Joust never simulates or that replays survive patches. Kettle server
(consumers of `PowerHistory.Last`, redaction) not traced. Card consumers of `PlayHistory` not
verified.
