# Rust: carrying a check's result to a later write

Researcher report, session 02h. Question: what Rust offers when a check reads state and a later
step writes the same state relying on what the check learned.

## Summary

Std fuses a check with the write that relies on it in two ways. Either one `&mut` borrow covers the
lookup and the write (Entry, `PeekMut`, `get_disjoint_mut`, `split_at_mut`), or the predicate runs
inside the mutating call (`pop_if`, `take_if`, `retain`, `extract_if`, `compare_exchange`).
Branded indices (`indexing`) keep only an in-bounds fact fresh across growth, not which element an
index points at. View types are not in stable Rust: a nightly lang experiment with no accepted
RFC. No Rust source found discusses parsed types going stale against state they don't own.

## Findings

### 1. Fused check and write in std

1. `HashMap::entry` / `BTreeMap::entry`, 1.0.0: "Gets the given key's corresponding entry in the
   map for in-place manipulation." https://doc.rust-lang.org/std/collections/struct.HashMap.html,
   https://doc.rust-lang.org/std/collections/btree_map/struct.BTreeMap.html. `OccupiedEntry`: "A
   view into an occupied entry in a `HashMap`." Its `remove`/`insert`/`get_mut` act on the found
   entry; "We can use the same Entry multiple times."
   https://doc.rust-lang.org/std/collections/hash_map/struct.OccupiedEntry.html. The variant is the
   proof, and it holds the map's `&mut` until the write.
2. `BinaryHeap::peek_mut`, 1.12.0: "Returns a mutable reference to the greatest item in the binary
   heap, or `None` if it is empty." `PeekMut::pop`, 1.18.0: "Removes the peeked value from the heap
   and returns it." https://doc.rust-lang.org/std/collections/binary_heap/struct.PeekMut.html. The
   guard is the "non-empty, this is the max" fact, holding `&mut`.
3. `Vec::pop_if`, 1.86.0: "Removes and returns the last element from a vector if the predicate
   returns `true`, or `None` if the predicate returns false or the vector is empty (the predicate
   will not be called in that case)." https://doc.rust-lang.org/std/vec/struct.Vec.html. Predicate
   inside the write.
4. `Option::take_if`, 1.80.0: "Takes the value out of the option, but only if the predicate
   evaluates to `true` on a mutable reference to the value."
   https://doc.rust-lang.org/std/option/enum.Option.html
