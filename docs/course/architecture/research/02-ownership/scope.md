Researcher report, session 02. Kept in full.

# Scope: ownership, invariants, application vs domain

## 1. Catherine West (kyren), RustConf 2018 closing keynote
Source (primary, author's blog, "very lightly edited text version" of the long-form draft, not a transcript of the delivered talk): https://kyren.github.io/2018/09/14/rustconf-talk.html

### 1a. Shape of game state, systems as functions
- "you can imagine the structure of these games more or less as a giant static global struct containing “all of the game state”, visible to all of the game’s code."
- Code comment in her sample: "Let's have a series of functions change the state of the game_state based on the previous game state and the input. We'll be very very fancy, and we'll give this very SIMPLE pattern an overly fancy name, and we'll call each of these functions a "system"."
- "Rust is rather good at letting you split borrows for different fields of a public struct, so since everything in our pretend-engine is just a giant nested tree of entirely public structs, you should always be able to read the state of the game and change exactly as much of the game as necessary in each system."
- "Let’s refer to this as the “UR-game-architecture”"
- Hedge: "So I’m NOT recommending that you make games this way!" and "But still, I’m genuinely not advocating this."
- Later: "I might have many “systems” (which remember here are just regular functions) in different files"

### 1b. What she criticizes
- Target is OO design applied to game entities / application-level structure, not data hiding per se. She lists encapsulation among OO principles: "Encapsulation - You should bind data together with the functions that operate on it, keeping both safe from outside interference and misuse."
- Explicitly keeps: "Data hiding - Being able to hide data to maintain invariants is invaluable, making a safe interface to unsafe code would not be possible without it. Very useful in the small, and with “library” code."
- Accessor growth: "It feels like every time a new requirement comes in, you have to take what may have once been sane interfaces and “poke more holes through them”." and "Lots of code has no clear place to be because it concerns multiple entities at once"
- "Data hiding has very limited utility for a game outside of just maintaining invariants at the edges of the code, where things are smaller and more contained. A lot, possibly even the vast majority of the interesting behavior in your game ends up spanning many data types, and does not naturally “belong” to any specific entity."
- Takeaway: "at the point where you’re not writing “library” code and are actually writing a game, data hiding is NOT generally useful, and just wastes a lot of effort and time."
- "Most behavior doesn’t “attach” to any data ... Stop conflating the data representation of your game with the systems that operate on it!"
- Borrowing: "This would be vastly easier if simply World was a struct with plain public members. This becomes MORE important the larger the structure is, and data hiding becomes less important the more into the “application” level you are."
- Interfaces: "They _amplify_ borrowing problems by _amplifying_ the amount you are forced to borrow!"
- Summary line: "Useful things like the dot operator and data hiding to maintain invariants in library-ish code aside, the large scale structure of our game is not really helped by OO."
Conclusion: she criticizes OO entity design / encapsulation at application scale. She endorses encapsulation for invariants in library code.

### 1c. Invariants, who may modify what, cost of all-public
She does discuss invariants:
- Cost of all-public: "Clearly there is a huge, gaping, obvious disadvantage in that the entire game state is visible and mutable to all systems and if something changes unexpectedly, you might have no idea where to look to find what is changing the game state improperly. Nobody is _really_ suggesting in the modern era that you write software with all state as effectively global."
- Strawman OO reasoning she voices: "What if there’s an invariant being held, where if the health drops below some value, some animation state is triggered? What about damage? Certainly the single responsibility principle states that a _monster_ shouldn’t be in charge of setting a _player’s_ health."
- Damage ownership: "Hmm, where should the damage system for players go? Probably a player should reduce their health since it’s _their_ health… or maybe monsters could do it since it’s _their_ damage region? I dunno, I’ll say it’s the player’s job."
- Sum types: "This is interesting, because it makes encapsulation less important, even with everything public, there are certain invariants that cannot be broken. There ARE invariants that aren’t expressible in this way though, an example being that the array of player ids probably need to point to Player type entities only!"
- "It’s clear that we lose some invariants here in that more potentially invalid entities could be created"
- SoA: "There are a few new invariants to maintain, namely that each component Vec is the same length"
- "Is it still an ECS if my components have methods? Yes, adding a component method is not the end of the world, especially if there is a small local invariant to maintain. SQL is all about data, but it also has stored procedures to maintain invariants."
- "We need some way to deal with the downsides of a single giant mutable public nested struct without making an OO mess."
[NOT FOUND] any rule assigning write-ownership of a field to one system.

### 1d. Why ECS / what it answers
- "The biggest problem we haven’t addressed from earlier is still that everything is kind of global still. More so, every “system” (for us, this is still just a fancy name for plain functions) depends on _all_ of the types that go into our game state, which may be quite large. ... still changing anything inside “GameState” potentially would affect every system at least _theoretically_."
- "I want to emphasize before we go on that this, like many steps before, is _optional_."
- Physics duplication: "the `physics_system` definitely _for sure_ has to understand that there are 3 separate entity types that all have physics. Whenever we add an entity type, probably this system has to change its implementation"
- Dynamic typing benefit: "you can add this component without “disturbing” your other systems, because without importing the new module, they can’t possibly “see” that the ECS has such a component anyway." She adds: "The justification for this may seem pretty weak, and it kind of is."
- "the two patterns go together well to limit the problem of “everything depending on everything else”."
- "I’m convinced that “ECS design” in games is so great because it forces you to think about your data rather than be stuck in OO mode, not because of the magic of “structs of arrays”."
- "ECS is just SQL for games."

### 1e. Indexes / generational indexes
- "everywhere where there would have been a pointer, instead store an index into some array."
- "you’re trading UB and potential crashes with pointers for “random but unspecified” behavior if you access the wrong or outdated index, and potentially panics. You’d be right, btw!"
- "You can do a LOT with indexes into Vecs. This is way easier than self borrowing or Rc<RefCell>."
- "It is not worth it, move on with your life, and just use Vecs and indexes."
- "We’ll get a “random other entity” in its place without being able to tell that it was in fact removed out from under us!"
- "Generational indexes are never re-used because the generation will always increment, yet the “real indexes” will always be “small”"
- On the allocator: "(There are faster ways to implement this, but this one is actually pretty much fine. Also note that this is an obviously GOOD use of data hiding)."

## 2. Meyer, class invariant
Primary: Meyer, "Applying 'Design by Contract'", IEEE Computer 1992, https://se.inf.ethz.ch/~meyer/publications/computer/contract.pdf (OCR text; section "Class invariants")
- "A class invariant is a property that applies to all instances of the class, transcending particular routines."
- "The invariant must be satisfied after the creation of every instance of the class ... This means that every creation procedure of the class must yield an object satisfying the invariant." (The text extractor did not return this exact phrase as a contiguous match, so check the PDF before quoting.)
- "The invariant must be preserved by every exported routine of the class (that is to say, every routine available to clients). Any such routine must guarantee that the invariant is satisfied on exit if it was satisfied on entry."
- "In effect, then, the invariant is added to the precondition and postcondition of every exported routine of the class."
- "Observable states ... are the states that immediately follow object creation, and any states subsequently reached after the execution of an exported routine of the object’s generating class. The invariant is the consistency constraint on observable states. (It is not necessarily satisfied in between these states.)"
- Note: "temporarily violated inside a routine" is not literal wording. The parenthetical above is the closest.
Primary: eiffel.org, https://www.eiffel.org/doc/eiffel/ET-_Design_by_Contract_%28tm%29%2C_Assertions_and_Exceptions
- "a class invariant must be ensured by every creation procedure (or by the default initialization if there is no creation procedure), and maintained by every exported routine of the class."
- "It must be satisfied on exit by any creation procedure, and is implicitly added to both the precondition and postcondition of every exported routine."
- Monitoring option: "`invariant` : same as `ensure`, plus class invariant on both entry and exit for qualified calls."
- OOSC2 1997 wording: [NOT FOUND] online in a legitimate copy.

## 3. Evans, application layer
- DDD Reference 2015 (https://www.domainlanguage.com/wp-content/uploads/2016/05/DDD_Reference_2015-03.pdf), fetched: "kept thin" / "coordinates tasks" / "does not contain business rules" are ABSENT. Closest text (Layered Architecture, p. 10): "Isolate the expression of the domain model and the business logic, and eliminate any dependency on infrastructure, user interface, or even application logic that is not business logic." and "Concentrate all the code related to the domain model in one layer and isolate it from the user interface, application, and infrastructure code. The domain objects, free of the responsibility of displaying themselves, storing themselves, managing application tasks, and so forth, can be focused on expressing the domain model." Repositories section: "Domain logic moves into queries and application layer code, and the entities and value objects become mere data containers."
- [SECONDARY] 2003 book, via Fowler https://martinfowler.com/bliki/AnemicDomainModel.html (no page given; "[his name for Service Layer]" is Fowler's insertion): "Application Layer: Defines the jobs the software is supposed to do and directs the expressive domain objects to work out problems. The tasks this layer is responsible for are meaningful to the business or necessary for interaction with the application layers of other systems. This layer is kept thin. It does not contain business rules or knowledge, but only coordinates tasks and delegates work to collaborations of domain objects in the next layer down. It does not have state reflecting the business situation, but it can have state that reflects the progress of a task for the user or the program." Domain Layer: "Responsible for representing concepts of the business, information about the business situation, and business rules. State that reflects the business situation is controlled and used here, even though the technical details of storing it are delegated to the infrastructure. This layer is the heart of business software." Same text on Microsoft Learn: https://learn.microsoft.com/en-us/dotnet/architecture/microservices/microservice-ddd-cqrs-patterns/ddd-oriented-microservice. Book page: [NOT FOUND].

## 4. Evans, aggregates and invariants
Primary: DDD Reference 2015, "Aggregates", p. 16:
- "It is difficult to guarantee the consistency of changes to objects in a model with complex associations. Objects are supposed to maintain their own internal consistent state, but they can be blindsided by changes in other objects that are conceptually constituent parts."
- "Choose one entity to be the root of each aggregate, and allow external objects to hold references to the root only (references to internal members passed out for use within a single operation only). Define properties and invariants for the aggregate as a whole and give enforcement responsibility to the root or some designated framework mechanism. Use the same aggregate boundaries to govern transactions and distribution. Within an aggregate boundary, apply consistency rules synchronously. Across boundaries, handle updates asynchronously."
- Assertions, p. 22: "State post-conditions of operations and invariants of classes and aggregates." "Assertions define invariants on aggregates."
- "spans AGGREGATES" sentence is ABSENT from the Reference.
[SECONDARY] 2003 book ch. 6, cited "DDD p128" by Vernon Part II (https://www.dddcommunity.org/wp-content/uploads/files/pdf_articles/Vernon_2011_2.pdf): "Any rule that spans AGGREGATES will not be expected to be up-to-date at all times. Through event processing, batch processing, or other update mechanisms, other dependencies can be resolved within some specific time."
[SECONDARY, blogs only] fuller passage: "Invariants, which are consistency rules that must be maintained whenever data changes, will involve relationships between members of the AGGREGATE. Any rule that spans AGGREGATES will not be expected to be up-to-date at all times. Through event processing, batch processing, or other update mechanisms, other dependencies can be resolved within some specified time. But the invariants applied within an AGGREGATE will be enforced with the completion of each transaction." (https://mcode.it/blog/2021-02-26-du_invariants/, https://medium.com/@philsarin/whats-the-point-of-the-aggregate-pattern-741a3132da5c). Discrepancy: "specific" (Vernon) vs "specified" (blogs). Check against the book.

## 5. Vernon
Part I, primary, https://www.dddcommunity.org/wp-content/uploads/files/pdf_articles/Vernon_2011_1.pdf, "Rule: Model True Invariants In Consistency Boundaries":
- "When trying to discover the aggregates in a bounded context, we must understand the model's true invariants. Only with that knowledge can we determine which objects should be clustered into a given aggregate."
- "An invariant is a business rule that must always be consistent. There are different kinds of consistency. One is transactional, which is considered immediate and atomic. There is also eventual consistency. When discussing invariants, we are referring to transactional consistency."
- "The consistency boundary logically asserts that everything inside adheres to a specific set of business invariant rules no matter what operations are performed. The consistency of everything outside this boundary is irrelevant to the aggregate. Thus, aggregate is synonymous with transactional consistency boundary."
- "Therefore, aggregates are chiefly about consistency boundaries and not driven by a desire to design object graphs."
- "the large cluster aggregate was designed with false invariants in mind, not real business rules. These false invariants are artificial constraints imposed by developers."
Part II, primary, same site, Vernon_2011_2.pdf, printed p. 9, under "Rule: Use Eventual Consistency Outside the Boundary":
- "Discussing this with Eric Evans revealed a very simple and sound guideline. When examining the use case (or story), ask whether it's the job of the user executing the use case to make the data consistent. If it is, try to make it transactionally consistent, but only by adhering to the other rules of aggregate. If it is another user's job, or the job of the system, allow it to be eventually consistent. That bit of wisdom not only provides a convenient tie breaker, it helps us gain a deeper understanding of our domain. It exposes the real system invariants: the ones that must be kept transactionally consistent."
Part III: not fetched.

## 6. Read-only computations vs owners of state
- CQS, Meyer, primary: *Eiffel: The Language* 3rd ed. draft, 2005, https://se.inf.ethz.ch/~meyer/ongoing/etl/feature.pdf: "A function should not change any object, except if the change only affects an object’s representation, not its abstract properties. Because language processing tools cannot easily know which properties are abstract, the ban on object-modifying functions is a methodological guideline — the Command-Query Separation principle — and not a language rule." and "The Command-Query separation principle, which suggests that queries should not change objects."
- [SECONDARY] OOSC2 p. 751 (snippets of an unofficial PDF only): "Command-Query Separation principle: Functions should not produce abstract side effects." and "asking a question should not change the answer."
- eiffel.org (not Meyer-authored): "You cannot reason about the integrity of an object if the act of asking a question changes the object." https://www.eiffel.org/doc/eiffel/Design_by_Contract_and_Assertions
- CQRS, Greg Young, primary: "Command and Query Responsibility Segregation uses the same definition of Commands and Queries that Meyer used and maintains the viewpoint that they should be pure. The fundamental difference is that in CQRS objects are split into two objects, one containing the Commands one containing the Queries." https://cqrs.wordpress.com/documents/cqrs-introduction/
- Functional core, imperative shell, Bernhardt 2012, primary: "This functional core is surrounded by a shell of imperative code: it manipulates stdin, stdout, the database, and the network, all based on values produced by the functional core." https://www.destroyallsoftware.com/screencasts/catalog/functional-core-imperative-shell/
- [NOT FOUND] a named principle stating that "a reader should not own the data". CQS and CQRS separate operations and models. Neither assigns ownership.

## Gaps
- 2003 book pages for the "kept thin" and "spans AGGREGATES" passages are unverified.
- The OOSC2 1997 text was not available from a legitimate source.
- Vernon Part III was not fetched.
- The PDF text extractor split some words at line breaks; quotes were rejoined.
