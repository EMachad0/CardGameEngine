# Ousterhout, Parnas and Rust privacy: quote check

Researcher report, session 01b. Kept in full.

## Summary

All ten items hold up in substance. Three need wording or location fixes: the interface definition is in §4.1, not §4.2; in the 2nd edition the splitting and joining methods section is §9.7, not §9.8; and the Parnas quote uses double quotes around "module". One part of item 10 rests on the Rust Book, not the Reference.

I couldn't reach a publisher or Google Books preview of Ousterhout's book. The Google Books API returned a quota error and search providers were rate-limited. For the book's English text I used two fan translation projects that reproduce it:
- **yingang.github.io/aposd2e-zh/en** reproduces the 2nd edition (2021). It has chapter intros, conclusions and the end-of-book summary with page numbers. Most section bodies are empty there.
- **Cactus-proj/A-Philosophy-of-Software-Design-zh** is an English and Chinese side-by-side text of the 1st edition (2018), with full section bodies.

Neither is publisher-authorized. Below, **[2e-text]** and **[1e-text]** mark quotes taken from these full-text copies. **SECONDARY** marks wording I found only in readers' notes. The Parnas PDF, Ousterhout's own CS190 lecture notes and the Rust docs are primary sources.

## Findings

**1. Module definition: VERIFIED [1e-text], §4.1 "Modular design".** Several readers' notes give the same wording.
> "For the purposes of this book, a module is any unit of code that has an interface and an implementation. Each class in an object-oriented programming language is a module. Methods within a class, or functions in a language that isn't object-oriented, can also be thought of as modules: each of these has an interface and an implementation, and modular design techniques can be applied to them. Higher-level subsystems and services are also modules; their interfaces may take different forms, such as kernel calls or HTTP requests."

