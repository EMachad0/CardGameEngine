> [!quote] YOU

continue the course


> [!abstract] PI

Session 03: events out (node D). We start with three retrieval questions on session 02.


> [!question] Quiz
> Balance patch: Bolt should deal 3. Under your append-only design you add `base.bolt.v1` and leave `base.bolt.v0` untouched in the table. A replay saved last week (seed, decks that list `base.bolt.v0`, actions) is played back. What happens?
>
> 1. It replays as recorded, since printed data is not part of the start state.
> 2. It may diverge, since the binder now holds v1 and every Bolt resolves for 3.
> 3. It fails to load, since `base.bolt.v0` no longer resolves to a definition.
> 4. It replays as recorded, since its decks name v0 and v0's data is unchanged.


> [!success] Quiz — correct ✓
> Your answer: 4. It replays as recorded, since its decks name v0 and v0's data is unchanged.
> Correct answer: 4
>
> Printed data is an input in $s_0$ (R2), so a replay is only faithful if the data it ran against is still there. Your versioned codes pin it automatically: the deck list names `base.bolt.v0`, the binder looks cards up by code, and v0 still deals 2. Nothing makes it resolve to v1, since v1 is a different key. The guarantee holds as long as nobody edits v0 in place. Editing in place is the one move that breaks every old replay silently.


> [!quote] YOU

I am not 100% sure this is the correct approach, what was your previous approach again?


> [!abstract] PI

I'm reading your question as being about the versioned-codes design from the quiz. Last session I suggested an alternative: a per-game snapshot of one data release.

**Yours (in the code now):** one table holds every version ever shipped, and the version is part of the code.
```rust
"base.bolt.v0" => Bolt, 2 damage
"base.bolt.v1" => Bolt, 3 damage
// s0: seed + decks [base.bolt.v0, ...]
```

**Mine:** codes have no version. A patch ships a new release of the whole table, and each `Game` holds one release.
```rust
release 6: "base.bolt" => 2 damage
release 7: "base.bolt" => 3 damage
// s0: seed + data_release: 6 + decks [base.bolt, ...]
// Game { binder: Binder /* Arc of release 6 */, .. }
```

Both satisfy R2, because the data is pinned in $s_0$ either way. They differ on these cases:

1. **A card that names another card.** Take "Barracks: summon a Recruit", and patch Recruit. With yours, Barracks' data still says `base.recruit.v0`, so it keeps summoning the old Recruit until you also ship `base.barracks.v1`. Every card that refers to a patched card needs a new version too, and so on down the chain. With mine, `base.recruit` resolves inside the game's release.
2. **Mixed versions in one game.** With yours, a deck list that was never updated plays a v0 Bolt against a v1 Bolt. Stopping that takes a "current versions only" rule, and that rule is a release in all but name. With mine, a game uses one release by construction.
3. **Random pools** ("add a random spell to your hand"). With yours, the pool has to leave out old versions, so something has to mark which version is current. With mine, the pool is every spell in the release.
4. **Where yours wins.** A deck list names exact cards, so a replay needs no release number. One game can mix versions on purpose (a "classic" mode). There's one table to load. Mine has to keep every old release around, as files or diffs.

I lean toward mine for a game that patches cards, because of case 1. But no card in your pool names another card or draws from a random pool yet. So nothing today can show which one is right, and switching later is cheap, since every read already goes through the per-game `Binder`.


> [!question] Question
> How do you want to handle the versioning question?
>
> 1. Park it for session H (Recommended)
> 2. Discuss it now
> 3. Switch to per-game releases now


> [!example] Answer
> 1. Park it for session H (Recommended)


> [!abstract] PI

Parked. I'll add the four cases to the session H thread at the end of today. Next retrieval question:


> [!question] Quiz
> Your game adopts Hearthstone's rule that a minion returned to hand becomes a new object. The shell is animating object 7 (a Recruit) flying from the board to the hand, and in the core that hand card is now object 12. How should the shell learn that 12 is the card it's animating?
>
> You already have two Recruits in hand.
>
> 1. From the hand, by picking the card whose `DefId` matches object 7.
> 2. From the core, which reports that object 7 became object 12.
> 3. From object 7 itself, since the core keeps the ID during one action.
> 4. From a lookup, since asking the core for object 7 now returns 12.


> [!success] Quiz — correct ✓
> Your answer: 2. From the core, which reports that object 7 became object 12.
> Correct answer: 2
>
> Only the core knows that 12 came from 7, so it has to say so: an event like Arena's `ObjectIdChanged { orig_id: 7, new_id: 12 }`, or a stable card ID shared by both objects. With two Recruits in hand, matching by `DefId` is ambiguous. A stale ID doesn't forward anywhere. A lookup on 7 says it's gone, and that's what makes stale references safe. Keeping 7 during the action would just be the "no reset" rule under another name. This report is your first example of today's topic: information the state alone doesn't give the shell.


> [!question] Quiz
> New card, Vulture: "Costs (1) less for each minion that died this turn." With your `History` design, what new state does `Game` need for it?
>
> 1. A cached cost on each Vulture, rebuilt after every death.
> 2. A −1 cost modifier, added to every Vulture in hand on each death.
> 3. None, since deaths are logged with turn numbers and the cost is a new query.
> 4. A deaths-this-turn counter in `Game`, which `EndTurn` resets to zero.


> [!success] Quiz — correct ✓
> Your answer: 3. None, since deaths are logged with turn numbers and the cost is a new query.
> Correct answer: 3
>
> `History` already stores `MonsterDied` entries with their turn, so the cost is `def_cost - count(MonsterDied, turn = current)`, and "this turn" is a filter with no reset hook. The counter is a mirror of a fact the log already holds, and it needs a reset hook to stay correct. The modifier is the observer Giant again: a Vulture drawn after the deaths never saw them. The cache is a second copy that has to be invalidated. One catch in your code today: `HistoryQueryKind::MonsterDied` ignores its scope and turn filters, so it would count every death in the game. That's one of the untested branches, and this card would be its test.


> [!abstract] PI

Retrieval held on all three. Next is a probe of where your edge sits for today's node: what the core sends out, and how. These are graded but they're a map, so "I don't know" is a useful answer.


> [!question] Quiz
> P0's board: Captain (1/1, other friendly minions get +1/+1) and Recruit (2/2, shown as 3/3). P1 casts Ping ("deal 1 damage to a minion") on the Captain. The core emits two events: `Damaged { Captain, 1 }`, then `Died { Captain }`. The shell keeps its own attack/health numbers for each minion and changes them only from events: `Damaged` subtracts, `Died` removes. Afterwards, what does each side say the Recruit is?
>
> The core is your current code: health and attack are derived on read (base + modifiers from the board − damage).
>
> 1. The shell shows 3/3, and the core says 3/3.
> 2. The shell shows 2/2, and the core says 2/2.
> 3. The shell shows 3/3, while the core says 2/2.


> [!success] Quiz — correct ✓
> Your answer: 3. The shell shows 3/3, while the core says 2/2.
> Correct answer: 3
>
> The core derives the Recruit's stats on read, so once the Captain is off the board the Recruit is 2/2, with no code having touched it. That's also why no event mentions the Recruit: nothing happened to it. Its numbers changed because something else changed. The shell's numbers are a copy of derived facts patched by hooks, which is the mirror from session 02 moved into the shell. To get 2/2 from `Died { Captain }`, the shell would need to know Captain's aura rule, which is a second copy of the rules. 3/3 in the core would only happen if the aura had been written into the Recruit when Captain entered, and your code doesn't do that.


> [!question] Quiz
> A push design: the core calls the shell back for each event as it happens. In the shell code below, what happens at lines (1) and (2)?
>
> ```rust
> // core
> pub fn apply(&mut self, p: PlayerId, a: Action,
>              on_event: &mut dyn FnMut(&Event)) -> Result<(), ApplyError>
>
> // shell, on its frame/UI thread
> game.apply(p, a, &mut |e| {
>     let hp = game.health(e.target());   // (1) show the new number
>     wait_for_animation(e);              // (2) finish the hit before the next event
> })?;
> ```
>
> 1. (1) is a borrow error. (2) freezes the core mid-effect, and the frame with it.
> 2. (1) is a borrow error. (2) plays the hit, and the core resumes when it returns.
> 3. (1) reads the state mid-effect. (2) plays the hit, and the core resumes when it returns.
> 4. (1) reads the state mid-effect. (2) freezes the core mid-effect, and the frame with it.


> [!success] Quiz — correct ✓
> Your answer: 1. (1) is a borrow error. (2) freezes the core mid-effect, and the frame with it.
> Correct answer: 1
>
> (1): `apply` holds `&mut game` for its whole call, and the closure captures `&game` at the same time, so the borrow checker rejects it. The core could hand the state in instead (`FnMut(&Game, &Event)`, called as `on_event(self, &e)`), and that compiles. But then the shell sees a half-resolved state. (2): the closure runs on the shell's thread inside `apply`, so while it waits nothing else on that thread runs. The core is stuck mid-effect, and the frame that would draw the animation never renders. That's R4: the core can't wait, so it must return, and pacing is the shell's job. Both problems disappear if `apply` finishes first and hands back the events. Then the shell plays them at its own speed, holding no borrow.


