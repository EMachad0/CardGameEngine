> [!quote] YOU

continue the architecture course rehersal questions not needed


> [!question] Question
> Run the 01d audit with the same setup as 01a's round 2?
>
> Setup, unchanged from 01a round 2:
> - Two auditors with the same prompt, each working alone over the whole `rules` crate. If they disagree, the doc is ambiguous. If both miss something, the doc doesn't lead a reader to it.
> - The doc is their only standard. They may read the source, the tests, the glossary, the ADRs, git history under `crates/`, and what they know about card games as domain evidence. They may not read `docs/course/`, `docs/issues/`, session logs or PRs, because those hold the earlier findings.
> - New this time: they first state the module tree by the doc's definition (so `game/` counts as one module), and they judge every module and file name against its siblings and the glossary.
> - Reports go to `research/01d-audit/audit-1.md` and `audit-2.md`. I compare both with each other and with the 16 findings from 01a, then bring you the merged list and the doc feedback.
>
> The open choice is the roadmap. The doc says to judge likely change by "what the roadmap plans", but its project section doesn't say where the roadmap lives. The card-game course plan and its open threads are the closest thing we have. They also list some 01a findings word for word (dead heroes in `board_characters`, `pick_targets` never cleared), so giving them to the auditors would spoil part of the test.
>
> 1. Same setup, no roadmap (Recommended)
> 2. Same setup, plus the card-game course plan and open threads as the roadmap


> [!example] Answer
> Other: I guess we will do at least 2 rounds, lets do the first without the roadmap and then at the end we do one with roadmap to see how they compare and prepare for later sessions to also include it


> [!abstract] PI

Round 1 runs without the roadmap. The last round adds the card-game course plan and open threads, so we can compare the two and decide how later audits use the roadmap. Starting the two auditors now.


> [!abstract] PI

Both auditors are running in the background. The reports will be `research/01d-audit/audit-1.md` and `audit-2.md`.

When they finish, I'll check their strongest claims against the code and compare the two reports with each other and with 01a's 16 findings. Then I'll bring you three things: the merged findings, the places where the doc couldn't decide a case, and what changed now that `game/` counts as one module.


> [!abstract] PI

Audit 1 is in: 18 findings, 9 of them high confidence, 18 names that fail and 13 doc feedback items. I'm waiting for audit 2 before I compare them.


> [!abstract] PI

Both reports are in, saved as `research/01d-audit/audit-1.md` and `audit-2.md`. I checked their strongest claims against the code.

## What changed since 01a

**Both auditors put the defined module first.** By the Rust section, `game` is one module. Its eight child files add `impl Game` blocks and hide nothing from each other. Both list about fifteen decisions in it and report "`game` holds many decisions that change independently" as a high-confidence finding. The evidence is the same in both: commits 8bcdccc, ae79835 and 995b2bb each changed one decision and touched five to seven files. In 01a every auditor counted files as modules, so this finding couldn't exist then.

**Comparison with 01a's 16 findings:**

- **Both auditors found 11 of them:** characters in play (1), the order stamp (2), the effect stack (3), the draft (5), mana (6), unit context (7), the card text walk (8), history (10), card kinds (11), trigger matching (12) and modifiers (13).
- **One auditor found 3:**
  - The Mourner bug (4), found by audit 2, which reproduced it. It's already filed on main as `docs/issues/dying-mourner-hears-older-death.md`.
  - Owner versus side (14), found by audit 1.
  - Checkpoints (16), found by audit 1.
- **Neither found 2:**
  - Interaction state destructured with `unreachable!` (15). I checked, and all three sites are inside `game`. The doc calls informal knowledge inside a module cheap, so a reader who takes `game` as one module has no reason to report it. That's doc item 1 below.
  - Nothing proves a `Binder` holds a validated table (9). Expected, because that's parse, don't validate, and the doc doesn't cover it yet (session 02b).

**New findings:**

