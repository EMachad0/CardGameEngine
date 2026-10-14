# General architecture literature

Researcher report, session 00. Kept in full.

## Summary
Almost every item has primary-source wording; exceptions are flagged **[UNVERIFIED]** or **[SECONDARY]**. Three findings correct common teaching: (a) Stevens/Myers/Constantine (1974) call intramodule strength "binding", not "cohesion", and the well-known coupling ladder (data/stamp/control/external/common/content) is generally attributed to Myers (1975), not that paper; (b) Fowler's "ContractTest" is a different idea from Rainsberger's "contract tests"; (c) Vernon himself calls his four aggregate rules "rules of thumb" and lists reasons to break them. Few sources give truly unconditional "where code belongs" rules; the cleanest are Martin's Dependency Rule/ADP and Cockburn's inside/outside rule.

## Findings

**1. Parnas (1972), KWIC paper.**
- Criterion: "We propose instead that one begins with a list of difficult design decisions or design decisions which are likely to change. Each module is then designed to hide such a decision from the others." Opening the same paragraph: "it is almost always incorrect to begin the decomposition of a system into modules on the basis of a flowchart."
- Modularization 1: Input, Circular Shift, Alphabetizing, Output, Master Control — "the criterion used was to make each major step in the processing a module."
- Modularization 2: Line Storage, Input, Circular Shifter, Alphabetizer, Output, Master Control — "made using 'information hiding' as a criterion. The modules no longer correspond to steps in the processing." "Every module in the second decomposition is characterized by its knowledge of a design decision which it hides from all others."
- The word "secret" does not appear in the extracted text (OCR spacing normalized). [PDF](https://www.win.tue.nl/~wstomv/edu/2ip30/references/criteria_for_modularization.pdf)

**2. Ousterhout, *A Philosophy of Software Design* (2nd ed.).** Book summary/chapter openings ([unofficial reproduction](https://yingang.github.io/aposd2e-zh/en/summary.html)); cross-checked against Ousterhout's own [CS190 lecture notes](https://web.stanford.edu/~ouster/cgi-bin/cs190-winter18/lecture.php?topic=modularDesign).
- Deep: "The best modules are those that provide powerful functionality yet have simple interfaces. I use the term deep to describe such modules." [SECONDARY, multiple readers' notes agree verbatim]
- Shallow Module (red flag list): "the interface for a class or method isn't much simpler than its implementation."
- Information hiding: "each module should encapsulate a few pieces of knowledge, which represent design decisions. The knowledge is embedded in the module's implementation but does not appear in its interface." [SECONDARY]
- Information Leakage: "a design decision is reflected in multiple modules."
- Temporal Decomposition: "the code structure is based on the order in which operations are executed, not on information hiding." Ch. 5 conclusion: "think about the different pieces of knowledge that are needed... design each module to encapsulate one or a few of those pieces of knowledge."
- Pass-Through Method: "a method does almost nothing except pass its arguments to another method with a similar signature."
- Classitis: "stems from the mistaken view that 'classes are good, so more classes are better.'" [SECONDARY]; lecture notes: "Classitis: small classes taken to the extreme"; "Classes in the range of 200-2000 lines are fine."
- Different layer, different abstraction: "If a system contains adjacent layers with similar abstractions, this is a red flag."
- General-purpose: "general-purpose interfaces are simpler and deeper than special-purpose ones"; sweet spot "somewhat general-purpose": "the module's functionality should reflect your current needs, but its interface should not."
- Define errors out of existence: "the semantics of operations can be modified so that the normal behavior handles all situations and there is no exceptional condition to report."
- All 14 red flags: Shallow Module, Information Leakage, Temporal Decomposition, Overexposure, Pass-Through Method, Repetition, Special-General Mixture, Conjoined Methods, Comment Repeats Code, Implementation Documentation Contaminates Interface, Vague Name, Hard to Pick Name, Hard to Describe, Nonobvious Code.
- Framed as "symptoms" that "suggest" a problem: heuristics, not laws.

**3. Constantine et al., cohesion and coupling.** Original paper (Stevens, Myers, Constantine, *IBM Systems Journal* 1974): "Binding is the measure of the cohesiveness of a module." Binding scale (weakest to strongest): coincidental, logical, temporal, communicational, sequential, functional — "Functional binding is the strongest type of binding." "The scale is not linear." "Control coupling" named; data/stamp/common/content coupling **not found** in the (partly garbled) extraction. [PDF](https://bitsavers.trailing-edge.com/pdf/ibm/IBM_Systems_Journal/132/ibmsj1302C.pdf). "Procedural" cohesion appears in the 7-level scale of *Structured Design* (Yourdon & Constantine, 1979) [SECONDARY]. Coupling ladder data, stamp, control, external, common, content usually attributed to Myers, *Reliable Software Through Composite Design* (1975) [SECONDARY, UNVERIFIED].

**4. Martin's package principles and the Dependency Rule.** ([butunclebob](http://butunclebob.com/ArticleS.UncleBob.PrinciplesOfOod))
- REP "The granule of reuse is the granule of release."
- CCP "Classes that change together are packaged together."
- CRP "Classes that are used together are packaged together."
- ADP "The dependency graph of packages must have no cycles."
- SDP "Depend in the direction of stability."
- SAP "Abstractness increases with stability."
- A "package" here means "a binary deliverable like a .jar file, or a dll as opposed to a namespace", i.e. a Rust crate, not a `mod`.
- Dependency Rule: "source code dependencies can only point inwards. Nothing in an inner circle can know anything at all about something in an outer circle." And: "There's no rule that says you must always have just these four [circles]. However, The Dependency Rule always applies." [Clean Architecture post](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)

**5. Cockburn, Hexagonal Architecture (2005).**
- Port: "A port identifies a purposeful conversation." "The protocol for a port is given by the purpose of the conversation... The protocol takes the form of an application program interface (API)."
- Adapter: "For each external device there is an adapter that converts the API definition to the signals needed by that device and vice versa."
- Core rule: "code pertaining to the inside part should not leak into the outside part."
- Primary/secondary: "ports and adapters show up in two flavors, which I'll call primary and secondary... They could be also called driving adapters and driven adapters." "A primary actor is an actor that drives the application... A secondary actor is one that the application drives." "The distinction... lies in who triggers or is in charge of the conversation." (He applies primary/secondary to both ports and adapters; driving/driven are his words for adapters.)
- His example: FIT discount app, `discount(amount) = amount * rate(amount)`; a FIT test or GUI drives one port; a constant, then a mock, then a real database serve the rate on the other. [Article](https://alistair.cockburn.us/hexagonal-architecture/)

**6. Bernhardt, Functional Core, Imperative Shell (2012-07-12).** A Twitter client whose "core is functional: managing tweets, syncing timelines..., rendering tweets to text," surrounded by "a shell of imperative code: it manipulates stdin, stdout, the database, and the network, all based on values produced by the functional core." The pure core "often naturally allows isolated testing with no test doubles," and the shell has "few conditionals." [DAS](https://www.destroyallsoftware.com/screencasts/catalog/functional-core-imperative-shell)

**7. DDD.** Evans, *DDD Reference* (2015) ([PDF](https://www.domainlanguage.com/wp-content/uploads/2016/05/DDD_Reference_2015-03.pdf)):
- Aggregate: "Cluster the entities and value objects into aggregates and define boundaries around each. Choose one entity to be the root of each aggregate, and allow external objects to hold references to the root only." Invariants enforced by "the root or some designated framework mechanism."
- Aggregate root: no separate entry; it is the "root" entity above.
- Bounded context: "A description of a boundary (typically a subsystem, or the work of a particular team) within which a particular model is defined and applicable."
- Ubiquitous language: "A language structured around the domain model and used by all team members within a bounded context."
- Entity: "When an object is distinguished by its identity, rather than its attributes, make this primary to its definition in the model."
- Value object: "When you care only about the attributes and logic of an element of the model, classify it as a value object... Treat the value object as immutable."
- Anti-corruption layer: "As a downstream client, create an isolating layer to provide your system with functionality of the upstream system in terms of your own domain model."
- Shared kernel: "Designate with an explicit boundary some subset of the domain model that the teams agree to share. Keep this kernel small."
- Context map: "Identify each model in play on the project and define its bounded context... Describe the points of contact between the models."

Vernon, *Effective Aggregate Design* (2011): four rules, Part III summary: "Model True Invariants In Consistency Boundaries"; "Design Small Aggregates"; "Reference Other Aggregates By Identity"; "Use Eventual Consistency Outside the Boundary (after asking whose job it is)." [Part III](https://www.dddcommunity.org/wp-content/uploads/files/pdf_articles/Vernon_2011_3.pdf). "aggregate is synonymous with transactional consistency boundary"; one aggregate per transaction "is a rule of thumb" ([Part I](https://www.dddcommunity.org/wp-content/uploads/files/pdf_articles/Vernon_2011_1.pdf)). Part II "Reasons To Break the Rules": UI convenience, lack of technical mechanisms, global transactions, query performance ([Part II](https://www.dddcommunity.org/wp-content/uploads/files/pdf_articles/Vernon_2011_2.pdf)).

**8. Package by feature vs by layer.**
- Martin, Screaming Architecture (2011): "When you look at the top level directory structure... do they scream: Health Care System... Or do they scream: Rails." [Post](https://blog.cleancoder.com/uncle-bob/2011/09/30/Screaming-Architecture.html)
- Bogard, Vertical Slice Architecture (2018): "Minimize coupling between slices, and maximize coupling in a slice." "New features only add code, you're not changing shared code." Caveat: it "assume[s] that your team understands code smells and refactoring." [Post](https://www.jimmybogard.com/vertical-slice-architecture/)
- Simon Brown, Package by Component (also *Clean Architecture* ch. 34): calls by-layer and by-feature both "suboptimal, with teams just switching from one extreme to the other." Key point: "if you make all types... public, the packages are simply an organisation mechanism... rather than being used for encapsulation"; wants to "use the compiler to enforce my architecture." Warns of the "Périphérique anti-pattern" (adapters calling each other directly when all infrastructure is in one tree). [Post](https://simonbrown.je/modular-monolith/)
- Strongest case for by-feature: change locality, domain visibility, more types can be private. For by-layer: technical-concern separation, easy enforcement of dependency direction; Brown's "packages are just folders if everything is public" applies to both (synthesis).

**9. Contract tests and "listen to the tests".**
- Rainsberger 2005 ([In brief](https://blog.thecodewhisperer.com/permalink/in-brief-contract-tests)): "a contract test case class is abstract... then I extend/subclass it and implement a Factory method or two to return instances of my own implementation." Later contrasts these tests of "the semantics of the contract" with tools like Pact that check "only... the syntax" ([2017](https://blog.thecodewhisperer.com/permalink/getting-started-with-contract-tests)).
- Fowler's ContractTest is a *different* idea: periodically checking that "calls against your test doubles return the same results as a call to the external service would" (renamed from "Integration Contract Test" in 2018). [Bliki](https://martinfowler.com/bliki/ContractTest.html)
- "Role tests": used by practitioners ([example](https://emmanuelvalverderamos.substack.com/p/role-tests-are-not-about-comparing)); coinage not verified (Sandi Metz, *POODR* ch. 9 is the likely source) [UNVERIFIED].
- Freeman & Pryce, GOOS ch. 20 "Listening to the Tests": "Sometimes we find it difficult to write a test... this usually means that our design can be improved—perhaps the class is too tightly coupled to its environment or does not have clear responsibilities." "we don't just ask ourselves how to test it, but also why is it difficult to test." [SECONDARY: O'Reilly preview snippet; page 403] ([O'Reilly](https://www.oreilly.com/library/view/growing-object-oriented-software/9780321574442/ch20.html))

**10. Known tensions.**
- Ousterhout vs Martin joint document (2024–25): Ousterhout says Clean Code "encourages programmers to create teeny-tiny methods that suffer from both shallow interfaces and entanglement." Martin: "We both value decomposition, and we both avoid entanglement; but we disagree on the relative weighting of those two values." They also disagree on comments and TDD. [GitHub](https://github.com/johnousterhout/aposd-vs-clean-code)
- Metz: "duplication is far cheaper than the wrong abstraction"; "prefer duplication over the wrong abstraction." [Blog](https://sandimetz.com/blog/2016/1/20/the-wrong-abstraction)
- Rule of three: Fowler, *Refactoring* (1999), crediting Don Roberts: "The third time you do something similar, you refactor." [SECONDARY] ([quote](https://bobnadler.com/articles/2014/01/26/the-rule-of-three.html))
- Ousterhout's "Repetition" red flag and "general-purpose is deeper" sit in tension with Metz and the rule of three.

**11. Data-oriented design.**
- Acton, CppCon 2014: "The purpose of all programs, and all parts of those programs, is to transform data from one form to another." "If you don't understand the data, you don't understand the problem." "Where there is one, there are many." "Software does not run in a magic fairy aether powered by the fevered dreams of CS PhDs." Three lies: "Software is a platform," "Code designed around model of the world," "Code is more important than data." [SECONDARY: transcribed in ACCU Overload 167; video not accessed](https://accu.org/journals/overload/30/167/teodorescu/)
- Fabian, *Data-Oriented Design* ([online](https://www.dataorienteddesign.com/dodbook/node2.html)): "The first principle: Data is not the problem domain." "The second principle: Data is the type, frequency, quantity, shape, and probability." Normalization: "as we normalise our data is a tendency to split data by dependency"; normalized tables have "only rows for things which matter" ([ch. 2](https://www.dataorienteddesign.com/dodbook/node3.html)). Existence-based processing ("Existential Processing" in the current edition) "attempts to provide a way to remove unnecessary querying about whether or not to process your data" ([ch. 3](https://dataorienteddesign.com/dodbook/node4.html)).
- Bearing on where code goes: DOD groups code by *transform over a data table*, not by domain noun, which works against "behaviour inside entities". Fabian also says it is "product and work-flow specific," not something you add as a library.

## Unconditional vs heuristic
Stated without caveats by their authors: Martin's Dependency Rule ("always applies"); ADP ("must have no cycles"); Cockburn's "inside should not leak into the outside"; Evans's "external objects ... hold references to the root only" (pattern directive, no listed exceptions). Near-unconditional: Parnas, "almost always incorrect" to decompose by flowchart. Heuristics with known exceptions: Ousterhout's red flags ("symptoms"; "Taking it too far" sections in ch. 5 and 10), Vernon's rules ("rules of thumb", four listed reasons to break), CCP/CRP/SDP/SAP (competing forces per Martin, not quoted here), Constantine's scales, Metz, rule of three, Clean Code function length, package-by-feature/layer/component (Bogard and Brown both state preconditions), FC/IS, DOD (Fabian: "product and work-flow specific").

## Sources
Kept: primary texts by Parnas, Cockburn, Martin, Bernhardt, Evans, Vernon, Rainsberger, Fowler, Metz, Bogard, Brown, Fabian; Ousterhout's own lecture notes and joint doc; the 1974 IBM SJ paper. Kept with flag: unofficial APOSD reproduction (summary and chapter openings only), readers' notes for exact Ousterhout sentences, ACCU Overload for Acton, Wikipedia/secondary for Myers and the rule of three. Dropped: SEO summaries and generic coupling tutorials.

## Gaps
No direct access to the APOSD subsections, GOOS ch. 20, Acton's video, *Refactoring*, or Myers 1975; check those quotes against physical copies before relying on them. The 1974 PDF lost pages, so its coupling terms are unconfirmed. "Role tests" coinage not established. Martin's own wording on the REP/CCP/CRP tension diagram (*Clean Architecture*) not pulled.
