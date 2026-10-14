# Audit 1: the rules crate against the file-structure convention

## Summary

By the doc's Rust rule the crate has nine small modules (`rng`, `turn`, `zones`, `ids`, `observer`, and `binder`, `object`, `definition`, `modifier` under `cards`) and one large one, `game`, which holds most decisions. The eight files under `game/` add `impl Game` blocks, so they are one module and hide nothing from each other.
Most findings are decisions known in several places inside `game`, or across `game` and `cards`, plus `_` filters that let a new variant through the build.
Three latent bugs were confirmed in a throwaway copy of the crate under `/tmp`: stale pick answers, a blind spot in the card validators, and fatigue damage that no damage trigger hears. No current card reaches any of them.
18 findings (9 high confidence), 18 names fail, 13 doc feedback items.

## Modules

I used the doc's Rust definition: a `mod` counts as a module when its private items hold the data its decision is stored in. A child file that adds `impl` blocks to its parent's type is part of the parent. I read "private" as "not visible outside the `mod`", so `pub(crate)` and `pub(super)`-to-the-crate-root count as public (doc feedback 2).

```
rules            lib.rs: the pub use list
├── game         game.rs and all of game/: condition, lookup, player, resolve,
│                state_check, triggering, view, zone_move
├── cards        mod.rs, no code of its own
│   ├── binder   private: the definition map
│   ├── object   private: ObjectId's u64, ObjectBag's counters and map
│   ├── definition  private: DefId's &'static str
│   ├── modifier private: Modifiers' Vec
│   ├── loader   no private data, only private validator functions
│   └── filter   no private data
├── zones        private: the contents of Deck, Hand, Board, Graveyard
├── turn         private: TurnOrder's fields
├── rng          private: Rng's state
├── observer     private: Views' &Game
├── ids          private: PlayerId's usize
└── no private data, so not modules: action, choice, effect, event,
    history, outcome, trigger
```

Decisions each one holds:

- `lib.rs`: which items are public.
- `game`: setup (3 cards each, shuffle, one `HERO` definition for both players, the built-in card table), legality, mana refill and cap, mana affordability and payment, turn start and end, effect interpretation, selector resolution, history recording, history queries, event reporting, trigger matching, the effect stack and the resolution loop, the resolving effect's context, the draft search, what a kind filter matches, the state check and the death rule, derived attack, health and cost, aura reach, zone transfers and order stamps, fatigue, checkpoint placement, and visibility. `player` declares `Player` and `PlayerInteractionState` with `pub(super)` fields, and `view` declares view types with `pub` fields, so neither hides anything from `game`.
- `cards::binder`: how definitions are stored and found. What each card kind has (health, attack, board presence, deck presence, auras), in exhaustive matches. Kind tests as `matches!` filters.
- `cards::object`: how object ids are minted, and the order-stamp counter. The `Object` record has `pub` fields and is handed out by `get_mut`, so its layout is not hidden.
- `cards::definition`: the representation of `DefId`. The `CardDef` layout is public.
- `cards::modifier`: the container of a modifier list. `Modifier` and `ModifierEffect` are public data.
- `cards::loader`: the built-in card table (public through `load_all`), the def id constants, the placeholder id and the validation rules. Only the validators are private, and they are code, not data (doc feedback 1).
- `cards::filter`: no private data. Holds the side rule in `fulfill_side_filter`.
- `zones`: each zone's container and order (deck top first, hand oldest first, board left to right). `Zones` has `pub` fields.
- `turn`: seat rotation and the turn count.
- `rng`: the PRNG algorithm.
- `observer`: observers get views, never the game.
- `ids`: the representation of `PlayerId`. `new` and `idx` are crate-visible, so `game` knows a `PlayerId` is an index.
- Data files: `action` (the `Action` layout), `event`, `outcome`, `choice` (choice, pick and amount data), `effect` (the effect vocabulary, pending and resolving effect records), `trigger` (trigger records and patterns), `history` (history records and the query vocabulary). The meaning of each is decided elsewhere, mostly in `game`.

