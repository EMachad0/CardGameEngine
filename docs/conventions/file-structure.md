# File structure

## When to use

Read this before creating, splitting, moving or naming a module, a file or a crate, and whenever
you decide where new code goes.

## General

### Coupling is shared knowledge of a decision

A decision is a choice the code makes where another was possible: a representation, a rule, a
meaning, an order or a policy.

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
- Steps still run, inside one coordinator that calls the modules in order. It knows the order and
  nothing about any format. An order belongs to whoever decides it. When it is a precondition of
  the callee, such as validating input before storing it, take it away from callers with one
  command. When it is the procedure itself, it lives in the coordinator. An order that is a rule of
  the domain is a decision, and gets a module like any other.
- Passing data between modules is fine when its layout is a decision you expect to stay put. The
  module that builds the data keeps the decision that moves.
- Make a decision its own module when its knowledge sits in more than one place, today or in
  planned work, and an interface of questions and commands can replace that knowledge while asking
  callers to know less than the data would. If only one place knows it, or the interface would
  mirror the fields, leave it where it is.

### Owners guard invariants

An invariant is a decision about stored data that holds whenever code outside its module can see
it. Inside one of the module's functions it may be false for a while. Any code that can write the
data can break it.

An owner is a module that privately owns all the data an invariant mentions, so it keeps the
invariant through every command on its interface. Data tied by an invariant shares one owner.

- A module that keeps an invariant over data it doesn't privately own is weak. Not breaking the
  invariant becomes informal knowledge in every other possible writer.
- Handing out write access to the data makes the invariant breakable from outside.
- Merge owners only for an invariant the domain needs. A rule nothing depends on unnecessarily grows
  its owner.
- An owner answers questions about its own data.
- A decision that combines several owners' data gets its own module with no data, under their
  lowest common ancestor. Inside one of the owners, it would make that owner depend on the others.

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
  can't read a child's. So code in a parent is open to every child, and child files that add
  `impl` blocks to their parent's type are part of the parent's module.
- Keep definitions in leaf modules, so that every module can hide its decision. A module with
  children is a directory whose `mod.rs` holds only `mod` declarations. A leaf's private items are
  hidden from its siblings and its parent, and siblings share only what they mark `pub(super)`.
  Clippy's `self_named_module_files` and `definition_in_module_root` check both.
- A directory's own code, such as its main type or how its children combine, goes in a leaf named
  for that decision. A leaf named like its directory claims everything its siblings hold, so its
  name fails.
- A module with no data holds free functions that take what they read as parameters. A struct
  earns its place only when it holds borrows that several of the functions share, or implements a
  trait.

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

## This project

- `docs/agents/domain.md` says where the glossary lives and how to use and extend it.
- `docs/conventions/testing.md` says where tests go and how they are named.
