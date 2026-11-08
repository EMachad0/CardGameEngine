## Scope research: invariants, stale reads, deferred events, Rust borrows

**Quote legend.** **[V]** means I checked the wording against the text of a page I fetched myself. **[S]** means I saw it only in a search-result snippet or excerpt. Text in a PDF may differ slightly from the printed page, for example where line-break hyphens were rejoined. Anything outside quote marks is my own summary or inference.

## Summary
- **Q1.** The ownership methodologies allow both one owner and nested owners. Each object packs and unpacks *itself*. The owner's pack and unpack only "commit" and "uncommit" the objects it owns. They don't do the owned objects' packing for them, so the claim you were given is partly wrong. Neither paper says it prefers nesting. DDD sources put enforcement on "the root". Microsoft's DDD guide is the only source I found that also gives local invariants to inner entities.
- **Q2.** Kleppmann's "premise" wording is correct, but it is in the *serializable snapshot isolation (SSI)* section of ch. 7, not in "Write Skew and Phantoms". I found no source that names the split between "still current" and "as of the read".
- **Q3.** The MTG claim is wrong as you stated it. Triggers are checked against the state "immediately after an event". Only putting them on the stack is deferred. Bevy 0.17 observers run immediately. Messages are read at fixed points later in the schedule.
- **Q4.** Confirmed. The 2026 Rust project goals include a view-types experiment.

## Findings

### 1. Several invariants inside one boundary

**1a. Barnett, DeLine, Fähndrich, Leino, Schulte, JOT 3(6) 2004** — https://www.jot.fm/issues/issue_2004_06/article2.pdf
- §3 **[V]**: "An implementation can modify the value of st only through the use of two new statements, pack and unpack ."
- §3 **[V]**, first restriction: "For now, we restrict each object invariant so that the only part of the state it can depend on are the fields of this (that is, the object whose invariant is being described). Later, we will make this restriction more liberal."
- §4 **[V]**, why the restriction exists: "If there is a valid T object t such that t . f = u , then the update of u . g may cause t ’s invariant to be broken without t being invalid. Therefore, we will arrange for any such t to be invalid whenever u is. To prevent u from being unpacked without regard for the state of t , we will make the packing of t put u into a committed state that can be changed back only by unpacking t ."
- §4 **[V]**: "Hence, we’ll arrange for the unpacking of t to take t . f from committed to valid."
- §4 **[V]**, on nesting: "the chains of committed objects form a hierarchy that corresponds to the hierarchy of layered abstractions."
- §4 **[V]**: "we’re able to loosen the restriction on object invariants so that an object’s invariant can depend on the fields of its compo- nents (and, transitively, the fields of their components)."
- §5 **[V]**, final rule: "An object invariant in a class T is now allowed to depend on any field of this declared in T or one of its superclasses. For any of those fields that is a rep field, the object invariant is also allowed to depend on any of its fields (and so on, transitively)."

**1a. Leino & Müller, ECOOP 2004** — https://pm.inf.ethz.ch/publications/LeinoMueller04.pdf
- Abstract **[V]**: "The invari- ant of an object is allowed to depend on the fields of the object, on the fields of all objects in transitively-owned contexts, and on fields of objects reachable via given sequences of fields."
- §1 **[V]**, the third kind of dependency: "Third, our methodology allows the invariant of X to depend on the fields of any specified object X . f 1 . · · · . f n ( n > 1 )—that is, an object reachable from X by a fixed sequence of field dereferences—provided certain visibility requirements are met."
- §2.1 **[V]**: "a committed object has exactly one owning object."
- §2.1 **[V]**, unpack definition: "unpack o from T ≡ assert o 6 = null ∧ o . inv = T ∧ ¬ o . committed ; o . inv := S ; foreach p ∈ Constit T ( o ) { p . committed := false }"
  - The "6 =" is how the PDF text extracted "≠".
