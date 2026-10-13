# File structure

## When to use

Read this before creating, splitting, moving or naming a module, a file or a crate, and whenever
you decide where new code goes.

## General

### Coupling is shared knowledge of a decision

Two pieces of code are coupled when one holds knowledge of a decision the other made. Changing the
decision then forces a change wherever that knowledge is held, to keep the program correct. A call
or an import is one way to hold it. A copied rule, a relied-on meaning and a required call order
are others, and none of them needs a call.

The knowledge is formal or informal:

- Formal knowledge is a reference, such as a name, a type or a match that lists every case. When
  the decision changes, the build fails at every place that holds it.
- Informal knowledge is an assumption or a copy. When the decision changes, the build passes, and
  only a test, a review or a bug finds the places that hold it.

Informal knowledge inside one module is cheap, because whoever changes the decision is already
editing that module. Across a module boundary, make it formal. Put it in a type, or in a lint where
a type can't hold it. Where neither can, pin it with a test.

### One place knows each decision

A change to a decision costs one edit for each place that knows it, plus the search for those
places. The build does the search only when the knowledge is formal. So the fewer places know the
answer, the cheaper the change. Ideally one place knows it, and every other place asks it a stable
question or gives it a stable command, by name.

- Give each question its own home, even when two questions share an answer today. The second can
  be built on the first. The day their answers differ, each changes alone.
- Prefer taking knowledge away from callers to making it formal. A caller that must call two
  functions in order still knows the decision, even when a type enforces it. One command that does
  both leaves the caller nothing to get wrong. Make formal only what callers must still know.

### Build modules around decisions, not steps

Before drawing a boundary, list the decisions the code makes. Build modules around them,
especially the ones likely to change or hard to get right. Each module hides its decision behind an
interface that holds still when the decision moves.

- A decision is likely to change when it has changed before, when the roadmap or the domain says
  it varies, or when it comes from outside your control: a file format, a provider's API, a
  regulation. Merge the decisions that change together. Leave stable, easy decisions where they
  are. A decision that never changes costs no edits, however many places know it.
- An interface is made of decisions too, the ones its callers know. Hiding one decision exposes
  others, so expose the ones you expect to stay put. No design makes every decision cheap to
  change.
- Don't split by processing step. A step hands its output forward as data, so every later step
  knows that data's layout, and when it was computed. A step that checks what a later step will do
  has to know that step's rule. Either way, one decision lands in several modules.
- Steps still run, inside one coordinator that calls the modules in order. It knows the order and
  nothing about any format.
- Passing data between modules is fine when its layout is a decision you expect to stay put. The
  module that builds the data keeps the decision that moves.

## Rust

### Making knowledge formal

- A private field or constructor, so only the owning module can create the value, and holding one
  proves the module made it.
- A newtype for a value with a meaning, such as an ID or an amount, so it can't be mixed up with a
  bare integer.
- An exhaustive `match` with no `_` arm, so adding a variant fails the build at every place that
  has to handle it.
- A clippy `disallowed_methods` or `disallowed_types` ban, for "only this place may call that".