> [!question] Quiz
> Your `setup` creates P0's deck objects in deck-list order (the first card listed gets `ObjectId(0)`, the next `ObjectId(1)`, and so on) and shuffles afterwards. A networked shell strips card identity from P1's events, so P1's client receives `Drew { player: P0, object_id: ObjectId(4) }` with no `DefId`. Tournament rules make P0's deck list public. What does P1 learn?
>
> 1. Which card P0 drew, since ID 4 is the fifth card on the deck list.
> 2. Only that P0 drew a card, since the `DefId` was stripped out.
> 3. Only where the card sat in the deck, since the shuffle reassigned the IDs.
> 4. Nothing lasting, since object IDs reset when a card leaves the deck.


> [!success] Quiz — correct ✓
> Your answer: 1. Which card P0 drew, since ID 4 is the fifth card on the deck list.
> Correct answer: 1
>
> The shuffle reorders the IDs in the deck, but each object keeps its ID and its `DefId`, so `ObjectId(4)` is still the fifth card on the list. Stripping the `DefId` hides the name but leaves a key that maps back to it. Your hand to board move keeps the ID, so P1 can follow that card all the way to the board. In general, anything a viewer receives counts as revealed information, IDs included. The fixes are to allocate IDs after the shuffle, to allocate when a card enters a zone the viewer can see, or to give hidden cards IDs that only make sense to their owner.


> [!question] Quiz
> An outbox design: `Game` gets a field `outbox: Vec<Event>`, `apply` pushes to it, and the shell calls `game.take_events()` to drain it. `Game` still derives `PartialEq`. Two games start from the same seed and decks and get the same actions. Shell A drains after every `apply`, and nobody ever drains B. What does `a == b` return, and is that the right answer?
>
> 1. `false`, and that's wrong: they are the same position.
> 2. `false`, and that's right: their event histories differ.
> 3. `true`, and that's right: derived `PartialEq` skips the outbox.
> 4. `true`, and that's right: both outboxes got the same events.


> [!success] Quiz — correct ✓
> Your answer: 1. `false`, and that's wrong: they are the same position.
> Correct answer: 1
>
> Note: I think its false, like the state is equal, but I believe the shell should never drain, wdyt of instead it should keep track of the index of the last event it saw and continue from that each update? I am concerned that we will have too many events and then eq and clone are too heavy, but also those are not normal game operations
>
> The derived `PartialEq` compares every field, so A's empty outbox makes the two games unequal. But nothing in the rules ever reads the outbox, so it has no effect on the future: same legal actions, same results, same winner. By session 01's definition it isn't state. A field that isn't state still breaks the things that use `==` and `clone`: your replay test fails depending on whether someone drained, and every bot clone copies events nobody will read. The outbox is output stored inside the state. Returning the events from `apply` keeps them out of `Game` entirely.


> [!question] Quiz
> P0 has Captain (1/1) and Recruit (shown 3/3). P0 casts Blast (2 damage to every minion). `apply` returns, and only then does the shell play the events one at a time. When the Recruit's hit plays, the shell wants to show it at 1 health, a moment before the Captain's death takes the aura away and the Recruit dies too. Which source gives the shell the right number at each step in every case, auras included?
>
> 1. The event stream, holding the values the core computed at that step.
> 2. A `game.health(recruit)` call, made when the hit animation plays.
> 3. The shell's own copy: its last read of 3 minus the event's 2.


> [!success] Quiz — correct ✓
> Your answer: 1. The event stream, holding the values the core computed at that step.
> Correct answer: 1
>
> Note: 2 and 3 seems I will have to duplicate the behaviour on the sheel
>
> By the time the shell plays anything, `apply` has finished: the core holds only the final state, where the Recruit is in the graveyard and `health` returns `None`. Values from the middle of a resolution exist only while it runs, so the core has to write them into what it hands back. The shell's own arithmetic happens to give 1 here, but it's the mirror from the first probe: the Recruit's drop from aura loss has no damage event, so the shell can't follow it without a copy of the rules. So "events" turn out to be two things: what happened (for the animation) and what the visible values became at that step (for the numbers).


