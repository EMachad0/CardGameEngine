# File structure

## When to use

Read this before creating, splitting, moving or naming a module, a file or a crate, and whenever
you decide where new code goes.

## General

### Coupling is shared knowledge of a decision

A decision is a choice the code makes where another was possible: a representation, a rule, a
meaning, an order or a policy. A rule is a decision about what the program must do or what its data
must hold.

A module is an enforced boundary that hides one decision, so that code outside it can't copy it.
Decisions that change together count as one.

- Data a module takes in or hands out is part of its interface: its layout, what each field means
  and the rules for writing it. A decision whose data crosses the boundary isn't hidden, because
  any caller can copy it.
- A file that declares only public types hides nothing. Its types are part of the interface of
  every module that takes them in or hands them out, and its name still has to pass.
- A module keeps the knowledge of its decision local, because all of its code serves that
  decision. A boundary that holds many decisions that change independently doesn't. Its code has
  to be sorted by decision before a change.
- A child module counts as one item in its parent, since its decision is hidden from the parent
  too. Judge a parent by its own code, which should hold one decision, often how its children
  combine.

Two pieces of code are coupled when one holds knowledge of a decision the other made. Changing the
decision then forces a change wherever that knowledge is held, to keep the program correct. A call
or an import is one way to hold it. A copied rule, a relied-on meaning and a required call order
are others, and none of them needs a call.

Coupling costs as much as the decision changes. Knowledge of a decision that never changes costs
no edits, however many places hold it.

The knowledge is formal or informal:

- Knowledge is formal for a change when that change fails the build at every place that holds it.
  A name, a type or a match that lists every case can make it formal.
- Knowledge is informal for a change when the build still passes after it, and only a test, a
  review or a bug finds the places that hold it. An assumption or a copy is informal for every
  change.

Judge each place against the changes you expect to the decision. Informal knowledge is acceptable
where its locality is high, meaning its places sit close together in code that serves the same
decision. A change then finds them all with a short search. Anywhere else, make it formal. Put it
in a type, or in a lint where a type can't hold it. Where neither can, pin it with a test. A test
shows that a change broke something, not where the decision is known, so it doesn't reduce the
places that know it.

### One place knows each decision

A change to a decision costs one edit for each place that knows it, plus the search for those
places. The build does the search only when the knowledge is formal. So the fewer places know the
answer, the cheaper the change. Ideally one place knows it, and every other place asks it a stable
question or gives it a stable command, by name.

- A question is one thing callers ask about a decision, and one decision can answer several. Give
  each question its own function, even when two share an answer today. The second can be built on
  the first. The day their answers differ, each changes alone. When no plausible change would make
  two answers differ, they are one question asked twice, and the two places are a copy.
- Prefer taking knowledge away from callers to making it formal. A caller that must call two
  functions in order still knows the decision, even when a type enforces it. One command that does
  both leaves the caller nothing to get wrong. Make formal only what callers must still know.

### Build modules around decisions, not steps

Before drawing a boundary, list the decisions the code makes. Build modules around them,
especially the ones likely to change or hard to get right. Each module hides its decision behind an
interface that holds still when the decision moves.

- Judge whether a decision is likely to change by reasoning about the domain. Look at what other
  systems in it already vary, what the roadmap plans, and what comes from outside your control.
  Merge the decisions that change together. Leave stable, easy decisions where they are.
- An interface is made of decisions too, the ones its callers know. Hiding one decision exposes
  others, so expose the ones you expect to stay put. No design makes every decision cheap to
  change.
- Don't split by processing step. A step hands its output forward as data, so every later step
  knows that data's layout, and when it was computed. A step that checks what a later step will do
  has to know that step's rule. Either way, one decision lands in several modules. When an
  interface needs separate steps, keep them as entry points, and move each rule they share into
  one module that both ask.
- Steps still run, inside code that calls the modules in order. That code knows the order and
  nothing about any format. If an order between its steps is an order rule, it is the rule's
  keeper. If every order it holds is free or forced by data, it is a coordinator and holds no
  rules.
- Passing data between modules is fine when its layout is a decision you expect to stay put. The
  module that builds the data keeps the decision that moves.