## Decisions likely to change

| Decision | Evidence |
|---|---|
| Card kinds (spell, minion, hero) | Hearthstone added weapons, hero cards and locations. MTG has artifacts, enchantments, lands, planeswalkers. Legends of Runeterra has landmarks and equipment. Commit 5273105 added `Hero` and touched binder, lookup, state_check, zone_move, view and resolve. |
| Effect vocabulary | Every set adds effects. The glossary already defines Restore, which has no `Effect` yet. ADR 0004 plans one match arm per interpreter. |
| Modifier kinds | Hearthstone keywords (Taunt, Divine Shield, Windfury, Stealth), set-stat effects, cost increases, Silence. Commits 1d3b955 and ae79835 each added one. |
| Trigger patterns | Start and end of turn, on draw, on summon, on heal, on attack, in every game named. Grew from none to three in ae79835. |
| History queries | Hearthstone counts cards drawn this turn, spells cast this game, Elementals played last turn. MTG has storm. |
| Filters and selectors | Adjacent minions, damaged minions, tribes, cost. Reshaped in 283f374. |
| Choice bounds and draft rules | `AtMost` and `AtLeast` are commented out at choice.rs:18-19. MTG "up to N targets", Hearthstone Discover. ADRs 0007, 0009 and 0010 superseded each other. |
| Resolution order | ADR 0013 was superseded by 0014. Hearthstone sorts by order of play, MTG lets a player order simultaneous triggers. Commit 995b2bb rewrote the loop across seven files. |
| Damage marking and the death rule | Hearthstone Armor, Divine Shield, Immune. MTG damage prevention. |
| Fatigue amount | Hearthstone fatigue grows 1, 2, 3. In MTG drawing from an empty library loses the game. |
| Mana: refill, cap, affordability, payment | Hearthstone Overload, The Coin, Wild Growth, Cho'gall (spells cost Health). Runeterra spell mana. |
| Opening setup | Hearthstone deals 3 or 4 cards plus The Coin, with a mulligan, and has class heroes. Runeterra has a mulligan. |
| Zones and their limits | Hearthstone hand limit 10 with overdraw, board limit 7, secrets, choosing a minion's position. MTG exile and the stack. |
| Ownership and control | Hearthstone Mind Control and Sylvanas, MTG Control Magic. ADR 0011 names "a change of control" as an input. |
| Which characters are in play and targetable | Hearthstone Dormant and Stealth, Runeterra Elusive, Hearthstone locations on the board. |
| Card source | `// TODO: loading cards from disk into the engine should be shells job` (loader.rs:89). A file format comes from outside. |
| Checkpoint placement | Moved in 283f374, 5273105, 8bcdccc, ae79835 and 995b2bb. It paces what a shell shows. |

Judged stable and left where they are: the PRNG (ADR 0001), the determinism bans, two players (`[Vec<DefId>; 2]`), seat rotation, id representations, events naming objects by id (ADR 0006), legality as `legal_actions` (ADR 0002), versioned card codes (ADR 0003).

## Findings

Every runtime check below was run in a throwaway copy of the crate under `/tmp`, with test cards added to the copy's table and a probe test file. Nothing in the repo was changed.

### 1. `game` holds many decisions that change independently

- Decision: every decision listed under `game` in Modules.
- Places: the data sits in `Game`'s fields, game.rs:32-43. Each child file reads and writes it directly. `history.entries` is written at triggering.rs:96 and read at resolve.rs:286. `effect_stack` is pushed at triggering.rs:34, zone_move.rs:64 and zone_move.rs:98, and popped at resolve.rs:233. `player.zones` is edited in game.rs, zone_move.rs, lookup.rs, condition.rs and view.rs. All of it is in module `game`. Informal for every change: nothing stops any file from reading any field.
- game.rs's own code holds setup (:56-108), legality (:110-201), mana refill and cap (:260-262) and turn start and end (:245-264). That is several decisions, not how its children combine.
- Rule: "A boundary that holds many decisions that change independently doesn't get this, since its code has to be sorted by decision before a change." Also "Judge a parent by its own code, which should hold one decision, often how its children combine." Also "a decision hides from the parent only in a type declared in a child."
- Fix: move the data of each decision likely to change into a type declared in a child of `game`, with private fields, so `Game` only combines them. Findings 2 to 16 name the candidates.
- Confidence: high. Commit 995b2bb changed one decision (how resolution pauses) and touched game.rs, resolve.rs, triggering.rs, zone_move.rs, player.rs, view.rs and lookup.rs.

