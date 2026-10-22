# Mutability control and stale checks: Python, Ruby, TypeScript, databases

Researcher report, session 02h. Question: what each language can do to stop a write to the data a
parsed fact describes, and how databases handle check-then-act.

## Summary

No language among the three stops another reference from writing to the data a parsed fact is
about. Python and TypeScript protections are mostly static or shallow. Ruby has a runtime `freeze`,
but it is shallow, and `Ractor.make_shareable` is the one deep-freeze tool found. Databases solve
the problem by fusing check and write: a conditional `UPDATE`, a row lock taken before the check,
or a version check, which is compare-and-swap. In the literature, King mentions mutable data only
in "avoid denormalized representations", and Noonan handles change by giving each changed value a
new type-level name. Neither addresses a parsed fact about external mutable state going stale.

## Findings

### 1. Python

1. Frozen dataclass assignment raises `FrozenInstanceError` (subclass of `AttributeError`): "Raised
   when an implicitly defined `__setattr__()` or `__delattr__()` is called on a dataclass which was
   defined with `frozen=True`." The docs also say: "It is not possible to create truly immutable
   Python objects. However, by passing `frozen=True`… you can emulate immutability."
   https://docs.python.org/3/library/dataclasses.html
2. `object.__setattr__`: the docs mention it only for `__init__`: "`__init__()` cannot use simple
   assignment to initialize fields, and must use `object.__setattr__()`." They don't say outright
   that it bypasses frozen. [UNVERIFIED as an explicit doc statement]
3. Shallowness is not stated on the dataclasses page. The general rule is in the data model: "The
   value of an immutable container object that contains a reference to a mutable object can change
   when the latter's value is changed; however the container is still considered immutable."
   https://docs.python.org/3/reference/datamodel.html. That a frozen instance's list field stays
   mutable is not explicit in the docs. [UNVERIFIED in docs]
4. No enforced privacy: "'Private' instance variables that cannot be accessed except from inside an
   object don't exist in Python." Also: "nothing in Python makes it possible to enforce data hiding
   — it is all based upon convention." https://docs.python.org/3/tutorial/classes.html
5. Name mangling: "Any identifier of the form `__spam`… is textually replaced with
   `_classname__spam`." Its purpose is "letting subclasses override methods without breaking
   intraclass method calls"; "the mangling rules are designed mostly to avoid accidents; it still is
   possible to access or modify a variable that is considered private." It does not apply in
   `exec()`/`eval()`, or with "`getattr()`, `setattr()` and `delattr()`, as well as when referencing
   `__dict__` directly." (same URL)
6. pyright `reportPrivateUsage`: "Protected class members begin with a single underscore… and can
   be accessed only by subclasses. Private class members begin with a double underscore… can be
   accessed only within the declaring class. Variables and functions declared outside of a class are
   considered private if their names start with either a single or double underscore, and they
   cannot be accessed outside of the declaring module." Default `"none"`, `"error"` in strict mode.
   https://raw.githubusercontent.com/microsoft/pyright/main/docs/configuration.md
7. mypy: no private-usage check among the optional error codes.
   https://mypy.readthedocs.io/en/stable/error_code_list2.html. Absence everywhere [UNVERIFIED].
