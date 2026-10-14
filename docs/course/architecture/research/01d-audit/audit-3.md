# Audit 3: the rules crate against the file-structure convention

## Summary

Nearly all behavior sits in one Rust module, `game`: nine files, 1,710 of the crate's 3,824 source lines, all reading `Game`'s private fields. Most decisions likely to change are spread across those files with no boundary between them.
20 findings, 10 of them high confidence. Two are live bugs. A death pass lets a minion that dies later in the pass hear an earlier death, against ADR 0012. ObjectIds also let an opponent name every hidden card in hand.
Four latent bugs were confirmed with throwaway programs in a temp copy of the crate, `/tmp/audit3`. The repo was not changed.
13 of the 28 module and file names fail. The roadmap names 4 findings outright and 8 more in part.

## Modules

I followed the doc's Rust section: a module is a `mod` whose private items hold its decision's data. A child file that adds `impl Game`, or declares types whose fields its parent can read, is part of the parent. A file whose types expose every field hides nothing, so I counted it as part of the crate root. `cards/mod.rs` declares no data, so `cards` is a directory and its children sit directly under the root.

- `rules`, the crate root. Its boundary is the `pub use` list in `lib.rs`. Its own code is `lib.rs` plus the files that hide nothing: `action`, `choice`, `effect`, `event`, `history`, `outcome`, `trigger`, `cards/filter`, and the open types in `cards/definition` (`CardDef`, selectors), `cards/object` (`Object`), `cards/modifier` (`Modifier`, `ModifierEffect`) and `zones` (`Zones`). Decisions: the public interface, and the layouts of actions, events, card text, choices, pending and resolving effects, history entries and outcomes.
  - `ids`: `PlayerId`'s integer. Hidden from outside the crate only, since `new` and `idx` are `pub(super)` from a root child, which is crate-wide.
  - `rng`: the PRNG algorithm.
  - `turn`: seat order and the turn counter.
  - `zones`: each zone's container and its order. Deck top first, hand oldest first, board left to right.
  - `observer`: what an observer can reach at a checkpoint, which is one view at a time.
  - `cards::definition`: the representation of `DefId`.
  - `cards::object`: id allocation and the order counter, both in `ObjectBag`.
  - `cards::modifier`: the container behind `Modifiers`.
  - `cards::binder`: how the table is stored, and per-kind answers through exhaustive matches.
  - `cards::loader`, with its macro child `static_card_definition`: the built-in card set, the code constants and the validation rules.
  - `game`: `game.rs` plus `condition`, `lookup`, `player`, `resolve`, `state_check`, `triggering`, `view` and `zone_move`. `player.rs` and `view.rs` declare types whose fields `game` reads, and the rest add `impl Game`. Decisions: setup, legality, turn start, how each action changes the game, effect interpretation, selector resolution, randomness use, the resolution loop, effect stack order, history recording and queries, choice candidates and look-ahead, filter meaning, derived stats and aura reach, trigger derivation and matching, zone moves and order stamps, damage, the state check and outcome, and visibility.

## Decisions likely to change

| Decision | Evidence | Roadmap |
| --- | --- | --- |
| Card set content | `loader.rs` changed in each of the last five feature commits. Every session adds cards. | yes |
| Card text vocabulary: verbs, patterns, selectors, modifier effects | Restore, `If`, `Repeat`, `ForEach`, an enter pattern, signed modifiers and replacements are planned. | yes |
| Where definitions come from, and how they are validated | Session H plans data files, validation and hot reload. `loader.rs:89` has a TODO saying loading is the shell's job. | yes |
| Card kinds and what each does in play | Hearthstone has weapons, locations and hero cards. MTG has artifacts, enchantments and planeswalkers. The roadmap plans hero replacement and tokens. | in part |
| Choice rules: bounds, fill order, "different", playability with no target | `AtMost`, `Back`, XMage-style tags and `REQ_TARGET_IF_AVAILABLE` are open threads. | yes |
| Where resolution values live | Sessions 06b and 06c decide it. | yes |
| Shape of the resolving effect | 06b replaces the single cursor with a frame stack. | yes |
| Effect stack order and push order | ADR 0014 already superseded ADR 0013. Enter-effect push order is open. Session K adds a Yu-Gi-Oh chain and session 08 adds responses. MTG orders triggers APNAP. | yes |
| Order of play as the tie-break | ADR 0014 cites Legends of Runeterra's board order. Hearthstone uses order of play. | no |
| Trigger patterns and matching | An enter pattern, hero damage that includes fatigue, and "immediately" reactions are planned. | yes |
| How damage is marked and reported | "One damage function", then replacements and prevention. | yes |
| What history stores and how it is queried | Untested History branches and a query cache are open threads. Many Hearthstone cards read history. | yes |
| What entering a zone resets | The zone-reset table is due when bounce arrives. A stack zone is open. | yes |
| Which player an object belongs to | Reveal over several players is session 06d. Control change is named as a future verb. Hearthstone has Mind Control. | in part |
| Who may act | Session 08 responses, 06d simultaneous picks. | yes |
| What a player may see | "Knowledge as state", and the ID allocation fix. | yes |
| Death and outcome rules | Mortally wounded minions, dead heroes still in play, N-player elimination, no cap on trigger loops. | yes |
| The play sequence | Enter effects against Hearthstone's After Summon phase. A resolving card is in no zone. | yes |
| Paying a cost | Hearthstone's Cho'gall and Bloodbloom make a spell cost Health instead of Mana. MTG has alternative and additional costs. | no |
| Aura reach | Hearthstone has adjacency auras, such as Dire Wolf Alpha, and auras on cards in hand, such as Sorcerer's Apprentice. | no |