### 2. History is recorded in one file, queried in another, and its data is public

- Decision: which events become history, and what each query counts.
- Places:
  - history.rs:10-15 and :43-46, module none (data file): `pub` fields, so the layout crosses every boundary.
  - triggering.rs:64-98, module `game`: the recording rule. The `_ => None` at :94 makes it informal for a new event that should be recorded.
  - resolve.rs:285-320, module `game`: the query rule. The player and turn filters are copied in both arms (:293-300 and :308-315). Formal for a new `HistoryQueryKind` (the copy failed at resolve.rs:292 only). Informal for a new `HistoryKind` (the copy built with only a dead-code warning).
  - triggering.rs:65 and resolve.rs:297, :312: both rely on `turn_count()` meaning the same thing at record time and at query time. Informal for a change to turn numbering.
  - `HistoryKind::CharacterEntered` is recorded (triggering.rs:84-93) and never read.
- Rule: "Make a decision its own module when its knowledge sits in more than one place, today or in planned work, and an interface of questions and commands can replace that knowledge while asking callers to know less than the data would."
- Fix: make `History` a module with private entries, one command `record(event, turn, object)` and one question `count(query, player, turn)`. To avoid depending on `Binder`, classify the entry at record time; ADR 0003 makes a definition immutable, so classifying early gives the same answer (doc feedback 12).
- Confidence: high.

### 3. What each card kind means is decided by several filters

- Decision: what each `CardDefKind` variant means for health, auras, death, graveyard and filters.
- Places:
  - binder.rs:34-100, module `cards::binder`: exhaustive matches. Formal.
  - binder.rs:50-60, module `cards::binder`: `is_spell`, `is_minion`, `is_hero` as `matches!` filters.
  - Callers that combine those filters to decide per kind, all informal for a new kind: lookup.rs:56-58 (which objects have health in play), lookup.rs:158 (only minions receive auras), zone_move.rs:105-110 (heroes stay in their zone on death), condition.rs:90-105 (kind filters), resolve.rs:301-306 and :316 (history counts), all in `game`; loader.rs:653-657 and :684-688 (`if let CardDefKind::Minion` to find auras), in `cards::loader`.
  - filter.rs:5-22, module `cards::filter`: `CharacterKindFilter` and `CardKindFilter` mirror `CardDefKind` by hand. Informal.
  - zone_move.rs:127 asks `has_deck_presence` to decide whether a card goes to the graveyard. That is a different question borrowing another one's answer.
- Rule: "When several filters together decide what each variant means, give that decision one exhaustive match." Also "Give each question its own function, even when two share an answer today."
- Fix: give `Binder` one question per use (`receives_auras`, `stays_on_death`, `goes_to_graveyard`, `matches_kind(filter)`), each an exhaustive match, and drop `is_*` from callers.
- Confidence: high. In the copy, adding `CardDefKind::Location` failed the build at the six matches in binder.rs only.

### 4. What each modifier does is decided by four `_` filters

- Decision: what each `ModifierEffect` variant changes.
- Places: lookup.rs:38-41 (cost), :76-79 (health), :102-105 (attack), :130-136 (granted triggers), all in `game`. Each is a `filter_map` with `_ => None`. Informal for a new variant: it is silently ignored everywhere.
- Rule: "When several filters together decide what each variant means, give that decision one exhaustive match."
- Fix: one exhaustive match on `ModifierEffect`, for example a function in `cards::modifier` that folds a modifier list into a summary of attack, health, cost and triggers, which the lookups read.
- Confidence: high. In the copy, adding `ModifierEffect::SetAttack` built with no error.