- **One character scan answers three questions** (audit 2, high confidence): what "all" hits, the random candidates, and the draft candidates. Stealth, Elusive and hexproof split them apart. It's the "own function per question" rule, and it refines finding 1.
- **The loader checks resolution's rules** (audit 1, medium). The validators know that `Chosen` needs a draft and that `ResolvePick` needs a `Reveal`. That's a check that copies the act's rule, the step split from 01a.
- **Fatigue marks damage by hand.** Both auditors found it, and in 01a I had dismissed it as cheap inside one file. Audit 1 also confirmed a latent bug: a `Damaged` trigger on a hero doesn't fire on fatigue. This is already a card-game open thread.
- **Latent bugs confirmed in `/tmp` copies:**
  - Adding a `Location` kind panics at `state_check.rs:15`.
  - `validate_pick_order` accepts a pick ID reused after it resolves. I confirmed this by reading the code.
  - A board aura reaches minions in every zone, including the hand. I confirmed `lookup.rs:158`: it checks `is_minion`, not board membership. Whether a cost aura should reach the hand is a rules question for the card-game course.
- **Low confidence, one auditor each:** the placeholder rule split between `definition` and `loader`, which import each other, and the side rule written twice.

**Names:** each auditor failed 18. They agree on `game`, `condition`, `lookup`, `resolve`, `triggering`, `zone_move`, `cards`, `definition`, `choice`, `ids`, `turn`, `history` and `modifier` (both missing from the glossary), and on the test files `cards`, `lookups` and `resolution`. They split on `cards::filter` and `cards::loader`: audit 1 fails both, audit 2 passes both.

**Disagreement on the card source.** Audit 1 reports that `Game` knows where definitions come from. Audit 2 says only one place knows it, so the doc says to leave it. I side with audit 2. When loading moves to the shell, the decision moves across the boundary, and no interface chosen today would hold still for that.

## Doc feedback, with proposed fixes

Your call on each:

1. **What counts as a place, and the "cheap" clause.** The doc never defines a place. Inside `game`, every copy is "cheap" unless you lean on the "many decisions" sentence. Audit 2 counted functions as places. Proposed fix: "Informal knowledge is cheap only in a module that holds one decision, or a few that change together. In a boundary that holds several, each function that knows a decision is a place." Report the boundary once, and each decision spread inside it as its own finding.
2. **A module without private data.** `loader` hides its validation rules as private functions and has no private data. The Rust bullet says "the data its decision is stored in", which is too narrow for a rule computed from its inputs. Proposed fix: "...whose private items include what its decision is stored in: its data, or its code when the decision is a rule computed from inputs." Also add: "A file that declares only public data hides nothing. Its layout is interface, and its name still has to pass."
3. **"Private" in Rust.** Proposed fix: "Private means not visible outside the `mod` and its children. `pub(crate)`, or a `pub` field on a `pub(crate)` type, is visible to the whole crate."
4. **Glossary versus technical names.** Proposed fix: "Domain concepts take their names from the glossary, and a concept it lacks gets added. Technical names such as `rng` or `observer` need no entry."
5. **Files of `impl` blocks**, your #13 verdict in the doc. Proposed fix: "Child files that only add `impl` blocks to the parent's type add names, not boundaries. To split a module, declare a type with private fields in the child."
6. **A trigger matcher on a tuple:** which enum must be exhaustive? Proposed fix: extend the Rust bullet to say "...one exhaustive match, on the type whose variants it decides."
7. **Tests.** `testing.md` already names `cards` as a topic module in `tests/spec/`, and it covers `model.rs` as an oracle. Proposed fix: a scope line, "Tests follow `docs/conventions/testing.md`", so the name rule doesn't judge test topics. Or does the name rule apply to topic modules too?
8. **A function that fits a sibling's name better** (`kill` in `zone_move`): name failure or misplacement? Proposed fix: "A function that fits a sibling's name better than its own is misplaced. Its file's name fails only when it didn't rule the function out." Optional.
9. **No change proposed:**
   - A parent with no code: "judge a parent by its own code" already passes it.
   - Several formal places, such as one exhaustive match per interpreter (ADR 0004): the doc already says each place costs an edit and the build does the search.
   - Load-time validation as a fourth tool: you said no in 01a. Audit 1's latent bug in the validator walk doesn't change that, since the walk is finding 8's job.