I judged these stable and left them alone: the RNG algorithm, which ADR 0001 and a reference test pin, `PlayerId` as an index, the zone containers, the mana cap and refill, and the observer interface from ADR 0006.

## Findings

### 1. The `game` module holds most of the crate's decisions

- Decision: all the `game` decisions listed under Modules.
- Places: `game.rs` and the eight files under `game/`, all in module `game`. Each adds `impl Game` or declares types whose fields `game` reads, at `player.rs:27-34` and `view.rs:8-72`, so no file hides anything from another. For example, `effect_stack` is pushed at `zone_move.rs:64`, `zone_move.rs:98` and `triggering.rs:34`, and popped at `resolve.rs:233`. `history.entries` is written at `triggering.rs:96` and read at `resolve.rs:286`. The knowledge is informal for every change: the build lets any file read or write any field.
- Evidence: each of the last five feature commits edited 3 to 7 of the 9 files. Commits 5273105, ae79835 and 995b2bb each touched 7.
- Rule: "A boundary that holds many decisions that change independently doesn't. Its code has to be sorted by decision before a change." The Rust section adds: "child files that add `impl` blocks to their parent's type are part of the parent's module".
- Fix: give each decision likely to change a child type with private fields. Findings 2 to 16 name them. `game` keeps `Game` and the coordinator that calls them in order.
- Confidence: high. Not named on the roadmap.

### 2. A death pass reports each death before it removes the next dead minion

- Decision: ADR 0012's order for a death pass. Read each death trigger, remove every dead object, then report each death, so that a removed object hears no other death.
- Places, all in `game`:
  - `state_check.rs:24-43` finds and sorts the pass, then calls `kill`.
  - `zone_move.rs:94-113`. `kill` pushes the deathrattles, then calls `destroy` and `emit` for each object in turn, inside one loop at lines 105-112. This is where the order is decided, and it is wrong.
  - `triggering.rs:17-20`. `emit` matches against whoever is on the board at that moment. It assumes the caller removed every dead object first, which is a relied-on meaning and informal for every change.
- Live bug. If a Recruit was played before a Mourner and both die to one Blast, the Mourner draws for the Recruit's death. A throwaway program run against an unmodified copy, through the public interface, showed `Died(Recruit)`, `Died(Mourner)`, then a `Drew` for P0. With the Mourner played first there was no draw. `a_mourner_dying_in_the_same_pass_draws_nothing` at `tests/spec/triggers.rs:97` tests only that second order.
- Rule: "A caller that must call two functions in order still knows the decision, even when a type enforces it. One command that does both leaves the caller nothing to get wrong." Also: "An order that is a rule of the domain is a decision, and gets a module like any other."
- Fix: make the whole pass one command next to the state check. It removes every dead object, then reports each death.
- Confidence: high. Not named on the roadmap. The roadmap says ADR 0012's rule "covers minions only", which assumes minions behave correctly.

### 3. Object ownership is stored twice