### 5. The trigger matcher lets a new pattern through

- Decision: which events each `TriggerPattern` matches.
- Places: triggering.rs:46-61, module `game`. The `(_, _) => false` arm at :60 covers every pattern, not only every event. Informal for a new pattern: it never fires and the build passes.
- Rule: "An exhaustive `match` with no `_` arm, so adding a variant fails the build at every place that has to handle it. This is for matches that decide something for every variant."
- Fix: match exhaustively on the pattern, and inside each arm test the event with `matches!` or `let else`.
- Confidence: high. In the copy, adding `TriggerPattern::TurnStarted` and `Event::Restored` built with no error.

### 6. Damage is marked in two places, and fatigue skips damage triggers

- Decision: how damage is marked on a character and reported.
- Places: zone_move.rs:262-277 (`damage`: marks, emits `Damaged`) and zone_move.rs:36-47 (fatigue: marks the hero directly, emits `FatigueDamaged`), both in `game`. The matcher at triggering.rs:57 only hears `Damaged`. Informal for any change to damage, such as Armor or Immune: the build passes if only one place changes.
- Rule: "Ideally one place knows it, and every other place asks it a stable question or gives it a stable command, by name." The glossary says "Fatigue: The damage a hero takes for a draw from an empty deck", and ADR 0008 says "a damage effect takes one path for every character".
- Fix: route both through one command that marks damage. Then decide, in the trigger matcher, whether `FatigueDamaged` matches `Damaged`. `damage` needs a source object and fatigue has none, which is an interface decision for you.
- Confidence: high.
- Latent bug. In the copy, a minion with a `Damaged(Character(friendly heroes))` trigger fired on Spark's hit and not on fatigue. No current card has a damage trigger that can match a hero: Rally Banner grants `Damaged(Itself)` to minions only.

### 7. The draft rules are copied, and the chosen list crosses four files

- Decision: choices fill in declaration order, the first unfilled one is open, and a choice is full when its bound is satisfied (ADR 0010).
- Places, all in `game`:
  - game.rs:157-198 and condition.rs:107-137: the same loop (count per choice, skip if satisfied, scan candidates, stop at the first unfilled). A copy. Informal.
  - condition.rs:18-25: counts and checks again.
  - The `Vec<(ObjectId, ChoiceId)>` layout is read at player.rs:22, game.rs:164-166 and :178-179, condition.rs:16-22 and :110-117, resolve.rs:86-94 and :125-129, view.rs:171. Formal for a layout change, informal for its meaning.
  - All three places treat "satisfied" as "full". `AtMost` is planned (choice.rs:18-19, `upper_bound` at choice.rs:29 is unused). With `AtMost`, zero picks is satisfied, so game.rs:168 and condition.rs:23, :118 would never offer a `Choose` for it. The build passes. Informal.
- Rule: "One place knows each decision." Also "Make a decision its own module when its knowledge sits in more than one place, today or in planned work."
- Fix: a `Draft` type declared in a child of `game`, with the chosen list private, answering `open_choice`, `candidates`, `can_finish` and `is_complete`, and handing out the targets per choice at `Play`.
- Confidence: high.

### 8. The effect stack's order is known at every push site

- Decision: newest first, with reactions pushed after what caused them (ADRs 0014, 0015).
- Places, all in `game`: pushes at triggering.rs:34, zone_move.rs:63-68 and zone_move.rs:96-103; the pop at resolve.rs:233. The call order in `play` (push before `Played`, zone_move.rs:63-74) and in `kill` (push death triggers before `Died`, zone_move.rs:96-111) only gives the ADR's result because the stack is last in, first out. The order of one event's matches comes from two sorts, game.rs:225 and lookup.rs:140. Informal for a change of discipline: swapping `pop` for a queue still builds and inverts both call orders.
- Rule: "A copied rule, a relied-on meaning and a required call order are others, and none of them needs a call." Also "Prefer taking knowledge away from callers to making it formal."
- Fix: an effect-stack type declared in a child of `game`, with a private `Vec` and commands named by moment (`push_play_effects`, `push_death_triggers`, `push_matches`), plus `next`.
- Confidence: medium. The spread is clear. How likely the order is to change again rests on one supersession and on MTG letting players order triggers.