- Make a decision its own module when its knowledge sits in more than one place, today or in
  planned work, and an interface of questions and commands can replace that knowledge while asking
  callers to know less than the data would. If only one place knows it, or the interface would
  mirror the fields, leave it where it is.

### Orders between steps

An order between two steps belongs to whoever decides it. To find out who, swap the two steps and
reason over every state the program can reach. An order kept at several sites looks free at each,
since the others still hold it. Swap it at every site together. Two runs of one step, such as two
iterations of a loop, count as two steps.

- If nothing observable changes, the order is free and carries no decision.
- If the swap doesn't compile, the second step takes the first one's output. The order is forced
  by data and carries no decision. Output passed through stored state forces nothing, since the
  swap compiles.
- If a step gets input its contract rules out, the order is a precondition of that step. Take it
  away from callers with one command of the callee's, such as one that validates input and then
  stores it.
- If every step keeps its contract and the behavior changes, the order is an order rule.

A state is reachable when some input the module accepts produces it, whether or not a caller sends
that input today. A caller can send a new accepted input without touching the module, so nothing
prompts a review of it. Widening what the module accepts is a change to the module.

Output counts as observable. An output belongs to the step that outputs it, so the order of the
outputs changes with the order of the steps, and the two are one decision. Steps that leave the
same state either way still have an order rule when their outputs show their order.

An order that carries no decision can sit anywhere, the coordinator included.

### Owners guard invariants

An invariant is a decision about stored data that holds whenever code outside its module can see
it. Inside one of the module's functions it may be false for a while. Any code that can write the
data can break it.

An owner is a module that privately owns all the data an invariant mentions, so it keeps the
invariant through every command on its interface. It keeps the rules for how that data may change
the same way. Data tied by an invariant shares one owner.

- Handing out write access to the data makes the invariant breakable from outside. That includes
  the field that holds the owner. Replacing the owner whole can't break its invariant. But putting
  an older value back undoes every change since, and putting any value in can break a rule that
  ties the owner to other data.
- Merge owners only for an invariant the domain needs. A rule nothing depends on unnecessarily grows
  its owner.
- An owner answers questions about its own data, and may ask other owners questions. Reading can't
  break their rules, and a stable question costs nothing.
- An owner's rules are about its own data, so a write to another owner never serves them. Code
  that writes several owners sits outside each of them, as the keeper of a rule that ties the
  writes or as their coordinator.
- A decision that reads several owners' data and belongs to none of them gets its own module with
  no data, under the lowest common ancestor of the code that asks it. Inside an owner or a caller,
  it would make that module hold two decisions.

### Rules no owner guards

A rule has no owner when its data can't share one, such as data outside the program's state, or
when the domain lets it be false for a while and a pass restores it at known points. A rule a pass
restores isn't an invariant. Neither is an order rule, since no stored data records the order its
steps ran in.

A rule no owner guards has a keeper, the one module whose code keeps it, such as a command that
makes the rule's writes together, the command that runs an order rule's steps, or the module that
decides when its pass runs. The keeper must be the only code that can make the writes the rule is
about. Any other code that can make them holds "go through the keeper" as informal knowledge, so a
keeper is weaker than an owner. For a rule a pass restores, the write to restrict is running the
pass. Give a rule an owner whenever its data can share one.

- Read the rule as implications, if A then B. A write that makes A true without B breaks it, so
  restrict A and leave B open. A rule that also runs the other way restricts both.
- An order rule, X before Y, reads as "if Y runs, X ran first", so restrict Y. When Y must also
  follow every X, the rule runs both ways.
- A rule about one value of a shared command restricts who can build that value. The restriction
  goes in the command's input, not in a type it hands out, since readers outside may build that
  type too.

### Parse, don't validate

A check validates when it returns nothing, or hands back the type it was given. What it learned is
lost, and every later step that relies on it holds "the check passed" as informal knowledge. A
check parses when it returns what it learned in a new type that later steps take. Where the build
checks types and only the check can build the new type, a step that takes it can't run without the
check. Whether the check's facts are still true when the step runs depends on the data they are
about.

- A later step relies on a check when, without it, the step could get input its contract rules
  out. The check's facts are then the step's precondition. A check that only decides whether the
  input is allowed establishes no precondition, so no step takes a value from it or checks it
  again. To make forgetting that check fail the build, the code that runs the steps for that input
  takes a value only the check can build.