8. Typing constructs are check-time only. "The Python runtime does not enforce function and
   variable type annotations." `Final`: "There is no runtime checking of these properties."
   `NewType`: "calling a `NewType` returns its argument unchanged"; "`some_value is
   Derived(some_value)` is always true at runtime." `ReadOnly`: "There is no runtime checking for
   this property." https://docs.python.org/3/library/typing.html. PEP 705: "will not be enforced by
   Python itself at runtime"; "This does not mean the items are immutable: a reference to the
   underlying dictionary could still exist with a different but compatible type in which those
   items have mutator operations." https://peps.python.org/pep-0705/
9. Immutable containers: frozenset, "These represent an immutable set." Immutable sequences:
   "these other objects may be mutable and may be changed; however, the collection of objects
   directly referenced by an immutable object cannot change." (datamodel). `MappingProxyType`:
   "Read-only proxy of a mapping. It provides a dynamic view on the mapping's entries, which means
   that when the mapping changes, the view reflects these changes."
   https://docs.python.org/3/library/types.html

### 2. Ruby

1. `freeze`: "Prevents further modifications to _obj_. A `FrozenError` will be raised if
   modification is attempted. There is no way to unfreeze a frozen object." `clone`: "Returns a
   shallow copy of `self`, including singleton class and frozen state." `dup`: "Returns a shallow
   unfrozen copy of `self`." https://docs.ruby-lang.org/en/3.4/Object.html. Shallowness is implied
   by the Ractor docs: "Even frozen objects can be unshareable if they contain (through their
   instance variables) unfrozen objects." https://docs.ruby-lang.org/en/3.4/Ractor.html
2. `Data`: "it is meant to be a storage for immutable atomic values. But note that if some of data
   members is of a mutable class, `Data` does no additional immutability enforcement." Example:
   `event.weekdays << 'Sat'`. Instances frozen only per the embedded C source (`OBJ_FREEZE(self)`).
   https://docs.ruby-lang.org/en/3.4/Data.html
3. `private_class_method`: "Makes existing class methods private. Often used to hide the default
   constructor `new`." https://docs.ruby-lang.org/en/3.4/Module.html. "A private method may only be
   called from inside the owner class without a receiver, or with a literal `self` as a receiver."
   https://docs.ruby-lang.org/en/3.4/syntax/modules_and_classes_rdoc.html. `send` bypassing is only
   hinted: "Unlike send, `public_send` calls public methods only." `instance_variable_set` "may
   circumvent the encapsulation intended by the author of the class." (Object docs)
4. `Ractor.make_shareable` deep-freezes: "+obj+ and all the objects it refers to will be frozen,
   unless they are already shareable." Warning: "the specification and implementation of this
   method are not mature and may be changed in the future."
   https://raw.githubusercontent.com/ruby/ruby/v3_4_0/ractor.rb
5. Sorbet: "Properties defined with `const` do not have setter methods, making it impossible to
   update these properties after construction." `prop` is "like a typed version of
   `attr_accessor`." "Constructors and setter methods are always runtime checked." "Attempts to
   write directly into the instance variables backing a prop are not validated."
   https://sorbet.org/docs/tstruct. "adding a `sig` wraps the method… validates the types of
   arguments… validates the return type." https://sorbet.org/docs/runtime
6. RBS runtime checking is an opt-in test tool: "The test installs instrumentations to spy the
   method calls and check if arguments/return values are correct."
   https://raw.githubusercontent.com/ruby/rbs/master/docs/sigs.md

### 3. TypeScript

1. `readonly`: "While it won't change any behavior at runtime, a property marked as `readonly`
   can't be written to during type-checking." Shallow: "doesn't necessarily imply that a value is
   totally immutable… It just means the property itself can't be re-written to." "`readonly`
   properties can also change via aliasing."
   https://www.typescriptlang.org/docs/handbook/2/objects.html
2. `as const`: "`const` contexts don't immediately convert an expression to be fully immutable."
   Example: `foo.contents.push(5); // ...works!`
   https://www.typescriptlang.org/docs/handbook/release-notes/typescript-3-4.html
3. `Object.freeze` is shallow: "a frozen object is not constant (freeze is shallow)." Writes fail
   "either silently or by throwing a `TypeError` exception (most commonly… in strict mode)."
   "Freezing an object with private elements does not prevent the values of these private elements
   from being changed."
   https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/freeze
4. Type assertions: "there is no runtime checking associated with a type assertion." Any cast can
   be forced: "`const a = expr as any as T;`"
   https://www.typescriptlang.org/docs/handbook/2/everyday-types.html. So any module can cast to a
   brand.

### 4. Databases and TOCTOU