### 9. The resolving effect's context is spread over four records, and pick answers never expire

- Decision: what a resolving effect carries until it finishes: its place, its chosen targets and its pick answers (ADR 0015: "its position and its context").
- Places:
  - effect.rs:80-85, data file: `ResolvingEffect` with `pub` `idx`.
  - player.rs:15-19, `game`: `PendingPick` holds `source` and `pick_id`.
  - object.rs:20, `cards::object`: `choice_targets`, written at resolve.rs:84-95, read at resolve.rs:360-368.
  - object.rs:21, `cards::object`: `pick_targets`, written at zone_move.rs:202-207, read at resolve.rs:171-177 with `find`. Never cleared.
  - Informal: each writer and reader relies on the other's layout and on when the data is valid.
- Rule: "Data a module takes in or hands out is part of its interface: its layout, what each field means and the rules for writing it. A decision whose data crosses the boundary isn't hidden."
- Fix: keep chosen targets and pick answers inside the resolving effect, so they end with it.
- Confidence: high.
- Latent bug. In the copy, a minion with a `Damaged(Itself)` trigger running `Reveal` then `ResolvePick` was hit twice by Ping. The second `ResolvePick` found the first answer: one card was twice in the hand, one twice in the deck, and the two cards revealed second were in no zone. No current card reaches it: Forage and Sweep are spells played once, Hoarder dies once.

### 10. Two validators walk a card's effect sequences, with the same blind spot

- Decision: where effect sequences live on a card definition.
- Places: loader.rs:645-672 (`validate_choice_on_trigger`) and loader.rs:678-698 (`validate_picks`), module `cards::loader`. Both walk triggers and minion auras by hand. Both miss a `GrantTrigger` in `modifier_effects` and one nested in an `Effect::Buff` or `Effect::AddFriendlyAura`. The runtime reads all of them at lookup.rs:119-142, in `game`. Informal: a new place for effects passes the build and every validator.
- Rule: "One place knows each decision." Also "When no plausible change would make two answers differ, they are one question asked twice, and the two places are a copy."
- Fix: one walker in `cards::definition` that yields every effect sequence on a card, recursing into granted triggers wherever a `ModifierEffect` appears, used by both validators.
- Confidence: high.
- Latent bug. In the copy, a spell that buffs friendly minions with a granted death trigger naming `Chosen(0)` passed `load_and_validate`. When a buffed minion died, the game panicked with "Choice not fulfilled" at resolve.rs:366. No current card grants a trigger through an effect.

### 11. The loader checks what resolution will do, so it knows resolution's rules

- Decision: a chosen target needs a draft, and `ResolvePick` needs an earlier `Reveal` in the same sequence.
- Places: loader.rs:626-643 (`chosen_id`, private) and loader.rs:700-723, in `cards::loader`. resolve.rs:360-368 (panics without a draft) and resolve.rs:171-186 (does nothing without a pick), in `game`. Formal for a new `Effect` or `CharacterSelector` (exhaustive matches). Informal for a change of rule, such as letting triggers choose targets as MTG does: resolution changes, the validator still rejects, and only a test finds it.
- Rule: "A step that checks what a later step will do has to know that step's rule... When an interface needs separate steps, keep them as entry points, and move each rule they share into one module that both ask."
- Fix: put the questions on `Effect` in `effect` (which choice it reads, which pick it opens or resolves) and have both the validator and resolution ask them.
- Confidence: medium.

### 12. "Characters in play" is answered in five places