**Deferred to session 02:** telling procedure from domain order, an order that callers express as call order around a third module (the stack and `emit`), and a decision module that needs another module's data (`History` asking `Binder` about card kinds).

Answer in plain text, for example "1 ok, 2 change X". Then I'll amend the doc and run round 2 with the card-game plan and open threads as the roadmap. After that you get the merged list for `COURSE.md`.


> [!quote] YOU

1 I dont get it, place on the file-structure doc means a hunk of code no? so on "A change to a decision costs one edit for each place that knows it, plus the search for those
places. The build does the search only when the knowledge is formal. So the fewer places know the
answer, the cheaper the change. Ideally one place knows it, and every other place asks it a stable
question or gives it a stable command, by name." literally means pieces of code u need to edit. 2 is it? here loader is not a good module because others can copy its decisions, I dont see the problem. 3 private simply means not public anywhere, I thought agents were smart enough to understand this. 4 this is glossary definition, it should not live here. 5 why do we need this change? what is wrong with current one. 6 again why this change. 7 I am fine with adding this link on this project section. 8 could be both I woudnt add it. 9 ok. apply the ones I decided on and lets discuss the others with more detail


> [!abstract] PI

I applied 7. The project section now has this line, uncommitted:

```markdown
- `docs/conventions/testing.md` says where tests go and how they are named.
```

3, 4 and 8 are dropped, and 9 needs no change. Here are the rest in more detail. After working through them, I'm dropping 2, 5 and 6 as well.

**1. You're right, and I named the wrong problem.** A place is a piece of code you have to edit, and the doc already says so. The real gap is two sentences that give different answers for a copy inside `game`:

- **Sentence A**, at the end of the coupling section: "Informal knowledge in the module that hides the decision is cheap. In any other module, make it formal."
- **Sentence B**, in the module bullets: "A boundary that holds many decisions that change independently doesn't get this, since its code has to be sorted by decision before a change."

Take 01a's finding 15. `resolve.rs:74`, `resolve.rs:127` and `zone_move.rs:194` each destructure `PlayerInteractionState` with `let ... else { unreachable!() }`. Each assumes which state the player is in at that moment. That's three copies of the transition rules, all inside `game`.

- Under A, `game` hides that decision from the rest of the crate, so the three copies are cheap and there's nothing to report.
- Under B, `game` holds about fifteen decisions, so it doesn't get the discount, and they're a finding.

Both auditors missed this finding. Audit 2 said its findings 7 and 12 to 16 depend on which reading wins. B is the one that matches the reason for the discount. Informal knowledge is cheap because whoever changes the decision reads the whole module anyway, and nobody reads 2,000 lines of `game` to change the transition rules.

Proposed fix, sentence A only: "Informal knowledge in a module that hides only that decision, or a few that change together, is cheap. Anywhere else, make it formal." Ok, or another wording?

**2. I agree, no change.** Your reading is consistent with the doc, and the code backs it up. The loader's validation rules are private functions, but they run over `CardDef`, whose fields are public. Anyone can read those fields and apply the rule again, and `resolve.rs` does: it panics on a `Chosen` with no draft, which is the same rule the validator checks (audit 1, finding 11). A rule over public data isn't hidden, whatever its own visibility. So `loader` isn't a module, and the doc says so correctly. Making it one means the inputs go private, which is the parse, don't validate node (02b).

**5. Nothing is wrong with the current text. Drop it.** The Rust bullet already says that child files adding `impl` blocks belong to the parent's module, and that a decision hides from the parent only in a type declared in a child. The prescription follows from those two sentences. Both auditors applied them correctly and treated `game` as one module. Only one asked whether splitting into such files helps, and the doc's facts answer that.