- Decision: which player an object belongs to. This decides which zones hold it and which side it is on.
- Places, in `game` unless noted:
  - The `Object.player_id` field at `cards/object.rs:16`, in `cards::object` but public.
  - Read to pick the zone to look in: `lookup.rs:27-31` (`mana_cost`), `lookup.rs:54-63` (`health`) and `lookup.rs:88-92` (`attack`). `kill` at `zone_move.rs:106-109` removes the object from the board of `player_id` at lines 121-125, and expects to find it there.
  - Read as the side owner: `triggering.rs:18-19`, `triggering.rs:110`, `resolve.rs:257`, `condition.rs:27-28` and `condition.rs:41-43`.
  - Zone's player read as owner instead: `condition.rs:54-63` uses the zone's `player.id`, and `lookup.rs:159-169` chooses friendly or hostile auras by the board's player.
  - Writers that place an object by a player argument and leave `player_id` alone: `zone_move.rs:9-20`, which `resolve.rs:180-185` calls with the source's owner, and `reveal` at `zone_move.rs:153-163`, which takes from the target player's deck.
  - All of this is informal for one change: "an object enters another player's zone".
- Latent bug. In a temp copy, Forage was changed to reveal from the enemy's deck. The picked card landed in the caster's hand with the enemy's `player_id`, and the caster's next `legal_actions` panicked at `game.rs:129` with "objects in hand always have mana cost". No current card reaches it.
- Rule: "Ideally one place knows it, and every other place asks it a stable question or gives it a stable command, by name."
- Fix: keep one source of truth. Either a zone move updates `player_id` in the same command, or ownership comes from the zone and the field goes.
- Confidence: high. Roadmap: in part. It names reveal over several players moving cards "into the source owner's zones" (06d) and control change, but not the split.

### 4. Resolution values are stored on the source object

- Decision: where the values a resolution produces live, which are chosen targets and picks, and for how long they last.
- Places: the fields at `cards/object.rs:20-21` live in `cards::object` but are public. Everything else is in `game`. On Play, `resolve.rs:84-95` writes `choice_targets`, and `resolve.rs:360-368` reads them for `Chosen`. `pick` appends at `zone_move.rs:202-207`, and `ResolvePick` reads the first match at `resolve.rs:171-177`. One step hands its output to a later one through the object. That is informal for "one source resolves twice" and for "a trigger reads its card's choice".
- Latent bug. In a temp copy, the Mourner's trigger became "whenever a friendly minion is damaged, reveal 2 and pick 1", and one Blast fired it twice from the same source. The second `ResolvePick` moved the first pick's cards again. One ObjectId ended up twice in the hand and another twice in the deck. No current card reaches it.
- Rule: "Don't split by processing step. A step hands its output forward as data, so every later step knows that data's layout, and when it was computed."
- Fix: keep the values in the resolving effect that made them, which ADR 0015 calls "its context", and read and write them only through it.
- Confidence: high. Roadmap: yes. It names both halves: `pick_targets` is never cleared, and targets stored on the source would collide.

### 5. Each validator walks a card's effect lists, and both miss two of them

- Decision: which effect lists a card definition holds.
- Places:
  - `validate_choice_on_trigger` at `loader.rs:645-672` walks triggers and minion auras.
  - `validate_picks` at `loader.rs:678-698` walks play effects, triggers and minion auras.
  - `validate_choice_not_found` at `loader.rs:614-624` walks play effects only.
  - `triggers` at `lookup.rs:119-142`, in `game`, also reads triggers granted through `CardDef.modifier_effects` (`definition.rs:29`) and through `Effect::Buff` and `Effect::AddFriendlyAura` (`resolve.rs:195-214`).
  - No validator walks those last two places. Each traversal is a copy, informal for "a card gets a new place to hold effects".
- Latent bug. In a temp copy, the Squire got a self-granted deathrattle in `modifier_effects` that names `Chosen(0)`. `load_and_validate` returned `Ok`, and when a Blast killed the Squire, resolution panicked at `resolve.rs:366` with "Choice not fulfilled". No current card reaches it.
- Rule: "Ideally one place knows it, and every other place asks it a stable question".
- Fix: one function in `cards::definition` yields every effect list of a definition, tagged as play or trigger, including nested `GrantTrigger`s. Every validator asks it.
- Confidence: high. Roadmap: in part. It flags `GrantTrigger` in `modifier_effects` for session H's validation, but not the gap.

### 6. The draft's next-choice rule is written twice

- Decision: `Choose` fills the first declared choice that is not yet filled (ADR 0010), and a target is offered only if the rest can still be filled.
- Places, all in `game`:
  - `legal_actions` at `game.rs:163-189` and `can_finish_choices` at `condition.rs:113-135` hold the same loop.
  - Counting targets per choice is copied at `game.rs:164-167`, `condition.rs:18-22` and `condition.rs:114-117`.
  - The draft's raw `Vec<(ObjectId, ChoiceId)>` from `player.rs:20-23` is also written at `resolve.rs:113-131` and read at `resolve.rs:69-95` and `view.rs:165-179`.
  - All informal for "change the fill order" and for "add a bound".
