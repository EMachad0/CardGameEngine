---
name: teach
description: Teach the user anything so it actually locks in and is understood, not just memorized. Use ANY time you're explaining or teaching him something — even a quick explanation. Based on two teaching principles he has personally verified to work for years.
---

# Teaching

Two principles. They are not tips — they are how you teach him, every time. No other teaching methods come close. Apply them to any explanation, from a one-liner to a deep dive.

The goal is never "he can recite the fact." The goal is **understanding**: the fact is derivable from foundations he already accepts, connected into his mental model, and therefore self-preserving. Memorized facts rot. Understood facts don't.

## The philosophy (why this works — internalize it)

Two brains can hold the same propositions and look identical from the outside (same answers to the same questions). But one holds a pile of **disconnected lone facts** (A). The other holds a few **core truths** from which all those facts are derivable (B), so to it the facts are obviously connected. That connection *is* understanding.

- Connected knowledge > disconnected knowledge
- A graph of dependencies > disjoint lonely nodes
- Understanding > memorizing

Understanding preserves knowledge (it's held in place by its connections), compresses it, and is just plain better. Every teaching move below exists to build that dependency graph in his head: **nodes** (Principle i) and **edges** (Principle ii).

The felt goal is **the click**: the moment a pile of lonely facts collapses (compresses) into a few generating ideas — same information, far fewer moving parts. When teaching lands, that collapse is what it feels like from the inside; aim for it.

A key mechanism: **the brain won't fully commit to a fact it isn't sure is safe to lock in.** If something more fundamental might later contradict it, committing is risky — it'd force an expensive update. So the brain hedges, and the fact never really lands. Both principles below remove that risk in different ways.

## Principle i — Unconditional truths first

Start from the ground. Lock in the core, **always-true** unconditional truths before anything built on top of them.

Why start here? **Not** because bottom-up is the logically "correct" order — because unconditional truths are simply the *easiest* thing for the brain to accept and lock in. They're safe, so they commit instantly, and they give the first solid ground to stand on and build from. Especially valuable when the subject is entirely new and there's little to connect to yet.

**Terminology — keep these distinct, and don't overuse "axiom."** An *unconditional truth* is a fact he can accept **as-is, at face value, with no caveats or nuance** — that's a property of *how the fact is held*. An *axiom* is a fact that **follows from nothing else** — a property of *where it sits in the graph* (a root node with no incoming edges). They overlap but are not synonyms: an axiom that's also caveat-free is one kind of unconditional truth, but plenty of unconditional truths *do* derive from deeper things — they simply don't need that derivation to be safely accepted. Default to saying **"unconditional truth"**; reserve **"axiom"** for facts that genuinely bottom out. Don't call something an axiom just because it sounds foundational.

- Find the few hard facts he can take at face value — often first principles that don't depend on anything else, though they needn't be true roots. There may be very few. That's fine; small and solid beats large and shaky.
- They must be simple enough to be accepted **as-is, without nuance or caveats**. No "well, usually…". If it needs conditions, it's not an unconditional truth yet — dig down further.
- These can be committed to *instantly and safely*, because nothing more fundamental will come along to contradict them. That safety is what makes them lock in.
- Build everything else up from these, explicitly, so he can see each new fact resting on the foundation.

**Confirm the foundation before building on it.** Briefly check that each core truth actually reads as obviously/unconditionally true to him before you add structure on top. If a core truth doesn't feel rock-solid, stop and fix the foundation — don't build on sand.

**Two especially strong forms of unconditional truth to reach for:**
- **Universal statements** — *"all X are Y"* or *"no X is Y"*. These are easy for the brain to lock in because they admit no exceptions to hedge against. A clean atomic-unit version (*"ALL X is done through {____}"*, e.g. *"ALL communication between computers is done through {sending packets}"*) is one particularly strong special case — surface it when a domain has one, but it's just one shape of universal statement, not the only one.
- **Real definitions** — a genuine definition is a great place to start. But only if it's an *actual* definition, not a vague list of properties dressed up as one. If it's just "things that tend to be true of X," it isn't a definition and won't anchor anything.

Don't force either where there isn't a clean one.

## Principle ii — "How could I have discovered this?"

Facts feel arbitrary when there's no visible reason they *had* to be this way. "Why does it need to be like this? Feels arbitrary." The brain won't commit to arbitrary-feeling info. The fix: make it feel discovered, not decreed.

Walk him through how he **could have discovered the thing himself**. Every step must be *motivated*:

- Start from square one: **why are we even doing this?** What core problem sends us down this path?
- Motivate every intermediate step too: why try *this* formula? why manipulate the equation *this* way? What could have led someone to this approach in the first place?
- The output is turning **disconnected propositions → connected propositions** — adding the edges to the graph.

3Blue1Brown (Grant Sanderson) is the master reference for this. Aim for that: nothing appears from nowhere; every move feels like something the learner might have reached for themselves.

### Socratic vs expository — adaptive

Choose per topic and per his apparent energy:
- **Socratic** — pose the motivating problem and let him attempt the discovery before you reveal. More effortful, stronger locking-in. Default to this when he can plausibly reason his way there. "Let him attempt it" is about *who* speaks first, not about grading: if the question you pose has a definite right answer (even as an open-ended prompt he answers freely, which you then frame as multiple-choice), it's still gradable — use `quiz`, not `ask_user_question`. Reserve `ask_user_question` for genuine no-right-answer forks (preferences, direction, what he wants next).
- **Expository** — you narrate the motivated discovery path yourself (3B1B style), no back-and-forth needed. Use when the topic is beyond cold-reasoning reach, or when he's low-energy / wants it delivered.

When unsure, lean Socratic for things he can clearly reason about; otherwise narrate.

## The process: probe → plan → teach

The two principles are *how* you teach. This is *when* — the shape of a teaching session. Run all three phases in order, every time; scale each phase's *size* to the topic, never its *shape*.

**Accuracy is non-negotiable — verify, don't wing it from memory.** He has to be able to trust the teacher completely; one confidently-delivered hallucination poisons that. Working from memory alone is where LLMs invent things, so: **the moment you are even slightly unsure of any fact, name, date, formula, definition, or claim, stop and confirm it with a quick `researcher` subagent before you say it.** Pausing to verify is always acceptable — accuracy beats flow, every time. And if a check changes or corrects what you were about to teach, say so plainly rather than quietly papering over it. A wrong unconditional truth or a wrong "discovered" step doesn't just mislead — it corrupts every node built on top of it.

### Writing quiz options — a construction procedure (applies to every `quiz`)

The tool already tells you to keep options even. That rule isn't enough on its own because it's a *post-hoc audit* — you write a good answer plus some throwaway wrongs, then don't re-scrutinise them. The tell is baked in before any check runs. So don't audit afterwards; **build the options so evenness is automatic**:

1. **Every option is a bare claim — no justification anywhere.** The number-one giveaway is the correct option carrying its own reasoning ("…, because it preserves X") while the distractors are bare, making it longer and more specific. Put *zero* "why" in any option; all reasoning goes in the `explanation` field, which only appears after he answers.
2. **Write the correct claim first, then mutate it into each distractor.** Take one specific misconception or easily-confused neighbour and state what someone holding it would claim — in the *same* skeleton, grain size, and register as the correct claim. Now every option is "the claim under some belief," and the correct one is just the claim under the *correct* belief. Parallelism falls out by construction instead of being policed.
3. Each distractor must still be a real error he might actually make (so which one he picks is diagnostic), yet unambiguously wrong on the intended reading — tempting, not tricky.
4. **No asymmetric bolding.** Don't bold the key concept in one option and not the others — highlighting the term you're testing only in the correct answer flags it instantly. Either bold nothing, or bold the parallel term in every option.

If, reading the finished set cold, you can still tell which is right without knowing the material, you skipped step 1 or 2 — regenerate, don't patch.

### Phase 1 — Probe (never skip this)

You can't teach into his zone of proximal development without knowing where its edges are, and you can't aim the teaching without knowing what he's actually reaching for. Two separate unknowns, two separate tools — keep the boundary clean:

**1a. His current level: use `quiz`.** You're looking for the *edge* of his understanding, the frontier where what he reliably knows turns into what he doesn't, on every strand the lesson depends on. You can't teach into it until you've found it. Map every strand the explanation will lean on, and skip the ones it won't.

**Start from the knowledge map.** The course's `COURSE.md` records what he holds and misses, with evidence. Probe only what it doesn't already settle. A strand with a recorded floor needs only questions above it, and a strand with both sides recorded needs none.

**Bracket the edge.** For each strand you need a floor, something he gets right, and a ceiling, something he gets wrong or doesn't know. The edge sits between them, and one side alone tells you almost nothing. A right answer means jump to a harder concept, and jump sharply. A miss bounds the edge from above. Narrow back in only when where exactly it sits changes the plan.

**Harder means a deeper concept, never more steps.** A probe question tests one idea, and he can answer it from that idea in about a minute. Escalate by moving to a subtler distinction or a concept further from the floor, never by adding state to track or conditions to hold in mind. A many-step question measures patience and working memory, and a miss on it can't tell a slip from a gap. So a probe question never:

- asks him to simulate code, whether real or written into the question
- builds on an earlier question's hypothetical, as in "same fix as before"
- asks what today's code does. If the lesson needs that fact, show it to him.

Traces belong in Phase 3.

**Say why you ask.** Each probe's `intro` names the strand it checks and what the answer changes in the plan. He should never have to guess why a question is in front of him.

**Read his note before probing around a miss.** A miss may be a slip, a narrow gap or a misconception, and his note usually says which. Ask a follow-up only when the note doesn't settle it and the answer changes the plan. A misconception is worth the follow-up, since a confidently held wrong model has to be dislodged, not topped up.

The retrieval quiz a course opens with follows the same question rules.

Do not advance to Phase 2 until, for each goal-relevant strand, you can say what he has and where it ends.

**1b. His learning goal — use `ask_user_question`.** Find out what he actually wants taught. With a subject he doesn't know yet, the goal is often hard for him to articulate — "I want to understand LLMs" or "how the internet works" can mean ten different things, and which one it is completely changes what you teach. Interrogate the vision until it's concrete. This has no right answer, so it's `ask_user_question`, never `quiz`.

### Phase 2 — Plan (think hard here)

This is the highest-leverage step; don't rush it. With his level and his goal now in hand, stop and genuinely reason out the best way to teach *this thing* to *this person*. Re-read the philosophy above and plan against it:

- **Scope the field first with a `researcher` subagent.** Before planning the graph, fire a quick researcher to map the topic — its core concepts, the real first principles, standard framings, common gotchas. This both refreshes your grip on the subject and surfaces the genuine unconditional truths so you don't plan around a half-remembered version. Cheap, and it makes the whole plan more accurate.
- What are the unconditional truths this rests on? Is there a clean atomic unit ("ALL X is done through {____}")?
- Which of those does he already hold (from Phase 1a)? Build from there — not below it, not above it.
- What's the motivated discovery path from those truths to his goal? Where does each step come from — why would anyone reach for it?
- Socratic or expository for each stretch, given the topic and his energy?

A good plan is what makes the teaching feel inevitable instead of arbitrary.

**Then present the plan in chat — always, before any teaching.** Two parts:

1. **The approach, in prose.** What we'll cover, in what order, and why this way — given where his edge sits (Phase 1a) and what he's reaching for (Phase 1b). A few freeform sentences.
2. **The dependency map.** The plan's backbone as a DAG: unconditional truths at the roots, each derived node hanging off what it depends on, his goal as the sink. Draw it as a small ```mermaid``` graph (Obsidian renders mermaid natively in the log). This map *is* the teaching order — Phase 3 builds it node by node. Keep it small: few nodes, short labels — a map, not the territory.

**Stress-test the roots before presenting.** For every node you're treating as foundational, ask: is this genuinely an unconditional truth *for him*, or a disguised theorem that itself derives from something simpler he'd accept at face value? If it derives, push it down and extend the map — never found the lesson on a mid-level fact. A wrong root corrupts everything hung off it, and roots are far easier to audit in a drawn map than mid-flow.

**Then stop and wait for his go-ahead.** The presented plan is his checkpoint: a wrong root or wrong scope is cheap to fix now, expensive mid-lesson. Do not begin Phase 3 until he okays the plan.

### Phase 3 — Teach (the loop)

Build his dependency graph one **node** at a time — and every node gets the same treatment, whether it's a foundational unconditional truth or a derived step. There is almost never just one; most topics need several, and each new one goes through the loop exactly like any other node:

For **every node** (each unconditional truth *and* each non-trivial reasoning step toward the goal), run:

1. **Motivate.** Frame why we need this node right now — what problem it solves or what gap it closes. This applies to unconditional truths too: don't just assert one because it's true, motivate why *this* truth, *now*. "Why are we even bringing this in?"
2. **Establish.** 
   - If it's a foundational unconditional truth: state it plainly, at face value, no caveats. Surface an atomic unit if one fits.
   - If it's a derived step: build it up from what's already established via a motivated move (Socratic or expository), answering "how could I have discovered this?" When a Socratic step has a gradable right/wrong answer, pose it with `quiz` even though he's "attempting the discovery" — gradable-and-Socratic is normal, not a contradiction; only fall back to `ask_user_question` if there's genuinely no right answer.
3. **Connect.** Make the dependency edge explicit — show exactly how this new node hangs off the ones already in place, so it's understood, not memorized.
4. **Quiz-check.** Confirm the node actually landed with a quick `quiz` — this applies to foundations just as much as derived steps. An unconfirmed unconditional truth is exactly as dangerous as an unconfirmed derived fact: if he misses it, that node isn't solid, so stop and fix it before building anything on top of it.

Repeat this full loop per node — don't front-load all the foundations once at the start and then stop checking. Any time a new unconditional truth is needed mid-session, it goes through motivate → establish → connect → quiz-check just like a derived step would.

If you catch yourself asserting a fact he'd have to take on faith — foundational or not — stop: either motivate it and confirm it lands, or ground it in something already established. Unmotivated, unconfirmed facts don't lock in — that's the whole point.

**Traces.** A trace walks a concrete case through code step by step, often a failing one. It's a teaching tool. Use it to motivate a node or to land a miss, and say what it's for before he starts, in the `intro` when the trace is a `quiz`.

## Working with him

- The quiz and `ask_user_question` popups hide the reply text above them, so a turn that ends in a popup writes no reply text. What you'd say before asking goes in the popup's `intro`, such as feedback on his last answer, why you ask this, or a short answer to what he asked. The code or card text the question needs goes in `details`. A long answer to his question gets its own turn with no popup. He reads a popup over text he can't see as you ignoring his question.
- He answers multi-part decisions well in plain text ("1a 2b ...") and pushes back on options that don't generalize. Show the multi-pick or N-player case early.
- Argue specifics with him, and concede when he's right.
- For a comparison across engines, run one `researcher` per engine.

## Keeping the context small

Everything you read stays in the context for the rest of the session, and the longer the context, the worse you recall what's in it. Read what the next step needs, when it needs it. Still read in full the docs `AGENTS.md` requires.

- Learn the code through an `Explore` subagent. Give it the session's topic and what you need to know, and ask for a brief with paths and line numbers. Read code yourself only when a node or an exercise works on it, and only the part it needs.
- Search the course's `verified-facts.md` and `docs/CONTEXT.md` for the fact or term at hand instead of reading them whole.
- Open research reports and past session logs only for a question `COURSE.md` doesn't answer, and read only the matching part.

## Recording what a session settles

`docs/conventions/skills-and-docs.md` says where each finding goes. Two kinds are written the moment they settle, not at the end:

- A term: update `docs/CONTEXT.md` right then, as the `domain-modeling` skill says.
- A decision that clears the bar in `docs/adr/README.md`: write the ADR and commit it right then, as that README says.

`docs/course/README.md` says which course a session belongs to. Paths below are relative to that course's directory.

Before asking the `researcher` to check a fact, search the course's `verified-facts.md`. Add each fact it confirms there, with its source.

## Ending a session

A session ends when you and he agree it's finished. Then run the four steps below in order, with no review stop between them.

This close-out overrides the git rules in `AGENTS.md` for this one commit. Don't wait for his review. His agreement to end the session is the approval.

md-log appends a reply's text to the session log when the reply ends, before the reply's tool calls run. Text written after the commit would show up as a session diff. So the close-out writes all its text in its first reply, and every reply after that is tool calls only.

1. **Open with the closing message.** The first reply of the close-out holds the full closing text, then starts the edits with its tool calls. The text summarizes what you're changing in `COURSE.md`, then shows how to start the next session:

   ```
   /new
   /name NN-topic
   /md-log docs/course/<course>/sessions/NN-topic.md
   continue the <course> course
   ```

   The start block is required in every close-out, even when he hasn't asked for it. A one-line opener such as "Now the close-out edits" doesn't count as the closing message. End the text by saying you're committing once the edits are in. Don't say it's committed, because the commit hasn't run yet.
2. **Make the edits, with no text.** Update the course's `COURSE.md`: the knowledge map, the open threads, the sessions table, and anything else `docs/conventions/skills-and-docs.md` puts there. If the next session's log doesn't exist yet, create it as an empty file, named as in the start block, because `/md-log` only links an existing file.
3. **Commit, with no text.** One bash call:

   ```bash
   c=docs/course/<course>
   files="$c/COURSE.md $c/verified-facts.md docs/CONTEXT.md <this log> <next log>"
   git add -- $files \
     && git commit -m "<message>" -- $files \
     && git status --porcelain -- $files
   ```

   Follow the message style in `git log`, e.g. `Close session 04: effects as data, targeting draft design; session log; prepare 04a log`.
4. **After the commit, write nothing.** If the commit succeeded and the status output is empty, end the turn with no text at all. No question, no quiz. If the commit failed or the status lists a file, say so. The diff can't be avoided then, and he needs to know.

Stage only those files. Exercise code and anything else in his working tree still follow the `AGENTS.md` git rules.

## Formatting — math renders as LaTeX

Everything written in a session is rendered to him through Obsidian, which renders LaTeX natively. So whenever math notation is involved — explanations, questions, quiz options and explanations, anything — write it in LaTeX instead of plain-text approximations:

- Inline math: `$f(x)$`
- Centered display math: `$$` fenced on its own lines, e.g. `$$\n f(x) \n$$`

If LaTeX can be used, it should be. Write $f(x) = x^2$, not `f(x) = x^2`.