Earlier in §4.1: "Modules can take many forms, such as classes, subsystems, or services." The 2e copy confirms the section title "4.1 Modular design" but has no body text. ([Cactus ch04](https://github.com/Cactus-proj/A-Philosophy-of-Software-Design-zh/blob/main/docs/ch04.md); [2e ch04](https://yingang.github.io/aposd2e-zh/en/ch04.html))

**2. Interface definition: CORRECTED (location and wording) [1e-text].**
- The sentence is in **§4.1**, not §4.2, and it reads "The interface consists of…", not "The interface to a module … consists of…":
  > "In order to manage dependencies, we think of each module in two parts: an interface and an implementation. The interface consists of everything that a developer working in a different module must know in order to use the given module. Typically, the interface describes what the module does but not how it does it."
- **§4.2 "What's in an interface?"** opens with: "The interface to a module contains two kinds of information: formal and informal. The formal parts of an interface are specified explicitly in the code, and some of these can be checked for correctness by the programming language." It goes on: "Each interface also includes informal elements. These are not specified in a way that can be understood or enforced by the programming language. … In general, if a developer needs to know a particular piece of information in order to use a module, then that information is part of the module's interface."
- Primary corroboration from Ousterhout's own lecture notes (paraphrase-level bullets, not book text): "_Interface_: anything about the module that must be known to other modules. Formal aspects (in the code)… Informal aspects… can only be described with comments." ([CS190 Modular Design notes](https://web.stanford.edu/~ouster/cgi-bin/cs190-winter18/lecture.php?topic=modularDesign))

**3. Cost/benefit and the rectangle picture: VERIFIED [1e-text], §4.4 "Deep modules".**
> "Module depth is a way of thinking about cost versus benefit. The benefit provided by a module is its functionality. The cost of a module (in terms of system complexity) is its interface."

> "To visualize the notion of depth, imagine that each module is represented by a rectangle, as shown in Figure 4.1. The area of each rectangle is proportional to the functionality implemented by the module. The top edge of a rectangle represents the module's interface; the length of that edge indicates the complexity of the interface."

The 2e summary gives "Modules should be deep (see p. 23)". ([summary](https://yingang.github.io/aposd2e-zh/en/summary.html))

**4. Simple interface over simple implementation: VERIFIED [2e-text], Chapter 8 intro (before §8.1).**
> "Most modules have more users than developers, so it is better for the developers to suffer than the users. … Another way of expressing this idea is that it is more important for a module to have a simple interface than a simple implementation."

- The 2e summary of principles reads: "It's more important for a module to have a simple interface than a simple implementation (see pp. 61, 74)."
- Ousterhout's lecture notes (primary) say "It's more important for a class to have a simple interface than a simple implementation" and "Simple APIs are more important than a simple implementation".
- ([2e ch08](https://yingang.github.io/aposd2e-zh/en/ch08.html))

**5. Chapter 9 "Better Together Or Better Apart?": VERIFIED with one location CORRECTION.**

- **Subdividing creates complexity: VERIFIED [2e-text], chapter intro, before §9.1.**
  > "However, the act of subdividing creates additional complexity that was not present before subdivision:"

  The four reasons follow as bullets. Each is quoted from its start:
  - "Some complexity comes just from the number of components… Subdivision usually results in more interfaces, and every new interface adds complexity."
  - "Subdivision can result in additional code to manage the components."
  - "Subdivision creates separation… Separation makes it harder for developers to see the components at the same time, or even to be aware of their existence."
  - "Subdivision can result in duplication…"

  ([2e ch09](https://yingang.github.io/aposd2e-zh/en/ch09.html))
- **Signs that code is related: VERIFIED [2e-text], same place.**
  > "Bringing pieces of code together is most beneficial if they are closely related. … Here are a few indications that two pieces of code are related:"
  - "They share information; …"
  - "They are used together: anyone using one of the pieces of code is likely to use the other as well. This form of relationship is only compelling if it is bidirectional. …"
  - "They overlap conceptually, in that there is a simple higher-level category that includes both of the pieces of code. …"
  - "It is hard to understand one of the pieces of code without looking at the other."
- **Splitting and joining methods: CORRECTED location.**
  - The section is **§9.8 in the 1st edition but §9.7 in the 2nd edition**. The 2e headings are "9.7 Splitting and joining methods" and "9.8 A different opinion: Clean Code". The 2e copy has no body text for §9.7, so the quotes below are from the 1st edition [1e-text]:
    > "However, length by itself is rarely a good reason for splitting up a method. In general, developers tend to break up methods too much. Splitting up a method introduces additional interfaces, which add to complexity. It also separates the pieces of the original method, which makes the code harder to read if the pieces are actually related. You shouldn't break up a method unless it makes the overall system simpler"

    > "Long methods aren't always bad. … Methods containing hundreds of lines of code are fine if they have a simple signature and are easy to read."

    > "When designing methods, the most important goal is to provide clean and simple abstractions. Each method should do one thing and do it completely."
  - On when to split, it gives two forms. The first is factoring out a subtask, which "makes sense if there is a subtask that is cleanly separable from the rest of the original method". The second is splitting into two methods visible to callers, which "makes sense if the original method had an overly complex interface because it tried to do multiple things that were not closely related." It names the red flag in passing: "that is a red flag ("Conjoined Methods")".
  - Red-flag box [1e-text]:
    > "Red Flag: Conjoined Methods — It should be possible to understand each method independently. If you can't understand the implementation of one method without also understanding the implementation of another, that's a red flag. This red flag can occur in other contexts as well: if two pieces of code are physically separated, but each can only be understood by looking at the other, that is a red flag."
  - The 2e summary wording differs: "Conjoined Methods: two methods have so many dependencies that its hard to understand the implementation of one without understanding the implementation of the other (see p. 75)." The missing apostrophe in "its" is in the source.
  - The 2e-only §9.8 (Clean Code) adds [2e-text]: "More functions means more interfaces to document and learn. If functions are made too small, they lose their independence, resulting in conjoined functions… Depth is more important than length".

  ([Cactus ch09](https://github.com/Cactus-proj/A-Philosophy-of-Software-Design-zh/blob/main/docs/ch09.md))

**6. Pass-Through Method and "different layer, different abstraction": VERIFIED.**
- Red-flag box, §7.1 "Pass-through methods" [1e-text]. Many notes give the same wording.
  > "A pass-through method is one that does nothing except pass its arguments to another method, usually with the same API as the pass-through method. This typically indicates that there is not a clean division of responsibility between the classes."
- 2e summary: "Pass-Through Method: a method does almost nothing except pass its arguments to another method with a similar signature (see p. 52)."
- The rule, chapter 7 intro [2e-text]:
  > "In a well-designed system, each layer provides a different abstraction from the layers above and below it"

  > "If a system contains adjacent layers with similar abstractions, this is a red flag that suggests a problem with the class decomposition."
- §7.6 Conclusion [2e-text]:
  > "The "different layer, different abstraction" rule is just an application of this idea: if different layers have the same abstraction, such as pass-through methods or decorators, then there's a good chance that they haven't provided enough benefit to compensate for the additional infrastructure they represent."
- 2e summary principle: "Different layers should have different abstractions (see p. 51)."
- ([2e ch07](https://yingang.github.io/aposd2e-zh/en/ch07.html))

**7. Overexposure: VERIFIED [1e-text].** The box sits in §5.7 "Example: defaults in HTTP responses".
> "Red Flag: Overexposure — If the API for a commonly used feature forces users to learn about other features that are rarely used, this increases the cognitive load on users who don't need the rarely used features."

2e summary: "Overexposure: An API forces callers to be aware of rarely used features in order to use commonly used features (see p. 36)." The 2e copy confirms the §5.7 heading. ([Cactus ch05](https://github.com/Cactus-proj/A-Philosophy-of-Software-Design-zh/blob/main/docs/ch05.md))

**8. Chapter 14 "Choosing Names": VERIFIED, with one wording fix.**
- **Section title:** "14.3 Names should be precise". This heading appears in both editions.
- **Vague Name box [1e-text]:**
  > "If a variable or method name is broad enough to refer to many different things, then it doesn't convey much information to the developer and the underlying entity is more likely to be misused."
- **Hard to Pick Name box [1e-text]:**
  > "If it's hard to find a simple name for a variable or method that creates a clear image of the underlying object, that's a hint that the underlying object may not have a clean design."
- **2e summary:** "Vague Name: the name of a variable or method is so imprecise that it doesn't convey much useful information (see p. 123)" and "Hard to Pick Name: it is difficult to come up with a precise and intuitive name for an entity (see p. 125)."
- **CORRECTED wording, SECONDARY:** the exact sentence "names should be precise" exists only as the section title. Notes quote "Good names have two properties: precision and consistency" from the chapter.
- ([Cactus ch14](https://github.com/Cactus-proj/A-Philosophy-of-Software-Design-zh/blob/main/docs/ch14.md); [schalkventer notes](https://github.com/schalkventer/reading-notes/blob/main/notes/philosophy-of-software-design.md))

**9. Parnas 1972: VERIFIED with a punctuation CORRECTION (primary PDF).**
- Section "What Is Modularization?", p. 1054 (PDF p. 2):
  > "In this context "module" is considered to be a responsibility assignment rather than a subprogram."

  The source uses double quotes around "module", not single. The PDF extraction hyphenates "sub- program".
- On work assignments and interfaces, section "Comparison of the Two Modularizations", subsection "General", p. 1055:
  > "The differences between the two alternatives are in the way that they are divided into the work assignments, and the interfaces between modules."
- Also from "The Criteria", p. 1056: "Every module in the second decomposition is characterized by its knowledge of a design decision which it hides from all others."
- ([PDF](https://www.win.tue.nl/~wstomv/edu/2ip30/references/criteria_for_modularization.pdf))

**10. Rust visibility: privacy rule VERIFIED in the Reference; the struct-fields part comes from the Book.**
- Reference, "Visibility and privacy":
  > "If an item is private, it may be accessed by the current module and its descendants."

  > "By default, everything is _private_, with two exceptions: Associated items in a `pub` Trait are public by default; Enum variants in a `pub` enum are also public by default."
- The Reference's example says `// Declare a public struct with a private field` but has no sentence stating that fields follow the same rule. ([Reference](https://doc.rust-lang.org/reference/visibility-and-privacy.html))
- The explicit statements are in **The Rust Book §7.3**:
  > "Items in a parent module can't use the private items inside child modules, but items in child modules can use the items in their ancestor modules."

  > "If we use `pub` before a struct definition, we make the struct public, but the struct's fields will still be private."
- So an `impl` block in a child module can read and write the private fields of a struct defined in its parent, and a parent or sibling cannot reach a child's private items. That is my inference from the descendant rule; neither page states it about `impl` blocks. ([Book 7.3](https://doc.rust-lang.org/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html))

## Gaps

- **No publisher or Google Books copy of the book.** All Ousterhout section-body quotes come from the two unofficial full-text copies, and most section bodies are 1st-edition text. For items 1, 2, 3 and 5 (§9.7), the 2nd edition could differ slightly from the 1st; the 2e chapter intros I could check matched exactly. To close this, check the printed 2nd edition at pp. 22–25 (ch. 4), 36, 52, 61, 74–75 and 123–125. The page numbers come from the book's own summary.
- **No independent check of the red-flag box text.** Every quote of the box wording traces to the same 1e text. The 2e summary uses different, shorter wording for each red flag; I've quoted both where they appear.
- **The Rust Reference has no statement about struct fields.** Only the Book says fields stay private on a `pub` struct, as quoted above.