- §2.2 **[V]**, universe types, as Leino & Müller describe Müller's thesis: "The state of a universe is encapsulated , that is, the fields of the universe’s objects can be modified only when control is in a method applied to an object within the universe. A universe can have several owner objects, all of which belong to the enclosing universe. The invariant of an object o is allowed to depend on the fields of all objects in o ’s universe and in all nested universes."

**What is right and wrong in the claim you were given**
- **Right:** an object's invariant may depend on the fields of the objects it owns, transitively.
- **Partly wrong:** "owned objects are packed and unpacked through their owner".
  - Each object has its own `pack` and `unpack`.
  - Packing the owner requires its components to be valid already, then marks them committed.
  - Unpacking the owner only uncommits them. They stay valid until they are unpacked themselves.
  - A committed object cannot be unpacked (`assert ¬o.committed`). So the order is owner first, then the owned object.
- **Overstated:** "may mention only owned state".
  - Leino–Müller also allow invariants over objects reached by fixed field paths, under visibility rules.
  - Universe types allow dependence on objects in the *same* universe as well as nested ones.

**Why owned state is the safe case.** If an invariant mentions an object nobody owns, someone can update that object and break the invariant while its holder still counts as valid. Ownership rules this out: it makes the owner invalid whenever the owned object is open for change. This is the §4 Barnett quote above.

**Nested owners or one owner?** This is my inference. Neither paper ranks the two options.
- A single object can state both rules over its `rep` fields.
- A nested design fits naturally: the inner object owns `balance` and `holds` and keeps rule 1, and the outer object owns `journal` plus the inner object and keeps rule 2 (rule 2 may read `inner.balance` because the inner object is `rep`).
- The cost: writing `balance` means unpacking the outer object, then the inner one, then re-packing the inner one (rule 1 is checked) and then the outer one (rule 2 is checked).

**1b. DDD**
- Evans, *DDD Reference* (2015), "Aggregates" **[V]** — https://www.domainlanguage.com/wp-content/uploads/2016/05/DDD_Reference_2015-03.pdf
  - "Objects are supposed to maintain their own internal consistent state, but they can be blindsided by changes in other objects that are conceptually constituent parts."
  - "Define properties and invariants for the aggregate as a whole and give enforcement responsibility to the root or some designated framework mechanism."
- Evans 2003 (blue book), through Jimmy Bogard's rule list, **[V]** on Bogard's page — https://lostechies.com/jimmybogard/2010/02/04/strengthening-your-domain-a-primer/
  - "The root Entity has global identity and is ultimately responsible for checking invariants".
  - This is Bogard's summary list. I could not check it against the book.
- Vernon, *Effective Aggregate Design* Part I **[V]** — https://www.dddcommunity.org/wp-content/uploads/files/pdf_articles/Vernon_2011_1.pdf
  - "The consistency boundary logically asserts that everything inside adheres to a specific set of business invariant rules no matter what operations are performed."
  - "Thus, aggregate is synonymous with transactional consistency boundary."
  - Part I does **not** say which object inside the aggregate enforces which rule.
- The closest source for "inner entities keep local rules, the root keeps the cross-entity ones" is Microsoft's *.NET Microservices* guide **[V]** — https://learn.microsoft.com/en-us/dotnet/architecture/microservices/microservice-ddd-cqrs-patterns/domain-model-layer-validations
  - "The main responsibility of an aggregate is to enforce invariants across state changes for all the entities within that aggregate."
  - "an order item object always has to have a quantity that must be a positive integer, plus an article name and price. Therefore, invariants enforcement is the responsibility of the domain entities (especially of the aggregate root)".
  - This is a vendor guide, not Evans or Vernon.

**1c. Meyer.** I did not reach the OOSC2 text.
- Two pages say Meyer calls this the "Indirect Invariant Effect", OOSC2 §11.14, pp. 403–406.
  - Bond docs **[V]** on that page (a third-party paraphrase): "object `a`'s invariant can be broken by an operation on object `b` that happens to hold a reference into `a`" — https://bond.hexdocs.pm/invariants.html
  - Boost mailing list **[S]** — https://listarchives.boost.org/Archives/boost/2004/01/58715.php
