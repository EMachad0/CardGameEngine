> [!quote] YOU

continue the architecture course


> [!abstract] PI

Session 02a: the `emit` exercise (node 3), then West's argument (node 4). First, a short retrieval round on session 02.


> [!question] Quiz
> Which code can break this invariant today?
>
> ```rust
> // history.rs
> pub struct History {
>     pub entries: Vec<HistoryEntry>,
> }
> ```
> Invariant: entries are append-only, in the order the events happened. Per-turn counts rely on it. Today only `Game::history_from_event` pushes to `entries`.
>
> 1. No code, since a `Vec` only grows through `push` and keeps order
> 2. Any code that can reach a `&mut History`, since `entries` is `pub`
> 3. Any code that can reach a `&History`, since `entries` is `pub`
> 4. Only `history_from_event`, the one place that pushes today


> [!success] Quiz — correct ✓
> Your answer: 2. Any code that can reach a `&mut History`, since `entries` is `pub`
> Correct answer: 2
>
> Whoever can write data can break its rules. With a `pub` field, every holder of `&mut History` can `insert`, `clear` or `sort` the entries. The set of possible writers matters, not today's callers. `&History` can only read, so it can break nothing. The fix is an owner: a private field and an `append`-style command.


> [!question] Quiz
> Where should the invariant "an object is in exactly one zone, across all players" be kept?
>
> Today each `Player` holds its own `Zones` (hero, hand, deck, board, graveyard). Forage can reveal a card from the enemy's deck, and a revealed card can end up in the picking player's hand.
>
> 1. In each `Object`, as a private field naming its current zone
> 2. In one owner holding every player's zones, behind a move command
> 3. In `Game`'s own code, since it already holds every player
> 4. In each player's `Zones`, made private, since each player owns its cards


> [!success] Quiz — correct ✓
> Your answer: 2. In one owner holding every player's zones, behind a move command
> Correct answer: 2
>
> An owner must privately own all the data its invariant mentions. This invariant mentions every player's zones, because a card can move between players. A per-player `Zones` can't stop the other player's zones from also holding the card, so the guarantee would be informal in every caller. `Game` holds far more than this invariant, so `&mut self` gives every one of its methods write access. A zone field on `Object` duplicates the zone lists, and the two copies would have to agree.