**6. Same, drop it.** The auditor asked which enum the tuple match `(pattern, event)` must be exhaustive on. Both auditors still reached the same answer: exhaustive on the pattern, filter the event inside each arm. The existing wording led them there, and a flagged ambiguity that two readers resolve the same way isn't a gap.

So the only open item is 1. Once you answer it, I'll show you the final doc diff for review. Round 2 then runs with the roadmap.


> [!quote] YOU

"Informal knowledge in a module that hides only that decision, or a few that change together, leads to cheap changes." wdyt?


> [!quote] YOU

actually "knowledge in a module that hides only that decision, or a few that change together" is repeting the definition of module , instead, I believe cheap is a property of the change, so "Informal knowledge shared within a module is acceptable, as changes stay cheap." wdyt?


> [!quote] YOU

ok, what I said is not true,
"Informal knowledge whose locality is high is acceptable as changes stay cheap." sounds true to me, as changes stay near each other.
"Good Modules, by their definition, keep locality high and changes cheap.
"Informal knowledge is inevitable, the simply fact that one code instruction comes after the other is a decision, informally enforced".


> [!abstract] PI

Your sequence of rewrites holds up. Each one fixes a real problem with the one before it. My take on the three statements, and then a wording.

**"Informal knowledge whose locality is high is acceptable as changes stay cheap."** True, and it names the property directly. The doc's cost model is one edit per place plus the search for those places. Informal knowledge gets no help from the build, so the search is what you pay. High locality keeps the search short. That fixes the ambiguity from item 1 without asking whether `game` counts as a module:

- The draft walk sits in `game.rs` and `condition.rs`. The `unreachable!` copies sit in `resolve.rs` and `zone_move.rs`. Each lives inside a 2,000-line boundary with fifteen decisions, so their locality is low.
- Fatigue's copy of damage is in the same file as `damage`. That's closer, but the file also holds moves, picks and `kill`, so the locality is middling. That matches both auditors reporting it at medium confidence.

**"Good modules, by their definition, keep locality high and changes cheap."** True for the decision the module hides, with two adjustments:

- "Good" is redundant. A boundary that doesn't keep its knowledge local isn't a module by the doc's definition, which is why the doc calls `game` "a boundary that holds many decisions".
- It holds only for changes to the hidden decision. A change to a decision the module exposes, its interface, isn't cheap. The doc already says "No design makes every decision cheap to change."

**"Informal knowledge is inevitable."** True, with one nuance on your example. Statement order is informal only when nothing forces it. When a line uses a value computed by an earlier line, the compiler enforces that order, so it's formal. Push before emit is informal because nothing ties the two lines together. Inevitability is why the doc needs a tolerance rule at all, rather than "make everything formal". I don't think it needs its own sentence, because the doc never claims that everything should be formal.

**Proposed wording.** The doc says the same thing in two places, so both change. Here's the coupling section:

```markdown
Judge each place against the changes you expect to the decision. Informal knowledge is
acceptable where its locality is high: its places sit close together, in code that serves the
same decision, so a change still finds them all cheaply. Anywhere else, make it formal. [...]
```

And the module bullet, which becomes the case where locality comes for free:

```markdown
- A module keeps the knowledge of its decision local, since all of its code serves that
  decision. A boundary that holds many decisions that change independently doesn't, since its
  code has to be sorted by decision before a change.
```

"In code that serves the same decision" is what rules out two adjacent copies in a file that holds everything else. Ok, or do you want to change the wording?


> [!quote] YOU

agreed, unslop it


> [!abstract] PI

Rewrapping the long line.


> [!abstract] PI

The doc diff, unslopped. Besides the punctuation, I split the colon sentence in two and made "cheaply" a concrete short search:

```diff
-- Informal knowledge inside a module is cheap, because all of its code serves its decision. A
-  boundary that holds many decisions that change independently doesn't get this, since its code
-  has to be sorted by decision before a change.
+- A module keeps the knowledge of its decision local, because all of its code serves that
+  decision. A boundary that holds many decisions that change independently doesn't. Its code has
+  to be sorted by decision before a change.

-Judge each place against the changes you expect to the decision. Informal knowledge in the module
-that hides the decision is cheap. In any other module, make it formal. Put it in a type, ...
+Judge each place against the changes you expect to the decision. Informal knowledge is acceptable
+where its locality is high, meaning its places sit close together in code that serves the same
+decision. A change then finds them all with a short search. Anywhere else, make it formal. Put it
+in a type, ...

 ## This project
 - `docs/agents/domain.md` says where the glossary lives and how to use and extend it.
+- `docs/conventions/testing.md` says where tests go and how they are named.
```

It isn't committed. If you approve, I'll commit it and start round 2. Round 2 runs two new auditors with the same prompt as round 1, plus the "Course plan" and "Open threads" sections of `docs/course/card-game/COURSE.md` as the roadmap. Their reports go to `audit-3.md` and `audit-4.md`.


> [!quote] YOU

approved, a module can have more than one decision do u think this should make us change the text?


> [!abstract] PI

The doc is committed as `3969941`, and `git status` shows nothing left to commit in it.

**Yes, I'd change it, in one place.** The new bullet doesn't break with more than one decision. In a module whose decisions change together, a change to one is a change to all of them, so whoever makes it reads the whole module anyway, and locality still holds. The issue is the wording. The definition allows the plural, and every later sentence uses the singular:

- The definition: "hides one decision, **or a few that change together**, so that code outside it can't copy **the decision**."
- The new bullet: "keeps the knowledge of **its decision** local, because all of its code serves **that decision**."
- The parent bullet: "which should hold **one decision**."
- The Rust section: "the data **its decision** is stored in."
- The names section treats parts that change together as one decision: "If a plausible change to one part leaves the others alone, they are separate decisions. [...] If they change together, name the concept."

So the doc counts decisions that change together as one in the names section, and as several in the definition. Take #10's verdict, the IDs and the card table in one module. Someone reading "all of its code serves that decision" could count two decisions there and conclude the module fails.

**Proposed fix**, at the definition only, so every later singular becomes exact:

```markdown
A module is an enforced boundary that hides one decision, so that code outside it can't copy it.
Decisions that change together count as one.
```

The alternative is to change nothing and read the singular as shorthand for the definition's "or a few". Round 1 shows that would mostly work, since neither auditor flagged it. I'd still make the edit. It's one sentence, and it makes the definition agree with the names section.

Apply it, rephrase it, or leave it?


> [!quote] YOU

apply it


> [!abstract] PI

Applied and committed as `17df72d`, with nothing left uncommitted in the doc. Round 2 now starts, with the roadmap.


> [!abstract] PI

Round 2 is running: two auditors, reports in `audit-3.md` and `audit-4.md`. The prompt has three changes from round 1:

- The card-game course plan and open threads are allowed as the roadmap. Nothing else under `docs/course/` is.
- Each finding says whether the roadmap already names it, and the decision table says which evidence came from the roadmap.
- They read the doc with today's two commits, and they may read `testing.md` because the project section now points to it.

When they finish, I'll compare round 2 with round 1. The questions are what the roadmap added, whether interaction state now gets found under the locality rule, and whether the doc feedback shrank. Then you get the merged list.


> [!quote] YOU

tbh I am fine if they dont find every issue, I think its right to do this changes iteractively, as changes may make other issues apparent


> [!abstract] PI

Agreed. That changes how I'll read round 2. A miss is a doc problem only when a rule clearly covers the case and the auditor still didn't apply it, as with interaction state under the old "cheap" sentence. If a miss just means nobody looked there, the refactor or the next audit will find it.

So after round 2 we stop. No third round to chase completeness. I'll merge the list, and you review it and the doc feedback. Then the list goes into the open threads as a working list, not a full inventory. Each refactor session will add whatever its changes bring to the surface.

I'll also record this in `COURSE.md` under "How sessions work" at the close: an audit is one pass per node, and it doesn't try to catch everything.


> [!abstract] PI