- Meyer 2016, arXiv abstract **[V]**: "Class invariants are both a core concept of object-oriented programming and the source of the two key open OO verification problems: furtive access (from callbacks) and reference leak." — https://arxiv.org/abs/1608.07637
- The abstract says the paper solves these problems "without any of the following: ownership; separation logic; universe types; object wrapping and unwrapping; …" **[V]**, same page. It is a counterpoint to 1a.

### 2. A value read before a write

All Kleppmann quotes come from ebrary.net, an unofficial mirror of DDIA (1st edition). I did not check them against the O'Reilly text.

- "Decisions based on an outdated premise" (SSI section of ch. 7) **[V]** — https://ebrary.net/64802/computer_science/decisions_based_outdated_premise
  - "Put another way, the transaction is taking an action based on a premise (a fact that was true at the beginning of the transaction, e.g., “There are currently two doctors on call”). Later, when the transaction wants to commit, the original data may have changed—the premise may no longer be true."
  - On the split you asked about: "When the application makes a query (e.g., “How many doctors are currently on call?”), the database doesn’t know how the application logic uses the result of that query. To be safe, the database needs to assume that any change in the query result (the premise) means that writes in that transaction may be invalid. In other words, there may be a causal dependency between the queries and the writes in the transaction."
- "Phantoms causing write skew" **[V]** — https://ebrary.net/64787/computer_science/phantoms_causing_write_skew
  - The write-skew section uses "precondition", not "premise": "The effect of this write changes the precondition of the decision of step 2."
- "Characterizing write skew" **[S]** — https://ebrary.net/64785/computer_science/characterizing_write_skew
  - "You can think of write skew as a generalization of the lost update problem."
- "Compare-and-set" **[V]** — https://ebrary.net/64782/computer_science/compare
  - "The purpose of this operation is to avoid lost updates by allowing an update to happen only if the value has not changed since you last read it. If the current value does not match what you previously read, the update has no effect, and the read-modify-write cycle must be retried."
  - The same page warns that if the `WHERE` clause reads an old snapshot, compare-and-set "may not prevent lost updates" **[V]**.
- CWE-367 (TOCTOU) **[V]** — https://cwe.mitre.org/data/definitions/367.html
  - "The product checks the state of a resource before using that resource, but the resource's state can change between the check and the use in a way that invalidates the results of the check."
- Fowler, *Optimistic Offline Lock* **[V]** — https://martinfowler.com/eaaCatalog/optimisticOfflineLock.html
  - "Optimistic Offline Lock solves this problem by validating that the changes about to be committed by one session don't conflict with the changes of another session."
  - "So long as the validation and the updates occur within a single system transaction the business transaction will display consistency."

**The distinction, in my words.** No source names it this way.
- *Premise* reads: the write is only valid if the value is still true at commit. CAS, a version check, `SELECT … FOR UPDATE` or serializable isolation is needed.
- *Snapshot* reads: the value is recorded "as of the read", for example copied into an event or a journal line. Nothing needs to stay true, so nothing goes stale.
- Kleppmann's point is that the database can't tell which kind a read is, so SSI treats every read as a premise.

### 3. Deferred reporting of events

**MTG Comprehensive Rules.** Mirror: https://friendswith.cards/en/rules/603. That page footer says "Rules effective as of Sep 25, 2026".
- 603.2 **[V]**: "Whenever a game event or game state matches a triggered ability’s trigger event, that ability automatically triggers. The ability doesn’t do anything at this point."
- 603.3 **[V]**: "Once an ability has triggered, its controller puts it on the stack as an object that’s not a card the next time a player would receive priority."
- 603.3b **[V]**: "If multiple abilities have triggered since the last time a player received priority, the abilities are placed on the stack in a two-part process."
- 603.10 **[V]**: "Normally, objects that exist immediately after an event are checked to see if the event matched any trigger conditions, and continuous effects that exist at that time are used to determine what the trigger conditions are and what the objects involved in the event look like. However, some triggered abilities are exceptions to this rule; the game “looks back in time” to determine if those abilities trigger, using the existence of those abilities and the appearance of objects immediately prior to the event."
- 603.10a **[V]**, from https://askjudge.app/rules/603.10a (the friendswith mirror lacked it): "Some zone-change triggers look back in time. These are leaves-the-battlefield abilities, abilities that trigger when a player sacrifices a permanent, abilities that trigger when a card leaves a graveyard, and abilities that trigger when an object that all players can see is put into a hand or library."

