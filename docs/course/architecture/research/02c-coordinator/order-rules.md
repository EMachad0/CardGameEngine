# Order rules: procedure vs domain order (research)

Marks: VERIFIED = wording read at the URL in this session. SECONDARY = from a search snippet,
a mirror, or someone else's report.

## 1. Connascence of execution and timing

- VERIFIED. https://connascence.io/execution.html
  "Connascence of execution is when the order of execution of multiple components is important.
  Common examples include locking and unlocking resources, where locks must be acquired and
  released in the same order everywhere in the entire codebase."
  "Connascence of execution can also occur when using objects that encapsulate a state machine,
  and that state machine only allows certain operations in certain states."
  Example: `email.send()` followed by `email.setSubject("Hello World")`. "The setSubject method
  cannot be called after the send method (at best it will do nothing)."
- VERIFIED. https://connascence.io/timing.html (the whole page is one sentence)
  "Connascence of timing is when the timing of the execution of multiple components is important."
  No example on the page.
- VERIFIED. https://connascence.io/pages/about.html
  The site is not written by Weirich or Rutherford. It lists them as influences: Rutherford's
  posts "have inspired much of the content on this website", and Josh Robb's 2015 talk "was the
  inspiration to build this site".
- NOT READ this session. Page-Jones, "Comparing techniques by means of encapsulation and
  connascence", CACM 35(9), Sept 1992, pp. 147-151. https://doi.org/10.1145/130994.131004
  (HTTP 403). Session 01's research read the paper and found that it does not name execution or
  timing (`research/01-coupling-and-hiding/sources.md`, 1b). Those kinds come later.

## 2. Seemann, "Design Smell: Temporal Coupling" (2011-05-24)

- VERIFIED. https://blog.ploeh.dk/2011/05/24/DesignSmellTemporalCoupling/
  Definition: "temporal coupling, which occurs when there's an implicit relationship between two,
  or more, members of a class requiring clients to invoke one member before the other."
  Examples: `EndpointAddressBuilder` (needs `Uri` set before `ToEndpointAddress()`), and a
  `Smell` class with `Initialize(string name)` then `Spread()`, where calling `Spread()` first
  throws `InvalidOperationException`. "The problem with the Smell class is that it doesn't
  properly protect its invariants. In other words, encapsulation is broken."
  Fix 1, constructor injection: "the name must be requested via the constructor".
  Fix 2, abstract factory: "Instead of mutating the (inconsistent) state of a class, the Create
  method (formerly known as Initialize) returns a new instance of the IFragrance interface".
  "improper use of the API can now be detected by the compiler instead of at run-time."
  The phrase "return a value the next step needs" is not in this post.
- VERIFIED. https://blog.ploeh.dk/2021/07/28/referential-transparency-fits-in-your-head/
  "In the terminology of Command Query Separation, Commands cause trouble. Queries, on the other
  hand, tend to cause little trouble. They return data which you can use as input for other
  Queries." "If the output of one function fits as the input for another, you can sequentially
  compose them."
- SECONDARY (search snippet of https://blog.ploeh.dk/page44/): on MazeBuilder, "the implied
  sequence coupling between members: you're expected to call BuildMaze before any of the other
  methods. A better design would be to remove that explicit step entirely, or else turn it into a
  factory that you have to call in order to be able to call the other methods."

## 3. Sequential coupling

- VERIFIED. https://en.wikipedia.org/wiki/Sequential_coupling
  "sequential coupling (also known as temporal coupling) is a form of coupling where a class
  requires its methods to be called in a particular sequence. This may be an anti-pattern,
  depending on context." "Methods whose name starts with Init, Begin, Start, etc. may indicate
  the existence of sequential coupling." Remedy: "Sequential coupling can be refactored with the
  template method pattern". It cites Andriy Buday, "Refactor: Sequential Coupling => Template
  Method", CodeProject, 2010. That page could not be extracted, so it is unread.
- SECONDARY (O'Reilly snippet, Mastering PHP Design Patterns, ch. 2):
  "Sequential coupling is where you create a class that has methods that must be called in a
  particular order."

## 4. A test for domain order vs procedural order

- Result: none of the sources found states an explicit test. The closest are below.
- VERIFIED (Evans, DDD 2003, ch. 5, as reproduced in
  https://github.com/Dawn-Breaker/Domain-Driven-Design-zh/blob/master/docs/ch5.md):
  "The application layer is responsible for ordering the notification. The domain layer is
  responsible for determining if a threshold was met". Also: "If a SERVICE were devised to make
  appropriate debits and credits for a funds transfer, that capability would belong in the domain
  layer. Funds transfer has a meaning in the banking domain language". This is a separation of
  responsibilities, not a test for order.
- VERIFIED. Khorikov, "Domain services vs Application services" (2016)
  https://enterprisecraftsmanship.com/posts/domain-vs-application-services/
  "domain logic is everything that is related to business decisions." "application services
  orchestrate those decisions". "What matters is whether or not those lines of code are
  responsible for making business decisions." On two calls: "The order in which these two methods
  are called doesn't matter either. If we were to re-arrange them [...] nothing would change in
  terms of the Atm's invariants, it would still remain valid. It's a strong sign there are no
  leaking implementation details". Branching as a sign of decisions: "There are no branches (if
  statements) that signalize the code makes any decisions". Flow: prepare data, execute the
  business decisions, then "Apply the results of the operation to the outside world." When a
  charge must succeed before cash is dispensed, the `if` in the application service "does
  represent domain logic", and he moves it into a domain service.
- VERIFIED. Bernhardt, "Functional Core, Imperative Shell" (2012)
  https://www.destroyallsoftware.com/screencasts/catalog/functional-core-imperative-shell
  The shell "manipulates stdin, stdout, the database, and the network, all based on values
  produced by the functional core." It "leads to an imperative shell with few conditionals".
  No test for order.
- SECONDARY. Wlaschin, Domain Modeling Made Functional (2018), ch. 7 "Modeling Workflows as
  Pipelines" (TOC: https://www.oreilly.com/library/view/domain-modeling-made/9781680505481/).
  A Russian blog review quotes his workflow sketch: "step 1 do ValidateOrder / If order is
  invalid then: stop / step 2 do PriceOrder / step 3 do SendAcknowledgmentToCustomer". Each step
  is typed by its input and output state (validated order, priced order). Book text not read.

## 5. Card games: order of simultaneous deaths written as rules

- VERIFIED (mirror of the MTG CR): https://magic-rulebook.vercel.app/rules/704
  704.3: "the game checks for any of the listed conditions for state-based actions, then performs
  all applicable state-based actions simultaneously as a single event. If any state-based actions
  are performed as a result of a check, the check is repeated".
- VERIFIED (same mirror): https://magic-rulebook.vercel.app/rules/603
  603.10: "Normally, objects that exist immediately after an event are checked [...] However,
  some triggered abilities are exceptions to this rule; the game "looks back in time" to
  determine if those abilities trigger, using the existence of those abilities and the appearance
  of objects immediately prior to the event."
  603.10a: "Some zone-change triggers look back in time. These are leaves-the-battlefield
  abilities, abilities that trigger when a card leaves a graveyard, and abilities that trigger
  when an object that all players can see is put into a hand or library." Its example: an
  artifact with "Whenever a creature dies, you gain 1 life" is destroyed along with two creatures,
  and "The artifact's ability triggers twice".
  603.3b: triggered abilities are put on the stack by "each player, in APNAP order [...] in any
  order they choose."
  mtgtools.io/rules/603 (VERIFIED, rule numbers stripped) and the Secrets of Strixhaven update
  bulletin (SECONDARY) add sacrifice triggers to the 603.10a list. The official 2026 PDF
  (media.wizards.com) was not read.
- VERIFIED. Hearthstone Wiki, Advanced rulebook (community document)
  https://hearthstone.wiki.gg/wiki/Advanced_rulebook
  Rule 4a: "After the outermost Phase ends, Hearthstone does an Aura Update (Health/Attack), then
  does the Death Creation Step (Looks for all mortally wounded [...] Entities and kills them,
  removing them from play simultaneously)".
  Rule 7: "If one or more Deaths happened after the outermost Phase ended, a new Phase (called a
  "Death Phase") begins, where Deaths are Queued in order of play. For each Death, all Death Event
  triggers (Deathrattles, on-Death Secrets and on-Death triggered effects) are Queued and resolved
  in order of play".
  Rule 8: "A Death Phase can have yet another Death Phase after it. This process repeats forever
  until no new Deaths occur".
  Example: Cult Master and Novice Engineer both hit by Mad Bomber, "both deaths occur
  simultaneously after the Phase ends", so Cult Master draws nothing.

## Sources

- Kept: connascence.io Execution, Timing and About pages. Definitions, and who runs the site.
- Kept: Seemann's 2011 temporal coupling post and his 2021 referential transparency post.
- Kept: Wikipedia "Sequential coupling", for the definition and the template method remedy.
- Kept: Khorikov, "Domain services vs Application services", the closest thing to an order test.
- Kept: the Evans ch. 5 text in the Dawn-Breaker GitHub repo, a reproduction of the book.
- Kept: Bernhardt's screencast page, the primary statement of functional core, imperative shell.
- Kept: MTG CR mirror and the Hearthstone Advanced rulebook, for the exact rule text.
- Dropped: Reddit threads on deathrattle order, dev.to, DZone and Substack connascence summaries.
- Not readable: CACM 1992 paper (HTTP 403), CodeProject Buday article, onestepback.org.

## Gaps

- Official MTG text: compare 603.10a against the current official PDF; the mirror's list may be
  older than the update that added sacrifice triggers.
- Wlaschin's wording: ch. 7 was not read.
- Order test: a direct search for a stated "domain order vs procedural order" test found nothing.
  Khorikov's "would reordering change the invariants?" is the nearest.