- Decision: which objects are characters in play.
- Places, all in `game`, all informal for a new kind of permanent or for Dormant and Stealth: game.rs:216-227 (heroes and boards, sorted by order), condition.rs:47-67 (heroes, then boards, unsorted), lookup.rs:53-65 (health only for the hero in its slot or a minion on a board), state_check.rs:9-22 and :29-36 (boards, then heroes separately), lookup.rs:158-172 (aura sources are boards). Also tests/spec/support.rs:303-318 lists the zones.
- Rule: "Give each question its own function, even when two share an answer today. The second can be built on the first."
- Fix: one question for membership; the trigger order and the targeting order become two questions built on it.
- Confidence: medium.

### 13. An object's owner is answered two ways

- Decision: which player an object belongs to.
- Places: `Object.player_id` (object.rs:16), read at triggering.rs:18-19, :68, :110, zone_move.rs:106-109, condition.rs:28, :42-43, lookup.rs:27-31, :54-63, :88-92, resolve.rs:257. The board's player, at condition.rs:56-63 and lookup.rs:159-165. Modules `game` and `cards::object`. Informal: both agree only while every object sits in its owner's zones.
- Rule: "When no plausible change would make two answers differ, they are one question asked twice, and the two places are a copy." Control change would make them differ. lookup.rs:28, :60 and :89 would then search the wrong player's zones, and `health` would return `None` for a board minion, which panics at state_check.rs:15.
- Fix: one question, `owner(object)`, and filter functions that take only the object id.
- Confidence: medium.

### 14. Zone membership and transfers have no single owner