- A fact about the parsed value's own data stays true while no code can write that data. The code
  that can build or write the type is the code trusted to keep its guarantee. When the type's
  shape rules out every wrong value and the build checks types, no code can build a wrong one, and
  nothing needs trusting. Prefer shape wherever the guarantee fits a layout. A relation between
  values doesn't, so the type needs an owner that keeps the guarantee as an invariant.
- A fact about data the parsed value doesn't hold stays true only until something writes that
  data, so one type can carry facts with different reaches. A fact about data nothing writes never
  goes stale, and its type can be stored and passed anywhere. "Checks before writes" covers the
  facts a write can change.
- A question that answers "maybe" to callers who already know the answer repeats the check on
  every call. Take the parsed type instead, and answer plainly.
- A reference that can outlive a write to its target's data names the target by a fact nothing
  writes, such as its identity. A fact a write can change, such as where the target is, stays out
  of the reference's type. Parse it where it's used.
- Don't hand a parsed type to code that can keep it while the data changes, or use it on other
  data. Take that code's input plain, and parse it where it arrives.

### Checks before writes

A write that relies on an earlier check holds the check's facts as informal knowledge, and any
write to their data in between makes them false.

- A command can check its own input and refuse it only if it makes the action's first write. An
  action that writes several owners checks every precondition before that first write, or it
  needs a rollback. So the code that runs the action knows the preconditions of every write after
  the first. A step that can fail whatever was checked, such as a network call, goes before the
  first write too, or the action needs a rollback. A call that changes another system's state
  counts as a write.
- A later write's precondition becomes formal when the write takes a value only the check can
  build. That value proves the check ran. It keeps the check's facts true only while it stops every
  other write to their data, up to the write it makes itself. Such a value belongs to the owner of
  that data and offers only the writes the check permits.
- When no value keeps the check's facts true up to the write, the write asks again the question
  its precondition rests on. Only a bug can make the answer false there, so the write fails on the
  spot. Refusing in its return type would make every caller handle a case that can't happen. A
  failure that a reachable state or another system can cause isn't a bug, so the write refuses it
  in its return type.
- A command whose input no type and no earlier check covers checks the input itself, so the check
  sits in one place instead of an assumption in every caller. If a reachable state or another
  system can send a wrong input, the command refuses it in its return type. If only a bug in a
  caller can, it fails on the spot. Doing nothing hides the bug.

### Names exclude

A module's name is the first part of its interface a reader sees. Readers use it to pick which
files to open when they look for a decision or a place for new code.

- A name passes when a reader, given any function, existing or new, can tell from the name alone
  whether it belongs. Check it on real functions. A name that most of them could fit fails, even
  when the glossary defines it.
- Judge a name against its siblings, the other names a reader picks between at the same level. A
  parent's name covers its children, and it passes when it rules out what its siblings hold.
- When no name passes, the code holds several decisions, or one decision the domain has no word
  for yet. If a plausible change to one part leaves the others alone, they are separate
  decisions. Split them until each part has a name that passes. If they change together, name
  the concept and add the term to the glossary.
- Take names from the glossary, the vocabulary that developers and domain experts share and keep
  precise.

## Rust

### Modules

- A module in the general sense is a `mod` whose private items include the data its decision is
  stored in.
- Privacy flows down the module tree: a child reads its parent's private items, and a parent
  can't read a child's. So code in a parent is open to every child.
- Keep a type's inherent `impl` blocks in the module that defines it, so its whole interface sits
  in one file. Code elsewhere that builds on the type takes it as a parameter.
- Keep definitions in leaf modules, so that every module can hide its decision. A module with
  children is a directory whose `mod.rs` holds only `mod` declarations. A leaf's private items are
  hidden from its siblings and its parent, and siblings share only what they mark `pub(super)`.
  Clippy's `self_named_module_files` and `definition_in_module_root` check both.
- A directory's own code, such as its main type or how its children combine, goes in a leaf named
  for that decision. A leaf named like its directory claims everything its siblings hold, so its
  name fails.