Audit 4 is in: 17 findings, 9 of them high confidence. The roadmap names 3 of them, and part of 4 more. I'm waiting for audit 3 before I compare.


> [!abstract] PI

Round 2 is in: `audit-3.md` (20 findings, 10 high confidence) and `audit-4.md` (17 findings, 9 high). Here's the comparison, three doc items for you, and the merged list.

## What the roadmap changed

- **It set a bar for "likely".** In round 1, domain reasoning admitted nearly everything. Audit 3 now counted a decision as likely to change when the roadmap plans it or a well-known card forces it. Confidence levels went down to match.
- **It tied findings to sessions.** Most findings now name the session that will hit them, such as 06b's frame stack, 06c's choices, the K chain or H's data files. That helps order the refactor.
- **It surfaced a few new findings:**
  - The resolving effect's cursor is public, and 06b plans a frame stack.
  - Trigger order and death order are two questions, since the K chain moves one and not the other.
  - An eliminated player's hero is still a target.
  - The card format is part of the crate's public interface.
- **I saw no spoiling.** Both auditors marked honestly what the roadmap already names, and they still confirmed each bug themselves in `/tmp` copies.
- **The live bug was found 2 times out of 4.** The Mourner bug was found by audits 2 and 3, and missed by 1 and 4. The interaction state's three `unreachable!` sites stayed missed in both rounds. Per your point, I'm not treating either as a doc gap.

## Doc feedback, three items for you

1. **Files with no private data.** All four auditors asked what `action`, `effect`, `event`, `history`, `trigger` and the other files of public types are. Audit 3 folded them into the crate root, which then holds many decisions. When you dropped item 2, I had bundled this with the loader point, which you were right to reject. This part stands on its own. Proposed wording: "A file that declares only public types hides nothing. Its types are part of the interface of every module that takes them in or hands them out, and its name still has to pass."
2. **Locality against new callers.** Audit 4 noticed that the five `update_order` calls all sit in `zone_move.rs`, so today's sentence calls them local and acceptable. It didn't report them. The other three did, under "prefer taking knowledge away from callers". Locality makes changing the decision cheap, but it doesn't stop the next move function from forgetting the stamp. Proposed addition after the locality sentence: "Locality keeps a change to the decision cheap, but new code can still miss it. When each new caller must repeat a step, take that step away from callers." No other auditor stumbled on locality. Audits 3 and 4 asked whether two files of one module count as close, and both concluded no, because the code doesn't serve the same decision. The sentence works as written there.
3. **Where the roadmap lives.** The doc says "what the roadmap plans", but nothing says where the roadmap is. I'd add a line to the project section: "The roadmap is the course plan and open threads in `docs/course/card-game/COURSE.md`." I'd also record in this course's `COURSE.md` that audits get it. Or record it in `COURSE.md` only, if you'd rather keep a course file out of the convention doc.

**No change proposed** for points that came back but are already decided:

- Which side of a tuple match must be exhaustive: 3 of 4 asked, and all picked the pattern.
- Misplaced functions and names across levels: 3 asked.
- Crate-wide `pub(super)`: 2 asked.

**Deferred to session 02:**

- A decision with no data of its own, or one that needs another module's data: 4 asked.
- Procedure versus domain order: 2 asked.

## Merged findings

"All" means all four auditors found it. The 01a number follows.