> [!question] Quiz
> After the untangling, where does `mana_cost` go?
>
> ```rust
> // game/lookup.rs, today
> pub fn mana_cost(&self, object_id: ObjectId) -> Option<u8> {
>     // Some only while the card is in its owner's hand:  locations
>     // printed cost:                                     binder
>     // cost modifiers on the object:                     objects
> }
> ```
> Owners: locations (every player's zones), objects (`player_id`, damage, modifiers), binder (card definitions). All three sit under `game`.
>
> 1. In the objects owner, since cost modifiers live on the object
> 2. In a module with no data under `game`, taking the three owners
> 3. In the locations owner, since it decides when there is a cost
> 4. In the binder, since the printed cost is the starting value


> [!success] Quiz — correct ✓
> Your answer: 2. In a module with no data under `game`, taking the three owners
> Correct answer: 2
>
> `mana_cost` writes nothing and combines three owners' data. Inside any one owner it would make that owner depend on the other two, so a change to the binder's interface would force an edit inside the objects owner. Reading is a reason to depend on an owner, not to be one. So it gets its own module with no data under their lowest common ancestor, `game`, taking shared references to the owners and returning the cost.


> [!abstract] PI

All three right, including the reader case, though it was recognition with options again. Production is the real check, and that's the exercise.

**Plan for today** (approved at the end of 02):
1. **Node 3 exercise: redraw `emit`**, in two small parts so it doesn't balloon like the last two exercises. Part 1 is history, part 2 is what's left of `emit`.
2. **Node 4: West's argument.** I present her case with quotes, and you judge where she and the doc really disagree.

---

## Exercise, part 1: history

`emit` does three things: it records history, reports to the observer, and matches triggers. We sketched matching last session. History is the part you haven't drawn yet, and it's one decision: what gets recorded and how it's counted. Today it's spread over two files (audit finding 13):

```rust
// game/triggering.rs, the writer, called first thing in emit
fn history_from_event(&mut self, event: &Event) {
    let turn = self.turn_order.turn_count();                          // reads turn order
    if let Some(entry) = match event {
        Event::Died { object_id } => Some(HistoryEntry {
            player_id: self.objects.get(*object_id).player_id,        // reads objects
            kind: HistoryKind::CharacterDied { object: self.objects.get(*object_id).clone() },
            turn,
        }),
        Event::Played { .. } => /* CardPlayed, same shape */,
        Event::BoardEntered { .. } => /* CharacterEntered, same shape */,
        _ => None,
    } {
        self.history.entries.push(entry);
    }
}

// game/resolve.rs, the reader, used by EffectAmount::History
fn history_query(&self, query: HistoryQuery, player_id: PlayerId) -> u8 {
    // filters self.history.entries by:
    //   scope: All players / Owner / Active (reads turn order)
    //   turn:  Current (reads turn order) / All
    //   kind:  SpellsPlayed -> CardPlayed whose def is a Spell (reads binder)
    //          MinionsDied  -> CharacterDied ...
}

// history.rs
pub(crate) struct History { pub entries: Vec<HistoryEntry> }
pub(crate) struct HistoryEntry { pub player_id, pub kind: HistoryKind, pub turn: u32 }
// HistoryKind variants each hold a full clone of the Object
```

After the untangling, the owners are: objects, locations, the binder, turn order, and a history owner.

**Your turn, in your own words:**

1. What is the history owner's data, and what invariant does it guard?
2. What is on its interface: which commands and which questions, with rough signatures?
3. `history_from_event` reads objects and turn order, and `history_query` reads the binder and turn order. Where does each of those reads happen in your design?


> [!quote] YOU

1 the entries, nothing really, but it would want to guard that the entries are sorted by when they happened. history_from_event informally holds another invariant, that any of those 3 events have a matching history entry. today it is the entries vec, the fields on the entries. On its own module with no data


> [!abstract] PI

Good start. Taking your answers in order:

**1. Right.** The data is the entries, and the invariant is append-only, in the order things happened. The second invariant you found is a good catch: every `Died`, `Played` and `BoardEntered` has an entry. It ties reported events to history, though, so the history owner can't keep it alone, because it only knows about events it's told about. `emit` keeps it today. It's the one command that both reports and records, and Clippy bans `obs.event` everywhere else. That's "one command takes the order away from callers", so this part of `emit` is already right.

**2. Not answered yet.** "The entries vec, the fields on the entries" is today's interface, and today the owner hands out its data. I need the redesigned one.

**3.** I read your answer as a single dataless module that reads objects and turn order to build an entry, and reads the binder and turn order to count. Correct me if I've got that wrong. If so, the history owner holds a `Vec`, an append and some way to read entries.

Now test that design against the likely change. In the card-game course you held "store history, derive the rest", where a new card that counts something (your Vulture case) is a new query with no new state. So a new query is the change to expect here.

**Your turn:**

1. Walk a new query through your design. Which modules change, and which of them need to know `HistoryEntry`'s layout?
2. Then give the history owner's interface, the commands and the questions with rough signatures, so that this change lands in as few places as you can get it.


> [!quote] YOU

2 your question before did not ask for the new design, but 3 yes, it would be a History { entries: Vec<HistoryEntry> } and the interface would be a push and a pub(crate) fn query(&self, query: HistoryQuery) -> u8 that would query the counts and HistoryQuery would have everything needed to query and would would similarly to resolve.rs history_query function, I think it could also hold a binder since the Binder is cheap to clone


> [!abstract] PI

Fair point on question 2. "Its interface" could be read as today's, so that's on my wording.

**What your design gets right.** Counting moves into the history owner, so it answers questions about its own data. A new query now changes `History` and the query type, and nothing else. That beats my reading of your last answer, where a dataless module would have counted.

**One small issue.** `HistoryQuery` is card text: `EffectAmount::History(HistoryQuery)` in `choice.rs` is what a card definition stores. If the asking player, the active player and the current turn go into it, runtime values end up inside card definitions. They belong in separate parameters, e.g. `count(&self, query: HistoryQuery, asking: PlayerId, active: PlayerId, turn: u32) -> u8`.

**The real question is the `Binder` inside `History`.** You're right that it's cheap. `Binder` wraps an `Arc<BTreeMap<DefId, CardDef>>`, and card definitions never change during a game, so the clone can't go stale either. Neither performance nor staleness is the cost.

**Your turn:**

1. Using the session 02 rules, what does `History` holding a binder cost?
2. What could you do instead? There's more than one option, so give me the one you'd pick and why.


> [!quote] YOU

on the issue I was thinking about creating a new HistoryQuery I forgot that one existed, maybe I would rename it HistoryQueryDef but separate parameters also work. 1 not sure, I guess now it holds informal knowledge that binder is cheap to clone and formally requires that the history is built after the History. I could always recieve it as parameter that would remove those


> [!abstract] PI

Either works for the query type. `HistoryQueryDef` would match `CardDef` and `TriggerDef`, so we can settle the name when the ticket gets written.

**On 1:** both costs you named are real but small. Knowing the clone is cheap means knowing `Binder`'s representation, which is informal. The build order is formal, so the compiler checks it. You missed the bigger cost, the one from the rule you approved last session:

> A decision that combines several owners' data gets its own module with no data. Inside one of the owners, it would make that owner depend on the others.

Counting "spells played" combines history's entries with the binder's card kinds. Whether the binder is stored or passed in, `History` still calls `binder.get(def_id).card_kind()`. So `History` depends on `Binder`'s interface, and a change there forces an edit inside `History`. Passing it as a parameter gets rid of the stored copy and makes the dependency visible in the signature, which is better. The dependency is still there, though.

So the rules pull against each other here:

- "An owner answers questions about its own data" wants counting inside `History`.
- "A combining decision goes in a dataless module" wants it outside, which forces `History` to hand out its entries again.

**Your turn, open-ended:** is there a design where `History` counts spells played without reading the binder at all? If so, what does it cost, given "store history, derive the rest"?


> [!quote] YOU

hmmm, query could recieve a filter closure, another module would then be responsible for combining the several owners or it could recieve a &[CardDef] data as parameter, it would depend on CardDef module but it is the data that it needs to read or instead of storing object we store another shape that contains all the information it needs to answer queries, I think I prefer this last one


> [!abstract] PI

That's a clean find: three options, and a sound pick. Quick verdicts:

- **Filter closure.** The closure takes `&HistoryEntry`, so the entry's layout is part of the interface again. What "spells played" means then lives in the caller.
- **`&[CardDef]` as a parameter.** Same dependency as `&Binder`, one level down.
- **Store a shape that answers the queries (your pick).** `History` reads nothing outside itself. It also fixes something we hadn't spotted: today each entry holds a full `Object` clone, so `History` depends on the objects owner's representation. After the untangling those fields are private, and the clone stops working.

**The cost,** which you didn't name: you decide at record time which facts get kept. If a new query needs a fact the entry doesn't hold, the recorder and `History` both change. This doesn't break "store history, derive the rest", since that rule was about storing events and deriving counts, and an entry like "player P played a spell on turn T" is still an event. The bet is that the facts queries ask about (who, which kind, which turn) stay stable. One thing gets better: a recorded fact is the fact as it was when the event happened, not a lookup done later.

---

## Part 2: what's left of `emit`

With part 1 and last session's matching sketch, `emit` now does this:

```rust
pub(crate) fn emit(&mut self, event: Event, obs: &mut impl Observer) {
    // 1. build the record: reads objects, binder, turn order; history.record(...)
    // 2. obs.event(&event)
    // 3. matched(&event, ..., &objects, &binder, ...) -> Vec<PendingEffect>; push on the effect stack
}
```

About fifteen places call it: `draw`, `damage`, `play`, `kill` and others. It keeps your invariant (every reported event is recorded), because it's the only command that reports.

**Your turn, open-ended:**

1. Does the bet above hold for you?
2. Where does `emit` live after the untangling, and what does its signature take? Say which owners it writes and which it only reads.
3. Is the order record, report, match a rule of the domain or just the procedure? Check the code: does anything in step 3 read what step 1 wrote?


> [!quote] YOU

1 what bet? 10$ on red?


> [!abstract] PI

Ha. I meant the stability bet from part 1's cost. The doc's version is "an interface is made of decisions too, ... expose the ones you expect to stay put."

When you store a narrow shape instead of the whole `Object`, you're betting that the facts history queries ask about stay these: **who** did it, **what kind of card**, and **which turn**. If a future card asks something else about past events, for example "minions that cost 5 or more died this game", the entry doesn't have the cost. Then the shape grows, and the recorder and `History` both change.

So question 1 is whether you expect future history cards to stay within who, kind and turn, or to need more facts about the card. Questions 2 and 3 still stand.


> [!quote] YOU

1 no, I expect the oposite actually, cards may want to query very specific things, it should be normal for a card to have a query only it does.
2 it needs to write to Observer and History and it needs to read ObjectBag, Binder, TurnOrder and the event that is to be emited, the Zones to see who is in play.
3 I dont think so, just a procedure. It is a rule that they should happen together tho


> [!abstract] PI

**1.** Then by your own forecast the bet fails, and that changes your part 1 pick. If most new history cards ask something specific to them, a narrow shape changes on almost every new card: the recorder, the entry and `History` together. The stability test you use for interfaces just overturned your design, which is exactly the job it's for.

**2.** The reads are complete. The writes miss one: step 3 pushes the matched triggers onto the **effect stack**. So `emit` writes `History`, the effect stack and the observer. You didn't say where it lives yet.

**3.** Right. Matching never reads history and recording never reads the stack, so the order is procedure. "They happen together" is your invariant from part 1, and the single command keeps it.

**Your turn:**

1. Redo part 1 with your forecast that queries vary per card. What does an entry store, and where does the query code live so that a new card's query lands in one place? Run the same stability test on each dependency the query code would have, the binder included.
2. Where does `emit` live? It writes three things and reads five.


> [!quote] YOU

2 I missed it cause I thought emit would return the PendingEffects but it makes sense. I would say it lives on its own module with no data. 1 I dont know then, I dont see other options then depending on object/binder so I can resolve the queries inside history module or passing the data on another type but then emit changes every time this new datatype changes. The only other option I see is to have the query present as data on card definition, as a data structure that could define a vast amount of queries, it sounds quite hard to prepare for any query tho.


> [!abstract] PI

**2.** Right, a module with no data. Your instinct to return the effects is right for the matching function: it decides and returns, and the stack's owner writes. `emit` is the command that keeps "they happen together", though. If it returned the effects, each of its roughly fifteen callers would have to push them, and your invariant would become informal in fifteen places. It takes three `&mut` and five `&`, so passing that list through fifteen callers is the open thread "emit needs most of `Game`". That one goes to 02b, with the coordinator.

**1.** You found the options. What was missing is the stability test applied to each dependency, not only to the interface. Run it:

- **A new query.** By your forecast, one per card, so it's likely. It has to land in one place: the query code, one match arm per query kind, the same way ADR 0004 handles effects.
- **The binder.** Its question is "the definition for this `DefId`". Every interpreter reads card text as data (ADR 0004), so this is about as stable as the crate gets. Depending on it costs almost nothing.
- **The objects owner's representation,** today's `Object` clone. It's about to go private in the untangling. Don't depend on it.

Then ADR 0005, "store what happened, derive the rest", decides what an entry holds. It says store only what **can't be recomputed**:

- Printed facts such as kind, cost and tribe can always be recomputed from the `DefId` through the binder, so don't store them.
- Facts as they were at the moment, such as a minion's attack when it died, can't be recomputed later. The recorder has to capture those.

That gives this design:

```rust
struct HistoryEntry { player_id: PlayerId, turn: u32, def_id: DefId, kind: HistoryKind } // private
                                     // plus moment-only values, added when a query needs one

impl History {
    fn record(&mut self, ...);  // called only by emit
    fn count(&self, query: HistoryQuery, asking: PlayerId, active: PlayerId, turn: u32,
             binder: &Binder) -> u8;
}
```

A new query about printed facts adds one arm in `History`. A new query about a moment-only fact also changes the recorder and the entry. That can't be avoided, since the fact has to be captured when it happens.

What this shows about the session 02 rule: "inside one of the owners, it would make that owner depend on the others" is a cost only when those owners' interfaces might change. The doc already says "a decision that never changes costs no edits, however many places know it", but the owners rule reads as if it had no exceptions.

Any pushback before a quick check?


> [!quote] YOU

wait, so this is a mix of the two then? I thought u were pressing me to find a cleverly solution, we depend on binder cause it costs nothing, that is good. but we did not avoid the change on emit every time a card wants to query a new field, e.g. count the minions with <= 3 atk played this game


> [!abstract] PI

Yes, it's a mix, and there's no clever trick behind it. My earlier question ("is there a design where `History` counts without reading the binder at all?") pushed you toward cutting the binder. That direction only made sense under the narrow-shape bet, and once you forecast per-card queries it was the wrong push. Depending on the binder is the right answer, and you'd already listed it as option (a).

On your example, "minions with ≤ 3 attack played this game", there are two readings, and the domain picks one:

- **Printed attack.** It's recomputed from the `DefId` through the binder, so you add one arm in `History` and `emit` doesn't change.
- **Attack at the moment it was played,** auras and buffs included. That value is gone afterwards, so the recorder has to capture it. Today's design can't answer this either: the `Object` clone holds damage and modifiers, but not the auras other characters gave it.

For the second reading, you can still avoid touching `emit` per query. Have the recorder store a snapshot of the card's stats at that moment, say attack, health and cost, using the same stat questions the rest of the game asks. Then a new query over any existing stat is one arm in `History`, and the recorder changes only when the game gets a new stat. A new stat changes the stat code anyway.

That moves the bet from "which queries exist", which by your forecast changes per card, to "which stats a card has", which changes rarely. Storing the snapshot still fits ADR 0005, because a value as it was at a moment can't be recomputed later.

Two questions:

1. In your game, does "≤ 3 attack played" mean printed or at the moment?
2. Does the snapshot bet hold for you?


> [!quote] YOU

1 lets change to "minios with <= 3 atk that died this game", at the moment. 2 stop using bet. I like it, it changes what changes affect it.


> [!abstract] PI

Noted, I'll stop saying "bet". "It changes which changes reach it" says it better anyway.

With your example, attack at the moment of death is in the snapshot, so that card is one arm in `History`, and `emit` doesn't change. One detail for the ticket: ADR 0012 removes the dead before deaths are matched, so the snapshot has to be taken before the minion leaves the board. Otherwise auras from the board no longer count.

Quick check:


> [!question] Quiz
> Which change reaches the recorder in `emit`?
>
> The design from the exercise:
>
> ```rust
> struct HistoryEntry {               // private to History
>     player_id: PlayerId,
>     turn: u32,
>     def_id: DefId,
>     kind: HistoryKind,              // played, died, entered
>     stats: Stats,                   // attack, health, cost at that moment
> }
>
> impl History {
>     fn record(&mut self, ...);      // called only by emit
>     fn count(&self, query: HistoryQuery, asking: PlayerId, active: PlayerId,
>              turn: u32, binder: &Binder) -> u8;
> }
> ```
>
> 1. A new stat, spell damage, and a card counting minions that died with it
> 2. A card counting spells played this game that cost 5 or more at the time
> 3. A card counting friendly minions that died during the previous turn
> 4. Binder storage switching from a `BTreeMap` to a `Vec` behind `get`


> [!success] Quiz — correct ✓
> Your answer: 1. A new stat, spell damage, and a card counting minions that died with it
> Correct answer: 1
>
> Only a new stat changes what has to be captured at the moment, so only it reaches the recorder (and the `Stats` shape). Cost at the time is already in the snapshot, and "previous turn" and "friendly" use the stored turn and player, so both are one new arm in `count`. The binder's storage is hidden behind `get`, a stable question, so nothing outside the binder changes.


> [!abstract] PI

Right. Only a new stat reaches the recorder.

That closes the `emit` exercise. Where it ended up:

- **History owner:** private entries holding who, turn, def, kind and a stats snapshot. It has `record` and `count`, and `count` takes `&Binder`.
- **`emit`:** a module with no data. It records, reports and pushes the matched triggers, keeps "every reported event is recorded", and the order of its steps is procedure. Its long parameter list goes to 02b.

**Your turn, the node's rule:** does this exercise change `file-structure.md`? Two places it touched:

- "A decision that combines several owners' data gets its own module with no data. Inside one of the owners, it would make that owner depend on the others." You ended up with a combining decision inside `History`, depending on `&Binder`.
- Choosing what an owner stores, so that the expected changes don't reach it.

If either needs a rule, state it in your own words, and I'll draft it into the doc.


> [!quote] YOU

"Dependency cost depends on stability, similar to decisions a dependency that never changes cost nothing"
I think both yours are not needed


> [!abstract] PI

Your rule holds, but the doc already says it once. The last sentence of the "likely to change" bullet in "Build modules around decisions" reads: "A decision that never changes costs no edits, however many places know it." Since the doc defines a dependency as knowledge of a decision, your rule follows from that sentence.

The gap is where that sentence sits. It's in the section about choosing boundaries, so a reader of the owners rule ("it would make that owner depend on the others") doesn't connect the two, and reads the cost as fixed. That's the reading I gave it in this exercise. So rather than add a sentence, I'd move the existing one in your words to where coupling is defined, so it covers every rule that mentions a dependency:

**In "Coupling is shared knowledge of a decision", after the paragraph ending "none of them needs a call":**

> Coupling costs as much as the decision changes. Knowledge of a decision that never changes costs no edits, however many places hold it.

**In "Build modules around decisions, not steps", the bullet loses its last sentence:**

> - Judge whether a decision is likely to change by reasoning about the domain: what other systems in it already vary, what the roadmap plans, and what comes from outside your control, such as a file format, a provider's API or a regulation. Merge the decisions that change together. Leave stable, easy decisions where they are.

I said "the decision changes" rather than "stability", because the doc never defines stability. Approve, or reword?


> [!quote] YOU

unslop the second and remore the exemples


> [!abstract] PI

I've applied both edits to `docs/conventions/file-structure.md`. I removed the examples from the bullet and the colon that joined its clauses. Diff:

```diff
@@ Coupling is shared knowledge of a decision
 are others, and none of them needs a call.
 
+Coupling costs as much as the decision changes. Knowledge of a decision that never changes costs
+no edits, however many places hold it.
+
@@ Build modules around decisions, not steps
-- Judge whether a decision is likely to change by reasoning about the domain: what other systems
-  in it already vary, what the roadmap plans, and what comes from outside your control, such as a
-  file format, a provider's API or a regulation. Merge the decisions that change together. Leave
-  stable, easy decisions where they are. A decision that never changes costs no edits, however
-  many places know it.
+- Judge whether a decision is likely to change by reasoning about the domain. Look at what other
+  systems in it already vary, what the roadmap plans, and what comes from outside your control.
+  Merge the decisions that change together. Leave stable, easy decisions where they are.
```

Approve, and I'll commit the doc as node 3's commit. Then we move on to West.


> [!quote] YOU

approved


> [!abstract] PI

Committed as `54d5d9f` on `architecture-02`. Only the session log is still uncommitted.


> [!abstract] PI

## Node 4: West's argument

**Why now.** In session 02 you asked whether "being a big state is good or bad", and you said you value hearing arguments against what we're learning. Catherine West's RustConf 2018 closing keynote is the best-known Rust argument against data hiding in applications. It's also easy to misquote: an earlier report of ours said she endorses all-public state, and the 02 research found she doesn't. So here is her case in her own words. All quotes are from her long-form text of the talk, kyren.github.io/2018/09/14/rustconf-talk.html.

**1. The starting point.** Games are "more or less a giant static global struct containing all of the game state". She sketches a "giant nested tree of entirely public structs", with "systems" that are "just regular functions" over it. Rust lets them "split borrows for different fields of a public struct". Then she hedges: "I'm genuinely not advocating this." She names its cost herself: "the entire game state is visible and mutable to all systems and if something changes unexpectedly, you might have no idea where to look."

**2. What she attacks: OO entity design at application scale.**
- "the vast majority of the interesting behavior in your game ends up spanning many data types, and does not naturally 'belong' to any specific entity."
- "Lots of code has no clear place to be because it concerns multiple entities at once."
- "every time a new requirement comes in, you have to take what may have once been sane interfaces and 'poke more holes through them'."
- Interfaces "*amplify* borrowing problems by *amplifying* the amount you are forced to borrow!"
- "Stop conflating the data representation of your game with the systems that operate on it!"
- Her verdict: "at the point where you're not writing 'library' code and are actually writing a game, data hiding is NOT generally useful, and just wastes a lot of effort and time."

**3. What she keeps.**
- "Being able to hide data to maintain invariants is invaluable ... Very useful in the small, and with 'library' code."
- Her generational index allocator is "an obviously GOOD use of data hiding".
- A component method is fine "if there is a small local invariant to maintain. SQL is all about data, but it also has stored procedures to maintain invariants."

**4. The problem she has left, and why she moves to ECS.** Every system "depends on _all_ of the types that go into our game state", so "changing anything inside 'GameState' potentially would affect every system at least _theoretically_." ECS limits "everything depending on everything else". She credits it mostly to the fact that "it forces you to think about your data rather than be stuck in OO mode".

---

**Your turn, open-ended.** Read her against `file-structure.md` as it stands (owners guard invariants, a decision that combines owners' data goes in a module with no data, data handed out is interface, hide decisions that are likely to change):

1. Where does she look like she disagrees with the doc, but actually agrees?
2. Where do they really disagree? Name the claim each side makes.


> [!quote] YOU

oh, I have watched her talk. 1 "the vast majority of the interesting behavior in your game ends up spanning many data types, and does not naturally 'belong' to any specific entity." and "Lots of code has no clear place to be because it concerns multiple entities at once." seem to be used as arguments against a textbook design like ours but we actually agree and actually solved it, behaviour that touches multiple modules lives on modules of its own with no data, that is what ecs systems are.
2 "depends on all of the types that go into our game state" our design disagree, it only depends on the public interfaces of the modules it can see


> [!abstract] PI

**1. Right.** Her complaint is about behavior attached to entities, and the doc builds modules around decisions instead, with combining decisions in modules that hold no data. Those are her systems. You missed three more places where you agree:

- **"Poke more holes".** Accessors multiply when an owner is drawn around a noun, and every new requirement needs another of its fields. The doc's owners are drawn around invariants and answer questions about their own data only. That's the fix for your 01b "Objects" module, which needed most of `Game`.
- **"Stop conflating the data representation with the systems".** That's "reading isn't owning", the habit you fixed in 02.
- **Invariants.** She calls hiding data to keep invariants "invaluable", and she names the cost of all-public state herself, "no idea where to look". That's the root "whoever can write data can break its rules".

**2. Not a disagreement yet.** "Depends on all of the types" is a problem she has with her own UR-architecture, and her whole move to ECS is meant to fix it. Both sides want systems to depend on less. They only fix it differently: you with explicit owners as parameters, she with ECS queries. A real disagreement is a claim one side makes that the other denies.

**Look again, open-ended.** Take her verdict, "at the point where you're not writing 'library' code and are actually writing a game, data hiding is NOT generally useful". Put it next to two doc rules: "data a module takes in or hands out is part of its interface" and "hide decisions likely to change".

1. In ECS, what does a system know about the components it queries? Which of those doc rules says something about that knowledge?
2. She splits "library" code from "application" code. Which one is the `rules` crate, and which is the Bevy front end?


> [!quote] YOU

1 the interface it exposes, which is formal knowledge. 2 I disagre with the split but I guess here rules would be the lib and bevy the application, we disagree with the "data hiding is NOT generally useful" it remains usefull as any required change costs less due to our design principles


> [!abstract] PI

**1. Half right.** In ECS a component is usually a plain struct with public fields, and a query hands the system `&T` or `&mut T`. So the system doesn't know an interface. It knows the **layout**. The doc says how much that costs:

- **The layout is formal.** Rename a field and the build finds every system that uses it.
- **The meaning and the write rules are informal.** Whether `health` is current or maximum, or that damage never goes negative, is copied into every system that reads or writes the field. Any system with `&mut T` can break it.
- **Formal still costs one edit per place.** The build finds the places, but each one still has to change.

The doc's line is "a decision whose data crosses the boundary isn't hidden". So here is the real disagreement. **She bets the data layout is the stable part and puts it in the open. The doc, after Parnas, hides a representation that's likely to change, even when no invariant ties it.** Session 03 picks this up as data-oriented design.

**2.** I agree with you on the split. The doc's test isn't scale. It's whether the decision is likely to change and how many places know it. One claim in your answer goes too far, though: "any required change costs less". The doc says the opposite: "No design makes every decision cheap to change." Hiding makes a change to the hidden decision cheap, and a change that crosses the interface costs more, because the interface changes too and then every caller. Her "poke more holes" is exactly that second kind. In games, new requirements often cut across entities, and that's her case for paying less for hiding.

**Your turn:**

1. Your verdict on your 02 question, in a sentence or two: is one big state good or bad?
2. Does West change anything in the doc, or does it already answer her?


> [!quote] YOU

1 it is not nescessarily bad, the problem arises if code start depending on state that changes often. 2 I think the doc already answers


> [!question] Quiz
> Which claim would West make and `file-structure.md` deny?
>
> 1. A system should depend on as few of the state's types as it can
> 2. Data that no invariant ties can be public, though its layout may change
> 3. Hiding data is worth its cost wherever it keeps an invariant
> 4. Behavior that spans several entities should live outside all of them


> [!success] Quiz — correct ✓
> Your answer: 2. Data that no invariant ties can be public, though its layout may change
> Correct answer: 2
>
> West keeps hiding for invariants and otherwise makes data plain and public, betting the layout is the stable part. The doc hides a decision that's likely to change, a representation included, even with no invariant behind it, because data handed out is part of the interface. The other three both sides hold: behavior spanning entities goes in systems (the doc's modules with no data), hiding pays for invariants, and depending on fewer types is why she moves to ECS and why the doc passes owners explicitly.


> [!abstract] PI

Right. (The explanation slipped and said "betting" again. Noted, I'll keep it out.)

**On your verdict:** your sentence covers half of it, the reading side. Code that depends on state whose decisions change often pays for every change. The other half is the write side, the root from 02: with one big `&mut` state, any code can break any invariant, which is her own "no idea where to look". So one big state is fine as a layout, because it's one tree of values you can clone and compare. It goes bad in two cases: when code reads layouts that change often, and when code that isn't an owner can write data an invariant ties.

**On 2:** agreed, no doc change. "Passing data between modules is fine when its layout is a decision you expect to stay put" is where her case fits, and the owner rules cover her invariants.

That finishes 02a's two nodes. Next on the plan is 02b: rules restored by a pass, the coordinator, your effect stack question, and the open thread on `emit`'s seven parameters. Shall I close the session here?


> [!quote] YOU

yes, reread the skill I did some changes on close session