1. CWE-367: "the resource's state can change between the check and the use in a way that
   invalidates the results of the check." Mitigations: "The most basic advice for TOCTOU
   vulnerabilities is to not perform a check before the use." "If you cannot perform operations
   atomically… try to limit the amount of time… This will not fix the problem." "Recheck the
   resource after the use call." "Ensure that some environmental locking mechanism can be used to
   protect resources effectively." "Ensure that locking occurs before the check, as opposed to
   afterwards, such that the resource, as checked, is the same as it is when in use."
   https://cwe.mitre.org/data/definitions/367.html
2. Atomic conditional write. PostgreSQL `UPDATE` reports "`UPDATE count`… If *count* is 0, no rows
   were updated by the query (this is not considered an error)."
   https://www.postgresql.org/docs/current/sql-update.html. Under Read Committed, a concurrent
   updater waits, then "The search condition of the command (the `WHERE` clause) is re-evaluated to
   see if the updated version of the row still matches the search condition."
   https://www.postgresql.org/docs/current/transaction-iso.html. Django's `update()` "returns the
   number of rows matched" and "prevents a race condition wherein something might change… between
   loading the object and calling `save()`."
   https://docs.djangoproject.com/en/5.2/ref/models/querysets/
3. Pessimistic locking: "`FOR UPDATE` causes the rows retrieved by the `SELECT` statement to be
   locked as though for update. This prevents them from being locked, modified or deleted by other
   transactions until the current transaction ends."
   https://www.postgresql.org/docs/current/explicit-locking.html. Django's `select_for_update()`
   errors in autocommit mode and is a no-op on SQLite.
4. Optimistic locking in Rails: "Each update to the record increments the integer column
   `lock_version`… an `ActiveRecord::StaleObjectError` exception is thrown if that has occurred and
   the update is ignored."
   https://api.rubyonrails.org/classes/ActiveRecord/Locking/Optimistic.html
5. Compare-and-swap, Java `compareAndExchange`: "Atomically sets the value to `newValue` if the
   current value… `== expectedValue`." `compareAndSet`: "False return indicates that the actual
   value was not equal to the expected value."
   https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/util/concurrent/atomic/AtomicReference.html

### 5. Literature

1. King (2019): "**Avoid denormalized representations of data, _especially_ if it's mutable.**
   Duplicating the same data in multiple places introduces a trivially representable illegal state:
   the places getting out of sync. Strive for a single source of truth." Sub-item: "If
   denormalization is absolutely necessary, use encapsulation to ensure a small, trusted module
   holds sole responsibility for keeping the representations in sync."
   https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/. Her "invalidate" refers to
   code changes caught at compile time. Her one mention of state is shotgun parsing's rollback ("such
   as rolling back a transaction in an RDBMS"). "Mutation", "mutability" and "stale" don't appear;
   no concurrency or TOCTOU.
2. Noonan (2018), Ghosts of Departed Proofs: mutable state appears only for the ST monad, local
   state "invisible from the outside", and shared regions where "sub-computations… may decide to
   share mutable reference cells." Maps are values: insertion gives a map with a new name (`JMap
   ks' k v`) and a combinator `Key ks k -> Key ks' k`, because "insertion results in a map with a
   new key set." Change produces a new name; old proofs carry over only through combinators the
   library supplies. "Without tooling to ensure that the user's proof remains valid, the software is
   left in a brittle state" refers to code evolution. https://kataskeue.com/gdp.pdf
3. No primary source read discusses a parsed value about external mutable state going stale.

## Gaps [UNVERIFIED]

- Python: frozen being shallow, and `object.__setattr__` bypassing it, as explicit doc statements.
- mypy: no private-usage check anywhere (only optional codes checked).
- Ruby: `send(:new)` bypassing `private_class_method`; `instance_variable_set` on a frozen object
  raising `FrozenError`.
- TypeScript: `Readonly<T>` page not fetched; branded types not in the official docs.
- Django: no built-in optimistic locking (only the QuerySet reference checked); `F()` race text.