1. **`game` is one module holding about 15 independent decisions.** All, high. New, because of the module definition. Fix: child types with private fields, while `game` keeps the coordinator. Session 02.
2. **The death pass reports deaths one at a time.** The Mourner bug, live. Audits 2 and 3 (01a 4). Already filed on main.
3. **Characters in play.** Computed in four or five places. One scan answers three questions: what "all" hits, the random candidates and the draft candidates. A dead hero hears its own death twice (confirmed), and an eliminated hero is still a target. All (01a 1).
4. **Order of play.** The stamp is `pub`, written at five sites and sorted at three, and its docstring disagrees with ADR 0012. Trigger order and death order are two questions. All (01a 2).
5. **Effect stack order** is known at three push sites. Audits 1, 2 and 3 (01a 3, #17).
6. **Draft.** The walk is copied in `legal_actions` and `can_finish_choices`, and the layout is read in five places. All (01a 5).
7. **`is_satisfied` answers "may stop" and "is full".** With `AtMost` restored, no `Choose` is ever offered (confirmed three times). All (01a 5).
8. **Mana and playing.** The check, the payment and the refill sit in three files. Audit 3 reads it as the glossary's Play split three ways. All (01a 6).
9. **Resolution answers live on `Object`.** `pick_targets` is never cleared, and the validator accepts a reused pick ID. Latent, confirmed. All (01a 7, roadmap 06b).
10. **The validators walk card text by hand.** They miss `modifier_effects` and nested grants, and the result is a confirmed panic. All (01a 8, #14).
11. **`loader` holds the card set, the IDs and validation.** `Binder::new` stores definitions without validating them. Audits 3 and 4 (01a 9).
12. **The validators copy resolution's rules:** `Chosen` needs a draft, and `ResolvePick` needs a `Reveal`. Audit 1. New.
13. **History** is recorded in one file and counted in another, and its entries are `pub`. All (01a 10, #16).
14. **Card kinds.** The `is_*` checks are combined in six places, the kind filters copy `CardDefKind`, and `has_deck_presence` is asked to mean "goes to the graveyard". A new board kind panics at `state_check.rs:15`. All (01a 11, #15).
15. **Trigger matching ends in `(_, _)`.** All (01a 12).
16. **Modifiers** are decided by four `_` filters. All (01a 13, #18).
17. **Owner.** `player_id` and the holding zone both answer it. Revealing from the enemy deck panics (confirmed). Audits 1, 3 and 4, and audit 2 found the side rule copied (01a 14).
18. **Damage.** Fatigue marks damage by hand, so damage triggers miss it. All. New to the list, because 01a had called it cheap. It's already a card-game open thread.
19. **Interaction state.** `view` and `pending_*` decide each state with `_` arms. Audit 4 (01a 15, whose three `unreachable!` sites stayed unfound).
20. **Checkpoints** are set at ten sites, two of them inside `play`. Audits 1 and 3, low confidence (01a 16).
21. **The resolving cursor is `pub`.** The caller has to clone, then advance. Audit 3. New, and the roadmap's 06b frame stack will change it.
22. **The card format is public.** `CardDef`, the loader and its errors are re-exported, and no test uses them. Audit 3, low. New.
23. **The placeholder rule** is split between `definition` and `loader`, which import each other. Audit 1, low. New.

**Routed elsewhere, not structure findings:**

- ObjectIds name hidden cards (audit 3, live). The card-game open threads already hold this as a policy question.
- Board auras reach minions in every zone, the hand included (audit 2, confirmed at `lookup.rs:158`). A rules question for the card-game course.

**Names.**

- Failed by every auditor:
  - top level: `game`, `cards`, `choice`, `ids`, `turn`
  - under `cards/`: `definition`, and `modifier`, which isn't in the glossary
  - under `game/`: `condition`, `lookup`, `resolve`, `triggering`, `zone_move`
- Split verdicts:
  - `loader`: three of four failed it.
  - `filter`, `history` (also missing from the glossary) and `static_card_definition`: two each.
  - `effect`, `trigger` and `player`: one each.

Answer 1 to 3 in plain text, and approve or edit the list. Then I'll write it into the open threads, replacing the 01a list, and we close the session.


> [!quote] YOU

1: agree with a bullet point for this. 2: I guess the correction is to prefer formal knowledge over informal knowledge I think this is clear on the doc and we dont need to change it. 3 this will change from project to project and course.md is kinda a roadmap but also not really, it is meant to be the course structure we are using it as roadmap because we dont have one, I wouldnt add anythign about it either. overall good.


> [!quote] YOU

how can I start next session?


> [!abstract] PI

Now the close-out edits to `COURSE.md`.