- Decision: which zone an object is in, and what a move does (remove, add, restamp, report).
- Places, in `game` unless noted:
  - `Zones` has `pub` fields (zones.rs:107-114, `zones`), so every move edits containers directly: zone_move.rs:27-29, :57-61, :90, :121-125, :139-142, :154-162, :222-223, :238-239.
  - Cards reach the hand at zone_move.rs:29 and :223. A hand limit (Hearthstone's 10) would need both. Informal.
  - Revealed cards sit in `PendingPick.options` (player.rs:18), in no zone.
  - The order stamp is renewed at five places (zone_move.rs:35, :138, :224, :240, :254). `draw` renews after reporting, the others before. object.rs:22 says the stamp changes on every move; ADR 0012 says it is set at each board entry. Only board order is ever read.
- Rule: "A copied rule, a relied-on meaning and a required call order are others." Also "Prefer taking knowledge away from callers to making it formal."
- Fix: one move command that owns removal, addition, stamping and the report, with the zone set private to `zones`.
- Confidence: medium.

### 15. Checkpoint placement is spread over ten call sites

- Decision: when observers get a checkpoint (ADR 0006: "at set moments").
- Places: resolve.rs:106, :111, :118, :130, :135, :259; zone_move.rs:76, :80; state_check.rs:51, :78. All in `game`. Informal: only the event tests find a change.
- Rule: "Steps still run, inside one coordinator that calls the modules in order. It knows the order and nothing about any format." Helpers like `play` and `remove_dead` hold part of the reporting procedure.
- Fix: move the checkpoint calls into `apply` and the resolution loop. Where one must fall inside a command, as between `Played` and the summon, name that moment in one place.
- Confidence: medium.

### 16. Affordability and payment are two places

- Decision: what a card costs a player and how it is paid.
- Places: game.rs:124-131 (affordable when cost is at most mana), resolve.rs:98-101 (subtract mana), game.rs:260-262 (refill and cap), all in `game`. Informal: paying with health or with spell mana changes the first two together, and the build passes if only one changes.
- Rule: "A question is one thing callers ask about a decision... Give each question its own function."
- Fix: a `can_afford` question and a `pay` command next to the mana data.
- Confidence: medium.

### 17. The placeholder rule is split across two files that import each other

- Decision: a `CardDef` built from `Default` has a placeholder id, and validation rejects it.
- Places: definition.rs:6 and :35-48 (`cards::definition`), loader.rs:40 and :576-581 (`cards::loader`). `definition` imports `loader`, which imports `definition`. Informal: the two halves only make sense together.
- Rule: "One place knows each decision."
- Fix: keep the placeholder and its check in `definition`, with the validator asking `def.is_placeholder()`, or drop `Default` when cards come from files.
- Confidence: low.

### 18. `Game` knows where card definitions come from

- Decision: the card source.
- Places: game.rs:59-61 (`setup` builds the table and unwraps validation), game.rs:25 and :70 (`HERO`), in `game`; definition.rs:15-21 (`DefId` wraps a `&'static str` with a `const` constructor), in `cards::definition`; loader.rs:42-87 and :94-555 (constants and table), in `cards::loader`. Formal for the build when the table moves, but `Game::new`'s signature changes with it.
- Rule: "Each module hides its decision behind an interface that holds still when the decision moves." The doc names "a file format" among things outside your control.
- Fix: one option is `Game::new` taking a binder built by the shell, with `loader` kept as the built-in source for tests. That changes the public interface, so it is your call.
- Confidence: medium.

## Names

Each fails against its siblings unless noted.

Top level (siblings: action, cards, choice, effect, event, game, history, ids, observer, outcome, rng, trigger, turn, zones):

1. `game`: most functions in the crate fit it. `fulfill_character_filter` (game/condition.rs:69) is in `game`, `fulfill_side_filter` (cards/filter.rs:45) is in `cards`.
2. `cards`: most of the crate is card text, and `effect`, `trigger` and `choice` hold card text too. `EffectSequence::as_slice` (effect.rs:54) reads a card's play effects, yet sits outside `cards`.
3. `choice`: holds pick types, which the glossary separates from choosing, and amount types. `AmountBound::is_satisfied` (choice.rs:23) and `EffectAmount::from` (choice.rs:9) are not about choices.
4. `ids`: says "identity newtypes" but holds only `PlayerId`. `ObjectBag::next_id` (cards/object.rs:34) mints `ObjectId` elsewhere.
5. `turn`: holds seat rotation. A reader looking for `Game::start_turn` (game.rs:254) opens it and finds nothing. The glossary's "seat" would pass.
6. `history`: passes the sibling test but fails the glossary check: the glossary has no History, though ADRs 0005 and 0011 use it.

Under `game` (siblings: condition, lookup, player, resolve, state_check, triggering, view, zone_move):

7. `condition`: not a glossary term. `can_finish_choices` (condition.rs:107) is a draft search, and the condition `fulfill_trigger_pattern` (triggering.rs:39) lives elsewhere.
8. `lookup`: `Game::mana` (game.rs:229) is a lookup outside it, and `death_rattles` (lookup.rs:112) is not a stat.
9. `resolve`: the glossary means carrying out card text. `history_query` (resolve.rs:285) and the `EndTurn` arm of `apply_action` are not, while `reveal` (zone_move.rs:145) is and sits elsewhere.
10. `zone_move`: `damage` (zone_move.rs:262) moves nothing. The name also clashes with the `ZoneMove` enum (effect.rs:11), which has two of its moves.
11. `triggering`: `history_from_event` (triggering.rs:64) records history.

Under `cards` (siblings: binder, definition, filter, loader, modifier, object):

12. `definition`: holds selectors. `PlayerSelectorFilter::all` (definition.rs:96) is a filter, next to `CharacterFilter::all` in `filter`.
13. `filter`: the same pair from the other side. A reader adding a player filter opens `filter` and finds the type is not there.
14. `loader`: loads nothing; it is the card table plus validation. `validate_pick_order` (loader.rs:700) checks card text.
15. `modifier`: fails the glossary check. The glossary has Buff and Aura, and `Modifiers::add` serves both (resolve.rs:199 and :209). The concept has no glossary word.

Tests under `tests/spec`:

16. `cards`: most tests are about a card. `triggers::bombers_deathrattle_deals_1_damage_to_every_minion` and `cards::blast_deals_two_damage_to_every_character` could swap files.
17. `lookups`: overlaps `derived`. `lookups::a_dead_minion_has_no_cost_attack_or_health` and `derived::cards_in_hand_have_a_cost_and_no_stats` test the same lookups by zone.
18. `resolution`: overlaps `triggers`. `resolution::a_deathrattle_resolves_after_the_rest_of_the_card_that_killed_its_minion` and `triggers::a_deathrattles_consequences_resolve_before_triggers_already_waiting` test the same order.

Pass: action, effect, event, observer, outcome, rng, zones, trigger, binder, object, player, state_check, view, and the other test files and the bench.

## Already following the rules

- `rng`, `turn` and `zones` hide their data in private fields, and each container keeps its own order.
- `ObjectId`, `DefId` and `PlayerId` are newtypes with private fields. Only `ObjectBag` mints object ids, and only `cards` mints def ids.
- clippy.toml bans `Observer::event` everywhere but `emit`, and bans `HashMap`, `HashSet` and the clocks. Both are "only this place may call that" made formal.
- `emit` is one command that records history, reports and matches, so no caller can forget one.
- `apply` checks `legal_actions` and then applies, so callers can't skip the precondition.
- The effect interpreters match exhaustively with no `_`: resolve.rs:147, loader.rs:627 and :636, loader.rs:703. So do the selectors (resolve.rs:334, :350) and `legal_actions` on the interaction state (game.rs:116).
- `Game::view` is the one place that decides visibility, and events carry ids only.
- `Views` hands out views, never the game.
- tests/spec/model.rs copies card data, but ADR 0003 makes each versioned card immutable, so the copy never needs an edit.

## Doc feedback

1. "A module in the general sense is a `mod` whose private items include the data its decision is stored in." A rule is a decision too, but `cards::loader` hides its validation rules as private functions and no data. I couldn't tell whether it counts as a module.
2. "Private" is unclear against Rust's visibility levels. `pub(super)` in a top-level file is crate-wide (ids.rs:8, :12), and `pub` fields on a `pub(crate)` struct are crate-wide (`Zones`, `Object`, `History`). I read "private" as not visible outside the `mod`.
3. "Names exclude" speaks of "A module's name", but the files under `game/` are not modules by the Rust section. The opening says the doc covers "naming a module, a file or a crate". I judged file names anyway.
4. Rust privacy forces every function that reads `Game`'s fields into `game`'s subtree. The doc says such files are part of the parent, but not whether splitting one module into `impl` files helps, is neutral, or hides the problem.
5. "A boundary that holds many decisions that change independently doesn't get this." The doc doesn't say whether to report one finding (split the module) or one per spread decision. I did both, so findings 2 to 16 overlap finding 1.
6. "This is for matches that decide something for every variant." The trigger matcher matches a tuple: it decides per pattern and filters per event. The doc doesn't say which enum must be exhaustive. I chose the pattern.
7. "Put it in a type, or in a lint where a type can't hold it. Where neither can, pin it with a test." Load-time validation (the loader's validators) is a fourth tool. The doc doesn't say how it counts, and finding 10 shows it can have blind spots no build sees.
8. "When it is the procedure itself, it lives in the coordinator. An order that is a rule of the domain is a decision, and gets a module like any other." Checkpoint placement and "push death triggers before reporting the death" fit either reading. The doc gives no test to tell procedure from domain rule.
9. "Take names from the glossary." The doc doesn't say whether technical names (rng, observer, loader, support) are exempt, or whether a domain concept missing from the glossary (history, modifier) makes a name fail. I failed those two and passed technical names.
10. "What the roadmap plans." The doc doesn't say where the roadmap is. I used a TODO, commented-out variants, ADRs and git history.
11. The doc doesn't say whether files under `tests/` follow the same rules, or how to treat an intended independent oracle like tests/spec/model.rs, which copies card stats and the Captain and Giant rules. ADR 0003 settled it here, not the doc.
12. "Make a decision its own module when... an interface of questions and commands can replace that knowledge." A `History` module answering "spells played" needs card kinds from `Binder`. Read literally, the doc leads to `History` depending on `Binder`, which I think is worse than classifying entries when they are recorded. The doc doesn't say how a decision module gets data another module owns.
13. "Judge a parent by its own code, which should hold one decision, often how its children combine." `cards/mod.rs` has no code. The doc doesn't say whether a parent with no code passes or fails.