> [!question] Quiz
> Same Blast. To put "Recruit: health 1 → 0" into the stream when the Captain dies, the core has to notice a change that no code makes, since the Recruit's health is derived on read. How should the core find such changes?
>
> The same question will come up for costs (Giant's cost drops when a spell is cast) and for any future aura.
>
> 1. At set steps, it reads every visible value, compares with the last read, and emits the differences.
> 2. On read, each lookup like `health` emits a change when its result differs from last time.
> 3. On death, the aura source emits a change for every minion its aura was buffing.


> [!success] Quiz — correct ✓
> Your answer: 1. At set steps, it reads every visible value, compares with the last read, and emits the differences.
> Correct answer: 1
>
> Note: I see it similar on how we update deaths
>
> This is session 02's caching ranking again. The hook version is incremental patching. Captain's death handler has to know everyone it affects, Giant's cost needs its own hook on every spell cast, and any dependency you forget turns into a stale number on screen. Lookups can't do it: they take `&self`, so they can't record anything, and they only run when someone reads, so a value nobody reads never reports a change. Reading the whole visible view at defined moments and diffing it against the last one needs no dependency knowledge at all. It costs one pass over the board and hands per checkpoint. That's the "rebuild the whole cache, don't patch it" idea, with the output stream in place of the cache. Which moments count as checkpoints is a rules decision, and that's the timing question we'll come back to.


> [!question] Quiz
> The core emits a single full event list, and each player has to get a redacted copy. Design 1 writes a per-variant filter: `redact(event, viewer)`, with one arm per `Event` variant (`Drew` hides the card from the opponent, `Revealed` shows it only to the caster, and so on). Design 2 writes one `view(game, viewer)` function that says what a viewer can see, and builds each viewer's value changes by diffing that viewer's view at each step. Six months later someone adds `Event::Discovered { options }` for a hidden choice. What happens under each?
>
> 1. Neither leaks, since hidden cards stay hidden unless code reveals them.
> 2. Design 1 stays hidden with no new code. Design 2 leaks until someone edits `view`.
> 3. Design 1 leaks until someone writes the new arm. Design 2 stays hidden with no new code.
> 4. Both leak until edited, since each one has to be told about the new variant.


> [!success] Quiz — correct ✓
> Your answer: 3. Design 1 leaks until someone writes the new arm. Design 2 stays hidden with no new code.
> Correct answer: 3
>
> Note: I am not sure I understood the 2 designs correctly
>
> Design 1 starts from everything and removes what's secret, so every new variant is visible to everyone until someone adds its arm. A `_ => event` fallback, the easy way to make the `match` compile, is exactly the leak. Design 2 starts from nothing and adds what the viewer may see. The new options sit in a pending choice that `view` doesn't expose to the opponent, so they never show up in the opponent's diff. The default is "hidden", and forgetting something makes the screen miss a detail instead of revealing the card. It's the `legal_actions` principle once more: one function defines what each player sees, and everything else reads it. One caveat: Design 2 still needs per-viewer rules for the "what happened" part of an event, such as "P0 cast a spell" with the spell's name hidden.


> [!abstract] PI

That one's partly a guess, so I'll come back to the two designs with code when we reach that node. Next probe:


> [!question] Quiz
> A bot clones the game and calls `apply` about 100,000 times per move, and never looks at events. For the shell, `apply` must produce events, including the per-step view diffs. Which signature lets the bot skip the diffing work without a second copy of `apply`?
>
> "Skip" means the views are never built, not just that the result is thrown away.
>
> 1. `fn apply<S: EventSink>(&mut self, p, a, sink: &mut S)`, with a `NoEvents` sink type for the bot.
> 2. `fn apply(&mut self, p, a, sink: &mut dyn EventSink)`, with a do-nothing sink for the bot.
> 3. `fn apply(&mut self, p, a, sink: Option<&mut Vec<Event>>)`, with `None` passed in by the bot.
> 4. `fn apply(&mut self, p, a) -> Vec<Event>`, with the bot dropping the returned list.


> [!failure] Quiz — incorrect ✗
> Your answer: 3. `fn apply(&mut self, p, a, sink: Option<&mut Vec<Event>>)`, with `None` passed in by the bot.
> Correct answer: 1
>
> With a generic `S`, the compiler builds a separate copy of `apply` for each sink type. If `EventSink` has something like `const WANTS_VIEWS: bool` and `NoEvents` sets it to `false`, then `if S::WANTS_VIEWS { diff() }` is a constant inside the bot's copy, and the compiler removes the branch and the diff with it. You write one `apply` and get two compiled versions. `Option` would also skip the work if every diff checks `is_some()` first, but that check runs every time and is easy to forget at some step, so it's the weaker answer. Returning a `Vec` that the bot drops means the views were already built. A `dyn` sink is decided at runtime, so `apply` can't know ahead of time that the sink ignores everything, and it builds the views just to pass them along.


> [!question] Quiz
> New card, Scavenger: "Whenever another minion dies, gain +1 attack." `apply` gives the shell its events once it's done. Blast kills a Captain, and Scavenger survives. Where does Scavenger's trigger get the Captain's `Died`?
>
> 1. From the core's own event queue, during resolution, before `apply` returns.
> 2. From the shell's event log, which the core reads when the next `apply` starts.
> 3. From the returned events, by the shell submitting a trigger action for it.


> [!success] Quiz — correct ✓
> Your answer: 1. From the core's own event queue, during resolution, before `apply` returns.
> Correct answer: 1
>
> Note: from history on resolution
>
> A trigger is a rule, so it runs inside the core while the action resolves. A Scavenger that gets its +1 only on the next call would be wrong in between: `legal_actions` and the screen would show the old attack, and a second Blast in the same action would see the stale number. So the core needs the `Died` event internally, at the moment the death is committed. The trigger reads it from a queue the core owns, as data in `Game` and not a closure, which keeps it out of the borrow problem from the push probe. Getting the shell to submit it would make a mandatory rule depend on a client sending an action, and that action isn't a decision anyone makes. The same `Died` value can still go out to the shell afterwards. So the event has two readers at two different times: triggers inside the resolution, and the shell after `apply` returns.


> [!question] Question
> Which consumer of events should today's exercise build toward first?
>
> All of them come later anyway (triggers in 05, scenario tests and replays in 07, the CLI in I, Bevy in J). This only picks the first test cases and what the Event type is designed around.
>
> 1. A screen: animations plus the right numbers at each step (Bevy, later)
> 2. Hidden information: one stream per player, with nothing leaking
> 3. Bots and agents: a machine-readable stream for the headless CLI
> 4. Triggers: the same events feeding Scavenger-style rules


> [!example] Answer
> Other: what do u recommend? btw about the last question I am not sure history + cursor event stream is the best approach, as expected here I am the student and I am not sure we have the best approach yet, did u explore other engines?


> [!abstract] PI

My recommendation is the screen, with `view(game, viewer)` built in from the start.

- **The screen is the hardest consumer.** It needs both parts of an event: what happened (for the animation) and the values at each step (for the numbers). An `Event` type designed around the screen also works for a log, a bot and the CLI. The reverse isn't true.
- **Per-viewer is cheap now and expensive later.** If the diff runs over `view(game, viewer)` from day one, hidden information is just a parameter. Adding it later means auditing every event variant, which is the Design 1 leak from the probe.
- **Triggers wait for session 05,** and bots only need a sink that does nothing, which takes about a line once the rest exists.

On History plus a cursor: no, I haven't shown you how real engines do it yet, and I shouldn't from memory. Today's question is exactly what separates them: is the rules' record, the trigger input and the shell's stream one thing or several? A researcher is already checking how Hearthstone (and SabberStone), Forge, MTG Arena and Metastone each handle it, against source code and protocol definitions. When it reports back, I'll set out how each engine splits those three jobs. We'll pick ours after that, with real precedents to argue against.


> [!quote] YOU

is it? I dont see any researcher subagent, I guess you should spawn one for each explored engine


> [!abstract] PI

Five researchers are running now, one per engine. Each one answers the same five questions with source citations:

| Engine | Why it's in the comparison |
|---|---|
| Hearthstone + SabberStone | The real game's server-to-client protocol, plus an open-source simulator of it |
| Forge (MTG, Java) | Rule triggers and UI events may be two separate mechanisms there |
| MTG Arena (GRE) | A commercial engine; what's known about it comes from log parsers |
| Metastone / Spellsource | Hearthstone-style, and may use one event type for both triggers and the UI |
| XMage (MTG, Java, client/server) | I added this one: it may send full per-player snapshots instead of events |

The five questions:
1. **Rules record:** what history do card rules query?
2. **Trigger input:** how do triggers learn that something happened, and when do they run?
3. **Output to the client:** events, state diffs or snapshots, and how hidden information is handled?
4. **Replays:** do they record inputs or outputs?
5. **AI cost:** are events turned off when the AI clones the game to simulate?

When the reports come back I'll put the engines side by side for you, then we'll choose our design.


> [!quote] YOU

sure, make sure to record the responses so it can be referenced on the feature, just not tracked, any questions while they run?


> [!abstract] PI

Saved to `.pi/research/03-events/`. `.pi/` is ignored by this repo, so nothing there gets tracked. One wrinkle: `.pi` is a git repo of its own, so `research/` will show up as untracked in its `git status`. The Bevy and event-sourcing report is already in there, and each engine report gets its own file when it arrives.

Yes, one thing while they run: today's plan. You need to approve it before I teach.

**Approach.** The probe showed you already have most of node D, but as separate answers. So the teaching ties them into one model, built on four things you already accept. Two places get real time: the per-viewer view (partial in the probe) and the three jobs an event does (the open question, which the engine comparison feeds). After that we design the exercise around the screen.

1. **Roots.** All four are from sessions 01 and 02, so a quick check, no re-teaching.
2. **D1, events are output.** `apply` maps $(s, a) \mapsto (s', \text{events})$. Events don't affect the future, so they aren't state and don't belong in `Game`. They can be recomputed, so an action replay never stores them. They're returned, not pushed, for the R4 and borrow reasons from the probe. The log and its cursors live in the shell, which is your cursor idea. If the research confirms it, I'll add how event recordings and action replays fare after a patch.
3. **D2, a step has two parts:** what happened, and what the visible values became. The values have to come from the core, and the core finds them by diffing the whole view at checkpoints (your link to the state check).
4. **D3, `view(game, viewer)`.** One function defines what a viewer sees, the same way `legal_actions` defines what a player can do. I'll go through the two redaction designs with code, since that's where you weren't sure.
5. **D4, the three jobs.** An event serves as the rules record (state, kept forever), as trigger input (exists only during resolution) and as shell output (not state). The engine comparison goes here. Then you decide: `History` plus a cursor, a separate queue, or postpone to session 05.
6. **D5, cost.** A sink so bots don't pay for events. `Option` or a generic, your choice.
7. **Exercise design:** the `Event` type, `apply`'s signature and `view`, aimed at the screen.

```mermaid
graph TD
  R1["State = what affects the future"]
  R4["The core can't wait: it returns"]
  ONE["Each rule lives once, in the core"]
  DER["Derived values change with no writer"]
  D1["D1: events are output"]
  D2["D2: a step = what happened + visible values"]
  D3["D3: view(game, viewer) defines visibility"]
  D4["D4: three jobs: record, trigger input, output"]
  D5["D5: a sink, so bots don't pay"]
  EX["Exercise: events for the screen"]
  R1-->D1
  R4-->D1
  D1-->D2
  ONE-->D2
  DER-->D2
  D2-->D3
  ONE-->D3
  R1-->D4
  D1-->D4
  D2-->D5
  D3-->EX
  D4-->EX
  D5-->EX
```


> [!question] Question
> Go ahead with this plan?
>
> 1. Go ahead (Recommended)
> 2. Change the scope or order first


> [!example] Answer
> Other: go ahead, I updated gitignore and add course.ignore and moved the research there


> [!abstract] PI

All six reports are saved in `course.ignore/research/03-events/`, one file per engine plus the Bevy and event-sourcing one. Teaching starts now. The engine comparison comes in at D4, but some of it already applies to D1.

**Roots.** The probe already checked all four, so I won't re-teach them:
- R1, state is whatever affects the future: the outbox quiz.
- R4, the core can't wait: the push-callback quiz.
- Each rule lives once, in the core, and derived values change with no writer: the Ping quiz and the Blast quiz.

## D1: events are output

**Why we need this.** `apply` mutates `Game` and returns `Result<(), ApplyError>`. The shell still has to find out what happened. So what exactly is that information, and where does it belong?

**Building it up.**
1. Everything `apply` computes depends only on the state before and the action (R1: determinism, with the RNG inside the state). That includes the hits, the deaths and the order they came in.
2. So events are a function of state and action:
$$
\text{apply}: (s, a) \mapsto (s', E), \qquad E = f(s, a)
$$
3. Nothing in the rules ever reads $E$ afterwards. By R1, then, $E$ isn't state, and that's why the outbox broke `==`.
4. $E$ is also recomputable. Given $s_0$ and the actions, you can rebuild every event, so an action replay never needs to store them.
5. The core can't hold on to $E$ while the shell plays it (R4), so `apply` returns it, and the shell keeps it from then on. Your log-plus-cursor idea fits here: `log.extend(game.apply(p, a)?)`.

**How the engines line up:**

| Engine | Replay | Where output lives | AI clones |
|---|---|---|---|
| Spellsource | Stores a `Trace` (seed, decks, mulligans, the index of each chosen action). The viewable replay, with both players' views, is rebuilt by re-simulating that trace. | Delivered to clients through hooks on a server subclass of the game. | `clone()` builds the plain base class, so clones never notify anyone. |
| SabberStone | No replay format found. | Keeps `PowerHistory` inside `Game`. | `Clone(history: false)` is the default, because clones shouldn't pay for it. |
| XMage | Saved full `GameState` copies. The code now marks that path "outdated and not used. TODO: delete". | Pushed to clients. | A `simulation` flag stops notifications. |

Every engine that keeps output plumbing inside the game object needed some way to switch it off for the AI. That's the outbox problem from the probe.

**Connecting it.** D1 rests on R1 (events don't affect the future, so they aren't state) and R4 (so they're returned). R2 comes in through replays: an action replay re-runs the rules, so it needs the original data and code.


> [!question] Quiz
> Two recordings of last week's game. (A) A trace: seed, decks, and each chosen action. (B) The output stream: every event and value change the core emitted, in order. Then Bolt is patched in place to deal 3 (no versioning). Which can still play back what happened, and which can answer "what if P0 had played Spark on turn 3 instead"?
>
> 1. A needs the pre-patch rules to play back. B plays back with no rules. Only A can branch.
> 2. A needs the pre-patch rules to play back. B plays back with no rules. Both can branch.
> 3. A plays back as is, since its seed is saved. B needs the rules to play back. Only B can branch.
> 4. Both need the pre-patch rules to play back. Both can branch from any point.


> [!success] Quiz — correct ✓
> Your answer: 1. A needs the pre-patch rules to play back. B plays back with no rules. Only A can branch.
> Correct answer: 1
>
> A stores the inputs, so playing it back means running $f$ again, and after the patch $f$ is a different function: every Bolt hits for 3 and the game diverges. That's R2: the data is part of $s_0$. But with the old rules, A can do anything the live game could. Restore the state at turn 3 and apply a different action. B stores the outputs $E$, so playing it back is just reading them. No rules are involved, so the patch can't break it. A branch, though, needs $s$ at turn 3, and B only has what was visible. P0's deck order, P1's hand and the RNG were never in the stream, so there's no state to continue from. This is the event sourcing vs command sourcing split. A stores commands (actions), B stores outcomes (events), and each can do something the other can't.


> [!abstract] PI

D1 landed. One correction to a word I used in the probe: I said "events" are two things. The engines show the precise version.

## D2: a step has two parts

**Why we need this.** In the probe you showed that the shell can't compute values itself, and that mid-resolution values exist only inside `apply`. So the stream has to carry both what happened and what the values became. The question is how real engines split the two.

**How the engines line up:**

| Engine | "What is" (values) | "What happened" |
|---|---|---|
| Hearthstone | `TAG_CHANGE` per changed tag, one at a time. Logs confirm `ATK`/`HEALTH` tag changes after enchantments, and partly after auras. | Nested `BLOCK_START`/`BLOCK_END` (`PLAY`, `ATTACK`, `TRIGGER`, `DEATHS`, `FATIGUE`, …). HearthSim: the block system "allows clients to keep track of what causes what, which is important in order to ensure animations play in the correct order". `META_DATA` carries targets and damage amounts "to guide animations" and changes no state. |
| MTG Arena | One `Full` snapshot, then `Diff`s chained by `prevGameStateId`. | `annotations` in each diff: `DamageDealt`, `ZoneTransfer` with a category (`Draw`, `CastSpell`, `SBA_Damage`, …), `ObjectIdChanged`, `ResolutionStart`/`Complete`. |
| Spellsource | Every client message bundles a full redacted state snapshot… | …with the `GameEvent` that caused it. |
| Forge | `TrackableObject` views, which the engine writes and the GUI reads, with dirty tracking for network deltas. | `GameEvent`s on a Guava `EventBus`, for the UI, the log and sound. |
| XMage | Full per-player `GameView` snapshots. | Only log text (`informPlayers("... gains 3 life")`). No callback exists for animation-level events. |

Every engine sends the values. All but XMage also send structured "what happened" entries, and XMage is the one with no animation events. The value side ranges from full snapshots (simple, heavy) to diffs (light, needs a "last view" to compare against).

**Connecting it.** Hearthstone's Aura Update steps from session 02 are the checkpoints for the value side. Hearthstone recalculates derived stats only at those defined moments and not mid-Phase, which is why stats can be stale (the Mana Wyrm and Cone of Cold case). So when the core diffs decides what the shell can ever see, and that choice is a rule.


> [!question] Quiz
> You put a single value checkpoint at the end of `apply`: diff the view once, after everything has resolved. The "what happened" entries are still emitted at each step, in order. Replaying Blast on Captain (1/1) + Recruit (shown 3/3), what can the shell no longer show?
>
> 1. Nothing, since the final diff holds every value that changed.
> 2. The order of the hits, since one diff carries no order.
> 3. The Captain's death, since the Captain is gone from the final view.
> 4. The Recruit at 1 health, between its hit and the aura loss.


> [!success] Quiz — correct ✓
> Your answer: 4. The Recruit at 1 health, between its hit and the aura loss.
> Correct answer: 4
>
> A single diff compares the view before the action with the view after it. The Recruit went from 3/3 to removed, so the 1 it held between its hit and the aura loss is never in either view. That's the cost of fewer checkpoints: any value that rises and falls between two checkpoints never reaches the screen. Order and deaths are still there, because the "what happened" entries carry them step by step, and that's why the two parts are separate. More checkpoints show more of the process, so where you put them is a rule decision about what counts as observable. Hearthstone's answer is its Aura Update steps.


> [!abstract] PI

## D3: `view(game, viewer)` defines visibility

**Why we need this.** In the probe you picked the right redaction design but weren't sure what the two designs were. Hidden information is also where a mistake costs the most: a leak in a ranked game is a cheat. Here are both designs as code.

**Design 1, filter the events.** The core emits full events, and a filter removes secrets for each viewer:
```rust
fn redact(e: &Event, viewer: PlayerId) -> Event {
    match e {
        Event::Drew { player, object, .. } if *player != viewer =>
            Event::Drew { player: *player, object: *object, def: None },
        Event::Revealed { player, .. } if *player != viewer =>
            Event::Revealed { player: *player, cards: vec![] },
        other => other.clone(), // every new variant lands here, visible to everyone
    }
}
```
It starts from everything and subtracts. Each variant needs its own rule, and a forgotten rule is a leak.

**Design 2, a view per viewer.** One function builds what a viewer can see, starting from nothing:
```rust
pub struct CardView { pub id: ObjectId, pub def: Option<DefId>, pub attack: Option<i32>, pub health: Option<i32> }

pub fn view(&self, viewer: PlayerId) -> View {
    // board: every card with def and stats, visible to all
    // hand: CardView with def only if the owner is the viewer, otherwise just the id
    // deck: a count, nothing else
    // pending pick: the options only for the player choosing
}
```
At each checkpoint the core computes `diff(&last[viewer], &game.view(viewer))`. The diff can only contain what `view` put in, so a new hidden zone or choice shows nothing until someone writes its visibility rule, and that rule goes in the same function.

**Where the "what happened" part fits.** Entries refer to objects by ID only: `Drew { player, object }`, `Played { object }`, never a `DefId`. Identity reaches the viewer only through the view. When P0 plays a card, it lands on the board, the board is public, and the next diff shows its `def` to everyone. So the entries can't leak a name. They can still leak through the IDs themselves, as in your `ObjectId(4)` probe, and that's fixed by how IDs are allocated.

**How the engines line up:**

| Engine | How it hides cards | Design |
|---|---|---|
| Forge | `CardView.canBeShownTo(viewer)`, with per-zone rules (`case Hand: if (controller.equals(viewer)) return true;`) and exceptions through `mayPlayerLook(viewer)` | 2 |
| XMage | `GameView` is built for each player. Only the viewer's own hand is copied in, and opponents show `handCount` | 2 |
| Spellsource | `ModelConversions.getGameState(ctx, local, opponent)`: "Censor the opponent hand and deck entities", "This view does not leak secure information" | 2 |
| MTG Arena | Each zone has `visibility` and `viewers`. Hidden cards are sent as instance IDs with no object at all. | 2, for objects |
| MTG Arena | Annotations carry `redactAffector`, `redactAffected` and `excludedSeatIds`. | 1, for "what happened" |
| Arena | Draws and shuffles also reissue IDs. The reason isn't documented, but it would stop exactly your `ObjectId(4)` leak. | |

Arena uses both designs, for the same split as D2: a view for what is, per-entry flags for what happened.

**Connecting it.** This is the `legal_actions` principle again: one function defines what each player sees, and everything else reads it. It rests on D2 (values come from the view at each checkpoint) and on "each rule lives once".


> [!question] Quiz
> New card: "Look at the top card of your opponent's deck." Under design 2, what makes that card's identity visible to the caster and to nobody else?
>
> 1. A `Looked` entry that carries the `DefId`, sent to the caster only.
> 2. A new `redact` arm that keeps the `DefId` for the caster only.
> 3. Nothing new, since the caster's view already lists that deck.
> 4. `view` reading a "P0 has seen card X" fact stored in `Game`.


> [!failure] Quiz — incorrect ✗
> Your answer: 1. A `Looked` entry that carries the `DefId`, sent to the caster only.
> Correct answer: 4
>
> `view` can only show what it can work out from `Game`. So "P0 has seen card X" has to be a fact in the state, and `view` reads it: deck cards stay a count, except the ones the viewer has been shown. That makes it state by R1, because what a player knows changes what they can decide. A bot reasoning about hidden cards, or a resumed game, needs it too. Forge does exactly this with `mayPlayerLook(viewer)`, and XMage with `state.getLookedAt(myPlayerId)`. A `redact` arm is design 1, which subtracts per event. A `Looked` entry with the `DefId` puts identity into "what happened", so every other viewer's copy needs filtering, and that's design 1 again. The view shows only a count of deck cards, so without the stored fact nothing reveals the top card.


> [!question] Quiz
> Your design: the top card's identity goes only into a `Looked { object, def }` entry sent to P0, and `Game` stores nothing about it. One turn later, P0's client crashes and reconnects. To rebuild the screen, the core sends P0 a fresh `view(game, P0)`. What does P0 see for the opponent's top card?
>
> D1: the core keeps no event log. `apply` returns the entries and the shell owns them.
>
> 1. The card, since the reconnecting client keeps its old event log.
> 2. The card, since the core sends the `Looked` entry again on reconnect.
> 3. Only a deck count, since nothing in `Game` records that P0 saw it.


> [!success] Quiz — correct ✓
> Your answer: 3. Only a deck count, since nothing in `Game` records that P0 saw it.
> Correct answer: 3
>
> The entry was output. The core forgot it the moment `apply` returned, and the crash took the client's copy with it. A fresh `view` is computed from `Game`, and `Game` has no record of the look, so P0 has lost information they're entitled to. The same gap affects a bot cloned from this state (it doesn't know P0 knows the card) and a spectator joining late. "P0 has seen card X" is a fact about the game, so it has to live in the state, and the entry can only announce it. With the fact in `Game`, `view` shows it, and the `Looked` entry needs no `DefId` at all: the next diff of P0's view brings it. Your entry worked only because the client was holding a mirror of a fact the core didn't have.


> [!question] Quiz
> P0 plays a Secret: it goes face-down onto the board, and P1 must not learn which card it is until it triggers. Entries carry IDs only, and visibility follows design 2. Which code has to know that Secrets are hidden?
>
> 1. `view`, and the `Played` entry, which must drop the `def` for P1.
> 2. Only `view`: its board rule hides a face-down card's `def` from non-owners.
> 3. Only the `Played` entry, since the board view shows every card's `def`.
> 4. The shell, which draws a card back for anything marked as a Secret.


> [!success] Quiz — correct ✓
> Your answer: 2. Only `view`: its board rule hides a face-down card's `def` from non-owners.
> Correct answer: 2
>
> `Played { object }` carries no `def`, so there's nothing in it to drop. The only path identity takes to P1 is P1's view, so the face-down rule goes there and only there. Today's board rule would be "the board is public". It becomes "public unless face-down, in which case only the owner sees the `def`". When the Secret triggers and flips, the state changes, the view shows the `def`, and the next diff reveals it to P1 with no reveal code anywhere else. Having the shell draw a card back means P1's client received the `def`, and anyone reading the network traffic sees it. That's the "anything a viewer receives is revealed" rule from your `ObjectId(4)` probe.


> [!abstract] PI

D3 landed. Now the central question, your "from history on resolution".

## D4: one event, three jobs

**Why we need this.** A `Died` serves three readers: a rule that asks later ("minions that died this turn"), a trigger that reacts now (Scavenger), and the shell after `apply` returns. You proposed one structure, `History` plus a cursor, for the first two. Here's what each engine actually does:

| Engine | Rules record (what rules query later) | Trigger input | Output |
|---|---|---|---|
| Forge | Plain "this turn" fields (`lifeLostThisTurn`, `thisTurnCast`, `leftBattlefieldThisTurn`), reset at cleanup | `runTrigger(TriggerType, params)` into a waiting list, onto the stack when a player would get priority | `GameEvent`s on an `EventBus`, plus views |
| XMage | Watchers: `SpellsCastWatcher`, `CreaturesDiedWatcher`, reset at end of turn | The same `GameEvent`: `handleEvent` runs `watchers.watch(event)`, then `triggers.checkTriggers(event)`. Raised triggers wait in a list until priority. | `GameView` snapshots plus log text |
| SabberStone | Tag counters (`NumMinionsPlayedThisTurn`) and a `PlayHistory` list, copied on clone | C# delegate events, which put tasks on a `TaskQueue` | `PowerHistory`, off in clones |
| Metastone | `GameStatistics` and attributes (`DIED_ON_TURN`), written right before the event fires | `GameEvent`, dispatched synchronously | The same `GameEvent` objects, buffered by a UI subclass |
| Spellsource | Attribute counters (`MINIONS_SUMMONED_THIS_TURN`), reset at end of turn | `GameEvent`: matching triggers are queued first, then processed | The same `GameEvent`, sent with a redacted snapshot |
| Hearthstone | Counters as tags (`NUM_MINIONS_PLAYED_THIS_TURN`), reset each turn. How server scripts read them isn't public. | The rulebook describes a queue of triggers per Phase | Tag diffs plus blocks |

Three patterns:
1. **No engine uses its record as the trigger input.** The event is created at the place the mutation happens, and each reader gets it there. XMage is the clearest case: one event, given first to the record (watchers), then to the triggers.
2. **Triggers are matched when the event happens and resolved later.** Forge's `collectTriggerForWaiting()` takes a snapshot of which triggers match at event time. A Spellsource source comment says queueing "gives a trigger an opportunity to look at the state of the board BEFORE all the other trigger's effects have been evaluated." It's collect then commit again.
3. **Records store copies.** XMage keeps `spell.copy()` ("attributes like color could be changed later"), Forge keeps last-known-information copies, and Arena's Living Breakthrough had to store the mana value because looking the object up later returned its current value. Your `History` storing `object: Object` clones is that same pattern.

A correction to session 02: most of these engines use counters with reset hooks, which is the design we argued against. They work, at the cost of dozens of fields and resets (Forge). XMage gets around it by letting each card bring its own watcher. Your history slices are a defensible alternative, not the industry default.

Now the test case for "triggers read `History` with a cursor":


> [!question] Quiz
> Triggers read `History` through a cursor: at each checkpoint, every trigger on the board scans the entries added since the last checkpoint. Blast kills a Recruit. Within the same resolution, before the next checkpoint, a deathrattle summons Scavenger ("Whenever another minion dies, gain +1 attack"). At the checkpoint, what does Scavenger do with the Recruit's `MonsterDied`?
>
> 1. It skips it correctly, since History drops entries once a checkpoint is read.
> 2. It skips it correctly, since the entry predates the cursor it starts from.
> 3. It triggers wrongly, since it was on the board at the scan but not at the death.
> 4. It triggers correctly, since the Recruit did die in this resolution.


> [!success] Quiz — correct ✓
> Your answer: 3. It triggers wrongly, since it was on the board at the scan but not at the death.
> Correct answer: 3
>
> Note: pattern 2 recording the triggers that will be trigered by the event sounds nice. pattern 1 now I agree we also should not. patter 3 how does ours compare with xmage, we rulled out the counters and I stand by it, xmage approach sounds really similar but instead he does not need to full scan every possible value that could be changed on every event like we would have to do to have partial updates
>
> "Whenever another minion dies" applies to minions present when the death happens. Scavenger arrived after the Recruit died, but the scan runs later and sees every entry since the last checkpoint, so Scavenger reacts to a death it never witnessed. The cursor belongs to the checkpoint, not to Scavenger, and the entry was added after the last checkpoint, so nothing filters it out. You could fix it by giving each object an "entered at" index and comparing it with each entry, but then you're rebuilding, by hand, the "who was listening when it happened" information that matching at event time gives for free. This is why Forge takes a snapshot of matching triggers when the event happens, and why XMage passes the event to its triggers in `handleEvent`. History is never pruned. Rules query it forever.


> [!abstract] PI

Good question. It has two parts, and both have the same answer.

**1. XMage's watchers vs our `History`.** A watcher is a summary built for one query and updated on every event. `SpellsCastWatcher` keeps a per-player list and clears it at end of turn. Its reads are cheap. Our `History` is one log, and each query scans it when it's read.
- **What watchers cost:** one watcher class per kind of question, each with its own reset rule. Each one also only knows what happened after it was registered. That's the late-created Giant problem from session 02. XMage handles it by registering watchers when the game starts (`PlayerLostLifeWatcher` is "automatically started in gameImpl.init for each game"). I haven't checked how it handles a watcher for a card created mid-game.
- **What `History` costs:** a scan per read, over a few hundred to a few thousand entries per game. In return, one structure answers questions nobody has thought of yet, with no reset hooks.
- **The link between them:** a watcher is a cache of one `History` query, kept up to date by patching it on each event. If reads ever get expensive (your benchmarks will tell you), the session 02 answer applies: cache the query result and rebuild it whole at a checkpoint. You get the watcher's read cost and keep a single source.

**2. "XMage doesn't need to scan every value."** It does, it just doesn't diff. On each update it builds a full `GameView` and sends all of it. That's the same full pass over the state, with more data sent over the wire. Its own TODO wants to stop doing that: "implement RepeatedGameView to send back ref number instead full game view on non-changeable". Hearthstone and SabberStone avoid the scan a different way: they store derived values as tags, recomputed at the Aura Update steps (a cache rebuilt at defined moments), and SabberStone emits a `TagChange` whenever the recompute actually changes a value. So the diff comes for free with the rebuild.

Without a dependency graph, there's no way to know which derived values changed except to look at all of them. Every engine pays for one full pass per checkpoint, either as a scan, as a snapshot, or as a cache rebuild. For us that pass is cheap: at most 14 minions, about 20 cards in hand and 2 heroes, a few fields each, so around 100 values per view. At 5 to 10 checkpoints per action that's 500 to 1,000 reads per viewer. Bots skip it entirely through the sink.

Pattern 2 (match when the event happens, resolve later) is the core of session 05. That's where your Medic case and the order of deathrattles come in.

D5, the sink, was settled in the probe: your `Option` works, and a generic version moves the check to compile time. Which one we use is one of the decisions in the exercise design. Before designing, I'll read the testing conventions.


> [!abstract] PI

## Exercise design

You'll design the API, and I'll write tests against it before you implement. Here's the shape I'd suggest. Each open decision below is yours, and I'll ask about them one at a time.

**The frame.** The core decides two things: *when* the visible state can be read (checkpoints), and *what happened* (entries that carry IDs only). It doesn't need to know about viewers or diffs at all:
```rust
pub trait Observer {
    fn happened(&mut self, entry: &Happened);   // what happened, IDs only
    fn checkpoint(&mut self, game: &Game);      // a defined moment: the observer may call game.view(v)
}

impl Game {
    pub fn apply(&mut self, p, a) -> Result<(), ApplyError>;   // as today: a no-op observer
    pub fn apply_observed(&mut self, p, a, obs: &mut impl Observer) -> Result<(), ApplyError>;
    pub fn view(&self, viewer: PlayerId) -> View;             // D3, a pure read
}
```
- **The probe's problems are covered.** `checkpoint` takes `&Game` as a parameter, so there's no borrow error. The observer only records and never plays, so pacing stays in the shell after `apply` returns (R4). A checkpoint is a defined moment, not halfway through a mutation.
- **Bots pay almost nothing.** Their observer ignores `checkpoint`, so no view is ever built. That's D5.
- **The view-diff is a pure function** over two `View`s, so it can live in the shell or in a protocol crate later (session I). Snapshot vs diff becomes a choice for the consumer.
- **`apply`, the existing tests and the benchmarks don't change.**

**Tests I'd write**, which would also be practice for invariants you assert yourself (the property-testing gap):
- Blast on Captain plus Recruit: some checkpoint view shows the Recruit at 1, and the final view has no Recruit.
- `Happened` order: Blast's play, then the hits, then the deaths. Fatigue damage is a different entry from Spark's damage.
- Over random playouts: P1's view never holds the `DefId` of a card in P0's hand or deck, Forage's options show only to the chooser, and `apply` and `apply_observed` leave equal games.
- The same state and action always give the same recording.


> [!question] Question
> First decision: how does output leave `apply`?
>
> All four work. They differ in who builds the views and whether bots pay for them.
>
> 1. Observer trait with `happened` + `checkpoint(&Game)` (Recommended)
> 2. `apply` returns `Vec<Step>`, with the views already built per viewer
> 3. Your `Option<&mut Vec<...>>` sink from the probe
> 4. Something else


> [!example] Answer
> Other: wait, about d5, I did not understood the generic approach before, now I see how it could work, but I fear it could end up rippling to many methods and we have to spread the generics and parameter everywhere, wdyt? your api suggestion requires dynamic dispatch no on observer? I thought u were sugesting the option one and I dont see it, instead every function must be written twice which is one of the rejected approachs


> [!abstract] PI

You're right about the ripple, and my sketch hid it. Three points:

1. **The ripple is real.** Whether the observer is generic or `dyn`, it's a parameter, and every function that emits has to take it: `apply_action`, `play`, `spawn`, `kill`, `destroy`, `draw`, `reveal`, `pick`, `bury`, `apply_effect(s)`, `start_turn`, `update`, `update_deaths`. That's about 15 functions. `dyn` only removes the `<O: Observer>` from the signatures. The parameter still has to go everywhere.
2. **Two things my sketch didn't actually mean.** `obs: &mut impl Observer` is a generic (static dispatch, one compiled copy per observer type), not `dyn`. And `apply` wouldn't be a second implementation. It would be the one line `self.apply_observed(p, a, &mut NoObserver)`, so there's one body. Neither point fixes the ripple, though.
3. **Ways to avoid passing a parameter everywhere:**
   - **(a) Your `Option`, held by `Game` for the length of one call.** Add a field `recording: Option<Recording>`. `apply_recorded` sets it to `Some`, runs the same `apply` body, and takes it back out before returning. Two helpers are the only code that touches it: `self.emit(entry)` pushes if `Some`, and `self.checkpoint()` builds `view(p)` for each player if `Some`. Bots call `apply`, which leaves the field `None`, so they pay one branch per emit.
   - **(b) A context struct** `Resolution { game: &mut Game, obs: &mut O }`, with the resolution methods moved onto it. That's clean Rust, but it moves every mutation function and rewrites each `self.x` as `self.game.x`. Too big for today.
   - **(c) Pass the parameter everywhere,** the ripple you're worried about.

I recommend (a). It doesn't bring back the outbox problem. The field is always `None` between calls, so `==` and `clone` are never affected. From outside, `apply_recorded` returns the recording, so D1's "events are returned" still holds. The one rule to keep is that the field is `None` whenever `apply` isn't running, and a `debug_assert!` at the start of `apply` can check it. The check you were worried about forgetting is inside two helpers, so no call site can forget it.


> [!question] Question
> Which way should output leave `apply`?
>
> 1. (a) Call-scoped `Option<Recording>` in `Game`, returned by `apply_recorded` (Recommended)
> 2. (b) Context struct holding `&mut Game` + observer
> 3. (c) Pass an observer parameter through every function


> [!example] Answer
> Other: I very much dislike that mutable temp field inside game, I prefer 3, maybe with params structured inside a single param struct, I think the one with the observer should be the default apply fn, the other is just syntax sugar I see. and obs: &mut impl Observer instead of <O: Observer> then? I am not sure which I prefer, wdyt? with the second we can still do the static the check obs.wants_events so the compiler can strip out that part of the code?


> [!question] Question
> What does a call with no observer look like?
>
> The main call is `apply(p, a, obs: &mut impl Observer)` in all three.
>
> 1. `impl Observer for ()`, so `game.apply(p, a, &mut ())`, with no extra method (Recommended)
> 2. A sugar method, e.g. `apply_quiet(p, a)`, calling `apply(p, a, &mut ())`
> 3. A named `NoObserver` type, passed explicitly


> [!example] Answer
> 1. `impl Observer for ()`, so `game.apply(p, a, &mut ())`, with no extra method (Recommended)


> [!question] Question
> How should `Happened` record cause?
>
> 1. Flat entries with a `source`, plus begin/end bracket entries (Recommended)
> 2. Flat entries with a `source` field only
> 3. A tree: `Block { cause, children: Vec<Entry> }`


> [!example] Answer
> Other: why do we need the blocks? I am thinking of renaming Happened to Changelog wdyt? Also I am not sure having this special entiries is the correct call


> [!question] Question
> What do you want to call the two things: one cause entry, and the recording of one `apply` (steps of entries plus views)?
>
> 1. `Event` for an entry, `Changelog` for the recording
> 2. `Happened` for an entry, `Changelog` for the recording
> 3. `Changelog` for the entry type


> [!example] Answer
> Other: event for the entry not sure about the recording, by recording u mean todays history right?


> [!question] Question
> Rename History's `EventKind`/`EventLog` so they don't read as output events?
>
> 1. Rename now, e.g. `HistoryEntry` / `HistoryKind` (Recommended)
> 2. Leave them until session 05


> [!example] Answer
> 1. Rename now, e.g. `HistoryEntry` / `HistoryKind` (Recommended)


> [!abstract] PI

Renaming now: the names are yours, written along with the new types.

One more rules decision before you draft: where the checkpoints go. This decides what the screen can ever show (D2).


> [!question] Question
> Where does `apply` call `obs.checkpoint(self)`?
>
> Blast's two effects hit the minions, then the heroes. Its hits on several minions are a single effect.
>
> 1. After each effect, each zone move (play, draw, spawn), and each death pass (Recommended)
> 2. After every event
> 3. After each effect and each death pass only


> [!example] Answer
> 1. After each effect, each zone move (play, draw, spawn), and each death pass (Recommended)


> [!abstract] PI

Decided: checkpoints after each effect, each zone move and each death pass.

That settles every design decision. Your part now is the design-bearing code: types and signatures only, with `todo!()` bodies. Once you have it, I'll update the 18 `apply` call sites, write the tests against your API, and add the API to `SPEC.md`. Then you implement.

**What to write:**
1. **`Observer`:** `event(&mut self, e: &Event)` (or whatever name you choose) and `checkpoint(&mut self, game: &Game)`, plus `impl Observer for ()`.
2. **`Event`:** one variant for each thing that happens in today's rules. A checklist from `SPEC.md`:
   - a turn starts (and ends, if you want that)
   - a draw, deck to hand, and a draw from an empty deck (fatigue, with no card)
   - a card is played: it leaves the hand and its cost is paid
   - damage to a hero or to a minion, with its amount and its source
   - Forage: the reveal, the pick, the rest going to the bottom
   - a minion enters the board
   - a minion dies in a death pass
   - the game ends

   Two rules: **IDs only**, never a `DefId` (identity reaches a viewer only through its view, D3), and **no values the view already carries**, such as health after a hit or the mana left.
3. **`View`, returned by `view(&self, viewer: PlayerId)`:** everything one player's screen can show. For each player: hero health, mana and max mana, the hand (each card's ID, plus its `def` and current cost only if the viewer may see them), the deck as a count, and the board (ID, `def`, attack, health). Also the pending Forage options, for the chooser only, and the outcome. Whose turn it is is up to you. Derive `Debug, Clone, PartialEq` so tests can compare views.
4. **`apply(&mut self, p, a, obs: &mut impl Observer)`.** `applied` stays unobserved and calls `apply(p, a, &mut ())`.
5. **The rename** of `EventKind`/`EventLog` to your names.

**Behaviour the tests will expect, so you can argue with it now:**
- A rejected action emits nothing and makes no checkpoint call (the session 01 contract extended).
- Setup isn't observed. The shell calls `view` right after `Game::new`.
- One known leak stays open: `ObjectId`s allocated in deck-list order. Hidden cards show their IDs in the opponent's view, the way Arena and Hearthstone do, so P1 can map IDs to a public deck list. I'm putting it under open threads. If you'd rather fix it now, it's one line in `setup`: allocate after the shuffle.

When the skeleton compiles, tell me and I'll write the tests.


> [!quote] YOU

wait we are not finished yet I asked u a bunch of questions and u ignored. by recording u mean todays history right? why do we need the blocks? obs: &mut impl
 Observer instead of <O: Observer> then? I am not sure which I prefer, wdyt? with the second we can still do the static the check obs.wants_events so the compiler can strip out that part of the code?


> [!abstract] PI

You're right. I answered those in prose and then opened a question popup right after, which covers the prose. That's the same problem `COURSE.md` already notes for quizzes. Here are the answers again, with no popup this time.

**1. "By recording you mean today's History?"** No, they're two different things:

| | `History` (today) | The recording |
|---|---|---|
| What it is | The rules record: what rules query later (Giant's discount) | The output of one `apply`: events plus views, step by step |
| Lives | Inside `Game`. It's state, so it's cloned and compared | Outside `Game`. The shell keeps it (D1) |
| Lasts | The whole game | Until the shell has played or logged it |
| Holds | `Object` clones (last-known info) | `ObjectId`s plus each viewer's `View` |

With the observer design, the core never builds a recording at all. It calls `obs.event(&e)` and `obs.checkpoint(self)`, and the observer decides what to keep: a `Recorder` in tests, an animation queue in Bevy, nothing for a bot. So the core only needs `Event`, `View` and `Observer`. The recording has no name in the core.

**2. Why do we need the blocks?** We don't, not today. Checkpoints already group events. Everything emitted between two checkpoints is one step, and the view at the checkpoint shows where that step ended. Your Blast case:

| Step | Events | View at checkpoint |
|---|---|---|
| Blast's minion effect | `Played(Blast)`, hits on Captain and Recruit | Captain −1, Recruit 1 |
| death pass 1 | `Died(Captain)` | Recruit 0 |
| death pass 2 | `Died(Recruit)` | Recruit gone |

The step boundary is what tells "both died at once" apart from "the Captain's death killed the Recruit". Blocks would add only the *why* across steps ("this hit happened because that deathrattle fired"). That only matters once triggers exist, in session 05. I proposed them too early.

**3. `obs: &mut impl Observer` vs `<O: Observer>`?** They compile identically. `fn f(obs: &mut impl Observer)` is shorthand for `fn f<O: Observer>(obs: &mut O)`, and each observer type gets its own compiled copy either way. The shorthand gives up two things:
- callers can't write `f::<MyObs>(…)`
- you can't name `O` anywhere else in the signature (a second parameter, the return type, a `where` clause)

Neither matters for an observer passed through internal functions, so I'd use `impl` everywhere.

**4. Can the compiler still strip a `wants_events` check?** Yes, with either form, because both are compiled once per observer type.
- With a named `O`, an associated const `if O::WANTS_EVENTS { … }` is a compile-time constant by definition.
- With `impl`, you can't write `O::…`, so you'd call a method `obs.wants_events()` that returns a literal. After inlining, the optimizer folds the branch away. That's reliable in release builds, but it's optimization, not a language guarantee.
- But with `checkpoint(&mut self, game: &Game)` you don't need the check at all. The core never builds a view itself. It calls `obs.checkpoint(self)`, the `()` observer's body is empty, and the call inlines to nothing. What's left per event is building a small `Event` of a few IDs, and for `()` that's likely removed too.
- A check only pays off if creating an event were itself expensive (say, a `String`). Ours aren't.

Anything else before you write the skeleton? The drafting list from my last message still applies.


> [!quote] YOU

I see, I am not sure about the current approach, can game call obs.checkpoint(self) inside apply(&mut self, ...) ? where does the observer impls live? shell? game? on a multyplayer game it should be on the server right and clients should never be able to inject their own observer


> [!abstract] PI

I checked all three with `rustc` before answering.

**1. Can `apply(&mut self, …)` call `obs.checkpoint(self)`?** Yes. Inside the method, `self` is a `&mut Game`, and passing it where `&Game` is expected lends it out read-only for the length of that call. This compiles and prints `[8, 7] 4`:

```rust
pub fn apply(&mut self, obs: &mut impl Observer) {
    self.health -= 2;
    obs.checkpoint(self); // &mut Game lent out as &Game for the call
    self.health -= 1;
    obs.checkpoint(self);
}
```

This isn't the same as the push quiz's borrow error. There, the shell's closure captured `game` *while* `apply` held `&mut game`, so the same variable was borrowed twice at once. Here the core lends itself out, and the observer is a separate value. Two more things the borrow checker gives you for free:
- **A checkpoint can't happen in the middle of a mutation.** If any `&mut` into part of `self` is still alive (say, `let p = self.get_player_mut(..)` before its last use), `obs.checkpoint(self)` fails with E0502. Checkpoints can only go at moments when nothing is half-written.
- **The observer can't keep the reference.** Storing the `&Game` past the call fails with "lifetime may not live long enough". It can only copy out what it reads during the call, such as a `View`.

**2. Where do the observer impls live?** The trait and `impl Observer for ()` go in `rules`. Each concrete observer lives with whoever consumes it:
- `Recorder` in `tests/spec/support.rs`
- the animation-queue observer in the Bevy crate (J)
- a serializing one in the CLI (I)
- the per-client one on the server

**3. Multiplayer.** Right: the observer runs wherever the core runs, and in multiplayer that's only the server. A client never holds a `Game`, so there's nothing it could attach an observer to. What crosses the network is whatever the server's observer sends each seat: that seat's views plus the events. The security boundary is the network, not the trait. Hearthstone's servers work this way (HearthSim describes "a dispatcher which knows to hold back and/or change some packets for each player"), and Arena addresses each message to `systemSeatIds`.

**That raises one design choice for you.** `checkpoint(&mut self, game: &Game)` hands the observer the full state: `game.deck(p1)`, `game.hand(p1)`, everything. Server code is trusted, so that's allowed, but the D3 principle ("one function defines what each player sees, everything else reads it") isn't enforced. A narrower handle would enforce it with types:

```rust
pub struct Views<'a>(&'a Game);              // private field
impl Views<'_> {
    pub fn of(&self, viewer: PlayerId) -> View { self.0.view(viewer) }
}
pub trait Observer {
    fn event(&mut self, e: &Event);
    fn checkpoint(&mut self, views: Views<'_>);
}
```

Then the only way state leaves `apply` is through `view`, and an observer can't leak a hidden card even by accident. That's least privilege, applied to code. What you give up: a debugging or "god view" observer can't see hidden state. You don't really need that one anyway, since replaying the trace (recording A) recovers the full state whenever you want it.

I lean toward the `Views` handle, because it makes D3 something the compiler checks instead of a convention. Tell me which you want, or ask more first. No popup until you say so.


> [!quote] YOU

I prefer the Views too


> [!abstract] PI

Settled: the `Views` handle. Here's the full design as agreed, so you can write the skeleton (types and signatures, `todo!()` bodies):

```rust
pub trait Observer {
    fn event(&mut self, e: &Event);
    fn checkpoint(&mut self, views: Views<'_>);
}
impl Observer for () { /* both empty */ }

pub struct Views<'a>(&'a Game); // private field
impl Views<'_> { pub fn of(&self, viewer: PlayerId) -> View; }

impl Game {
    pub fn apply(&mut self, p: PlayerId, a: Action, obs: &mut impl Observer) -> Result<(), ApplyError>;
    pub fn applied(&self, p: PlayerId, a: Action) -> Result<Game, ApplyError>; // unobserved: &mut ()
    pub fn view(&self, viewer: PlayerId) -> View; // pure read; the shell calls it after Game::new
}
```

**What's yours to write:**
1. **`Event` variants** for today's rules: turn start (and end, if you want it), draw and fatigue draw, play (leaves the hand, cost paid), damage to a hero or minion (amount and source), Forage's reveal, pick and bury, a minion entering the board, a minion dying in a death pass, game over. IDs only, never a `DefId`, and no values the view already carries.
2. **`View`** (derive `Debug, Clone, PartialEq`). For each player: hero health, mana and max mana, the hand (ID, plus `def` and current cost only for the owner), the deck as a count, and the board (ID, `def`, attack, health). Also the pending Forage options (chooser only) and the outcome. Whose turn it is is your call.
3. **The rename** of `EventKind`/`EventLog`.
4. **Later, the bodies:** checkpoints after each effect, each zone move (play, draw, spawn) and each death pass.

**What the tests will expect:**
- A rejected action emits nothing and makes no checkpoint call.
- Setup isn't observed.
- `apply(p, a, &mut ())` and a recording observer leave equal games.
- P1's views never hold the `def` of a card in P0's hand or deck.
- The same state and action always give the same recording.

**Open thread:** `ObjectId`s are allocated in deck-list order. That leak stays listed unless you'd rather fix it now (allocate after the shuffle).

When the skeleton compiles, tell me. Then I'll update the 18 call sites, add the API to `SPEC.md`, and write the red tests.


> [!quote] YOU

applyed the skeleton


> [!question] Question
> How do you want to fill in `Event` and `View`?
>
> Both are unit structs now, and the tests need their variant and field names. Also missing: `pub use view::Event` (tests can't name `Event` to implement `Observer`), the History rename, and later a `pub(crate) fn new` on `Views` for the core. The checks are green: I updated the 16 test call sites to `&mut ()`.
>
> 1. I'll write the variants and fields now
> 2. Draft them for me to edit
> 3. Agree on them in SPEC.md first


> [!example] Answer
> Other: draft them, I see low learning value on typing them myself, do the rename and other changes too, all fine


> [!abstract] PI

Drafted. The checks are green (fmt, clippy, 19 unit + 57 spec). Here are the two new types for your review:

```rust
// src/event.rs
pub enum Event {
    TurnStarted { player_id },
    TurnEnded { player_id },
    Drew { player_id, object_id },
    Played { player_id, object_id },
    Damaged { target: Target, amount: u8, source: DamageSource },
    Revealed { player_id, object_ids: Vec<ObjectId> },
    Picked { player_id, object_id },
    Buried { player_id, object_id },      // one per card, bottom order
    Entered { player_id, object_id },
    Died { object_id },
    GameEnded { outcome: Outcome },
}
pub enum Target { Hero(PlayerId), Monster(ObjectId) }
pub enum DamageSource { Card(ObjectId), Fatigue }

// src/view.rs
pub struct View { viewer, active_player, players: Vec<PlayerView>, outcome: Option<Outcome> }
pub struct PlayerView {
    player_id, hero_health: i32, mana: u8, max_mana: u8,
    hand: Vec<HandCard>, deck_size: usize, board: Vec<BoardCard>, revealed: Vec<RevealedCard>,
}
pub struct HandCard { object_id, face: Option<Face> }         // None unless the viewer holds it
pub struct Face { def_id, mana_cost: u8 }
pub struct RevealedCard { object_id, def_id: Option<DefId> }  // None unless the viewer is choosing
pub struct BoardCard { object_id, def_id, attack: i32, health: i32 }
```

**Choices I made inside the draft. Change any of them:**
1. **Fatigue is a `Damaged` with `source: Fatigue`**, not an event of its own. Spark and fatigue are then told apart by their source, which is exactly what the shell needs to pick an animation.
2. **`Face` groups the `def` and the cost**, so a hidden card can't end up showing one without the other. That would be a leak, because the cost narrows down which card it is.
3. **The opponent sees the `object_id`s of hidden and revealed cards** (the Arena/Hearthstone policy, so the deck-list allocation leak stays open).
4. **`Target::Monster`** follows your code's word, not SPEC's "minion".
5. **A Forage that reveals nothing emits nothing**, matching SPEC's "nothing happens". **`Buried` is one event per card.**
6. **No graveyard in `View`**, since SPEC has no graveyard query yet.

**The other changes:**
- `Event` and the view types are re-exported from `lib.rs`. Unexported, they'd trip your `unreachable_pub` lint.
- The rename: `EventKind` is now `HistoryKind`, `EventLog` is now `HistoryEntry` with a `kind` field, and `History.logs` is now `entries`.
- `Game::view` is `todo!()`.
- I did **not** add `Views::new`. Nothing calls it yet, so it would fail clippy as dead code. Add it when you write the checkpoint calls: `pub(crate) fn new(game: &Game) -> Views<'_>`.

If this looks right, I'll add the API and the event/view rules to `SPEC.md` and write the red tests. Reply with OK or with your changes.


> [!quote] YOU

disagree with u I think fatigue damage should be an event of its own, I also did a couples of changes, u can update the spec, write tests and commit the parts as u see fit


> [!abstract] PI

Now `SPEC.md`:


> [!abstract] PI

Now the tests. First the shared `Recorder` in `support.rs`:


> [!abstract] PI

The tests are written and validated. With a throwaway reference implementation in `/tmp/d3ref`, all of them go green (19 unit + 83 spec) and clippy is clean, so a red test means missing behaviour, not a bug in the test. I haven't copied that implementation into the repo. The bodies are yours.

**Committed:** `2d6307e`, your skeleton plus the rename and the `&mut ()` call sites (green). I accepted your `FatigueDamaged { amount, player_id }` and `BoardEntered`, removed the stale `DamageSource` re-export that broke the build, and added a missing colon in `event.rs`'s module doc.

**Not committed yet: `SPEC.md` and the tests.** These are my code and you haven't reviewed them, and they're red, which fails the checks that `CLAUDE.md` requires before a commit. I suggest committing them together with your implementation, so that commit is green.

**`SPEC.md`:** a new "Events and views (node D)" section, the API, and the layout rows. Three rules in it are my additions, so check them:
1. The last call of every accepted `apply` is a checkpoint, so its views equal `view(v)` afterwards.
2. `Damaged.amount` is the printed amount, even when it's more than the target has left.
3. The zone-move checkpoints include reveal, pick and each bury.

**Tests:** a `Recorder` and `observe(game, p, a)` in `support.rs`.

| Module | Tests | What they pin |
|---|---|---|
| `events.rs` | 16 | end turn → `TurnEnded`, `TurnStarted`, `Drew`; fatigue; Spark; Wild Bolt across 50 seeds (the event agrees with the hero that lost health); minion entry; Blast hits every character before any death; the Captain dies before the Recruit; checkpoints show the Recruit at `[3, 1, 0, gone]`; Forage reveal, empty Forage, pick plus bury; exactly one `GameEnded`, in the last `apply`; a rejected action reports nothing; final checkpoint = `view` |
| `views.rs` | 7 | viewer, active player and player ids; your own hand with faces and costs; only IDs for the opponent's hand; public health, mana and deck sizes; the board with buffed stats for both viewers; Forage options for the chooser only; outcome |
| `properties.rs` | +4 | over 20 random playouts: observing leaves the same game; same game and action give the same steps; every `apply` ends with a matching checkpoint; hidden cards show only to whoever may see them |

The run: 23 red. 3 of the new tests pass already (a rejected action reports nothing, observing leaves the same game, same steps). They're guards and will stay green once events exist.

**One pointer for the implementation:** expect `E0502` in `draw`, `reveal` and `pick`. Each holds `let player = self.get_player_mut(..)` across the place where a checkpoint has to go. That's the compiler enforcing what we saw earlier: a checkpoint can't happen while something is half-written.

Do you want to implement now, or should I close the session here (update `COURSE.md`, commit the log) and you implement next time?


> [!quote] YOU

finished, commit everything, one question that I got during the implementation: it is not clear to me when to call checkpoint, on every event?


> [!abstract] PI

All three checks pass: fmt, clippy, and 19 unit + 83 spec tests. Before I commit, I'm reviewing your implementation.


> [!abstract] PI

Committed as `a2f8ec7` (green). Before I answer your question, I'll check one thing in your checkpoint placement with a throwaway probe:


> [!abstract] PI

No, not on every event. Here's the rule, and then what your code actually does. The probe turned up something about my own spec.

**When to call `checkpoint`.** A checkpoint is a step boundary. It says: "the events since the last checkpoint happened together, so show the state now." At any spot, the question is whether the screen should be able to show the state *between* what came before and what comes next:
- **Simultaneous** things go in one step, with no checkpoint between them. Blast's hits are one sentence of card text, so they land together.
- **Sequential** things get a checkpoint between them. The Captain's death and the Recruit's death come from separate passes, so the screen should show the Recruit at 0 in between.
- **Only where the state is settled.** The borrow checker enforces the mechanical half of this.

A checkpoint after every event would treat everything as sequential, which is the option you rejected earlier (Blast's hits playing one by one). So checkpoints belong where simultaneity is decided: at the end of a card's resolution, after a zone move, after a death pass, and once at the end of `apply`. They don't belong in leaf loops such as the per-target loop in `apply_effect`.

**What your code does with Blast** (probe output):

| Step | Events | Recruit | Captain | Heroes |
|---|---|---|---|---|
| 0 | `Played` | 3 | 1 | 10, 10 |
| 1 | `Damaged` ×4 | 1 | −1 | 8, 8 |
| 2 | `Died` | 0 | gone | 8, 8 |
| 3 | `Died` | gone | gone | 8, 8 |
| 4 | none | gone | gone | 8, 8 |

That's correct by the rule above. But it **diverges from `SPEC.md`**, which says "after each effect". Your Blast data has two `Effect`s (minions, then heroes), so the spec asks for a step where the minions are hit and the heroes aren't yet. My tests didn't catch it. And thinking it through, **your version is the better rule.** Blast's text is one sentence, so its hits are simultaneous, as Hellfire's are in Hearthstone. Splitting it into two `Effect`s is an encoding detail, and checkpointing per `Effect` would leak that encoding onto the screen. A card whose text really is sequential ("Deal 2. Then draw a card.") needs a boundary that the *data* expresses. That's a question for session 04 (effects as data).

I'd change the spec to: "after a card's resolution, each zone move, each death pass, and at the end of `apply`", and add a test pinning that Blast's four hits share one step.

**Review of the rest.** None of these are covered by tests, and all of them are your code:
1. **`spawn` reports `BoardEntered` *after* `on_board_enter` effects run**, so an enter effect would show up before the minion entered. Emit the event right after `board.add`. No current card has such an effect.
2. **`kill` runs the deathrattle before reporting `Died`.** It's the same root as the one-at-a-time death loop, so it goes in the session 05 opener with the Medic. No change today.
3. **`reveal`** moves `options` into the event, destructures it back out, and has an `unreachable!()` branch, all to avoid cloning two IDs. `object_ids: options.clone()` is simpler.
4. **Unused parameters added ahead of need:** `emtomb(_obs)` and `board_card(_viewer)`. That's the session 02 speculative-branch pattern; drop them until a rule needs them.
5. **`update_outcome` emits `GameEnded` whenever an outcome exists,** not when it first appears. It's safe only because nothing is legal after the end. Guard it with `self.outcome.is_none()`.
6. **`play` now records `CardPlayed` in `History` before the effects resolve** (before your change it was after). So a spell counts as cast while it's still resolving. That's harmless today, and it matches Hearthstone's "played" counters. Flagging it because it's a behaviour change.
7. **Nit:** `match cond { true => Some(..), false => None }` can be `cond.then(|| ..)`.

Two decisions for you:
- **A.** Change `SPEC.md` to your checkpoint rule plus the Blast test (my recommendation), or keep "after each effect" and add the checkpoint in `apply_effects`?
- **B.** Points 1, 3, 4, 5 and 7: will you fix them, or should I (you review before the commit)?

After that I'll close the session: `COURSE.md` (knowledge map, decisions, open threads, verified facts) and the log commit.


> [!quote] YOU

I am not sure the checkpoints are correct but it is easy to add more later. 3 but this way we dont clone so compiler can optimize no? 4 I prefer to keep them. 6 intended. rest was fixed, A update yes


> [!abstract] PI

The new test pins the rule. It fails against my per-effect reference implementation (`[2, 2]`), passes on your code, and the checks are all green (19 unit + 84 spec). Committing:
