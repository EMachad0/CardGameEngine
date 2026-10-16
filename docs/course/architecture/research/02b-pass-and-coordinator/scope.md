# Research: pass, coordinator, event dispatch, stack vs effect application

Scope: primary sources for session 02b. "Verified" means read at the cited URL during this research.
"Secondary" means the quote comes through someone else's reproduction of the primary text.

## A. Evans: application layer, domain services

### A1. Application layer (DDD, 2003, ch. 4 "Isolating the Domain", Layered Architecture table)

> "Application Layer: Defines the jobs the software is supposed to do and directs the expressive
> domain objects to work out problems. The tasks this layer is responsible for are meaningful to
> the business or necessary for interaction with the application layers of other systems. This
> layer is kept thin. It does not contain business rules or knowledge, but only coordinates tasks
> and delegates work to collaborations of domain objects in the next layer down. It does not have
> state reflecting the business situation, but it can have state that reflects the progress of a
> task for the user or the program."

> "Domain Layer (or Model Layer): Responsible for representing concepts of the business,
> information about the business situation, and business rules. [...] This layer is the heart of
> business software."

- Via Fowler, AnemicDomainModel: https://martinfowler.com/bliki/AnemicDomainModel.html
  (Fowler adds "[his name for Service Layer]"). The same text is quoted on Microsoft Learn:
  https://learn.microsoft.com/en-us/dotnet/architecture/microservices/microservice-ddd-cqrs-patterns/ddd-oriented-microservice
- Note: SECONDARY. The book itself was not consulted. Two independent sources quote identical
  wording. Usually cited as p. 70; page not verified.

### A2. Application vs domain services (DDD 2003, ch. 5 "Services", "Services and the Isolated Domain Layer")

> "It can be harder to distinguish application SERVICES from domain SERVICES. The application
> layer is responsible for ordering the notification. The domain layer is responsible for
> determining if a threshold was met [...]"

> "Many domain or application SERVICES are built on top of the populations of ENTITIES and
> VALUES, behaving like scripts that organize the potential of the domain to actually get
> something done."

- Source: unofficial transcription of the book,
  https://github.com/Dawn-Breaker/Domain-Driven-Design-zh/blob/master/docs/ch5.md
- Note: SECONDARY, unofficial copy. Check against the printed book before quoting on a slide.

### A3. Domain service pattern

Book (2003), ch. 5:

> "When a significant process or transformation in the domain is not a natural responsibility of
> an ENTITY or VALUE OBJECT, add an operation to the model as a standalone interface declared as
> a SERVICE. Define the interface in terms of the language of the model and make sure the
> operation name is part of the UBIQUITOUS LANGUAGE. Make the SERVICE stateless."

> "A good SERVICE has three characteristics. 1. The operation relates to a domain concept that
> is not a natural part of an ENTITY or VALUE OBJECT. 2. The interface is defined in terms of
> other elements of the domain model. 3. The operation is stateless."

- Same unofficial copy as A2. SECONDARY.

DDD Reference (2015), "Services", p. 14 (PDF p. 21):

> "Sometimes, it just isn't a thing. Some concepts from the domain aren't natural to model as
> objects. Forcing the required domain functionality to be the responsibility of an entity or
> value either distorts the definition of a model-based object or adds meaningless artificial
> objects. Therefore: When a significant process or transformation in the domain is not a
> natural responsibility of an entity or value object, add an operation to the model as a
> standalone interface declared as a service. Define a service contract, a set of assertions
> about interactions with the service. [...] Give the service a name, which also becomes part
> of the ubiquitous language."

- https://www.domainlanguage.com/wp-content/uploads/2016/05/DDD_Reference_2015-03.pdf
- Note: VERIFIED (primary PDF). The Reference writes "entity or value object" in lowercase. It
  dropped "Make the SERVICE stateless" in favour of a "service contract".

### A4. DDD Reference (2015) on the application layer

The Reference has no "Application Layer" definition. Layered Architecture (p. 10) only says:

> "Concentrate all the code related to the domain model in one layer and isolate it from the
> user interface, application, and infrastructure code. The domain objects, free of the
> responsibility of displaying themselves, storing themselves, managing application tasks, and
> so forth, can be focused on expressing the domain model."

- Note: VERIFIED, same PDF. For "thin, coordinates, no business rules", cite the 2003 book
  (A1), not the Reference.

## B. Domain event dispatch

### B1. Udi Dahan, "Domain Events – Salvation" (2009-06-14)

Mechanism: raise immediately, through a static class. Handlers are resolved from a container,
plus thread-static callbacks.

> "In your domain entities, when a significant state change happens you'll want to raise your
> domain events like this: `DomainEvents.Raise(new CustomerBecamePreferred() { Customer = this });`"

> "The answer is that DomainEvents is a static class. 'OMG, static?! But doesn't that hurt
> testability?!' No, it doesn't."

> "`foreach(var handler in Container.ResolveAll<Handles<T>>()) handler.Handle(args);`" (inside `Raise`)

Trade-offs he names:

> "Please be aware that the above code will be run on the same thread within the same
> transaction as the regular domain work so you should avoid performing any blocking
> activities, like using SMTP or web services. Instead, prefer using one-way messaging [...]"

> "you can have multiple classes handling the same domain event. [...] you don't need to change
> any code – just write a new handler."

> "These handler classes do not belong in the domain model. Nor do they belong in the service layer."

> "When used server side, please make sure that you add a call to ClearCallbacks in your
> infrastructure's end of message processing section."

Later author comments on the same page:

> "Be careful of using static – it is particularly problematic in async/await."

> "These days, instead of recommending domain events I prefer to model things as sagas with NServiceBus."

- https://udidahan.com/2009/06/14/domain-events-salvation/ (bot wall). Read through
  https://web.archive.org/web/2023/https://udidahan.com/2009/06/14/domain-events-salvation/
- Note: VERIFIED through the Wayback Machine copy (Dec 2023 snapshot), not the live page.

### B2. Jimmy Bogard, "A better domain events pattern" (2014-05-13)

Mechanism: the entity keeps a list of events. The infrastructure dispatches them before commit
(e.g. in EF `SaveChanges`).

> "Because the domain events class is static, it also dispatches to handlers immediately. This
> makes testing our domain model difficult, because side effects, however crazy, are executed
> immediately at the point of raising the domain event."

> "Instead of dispatching to a domain event handler immediately, what if instead we recorded
> our domain events, and before committing our transaction, dispatch those domain events at that point?"

> "Just before we commit our transaction, we dispatch our events to their respective handlers.
> With this approach, we decouple the raising of a domain event from dispatching to a handler
> [...] If you want immediate side effects with other entities, just pass those in as arguments."

> "We can dispatch synchronously [...] or asynchronously by persisting our events as JSON and
> having an offline process pick up those undispatched events [...]"

> "beware of static, opaque dependencies [...] Separating the two concerns of raising versus
> dispatching keeps our domain model fully encapsulated"

- https://lostechies.com/jimmybogard/2014/05/13/a-better-domain-events-pattern/
- Note: VERIFIED. The code samples did not come through the fetch; only the prose was read.

## C. MTG Comprehensive Rules, 704 (effective September 25, 2026)

> "704.1. State-based actions are game actions that happen automatically whenever certain
> conditions (listed below) are met. State-based actions don't use the stack."

> "704.1a Abilities that watch for a specified game state are triggered abilities, not
> state-based actions. (See rule 603, "Handling Triggered Abilities.")"

> "704.2. State-based actions are checked throughout the game and are not controlled by any player."

> "704.3. Whenever a player would get priority (see rule 117, "Timing and Priority"), the game
> checks for any of the listed conditions for state-based actions, then performs all applicable
> state-based actions simultaneously as a single event. If any state-based actions are
> performed as a result of a check, the check is repeated; otherwise all triggered abilities
> that are waiting to be put on the stack are put on the stack, then the check is repeated.
> Once no more state-based actions have been performed as the result of a check and no
> triggered abilities are waiting to be put on the stack, the appropriate player gets priority.
> This process also occurs during the cleanup step (see rule 514), except that if no
> state-based actions are performed as the result of the step's first check and no triggered
> abilities are waiting to be put on the stack, then no player gets priority and the step ends."

> "704.4. Unlike triggered abilities, state-based actions pay no attention to what happens
> during the resolution of a spell or ability."

- Index: https://magic.wizards.com/en/rules
- TXT: https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt
- Note: VERIFIED against the official TXT. The filename date changes with each CR update, so
  re-check the link from the rules page before the session.

## D. Forge and XMage: stack vs effect interpretation

### D1. Forge (github.com/Card-Forge/forge, master)

Who decides what resolves next:
- `forge-game/src/main/java/forge/game/zone/MagicStack.java`. `MagicStack` holds a
  `Deque<SpellAbilityStackInstance>`. `resolveStack()` takes the top with `peekAbility()`,
  checks fizzling with `hasFizzled(sa, null)`, then hands off to `AbilityUtils.resolve(sa)`
  (or to `sa.resolve()` when there is no API). It then fires `GameEventSpellResolved`, runs
  `game.getAction().checkStaticAbilities()`, and calls `finishResolving` (remove from stack,
  unfreeze, `game.getPhaseHandler().onStackResolved()`).
- `MagicStack` is constructed with the whole `Game` (`public MagicStack(Game gameState)`) and
  does more than order the stack. Its `add()` runs cast triggers, sets priority, and resolves
  mana abilities straight away through `AbilityUtils.resolve(sp)`.

Who calls the stack, and where SBAs run:
- `forge-game/src/main/java/forge/game/phase/PhaseHandler.java`. When priority has gone around
  with every player passing and the stack is not empty:
  `else if (!game.getStack().hasSimultaneousStackEntries()) { game.getStack().resolveStack(); }`.
- `checkStateBasedEffects()` in the same file loops
  `do { game.getAction().checkStateEffects(false, allAffectedCards); ... } while (game.getStack().addAllTriggeredAbilitiesToStack());`.
  Its source comment reads "CR 704.3 Whenever a player would get priority, the game checks
  ... for state-based actions". `GameAction` was not opened.

Who interprets the effect:
- `forge-game/src/main/java/forge/game/ability/AbilityUtils.java`, `public static void resolve(final SpellAbility sa)`.
  It gets `final Game game = pl.getGame();`, checks conditions and "unless" costs, calls
  `sa.resolve()`, then `resolveSubAbilities(sa, game)`, which walks the sub-ability chain.
- `forge-game/src/main/java/forge/game/ability/AbilityApiBased.java`:
  `effect = api.getSpellEffect();` ... `public void resolve() { effect.resolve(this); }`.
- `forge-game/src/main/java/forge/game/ability/SpellAbilityEffect.java`:
  `public abstract class SpellAbilityEffect { public void resolve(SpellAbility sa) {} ...`.

What the interpreter can reach: the effect receives only the `SpellAbility`, but reaches the
whole game through it: `Game game = sa.getHostCard().getGame();`,
`sa.getActivatingPlayer().getGame().getTriggerHandler()...`.

- Note: VERIFIED from raw source on master. Line numbers move, so cite by method name.

### D2. XMage (github.com/magefree/mage, master)

Who decides what resolves next:
- `Mage/src/main/java/mage/game/stack/SpellStack.java`:
  `public class SpellStack extends ArrayDeque<StackObject>`. Its `resolve(Game game)` does
  `top = this.peek(); top.resolve(game);`.
- The main loop goes through `Mage/src/main/java/mage/game/GameImpl.java` instead.
  `playPriority` runs `if (allPassed()) { if (!state.getStack().isEmpty()) { resolve(); ... applyEffects(); ...`.
  `protected void resolve()` does `top = state.getStack().peek(); applied = top.resolve(this);`,
  then `state.getStack().remove(top, this)` and handles simultaneous events.

Where SBAs run: in `GameImpl.playPriority`, `checkStateAndTriggered(); applyEffects();` runs
before each `player.priority(this)`. The javadoc of `checkStateAndTriggered()` paraphrases CR
117.5 ("performs all applicable state-based actions as a single event"). The method loops
`checkStateBasedActions()`, then `checkTriggered()`.

Stack object to ability:
- `Mage/src/main/java/mage/game/stack/StackObject.java`: `boolean resolve(Game game);`
- `Mage/src/main/java/mage/game/stack/StackAbility.java`: `resolve(Game game)` checks
  `ability.getTargets().stillLegal(ability, game)`, calls `ability.resolve(game)`, then
  `game.getStack().remove(this, game)`. Otherwise it fizzles through `counter(null, game)`.
- `Spell.java` was not opened (the spell path).

Who interprets the effect:
- `Mage/src/main/java/mage/abilities/AbilityImpl.java`. `resolve(Game game)` calls
  `resolveMode(game)` for each selected mode, which runs:
  `for (Effect effect : getEffects()) { ... if (effect instanceof OneShotEffect) { boolean effectResult = effect.apply(game, this); ... } else { game.addEffect((ContinuousEffect) effect, this); } ... game.processAction(); }`
- `Mage/src/main/java/mage/abilities/effects/Effect.java`: `boolean apply(Game game, Ability source);`

What the interpreter can reach: every `Effect.apply` receives the whole `Game` plus the source
`Ability`. Only one-shot effects are applied. Continuous effects are registered with the game
(`game.addEffect`), not applied by the ability.

- Note: VERIFIED from raw source on master.

### D3. Takeaway for topic (4)

In both engines the stack type only decides "top next" (LIFO peek). Effect interpretation lives
elsewhere: Forge `AbilityUtils.resolve` to `SpellAbilityEffect.resolve(sa)`; XMage
`AbilityImpl.resolveMode` to `Effect.apply(game, source)`. That code gets the whole game.
Neither stack is a pure ordering structure, though. Forge's `MagicStack` holds `Game` and
triggers rules work itself. XMage's resolve loop lives in `GameImpl`, not `SpellStack`.

## E. Orchestration vs choreography

### E1. Sam Newman, Building Microservices (O'Reilly, 1st ed. 2015), ch. "Integration", p. 17

> "With orchestration, we rely on a central brain to guide and drive the process, much like the
> conductor in an orchestra. With choreography, we inform each part of the system of its job,
> and let it work out the details, like dancers all finding their way and reacting to others
> around them in a ballet."

> "The downside to this orchestration approach is that the customer service can become too much
> of a central governing authority. It can become the hub in the middle of a web, and a central
> point where logic starts to live. I have seen this approach result in a small number of smart
> god services telling anemic CRUD-based services what to do." (pp. 18-19)

> "The downside is that the explicit view of the business process we see in Figure 2-2 is now
> only implicitly reflected in our system." (choreography, p. 19)

- NGINX/F5 sponsored preview edition PDF:
  https://cdn.studio.f5.com/files/k6fem79d/production/127ba8fd31c38d7e575ed000a58872a6c87be19a.pdf
- Note: VERIFIED against that preview. Page numbers are the preview's. The full 1st edition
  puts the section around p. 43, per secondary sources. The 2nd edition (2021) rewords it;
  not checked.

### E2. Hohpe & Woolf, Enterprise Integration Patterns, "Process Manager"

> "How do we route a message through multiple processing steps when the required steps may not
> be known at design-time and may not be sequential?"

> "Use a central processing unit, a Process Manager, to maintain the state of the sequence and
> determine the next processing step based on intermediate results."

> "The downside of this central control element is the danger of turning the Process Manager
> into a performance bottleneck."

- https://www.enterpriseintegrationpatterns.com/patterns/messaging/ProcessManager.html
- Note: VERIFIED (official pattern page; the full text is in the book). The routing logic sits
  inside the Process Manager ("Based on the rules inside the Process Manager"). That differs
  from a coordinator that "holds no rules", and the difference is worth pointing out in class.

## Gaps

- Evans 2003 book text (A1 to A3) was not checked against a print or licensed edition, and the
  page numbers are unconfirmed.
- Forge `GameAction.checkStateEffects` and XMage `Spell.resolve` / `checkStateBasedActions`
  bodies were not opened.
- Newman 2nd edition wording was not checked.