- Rule: "When an interface needs separate steps, keep them as entry points, and move each rule they share into one module that both ask."
- Fix: a `Draft` type in its own child module. It keeps the chosen list private and answers next choice, completeness and targets per choice. `legal_actions`, `apply` and `view` ask it.
- Confidence: high. Not named on the roadmap. The roadmap plans changes that hit both copies: `AtMost`, `Back`, and tags.

### 7. One function answers two questions about a choice

- Decision: when a choice can take another target, and when it is complete enough to play.
- Places, in `game` unless noted:
  - `check_choice_count_fulfilled` at `condition.rs:35-45` is built on `AmountBound::is_satisfied` at `choice.rs:23-27`, in the crate root.
  - It is asked as "complete" at `game.rs:168-174` and `game.rs:190-194`.
  - It is asked as "full, list no more candidates" at `condition.rs:23-25` and `condition.rs:118-122`.
  - `AmountBound::upper_bound` at `choice.rs:29-33` answers the second question, and nothing calls it.
  - Informal for "add `AtMost` or `AtLeast`".
- Latent bug behind the commented-out variant. In a temp copy, with `AtMost` restored and Twin Shot set to "up to 2", a draft with two minions on the board offered only `Play` and `Cancel`, never `Choose`.
- Rule: "Give each question its own function, even when two share an answer today."
- Fix: two functions, "can take another" built on `upper_bound` and "complete" built on `is_satisfied`. Each caller asks the one it means.
- Confidence: high. Roadmap: yes, "His first AtMost counted as filled before any choice, so its candidates were never listed."

### 8. Trigger matching ends in a catch-all arm

- Decision: which events each trigger pattern matches.
- Places: `fulfill_trigger_pattern` at `triggering.rs:46-61`, in `game`, matches on the pair of pattern and event and ends in a `(_, _)` arm that returns false. This is informal for "add a pattern". In a temp copy, adding `TriggerPattern::Entered` built clean, and the new pattern would never match.
- Rule: "An exhaustive `match` with no `_` arm, so adding a variant fails the build at every place that has to handle it. This is for matches that decide something for every variant."
- Fix: match on the pattern exhaustively, and test the event inside each arm.
- Confidence: high. Not a bug today. Roadmap: in part. It plans the enter pattern and a hero-damage trigger that must also match fatigue.

### 9. Damage is marked in two places

- Decision: how damage is marked on a character and reported.
- Places, both in `game`: `damage` at `zone_move.rs:262-277` reports `Damaged`. Fatigue inside `draw`, at `zone_move.rs:37-46`, adds to `object.damage` inline and reports `FatigueDamaged`. Both are in one file, but in functions that serve different decisions. Informal for replacement, prevention and damage triggers.
- Latent bug, confirmed by reading. `TriggerPattern::Damaged` matches only `Event::Damaged` (`triggering.rs:57`), so a "whenever your hero takes damage" trigger would miss fatigue. No hero holds a trigger today, because auras reach minions only (`lookup.rs:158`).
- Rule: "A change to a decision costs one edit for each place that knows it, plus the search for those places."
- Fix: fatigue calls `damage`, and the event carries the cause.
- Confidence: high. Roadmap: yes, "One damage function".

### 10. History is written in one file and read in another

- Decision: what history stores and how a query counts it (ADR 0005).
- Places:
  - `history.rs:3-46`, crate root: every field is public.
  - `history_from_event` at `triggering.rs:64-98`, in `game`, writes the entries. The entry's player means the dead object's owner for a death at line 68, and the player who played the card for a play at line 78.
  - `history_query` at `resolve.rs:285-320`, in `game`, reads them. It filters by player and turn and asks the binder whether a card is a spell or a minion.
  - All informal for "add a query kind", "add an entry kind" and "cache a query".
- Rule: "A decision whose data crosses the boundary isn't hidden, because any caller can copy it." Also: "Make a decision its own module when its knowledge sits in more than one place, today or in planned work".
- Fix: `history.rs` owns `History`, with private entries, one `record` command called from `emit`, and one `count` question. Recording the card kind at record time removes `count`'s need for the binder.
- Confidence: high. Not named on the roadmap. It lists the untested History branches and a cache, not the split.