5. `Vec::retain`, 1.0.0: "Retains only the elements specified by the predicate… This method
   operates in place, visiting each element exactly once in the original order." `retain_mut`
   1.61.0. `Vec::extract_if`, stabilized in 1.87.0
   (https://blog.rust-lang.org/2025/05/15/Rust-1.87.0/): "Creates an iterator which uses a closure
   to determine if an element in the range should be removed. If the closure returns `true`, the
   element is removed from the vector and yielded."
6. `slice::get_disjoint_mut`, 1.86.0: "Returns mutable references to many indices at once… Returns
   an error if any index is out-of-bounds, or if there are overlapping indices… This method does a
   O(n^2) check." https://doc.rust-lang.org/std/primitive.slice.html. `HashMap::get_disjoint_mut`,
   1.86.0: "Attempts to get mutable references to `N` values in the map at once… For soundness, at
   most one mutable reference will be returned to any value… Duplicate keys panic!" The disjointness
   check and the split happen in one call.
7. `split_at_mut`, 1.0.0: "Divides one mutable slice into two at an index… Panics if `mid > len`."
8. `compare_exchange`, 1.10.0: "Stores a value into the atomic integer if the current value is the
   same as the `current` value. The return value is a result indicating whether the new value was
   written and containing the previous value."
   https://doc.rust-lang.org/std/sync/atomic/type.AtomicUsize.html

### 2. Borrow facts

9. The Book: "At any given time, you can have *either* one mutable reference *or* any number of
   immutable references." https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html.
   E0502: "A variable already borrowed with a certain mutability (either mutable or immutable) was
   borrowed again with a different mutability." E0499: "A variable was borrowed as mutable more
   than once." https://doc.rust-lang.org/error_codes/E0502.html,
   https://doc.rust-lang.org/error_codes/E0499.html
10. Indexing: "an index expression `a[b]` is equivalent to `*std::ops::Index::index(&a, b)`, or
    `*std::ops::IndexMut::index_mut(&mut a, b)` in a mutable place expression context."
    https://doc.rust-lang.org/reference/expressions/array-expr.html. `fn index(&self, index: Idx)
    -> &Self::Output;` https://doc.rust-lang.org/std/ops/trait.Index.html. "Borrows the whole
    `Vec`" follows from `&a` / `&self`, not stated outright.
11. NLL, RFC 2094: "the lifetime of a reference lasts only for those portions of the function in
    which the reference may later be used (where the reference is **live**…)".
    https://rust-lang.github.io/rfcs/2094-nll.html. The Book: "a reference's scope starts from
    where it is introduced and continues through the last time that reference is used."
12. Niko Matsakis: "You are then giving away access to `self` in order to call `signal_event`… The
    danger here is that `signal_event` may mutate `self.widgets`… But the compiler doesn't know
    that, because it only examines one function at a time."
    https://smallcultfollowing.com/babysteps/blog/2018/11/01/after-nll-interprocedural-conflicts/

### 3. Typestate by move

13. Cliffle: "`drop` takes its argument *by value*… the argument gets *moved into* the `drop`
    function, and the caller loses access to it." "the *previous state*… *can no longer be used.*
    This is because Rust's notion of 'moving' a value causes the previous owner to lose access to
    the value." "each operation *consumes* `self` and either produces a new object in a certain
    state." https://cliffle.com/blog/rust-typestate/
14. That a moved-in proof stays fresh is the researcher's inference, not a source: once moved into
    a type with private fields, only its methods can write the data.

### 4. Branded indices

15. `indexing` (bluss): "Sound unchecked indexing in Rust using 'generativity'… indices, pointers
    and ranges that are trusted to be in bounds… ***This is an experiment.*** The API is all of
    inconsistent, incomplete and redundant." "The indices, pointers and ranges are freely copyable
    and do not track mutability or exclusive access themselves. All access to the underlying data
    goes through the Container." https://docs.rs/indexing/latest/indexing/
16. It allows length changes: "Add one element to the underlying storage, and return its index.
    All outstanding indices remain valid, only the length of the container is now larger."
    "Insert one element… All outstanding indices remain valid (in bounds), but elements have
    shifted." https://docs.rs/indexing/latest/indexing/container/struct.Container.html. Only the
    in-bounds fact survives a write.
17. Last release 0.4.1, 2019-09-11. Changelog: "Remove the ability to clone non-`FixedLength`
    Containers, because allowing to clone a container was wrong in the presencen of the length
    changing .push()/.insert() methods." https://lib.rs/crates/indexing
18. Gankra's thesis: "It should be noted that this trick is very delicate, and I don't expect it to
    see much use." "The indices into an array are only valid as long as the array's length doesn't
    change." "its soundness relies on a pretty deep understanding of Rust… So we really don't
    recommend doing this. Also, whenever this interface catches an error, it produces nonsensical
    lifetime errors." https://faultlore.com/blah/papers/thesis.pdf
19. `generativity`: "Create a trusted carrier with a new lifetime that is guaranteed to be unique
    among other trusted carriers." Brands only. https://docs.rs/generativity/latest/generativity/
20. `qcell`, `ghost-cell`: mutable access needs `&mut` on the owner, so a read-made reference can't
    live across a write either. qcell: "the restrictions are much stricter than they really need to
    be. For example it's not possible to borrow from more than a few different cells at the same
    time." https://docs.rs/qcell/latest/qcell/. ghost-cell: "Extensions to `GhostCell`, such as
    `GhostCursor`, are not proven." https://docs.rs/ghost-cell/latest/ghost_cell/

### 5. View types and partial borrows

21. Niko, 2021: "To be clear, this isn't an RFC or a proposal, at least not yet."
    https://smallcultfollowing.com/babysteps/blog/2021/11/05/view-types/. Later:
    https://smallcultfollowing.com/babysteps/blog/2025/02/25/view-types-redux/ and
    https://smallcultfollowing.com/babysteps/blog/2026/03/21/view-types-max-min/. The 2026 Project
    Goal "View types experiment" is accepted, to "implement a few of the competing syntaxes so that
    people can experiment." https://goals.rust-lang.org/2026/view-types-experiment.html. Feature gate
    PR #155939, merged 2026-05-01: "Usage of view types on `pub` or `pub(...)` functions is a hard
    error." "Partial Types (v3)" RFC #3736 is open. Not stable, no accepted RFC.
    Crates: `partial_borrow` ("Multiple simultaenous mutable partial borrows are possible, so long as
    each field is only mutably accessible via (at most) one of them"; relies on integer/pointer
    conversions) and `borrow` by wdanilo. Both let a proof hold a shared borrow of one field while
    another is written; neither lets the field the proof reads be written.

### 6. Parse, don't validate in Rust writing

22. Palmieri, Zero To Production ch. 6, shows `started_well.0 = "".to_string();` breaking an
    invariant after `parse`, warns that exposing `&mut` "allows them to perform *arbitrary* changes
    to value itself, potentially breaking our invariants!", and keeps fields private with by-value
    or `&` accessors.
    https://www.lpalmieri.com/posts/2020-12-11-zero-to-production-6-domain-modelling/
23. C-NEWTYPE: "Newtypes can statically distinguish between different interpretations of an
    underlying type." Nothing on staleness.
    https://rust-lang.github.io/api-guidelines/type-safety.html
24. Comprehensive Rust: "consumers can't modify the underlying value since `&str`… restricts them
    to read-only access."
    https://google.github.io/comprehensive-rust/idiomatic/leveraging-the-type-system/newtype-pattern/parse-don-t-validate.html
25. None addresses a proof about state outside the parsed value going stale. The closest is
    Gankra's "temporal validity".

## Gaps [UNVERIFIED]

- `Vec::extract_if`'s version comes from the 1.87 release notes only.
- `VacantEntry` and BTreeMap entry types not read.
- The thesis section number 6.3.
- Whether `generativity` claims anything about mutation.
- The view-types tracking issue #155938 not checked.
