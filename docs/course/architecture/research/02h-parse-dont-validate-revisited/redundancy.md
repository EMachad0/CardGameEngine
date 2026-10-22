# Rechecking a precondition: Meyer and the counterpositions

Researcher report, session 02h. Question: Meyer's position on a routine re-testing its own
precondition, and primary-source counterpositions.

## Summary

Meyer: a routine never re-tests its precondition; the check is "in the Require, or ... in an If
instruction in the body of the routine, but never in both." Redundant checks add complexity, which
hurts reliability. A precondition violation is a client bug. Runtime precondition checking is a
compilation option, mainly for debugging. The Pragmatic Programmer says to keep assertions on in
production. Rust's std splits `assert!` from `debug_assert!`.

## 1. Non-Redundancy principle (OOSC2)

- Wording, secondary only [UNVERIFIED against the book]: "Under no circumstances shall the body of
  a routine ever test for the routine's precondition." Placed in §11.6, p. 343 by
  https://bond.hexdocs.pm/faq.html,
  https://stackoverflow.com/questions/5721443/when-to-add-a-precondition-and-when-to-only-throw-an-exception,
  https://www.eecs.yorku.ca/course_archive/2011-12/W/3311/slides/15-DesignContract.pdf
- Artima interview (2004): Venners put it as "under no circumstances shall the body of a routine
  check for the precondition of the routine." Meyer: "I have found it far more effective to define
  for each operation the conditions under which it will work, and leave the behavior undefined in
  other cases." https://www.artima.com/articles/failure-preconditions-and-reuse
- Meyer, "Design by Contract" (1991 chapter): "If both the client and the routine check for the
  same conditions, the resulting redundancy, when accumulated over a large system, will yield the
  complexity and unwieldiness that are so characteristic of today's software." "one can hardly find
  the islands of useful processing in oceans of error-checking code."
  https://se.inf.ethz.ch/~meyer/publications/old/dbc_chapter.pdf

## 2. "Applying 'Design by Contract'", IEEE Computer 25(10), Oct 1992, pp. 40-51

https://se.inf.ethz.ch/~meyer/publications/computer/contract.pdf (OCR; columns interleaved on
pp. 43-44)

- Defensive programming is advice "to include as many checks as possible, even if they are
  redundant with checks made by callers." Reply: "Adding possibly redundant code 'just in case' only
  contributes to the software's complexity - the single worst obstacle to software quality in
  general, and to reliability in particular." "more sources of things that could go wrong at
  execution time, hence the need for more checks, and so on ad infinitum."
- "This is an absolute rule: Either you have the condition in the Require, or you have it in an If
  instruction in the body of the routine, but never in both. This principle is the exact opposite
  of the idea of defensive programming, since it directs programmers to avoid redundant tests."
- "the answer depends on a compilation option. For each class, you may choose from various levels
  of assertion monitoring: no assertion checking, preconditions only (the default), preconditions
  and postconditions, all of the above plus class invariants, or all assertions." "Under the
  monitoring options, the effect of an assertion violation is to raise an exception."
- "Assertion monitoring, then, is a way to call the developer's bluff ... This yields a productive
  approach to debugging, testing, and quality assurance." Precondition checking is the default
  because client clusters "may still be 'young' and contain bugs".

## 3. Whose bug; demanding and tolerant

- "A precondition violation indicates a bug in the client (caller). The caller did not observe the
  conditions imposed on correct calls." "A postcondition violation is a bug in the supplier
  (routine)." (1992, p. 44)
- "any runtime violation of an assertion is not a special case but always the manifestation of a
  software bug." (1992)
- "several styles of writing routines are possible, ranging from 'demanding' ones where the
  precondition is strong (putting the responsibility on clients) to 'tolerant' ones where it is
  weak (increasing the routine's burden)." (1992)
- "The standard recommended style in Eiffel is on the demanding side." (1991)

## 4. Counterpositions

- Pragmatic Programmer, 20th anniversary ed. (2019), Topic 24, Tip 38 "Crash Early": "A dead
  program normally does a lot less damage than a crippled one." Topic 25, Tip 39 "Use Assertions to
  Prevent the Impossible": rejects "Assertions are a debugging facility"; "Turning off assertions
  when you deliver a program to production is like crossing a high wire without a net"; "turn off
  only those assertions that really hit you." (pp. 115-117)
  https://cdn.ttgtmedia.com/rms/pdf/pragmatic_programmer_ch4.pdf
- Rust `assert!`: "Assertions are always checked in both debug and release builds, and cannot be
  disabled ... Unsafe code may rely on `assert!` to enforce run-time invariants that, if violated
  could lead to unsafety." https://doc.rust-lang.org/std/macro.assert.html
- Rust `debug_assert!`: "only enabled in non optimized builds by default ... Replacing `assert!`
  with `debug_assert!` is thus only encouraged after thorough profiling, and more importantly, only
  in safe code!" https://doc.rust-lang.org/std/macro.debug_assert.html

## Gaps [UNVERIFIED]

- OOSC2 not read: Non-Redundancy wording, §11.6, p. 343.
- "Demanding vs. tolerant" wording in OOSC2.
- "The main application of runtime assertion monitoring, then, is debugging" not matched in the
  stored text.