### 11. The effect stack is a bare `Vec` pushed from three functions

- Decision: the order pending effects resolve in (ADR 0014), and the order each moment pushes them in.
- Places, all in `game`:
  - The field is declared at `game.rs:41`.
  - Pushes happen in three places. `zone_move.rs:63-68` pushes play effects before `Played`. `zone_move.rs:96-103` pushes deathrattles before deaths. `triggering.rs:34` pushes matched triggers.
  - The pop is at `resolve.rs:233`.
  - The order of one event's triggers comes from the sorts at `game.rs:225` and `lookup.rs:140`.
  - Informal for "change the stack order", for example to a queue, APNAP or a chain.
- Rule: "An order that is a rule of the domain is a decision, and gets a module like any other."
- Fix: an `EffectStack` type with private storage, one command for each moment that pushes, and a `next` question. The sort by order of play moves in with it.
- Confidence: medium. Roadmap: in part. It names `play`'s push order for enter effects, and the session K chain.

### 12. Card kind meaning is spread over boolean filters

- Decision: what each card kind is in play.
- Places:
  - In `cards::binder`, the exhaustive matches at `binder.rs:34-100` are formal. The filters `is_spell`, `is_minion` and `is_hero` at `binder.rs:50-60` are not.
  - In `game`, those filters are combined at `condition.rs:90-105`, `lookup.rs:56-58`, `lookup.rs:158`, `zone_move.rs:107-110`, `resolve.rs:304` and `resolve.rs:316`.
  - `filter.rs:5-22`, crate root: `CharacterKindFilter` and `CardKindFilter` copy the kinds by hand.
  - `state_check.rs:9-36` finds dead minions by board scan and dead heroes by hero health.
- In a temp copy, adding `CardDefKind::Weapon` failed the build only at six matches in `binder.rs`. Every other place compiled and gave the new kind an answer nobody chose. For example, a new kind with board presence would have no health there (`lookup.rs:56-65`), and `find_dead` would panic at `state_check.rs:15`.
- Rule: "When several filters together decide what each variant means, give that decision one exhaustive match."
- Fix: the binder answers each use as its own question through an exhaustive match: gets auras, stays in its zone when it dies, has health in play, matches a kind filter. Callers stop combining `is_*` checks.
- Confidence: medium. Not named on the roadmap. The evidence is the domain, plus hero replacement on the roadmap.

### 13. Zone entry and order of play are copied into each move

- Decision: what entering a zone does to an object, and how order of play is read. Today entry sets an order stamp. A reset is planned.
- Places:
  - The public `Object.order` field at `cards/object.rs:22-23`, and `update_order` at `object.rs:75-79`, in `cards::object`.
  - In `game`, `update_order` is called at `zone_move.rs:35` in `draw`, after the event. It is also called before the event at `zone_move.rs:138`, `224`, `240` and `254`.
  - Three sorts read the field: `game.rs:225`, `lookup.rs:140` and `state_check.rs:42`.
  - ADR 0012 defines the stamp as board entry, while the field's docstring says it updates on every move between zones.
  - Informal for "add a zone move" and "reset on zone change".
- Rule: "Prefer taking knowledge away from callers to making it formal."
- Fix: `ObjectBag` gets one `enter` command that stamps the object, and later resets it, plus an order-of-play comparison. The field becomes private.
- Confidence: medium. Roadmap: in part, "Raise this when bounce arrives (the zone-reset table)."

### 14. Playing a card is split over three files

- Decision: the play sequence. The glossary defines it as "Paying a card's cost, taking it from the hand and resolving it."
- Places, all in `game`:
  - `game.rs:124-144` checks affordability and offers `Play` when a card has no choices.
  - `resolve.rs:63-103` writes the draft's targets, pays at lines 98-101 and calls `play`.
  - `zone_move.rs:51-82` takes the card from the hand, queues its effects, reports it, sets checkpoints and enters the board.
  - `game.rs:190-194` offers `Play` from a draft without checking the cost again. It relies on nothing changing cost or mana during a draft.
  - Informal for Health costs, enter-effect order and a stack zone.
- Rule: "Don't split by processing step." Also: "An order that is a rule of the domain is a decision, and gets a module like any other."
- Fix: one `play` command holds the sequence. `legal_actions` asks a `can_pay` question that sits next to `pay`.
- Confidence: medium. Roadmap: in part. It names the enter-effect order and the fact that a resolving card is in no zone.

### 15. Characters in play are listed by two functions