**Correction.** MTG does **not** match triggers against the state a step leaves behind.
- Matching happens per event, "immediately after" it, or "immediately prior" to it for look-back triggers.
- Only *stack placement* waits until priority (603.3).
- So MTG is a model of "match at the event's moment, act later", not "match against the final state".
- When the effects run, they read whatever the state is then. That is my inference; see also 603.6, a zone-change trigger looking for the object in its new zone **[S]**.

**Bevy 0.17**
- Release notes **[V]** — https://bevy.org/news/bevy-0-17/
  - "In Bevy 0.17, Event is now exclusively the name/trait for the concept of something that is "triggered" and "observed". Message is the name / trait for something that is "buffered": it is "written" via a MessageWriter and "read" via a MessageReader."
  - The release notes put backticks and links around the type names; I dropped them here.
- `Event` docs **[V]** — https://docs.rs/bevy/0.17.1/bevy/ecs/event/trait.Event.html
  - "This causes any Observer watching for that Event to run immediately, as part of the World::trigger call."
- `Commands` docs **[V]** — https://docs.rs/bevy/0.17.1/bevy/ecs/system/struct.Commands.html
  - "all queued commands are automatically applied in sequence when the ApplyDeferred system runs".
  - My inference: `Commands::trigger` therefore runs observers when the command queue is applied, not inside the system that called it.
- `Message` docs **[V]** — https://docs.rs/bevy/0.17.1/bevy/ecs/message/trait.Message.html
  - "Unlike Events triggered for observers, messages are evaluated at fixed points in the schedule rather than immediately when they are sent. This allows for more predictable scheduling, and deferring message processing to a later point in time."
- `Messages` docs **[V]** — https://docs.rs/bevy/0.17.1/bevy/ecs/message/struct.Messages.html
  - "If no ordering is applied between writing and reading systems, there is a risk of a race condition."
  - "If messages are not handled by the end of the frame after they are updated, they will be dropped silently."
- My inference: a `MessageReader` system that queries components sees the world as it is when the reader runs, not when the message was written. Your risk is this case.

**Events that carry state vs listeners that read it back**
- Fowler, *What do you mean by "Event-Driven"* **[V]** — https://www.martinfowler.com/articles/201701-event-driven.html
  - On Event Notification: "The receiver knows something has changed, may get some minimal information on the nature of the change, but then issues a request back to the sender to decide what to do next."
  - Event-Carried State Transfer is presented as the alternative, where the event contains the data.
- Microsoft, domain events **[V]** — https://learn.microsoft.com/en-us/dotnet/architecture/microservices/microservice-ddd-cqrs-patterns/domain-events-design-implementation
  - "Instead of dispatching to a domain event handler immediately, a better approach is to add the domain events to a collection and then to dispatch those domain events *right before* or *right* *after* committing the transaction".
  - It discusses transaction scope. It does not discuss handlers seeing state from after the event.
- I found no source that uses "temporal coupling" for this specific dependency on what listeners read. The temporal-coupling pages I found cover call ordering, for example https://enterprisecraftsmanship.com/posts/temporal-coupling-and-immutability/ **[S]**.

### 4. Rust borrow granularity