- A module with no data holds free functions that take what they read as parameters. A function
  that takes the whole holder instead hides what it reads, can start reading more with no change to
  its signature, and keeps its callers from writing any part of the holder while it runs. A struct
  earns its place only when it holds borrows that several of the functions share, or implements a
  trait. It also earns it when many callers would pass the same parameters. One function builds it
  from the owners, so only that function knows what the module reads and writes.

### Paths

From any place in the code, only one path to a definition is allowed.

- Inside the crate, import a definition from the module that defines it, spelled from `crate::`.
  `super::` and `self::` are a second spelling of the same path. The exception is `use super::*;`
  in a test module.
- Don't re-export inside the crate. A re-export hides which module owns an item, and that is part
  of the module's interface, not a decision.
- The public API is one leaf, `prelude.rs`, holding the crate's whole `pub use` list. `lib.rs`
  holds only `mod` declarations. With `unreachable_pub` on, a `pub` item the prelude doesn't list
  fails the checks.
- Nothing inside the crate imports through the prelude. Other crates import from it by name,
  never with a glob, since a glob next to another crate's prelude makes shared names ambiguous.
  rust-analyzer ranks paths through a module named `prelude` last, so its auto-import suggests the
  defining path.
- Visibility can't hide the prelude from its own crate, and Clippy bans items, not paths. A text
  search in the checks enforces these rules.

### Making knowledge formal

These are ways to make knowledge formal where the general rules call for it, not rules of their
own.

- A private field or constructor, so only the owning module can create the value, and holding one
  proves the module made it.
- A newtype for a value with a meaning, such as an ID or an amount, so it can't be mixed up with a
  bare integer.
- An exhaustive `match` with no `_` arm, so adding a variant fails the build at every place that
  has to handle it. This is for matches that decide something for every variant. A filter that
  only cares about some variants can use `_` or `matches!`, and a new variant passes it silently.
  When several filters together decide what each variant means, give that decision one exhaustive
  match.
- A clippy `disallowed_methods` or `disallowed_types` ban, for "only this place may call that".

### Checks before writes

- A proof that holds `&mut` to an owner keeps the check's facts fresh up to its write, as
  `HashMap`'s `OccupiedEntry` does. The owner defines it, with the borrow in a private field and
  methods only for the writes the check permits, each taking `self`. A check that lists many
  options can't hand these out, because only one `&mut` to the owner can exist at a time. A proof
  that holds a shared borrow serves only reads, because it has to end before any write.
- An owned token isn't `Clone` or `Copy`, and the write takes it by value, so each check serves one
  write. A token whose fact is about data nothing writes is a parsed value, and copying it is fine.
- Name a proof for the fact its check established. A proof that holds `&mut` adds the `Mut`
  suffix std uses for exclusive borrows. An owned token puts the fact in the past tense, since it
  holds the fact only as the check saw it.
- Fail on the spot by panicking. The message says what should have made the fact true, in std's
  "expect as precondition" style.

### Restricting callers

No visibility opens an item to one sibling only. `pub(super)` opens it to the parent's whole
subtree, and `pub(in path)` accepts only an ancestor. So a keeper that must be the only caller of a
command picks its tool by where the command lives. No tool restricts an item to one function, only
to a module. Inside that module, which code may call or build it is informal knowledge, acceptable
where its locality is high.

- A command whose only rightful caller shares its file is private.
- Enforce the rule through nesting when the command has one rightful caller and no reason to live
  anywhere else. Its module and the keeper's share a parent that holds only the keeper and the
  commands it guards, and the command is `pub(super)`. When a guarded command keeps a rule of its
  own, its module is the parent of the commands it guards, so a chain of keepers nests one level
  per link.
- A token earns its place when the command has to live outside the keeper's subtree. The command
  takes a value with a private field, which only the keeper's subtree can build.
- A Clippy ban fits when neither the tree nor a signature should carry the rule. An `#[expect]`
  marks each rightful call, including calls from the defining module's unit tests.
- For a rule about one value of a shared command, put the token inside the value: the variant
  holds a type with a private field. When the command's input is also public output, decouple
  them. An internal input type carries the token, and the public type is built from it.

## This project

- `docs/agents/domain.md` says where the glossary lives and how to use and extend it.
- `docs/conventions/testing.md` says where tests go and how they are named.