- Decision: which characters are in play.
- Places, all in `game`: `board_characters` at `game.rs:216-227` lists them in order of play. `scan_characters_with_filter` at `condition.rs:47-67` lists heroes, then boards. `health` at `lookup.rs:53-65` decides per object. The death test `health <= 0` is copied three times in one file, at `state_check.rs:16`, `33` and `67`. Informal for "dead heroes leave play" and "skip mortally wounded minions". That second change would add a fourth copy in `condition.rs`.
- Rule: "Give each question its own function, even when two share an answer today. The second can be built on the first."
- Fix: one `characters_in_play` question, with the ordered list and the filtered list built on it.
- Confidence: medium. Roadmap: in part. It names dead heroes in `board_characters` and mortally wounded minions.

### 16. The resolving effect's cursor is public, and its caller knows the step order

- Decision: how a resolving effect tracks its place. Today it is one index. A frame stack is planned.
- Places: `effect.rs:80-107`, crate root, with a public `idx`. `continue_resolving_effect` at `resolve.rs:250-258`, in `game`, reads `curr_effect`, clones it, reads `object_id` and advances before it applies the effect, so that a pause resumes after that effect. Informal for "frame stack".
- Rule: "A caller that must call two functions in order still knows the decision, even when a type enforces it. One command that does both leaves the caller nothing to get wrong."
- Fix: private fields and one `next_effect` command that returns the source and the effect and advances.
- Confidence: medium. Roadmap: in part, "a resolving effect has one cursor today."

### 17. ObjectIds tell the opponent which hidden card is which

- Decision: what each player may see (ADR 0006).
- Places: `hand_card` at `view.rs:114-123`, in `game`, shows the ids of hidden cards and assumes an id says nothing about the face. `setup` at `game.rs:63-80`, also in `game`, allocates ids in deck-list order and shuffles only afterwards, at lines 96-100. `next_id` at `cards/object.rs:34-38` counts up from zero. Informal for every change.
- Live bug. A throwaway program run against an unmodified copy, using only the public interface, had P1 guess each hidden card in P0's opening hand from its id and the deck list. Across 200 seeds it named all 800.
- Rule: "Ideally one place knows it, and every other place asks it a stable question or gives it a stable command, by name."
- Fix: shuffle the definitions before allocating ids. Setup would then be one command that cannot get the order wrong.
- Confidence: medium as a structure finding, high as a bug. Roadmap: yes.

### 18. `loader.rs` holds the card set and the rules of a valid card

- Decision: the content of the card set, the validation rules, and the source of definitions. All three change for different reasons.
- Places: in `cards::loader`, the code constants at `loader.rs:42-87`, the card list at `94-555`, validation at `565-723` and errors at `20-38`. In `game`, the source is chosen at `game.rs:60`.
- Evidence: each of the last five feature commits changed `loader.rs`, by 64 to 267 lines. Commit 995b2bb added three cards and the pick validators to the one file.
- Rule: "A boundary that holds many decisions that change independently doesn't. Its code has to be sorted by decision before a change."
- Fix: one file for the card set and one for validation. `loader` keeps only loading, once data files arrive.
- Confidence: medium. Not named on the roadmap, which plans the session H work.

### 19. The card format is part of the crate's public interface

- Decision: the layout of a card definition.
- Places: `lib.rs:28-31` re-exports `CardDef`, `CardDefLoader` and `CardDefError`. `CardDef`'s fields at `definition.rs:23-33` are public, and through them so are the effect, selector, choice and trigger types. No test or bench uses these three. They use only the `static_card_definition` constants.
- Rule: "An interface is made of decisions too, the ones its callers know. Hiding one decision exposes others, so expose the ones you expect to stay put." Session H plans to change this format.
- Fix: drop the three from the `pub use` list until a shell needs them. Many `pub` items would then become `pub(crate)`.
- Confidence: low. Not named on the roadmap.

### 20. Checkpoints are set in ten places across three files

- Decision: where a step ends in the reported stream. ADR 0006 calls these "set moments".
- Places, all in `game`: `resolve.rs:106`, `111`, `118`, `130`, `135` and `259`. `zone_move.rs:76` and `80`, inside `play`. `state_check.rs:51` and `78`.
- Rule: "Steps still run, inside one coordinator that calls the modules in order." Also: "When it is the procedure itself, it lives in the coordinator." `play` is not the coordinator.
- Fix: move the two checkpoints out of `play` and into the coordinator that calls it.
- Confidence: low. Not named on the roadmap.

## Names