- **Confirmed.** Borrows are fixed by the function signature. With parameters `a: &mut A, b: &mut B`, both are usable in every `match` arm. Nothing narrows them per arm. This is a language fact; the wording is mine.
- Rust Project Goals 2026, *View types experiment* **[V]** — https://goals.rust-lang.org/2026/view-types-experiment.html
  - Status: "Accepted". Contact: Sasha Pourcelot.
  - "Rust currently does not allow tracking borrows across function calls: when a reference to an object is passed to a function, the borrow checker considers the entire object to be used by the function. Developers work around this by passing individual fields of a struct as arguments instead of passing the whole struct."
  - Listed drawback **[V]**: "Methods have to be turned into associated functions or free functions, which sometimes makes the code harder to understand".
- Roadmap *The Borrow Checker Within* (2026) **[V]** — https://goals.rust-lang.org/2026/roadmap-borrow-checker-within.html
  - "enabling methods to borrow disjoint fields of a struct simultaneously (“view types”)".
  - "view types need modeling and have open design questions around strong updates".
- Splitting into helpers, users.rust-lang.org (2018) **[V]** — https://users.rust-lang.org/t/approach-to-modify-a-struct-using-helpers-within-the-struct/16012
  - "`apply_turn(&mut self, ...)` is probably fine itself - it’s the entrypoint for mutation. Once you’re inside apply_turn, you can borrow individual fields of `self` separately."
  - "Instead of having “one big self” container of data that you borrow from, borrow from the individual fields."
- Niko Matsakis posts (verified earlier, cited only):
  - https://smallcultfollowing.com/babysteps/blog/2021/11/05/view-types/
  - https://smallcultfollowing.com/babysteps/blog/2024/06/02/the-borrow-checker-within/
  - https://smallcultfollowing.com/babysteps/blog/2025/02/25/view-types-redux/ ("View types redux and abstract fields", 2025-02-25)
- Also from 2025: Pak et al., "View Types in Rust", SPLASH Companion '25 **[S]** — https://dl.acm.org/doi/10.1145/3758316.3765481

**Practical pattern** (my inference): have each arm call a helper that takes only the one `&mut` it should write. This is the "pass only the fields" workaround the goal document describes.

## Sources
- **Kept:** Barnett et al. JOT 2004 PDF and Leino–Müller 2004 PDF — primary sources for Q1a.
- **Kept:** Evans *DDD Reference* (CC-BY) and Vernon essay Part I — primary DDD sources.
- **Kept:** Microsoft .NET DDD guide — the only source found for local invariants on inner entities.
- **Kept:** ebrary DDIA pages — a mirror, but the text matches the book's section titles.
- **Kept:** CWE-367 and Fowler's PoEAA catalog entry.
- **Kept:** friendswith.cards and askjudge — unofficial mirrors of the Comprehensive Rules.
- **Kept:** Bevy release notes and docs.rs 0.17.1; Rust Project Goals 2026.
- **Dropped:** Medium and Chinese DDIA notes (second-hand), Baeldung and seedstack (they restate Evans), the Unity forum (off-topic), Meyer's arXiv PDF (I could not extract it).

## Gaps / claims I could not verify
1. Evans blue-book wording ("ultimately responsible for checking invariants", and the "Invariants … will involve relationships between members of the AGGREGATE" line). I saw them only second-hand.
2. Meyer OOSC2 §11.14, "Indirect Invariant Effect": no primary text reached. Meyer 2016/2024 paper bodies: not read.
3. Müller's universe-types thesis itself. I only have Leino & Müller's summary of it.
4. Whether Spec#/Boogie or Leino–Müller *prefer* nested owners over one owner. They say nothing explicit, so the answer is open.
5. A source that explicitly separates "the write needs the value still current" from "the write wants it as of the read". None found.
6. A source naming the risk of listeners reading state that changed after the event, under "temporal coupling" or batching. None found.
7. DDIA wording against the official O'Reilly edition, or against the 2nd edition.
8. MTG rule text against the official Wizards PDF. I used mirrors; the friendswith footer date is "Sep 25, 2026".
9. The exact timing of `Commands::trigger` observers in Bevy 0.17. This is inferred from the `Commands`/`ApplyDeferred` docs.
10. The SPLASH '25 "View Types in Rust" paper. Seen in a snippet only.