I judged 28 names: 14 at the root, 6 under `cards/` and 8 under `game/`. I skipped `lib.rs` and `cards/mod.rs`. These 13 fail.

1. `game`. A parent's name must rule out what its siblings hold, and this one doesn't. `Game::history_query` (`resolve.rs:285`) and `Game::history_from_event` (`triggering.rs:64`) live under `game`, though a sibling is called `history`. `Game::bury` (`zone_move.rs:232`) lives under `game` beside a sibling called `zones`.
2. `turn`. `Game::start_turn` (`game.rs:254`) holds the turn's rules, mana and the draw, and lives in `game.rs`. `turn.rs` holds seat order, as its own docstring says. `turn_order` would match `TurnOrder` and the glossary's "A place in the turn order".
3. `ids`. It holds only `PlayerId`. `ObjectBag::next_id` (`cards/object.rs:34`) makes ObjectIds, `DefId::new` is in `cards/definition.rs`, and `ChoiceId` and `PickId` are in `choice.rs`.
4. `choice`. `EffectAmount::from` (`choice.rs:9`) serves every verb, and the roadmap already says so. `PickTarget`, written by `Game::pick` (`zone_move.rs:181`), is a pick, and the glossary keeps Pick apart from Choose.
5. `cards`. Card text is split between it and its siblings `effect`, `choice` and `trigger`. `CharacterFilter::all` (`cards/filter.rs:37`) is under `cards`, while `AmountBound::is_satisfied` (`choice.rs:23`) for the same declared choice is not. `ModifierEffect` is under `cards`, and `Effect` is not.
6. `cards/loader.rs`. `CardDefLoader::load_all` (`loader.rs:94`) is the card set itself, not a loading step. A reader looking for the Bomber's text would not open a loader, and the file's TODO says loading belongs to the shell.
7. `cards/definition.rs`. `PlayerSelectorFilter::all` (`definition.rs:96`) is a filter, yet its sibling `filter.rs` holds the other filters. `CharacterSelector` (`definition.rs:111`) is, per the glossary, "The part of an effect that says what it acts on".
8. `cards/modifier.rs`. "Modifier" is not in the glossary. `Game::modifiers` (`lookup.rs:144`) collects printed self-modifiers, buffs and auras. `Modifiers::add` stores a buff at `resolve.rs:199` and an aura at `resolve.rs:209`.
9. `game/condition.rs`. The word is not in the glossary, and almost any check could fit it. `Game::fulfill_trigger_pattern` (`triggering.rs:39`) is a condition, yet it lives in `triggering`. `can_finish_choices` (`condition.rs:107`) is draft legality.
10. `game/lookup.rs`. `Game::mana` (`game.rs:229`) and `Game::effect_amount` (`resolve.rs:278`) are lookups that live elsewhere. `Game::triggers` (`lookup.rs:119`) is none of the "attack, health and cost" its docstring promises.
11. `game/resolve.rs`. The glossary's Resolve means "To carry out a card's text". `Game::history_query` (`resolve.rs:285`) and the `Cancel` arm of `apply_action` (`resolve.rs:132`) carry out no card text. `Game::damage` (`zone_move.rs:262`), which carries out `Damage`, lives in `zone_move`.
12. `game/triggering.rs`. `Game::history_from_event` (`triggering.rs:64`) and the observer report in `emit` (`triggering.rs:16`) have nothing to do with triggers.
13. `game/zone_move.rs`. `Game::damage` (`zone_move.rs:262`) and `Game::pick` (`zone_move.rs:181`) move nothing between zones.

These pass: `action`, `effect`, `event`, `history`, `observer`, `outcome`, `rng`, `trigger`, `zones`, `binder`, `filter`, `object`, `player`, `state_check` and `view`.

## Already following the rules

- `rng`, `turn` and `zones` keep their data private and expose commands named for the domain, such as `pop_front` for "take the top card".
- `PlayerId`, `ObjectId` and `DefId` are newtypes with private fields. Only `ObjectBag` makes ObjectIds.
- `apply` runs the legality check and the change as one command (`resolve.rs:39-54`). `apply_action` is `pub(super)`.
- `emit` is the one command that records history, reports to the observer and matches triggers. A clippy `disallowed_methods` ban plus the `#[expect]` at `triggering.rs:12-15` makes "only `emit` reports" formal.
- `clippy.toml` bans `HashMap`, `HashSet` and the clock, which makes ADR 0001 formal.
- These matches are exhaustive with no `_` arm: `apply_effect` (`resolve.rs:147`), the selector matches (`resolve.rs:334`, `350`), `chosen_id` (`loader.rs:627`, `636`), `validate_pick_order` (`loader.rs:703`), the kind matches in `binder.rs` and `legal_actions` on the interaction state (`game.rs:116`).
- Informal copies kept close together, which the doc accepts: the owner check in `view.rs:115` and `149`, the turn filter repeated inside `history_query`, and the death test in `state_check.rs`.
- One place each, so left alone: the hero definition (`game.rs:70`), the card source (`game.rs:60`), the mana cap (`game.rs:261`) and `PlayerId` as an index (`game.rs:66`, `267`).
- Crate tests use only the public interface, and unit tests sit in `rng`, `turn`, `zones`, `object` and `loader`, as `testing.md` asks.

## Doc feedback

1. "A module in the general sense is a `mod` whose private items include the data its decision is stored in." The doc doesn't say what a `mod` with no private data is, or where its items belong. `action`, `choice`, `effect`, `event`, `history`, `outcome`, `trigger`, `cards/filter` and `cards/mod` have none. I counted them as part of the crate root, which then holds many layouts. "Judge a parent by its own code, which should hold one decision" would flag that, while ADR 0004 wants card data shared.
2. The Rust section speaks of parent and child, not of the crate. A `pub(super)` item in a root child, such as `PlayerId::new` at `ids.rs:8` or `Graveyard::add` at `zones.rs:102`, is visible to the whole crate. Is the crate a module whose private data is every `pub(crate)` item? I couldn't tell.
3. "A decision hides from the parent only in a type declared in a child" reads as necessary, not sufficient. `player.rs` declares types with `pub(super)` fields, so I counted it as part of `game`. The doc could say a child hides a decision only when the fields are private to the child.
4. "A filter that only cares about some variants can use `_`." There is no test for telling a filter apart from a decision for every variant. `history_from_event` (`triggering.rs:66-95`) maps three events to entries and the rest to none. Does it decide, for every event, whether it is recorded? A match on a pair, as at `triggering.rs:46`, can be exhaustive on one axis only, and the doc doesn't cover that either.
5. "Judge whether a decision is likely to change by reasoning about the domain: what other systems in it already vary". In card games almost everything varies between Hearthstone, MTG, Runeterra and Yu-Gi-Oh: stack order, costs, zones, kinds and death timing. Read literally, every decision is likely to change. I counted a decision as likely when the roadmap plans it, or when a well-known card in one of those games forces it. The doc gives no threshold.
6. "Informal knowledge is acceptable where its locality is high, meaning its places sit close together in code that serves the same decision." When one Rust module spans nine files, are two places in sibling files close? What about two places in one file but in functions that serve different decisions, like fatigue and `damage` in `zone_move.rs`? I treated only same-file places serving the same decision as high locality.
7. Names. When a name is right but the functions it claims live elsewhere, as with `history` and `history_query`, is that the name failing or the parent `game`? I charged the parent, following "it passes when it rules out what its siblings hold". The doc could say so.
8. Two rules pulled different ways on `effect.rs`. The glossary defines Effect, Pending effect and Resolving effect, so the name passes the names rule. The file holds two decisions that change independently, the card text vocabulary and the resolution cursor. But it hides no data, so by the Rust section it isn't a module and the decision rule doesn't apply to it. I passed the name and filed the cursor as finding 16.
9. "Make a decision its own module when its knowledge sits in more than one place". The doc doesn't say what to do when the decision's rule needs data another module owns. History counting needs the binder's kinds and the turn counter, and a `Draft`'s candidates need the board. You could pass the data in, store it at record time, or take a closure. Each choice changes what callers know.
10. "An order belongs to whoever decides it. ... When it is the procedure itself, it lives in the coordinator." Where steps end in the reported stream is procedure, but it is also a contract that tests pin and shells read. I couldn't tell which sentence applies, so finding 20 has low confidence.
11. Following the doc on finding 17 led to a fix that is a reorder in setup, not a move. The doc covers the coupling ("a relied-on meaning"), but it doesn't say whether an audit should report a breach whose structure is fine and whose order is wrong. I reported it, because the bug is live.
12. "Take names from the glossary." `modifier` has no glossary term. Buff and Aura are terms, but the type also holds cost reductions and granted triggers. The doc says to add a term when the parts change together. A cost reduction and a buff share the derive-on-read mechanism and differ in what they change, so I couldn't tell whether the type is one decision with no word yet, or several.
