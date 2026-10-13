# Connascence, Parnas's changeability, Ousterhout on leakage and names

Researcher report, session 01. Kept in full.

## Summary

I confirmed most of the wording from primary sources. Parnas 1972 (full PDF), Page-Jones 1992 (full CACM scan), connascence.io, the Go blog, Dave Cheney, Google's Go style guide and matklad are all [PRIMARY]. The *Philosophy of Software Design* book text is not legally online, so its quotes are [SECONDARY], with partial [PRIMARY] backing from Ousterhout's own CS190 notes. Check those against your print copy before quoting.

Two corrections. Your connascence definition is not Page-Jones's wording. It is the later paraphrase used by Wikipedia and Richards & Ford. And the full taxonomy and the three properties (strength, degree, locality) are not in the 1992 paper.

## Findings

### 1. Connascence

**1a. Definition. Your recollection is wrong as a quote.**
- **Page-Jones 1992 [PRIMARY]**: "I say that two elements of software are connascent if they are "born together" in the sense that they somehow share the same destiny. More explicitly, I define two software elements A and B to be connascent if there is at least one change that could be made to A that would necessitate a change to B in order to preserve overall correctness."
  - Source: M. Page-Jones, "Comparing Techniques by Means of Encapsulation and Connascence," *CACM* 35(9), Sept 1992, pp. 147–151; the definition is on p. 148.
  - Read at: the scan connascence.io links to, via [archive.org](https://web.archive.org/web/2016id_/http://wiki.cfcl.com/pub/Projects/Connascence/Resources/p147-page-jones.pdf).
  - "Correctness" is hyphenated across a line break in the scan; I rejoined it.
- **Later book version [SECONDARY]**: "Connascence between two software elements A and B means either 1. that you can postulate some change to A that would require B to be changed (or at least carefully checked) in order to perserve [sic, blog typo] overall correctness, or 2. that you can postulate some change that would require both A and B to be changed together in order to preserve overall correctness."
  - Source: *Fundamentals of Object-Oriented Design in UML* (1999), p. 214, as quoted by [iformattable blog](http://iformattable.blogspot.com/2009/05/connascence-underlying-principle-of-oo.html).
- **The 1995/96 *What Every Programmer Should Know About OOD* wording: [NOT FOUND]**. I could not get the text.
- **Your wording**: "two components are connascent if a change in one would require the other to be modified in order to maintain the overall correctness of the system."
  - This is the phrasing on [Wikipedia-derived pages](https://handwiki.org/wiki/Connascence) and in reader notes on Richards & Ford, *Fundamentals of Software Architecture* (2020) ([notes](https://github.com/zhangjunhd/reading-notes/blob/master/software/FundamentalsOfSoftwareArchitecture.md)).
  - Do not put it in quotation marks as Page-Jones.
- **Note on the book's date**: connascence.io says the book came out in 1996. Others say 1995.

**1b. Kinds, properties and rules**
- **Kinds named in 1992, p. 149 [PRIMARY]**: "Here we see two forms of connascence: name and type. Other forms include the following: • Value, which indicates that two software elements must contain the same absolute values. • Position, which refers to the specific location of two software elements in the overall code structure. … • Algorithm, which means that two software elements must agree on some common algorithm for their correct execution. • Meaning, which implies that two software elements must be defined to have identical semantic (rather than merely syntactic) structure."
  - For object orientation (pp. 149–150) the paper adds "Class membership", "Class variables" and "An odd kind of connascence derived from polymorphism".
- **What the 1992 paper does not contain**:
  - the words "static" or "dynamic";
  - execution, timing, identity or convention;
  - strength or degree as named properties.
  - Its only gradient is "explicitness" (p. 150): "some forms of connascence (e.g., connascence, by name) are readily apparent from the code itself … Other cases (e.g., connascence by polymorphism) are usually obscure … this latter, implicit form of connascence will probably incur the greatest development and maintenance costs."
- **The rule, 1992 p. 148 [PRIMARY]**: "Eliminate any unnecessary connascence and then minimize connascence across encapsulation boundaries by maximizing connascence within encapsulation boundaries."
- **The static/dynamic taxonomy** (Name, Type, Meaning, Algorithm, Position / Execution, Timing, Value, Identity, plus "Contranascence"):
  - [PRIMARY for Weirich]: it is the table of contents of Jim Weirich's "Connascence and Java" (© 2001), which cites the Page-Jones books ([archive](https://web.archive.org/web/20160117203207/http://onestepback.org/articles/connascence/index.html)).
  - [NOT FOUND in Page-Jones's own words].
  - "Convention" does not appear as a kind in the 1992 paper, in Weirich's list or on connascence.io. All three use "Meaning".
- **The book's three guidelines [SECONDARY]**: "1. Minimize overall connascence by breaking the system into encapsulated elements. 2. Minimize any remaining connascence that crosses encapsulation boundaries. 3. Maximize the connascence within encapsulation boundaries."
  - Quoted via [Codesai/dev.to](https://dev.to/trikitrok/about-connascence-17ko).
- **Weirich's rules [SECONDARY]**: "Rule of Locality: As the distance between software elements increases, use weaker forms of connascence" and "Rule of Degree: Convert high degrees of connascence into weaker forms of connascence."
  - From slides as seen in a [Scribd snippet](https://www.scribd.com/document/92565997/Connascence-key). These are Weirich's rules, not Page-Jones's.

**1c. connascence.io**
- **Who wrote it**: the site is not by Weirich or Rutherford. It credits Josh Robb's 2015 talk as "the inspiration to build this site", and says Rutherford's posts "inspired much of the content".
- **It gives no "connascent if…" definition [PRIMARY]**. Its About page says: "Connascence is a software quality metric that attempts to measure coupling between entities." ([about](https://connascence.io/pages/about.html))
- **Homepage on the three axes**: "1. Strength. Stronger connascences are harder to discover, or harder to refactor. 2. Degree. An entity that is connascent with thousands of other entities is likely to be a larger issue than one that is connascent with only a few. 3. Locality. Connascent elements that are close together in a codebase are better than ones that are far apart." ([home](https://connascence.io/))
- **Locality page**: "Locality matters! Stronger connascences are more acceptible [sic] within a module. Weaker connascences should be used between entities that are far apart (in separate modules or even codebases)." ([locality](https://connascence.io/locality.html))
- **Strength page**: "Static connascences are considered to be weaker than dynamic connascences…" ([strength](https://connascence.io/strength.html))
- **Degree page**: "The degree of a piece of connascence is related to the size of its impact. Does the coupling in question affect 2 entities, or 200?" ([degree](https://connascence.io/degree.html))

### 2. Parnas 1972 [PRIMARY]

D. L. Parnas, *CACM* 15(12), Dec 1972, pp. 1053–1058, read at the [TU/e PDF](https://www.win.tue.nl/~wstomv/edu/2ip30/references/criteria_for_modularization.pdf). Line-break hyphens are rejoined. "rather that" is in the scan. "FIND" is OCR'd as "rIND".

**2a. Decisions likely to change (p. 1055)**: "Changeability. There are a number of design decisions which are questionable and likely to change under many circumstances. This is a partial list.
1. Input format.
2. The decision to have all lines stored in core. For large jobs it may prove inconvenient or impractical to keep all of the lines in core at any one time.
3. The decision to pack the characters four to a word. In cases where we are working with small amounts of data it may prove undesirable to pack the characters; time will be saved by a character per word layout. In other cases we may pack, but in different formats.
4. The decision to make an index for the circular shifts rather that actually store them as such. Again, for a small index or a large core, writing them out may be the preferable approach. Alternatively, we may choose to prepare nothing during CSSETUP. All computation could be done during the calls on the other functions such as CSCHAR.
5. The decision to alphabetize the list once, rather than either (a) search for each item when needed, or (b) partially alphabetize as is done in Hoare's FIND [2]. In a number of circumstances it would be advantageous to distribute the computation involved in alphabetization over the time required to produce the index."

**2b. The two modularizations under change (pp. 1055–1056)**: "By looking at these changes we can see the differences between the two modularizations. The first change is confined to one module in both decompositions. For the first decomposition the second change would result in changes in every module! The same is true of the third change. In the first decomposition the format of the line storage in core must be used by all of the programs. In the second decomposition the story is entirely different. Knowledge of the exact way that the lines are stored is entirely hidden from all but module 1. Any change in the manner of storage can be confined to that module!"

It continues: "The fourth change is confined to the circular shift module in the second decomposition, but in the first decomposition the alphabetizer and the output routines will also know of the change. The fifth change will also prove difficult in the first decomposition. The output module will expect the index to have been completed before it began. The alphabetizer module in the second decomposition was designed so that a user could not detect when the alphabetization was actually done. No other module need be changed."

**2c. Interfaces**
- **p. 1056**: "Independent Development. In the first modularization the interfaces between the modules are the fairly complex formats and table organizations described above. These represent design decisions which cannot be taken lightly. … In the second modularization the interfaces are more abstract; they consist primarily in the function names and the numbers and types of the parameters."
- **Also p. 1056**: "In the first decomposition the criterion used was to make each major step in the processing a module." And: "Every module in the second decomposition is characterized by its knowledge of a design decision which it hides from all others. Its interface or definition was chosen to reveal as little as possible about its inner workings."
- **Conclusion**: "We propose instead that one begins with a list of difficult design decisions or design decisions which are likely to change. Each module is then designed to hide such a decision from the others."

### 3. Ousterhout, *A Philosophy of Software Design*

**Sourcing for this item**:
- I found no legal full text online. Book quotes are [SECONDARY]: they are consistent across several reader notes, and I also saw them in search snippets from unauthorized PDF copies, which I am not linking.
- Page numbers: [NOT FOUND].
- His [CS190 Winter 2018 notes](https://web.stanford.edu/~ouster/cgi-bin/cs190-winter18/lecture.php?topic=modularDesign) are [PRIMARY] but in bullet form.
- I could not get the 2018 Google talk transcript.

**3a. Back-door leakage (§5.2) [SECONDARY]**: "If a piece of information is reflected in the interface for a module, then by definition it has been leaked; thus, simpler interfaces tend to correlate with better information hiding. However, information can be leaked even if it doesn't appear in a module's interface. Suppose two classes both have knowledge of a particular file format (perhaps one class reads files in that format and the other class writes them). Even if neither class exposes that information in its interface, they both depend on the file format: if the format changes, both classes will need to be modified. Back-door leakage like this is more pernicious than leakage through an interface, because it isn't obvious."
- The last two sentences are confirmed by [ashishb.net](https://ashishb.net/book-summary/a-philosophy-of-software-design/). The rest I saw only in snippets.
- Lead-in [SECONDARY]: "Information leakage occurs when a design decision is reflected in multiple modules. This creates a dependency between the modules: any change to that design decision will require changes to all of the involved modules." ([Hofstetter](https://books.danielhofstetter.com/a-philosophy-of-software-design/))
- Red-flag box [SECONDARY]: "Information Leakage: Information leakage occurs when the same knowledge is used in multiple places, such as two different classes that both understand the format of a particular type of file." ([sportebois](https://dev.to/sportebois/software-design-red-flags-wisdom-nuggets-from-john-ousterhout-43i2))
- CS190 notes [PRIMARY]: "Back-door leakage: not visible in the interface".

**3b. Temporal decomposition (§5.3) [SECONDARY]**: "One common cause of information leakage is a design style I call temporal decomposition. In temporal decomposition, the structure of a system corresponds to the time order in which operations will occur. Consider an application that reads a file in a particular format, modifies the contents of the file, and then writes the file out again. With temporal decomposition, this application might be broken into three classes: one to read the file, another to perform the modifications, and a third to write out the new version. Both the file reading and file writing steps have knowledge about the file format, which results in information leakage."
- Sources: [Douban quotes](https://book.douban.com/subject/30218046/blockquotes) and [Medium notes](https://medium.com/@programmerraja/a-philosophy-of-software-design-book-notes-6391fdc23dd5). I did not verify the sentence that follows.
- Red-flag box [SECONDARY, sportebois]: "In temporal decomposition, execution order is reflected in the code structure: operations that happen at different times are in different methods or classes. If the same knowledge is used at different point in execution, it gets encoded in multiple places, resulting in information leakage."

**3c. Formal and informal interface (§4.1) [SECONDARY]**: "The interface to a module contains two kinds of information: formal and informal. The formal parts of an interface are specified explicitly in the code, and some of these can be checked for correctness by the programming language. For example, the formal interface for a method is its signature, which includes the names and types of its parameters, the type of its return value, and information about exceptions thrown by the method. […] Each interface also includes informal elements. These are not specified in a way that can be understood or enforced by the programming language. The informal parts of an interface include its high-level behavior, such as the fact that a function deletes the file named by one of its arguments."
- Source: [Hofstetter](https://books.danielhofstetter.com/a-philosophy-of-software-design/). The page has the typo "fat"; I corrected it to "fact".
- The book goes on to mention ordering constraints, but I did not verify that sentence.
- CS190 [PRIMARY]: "Formal aspects (in the code): method signatures, public variables, etc." / "Informal aspects: overall behavior, side effects, constraints on usage, etc." / "Informal aspects can only be described with comments".

**3d. Cost/benefit, shallow modules and classitis (Ch. 4)**
- **Cost/benefit [SECONDARY]**: "Module depth is a way of thinking about cost versus benefit. The benefit provided by a module is its functionality. The cost of a module (in terms of system complexity) is its interface." ([dev.to](https://dev.to/nicolastrahan/the-big-idea-from-a-philosophy-of-software-design-oge), [slides.com](https://slides.com/sebastiangebski/how-to-design-a-calendar))
  - The words "Module depth" at the start appear in my snippets only as "depth is a way…", so check them.
- **Shallow Module red flag (§4.6) [SECONDARY, sportebois]**: "A shallow module is one whose interface is complicated relative to the functionality it provides. Shallow modules don't help much in the battle against complexity, because the benefit they provide (not having to learn about how they work internally) is negated by the cost of learning and using their interfaces. Small modules tend to be shallow."
- **Shallow method example [SECONDARY]**: `private void addNullValueForAttribute(String attribute) { data.put(attribute, null); }` ([softengbook](https://softengbook.org/articles/deep-modules); Ousterhout-group [slides](https://cs.stanford.edu/~hq6/files/Great%20Programmers%20Long%20(Aug).pdf))
- **Classitis [SECONDARY]**: "The extreme of the "classes should be small" approach is a syndrome I call classitis, which stems from the mistaken view that "classes are good, so more classes are better." In systems suffering from classitis, developers are encouraged to minimize the amount of functionality in each new class: if you want more functionality, introduce more classes." ([edrperez](https://edrperez.com/opiniones-de-libros/a-philosophy-of-software-design), [contemplating.dev](https://contemplating.dev/posts/encapsulation-is-a-bitter-pill))
  - The book's example is Java I/O (FileInputStream / BufferedInputStream / ObjectInputStream).
- **CS190 [PRIMARY]**:
  - "Shallow class … Example: linked list"
  - "Classitis: small classes taken to the extreme"
  - "Each class adds the least possible amount of functionality to existing classes"
  - "Bad example: Java libraries"
  - "Every class and method introduces complexity with its interface"

**3e. Choosing Names, Ch. 14 §14.3 [SECONDARY]**
- **Vague Name box**: "If a variable or method name is broad enough to refer to many different things, then it doesn't convey much information to the developer and the underlying entity is more likely to be misused."
- **Hard to Pick Name box**: "If it's hard to find a simple name for a variable or method that creates a clear image of the underlying object, that's a hint that the underlying object may not have a clean design."
  - Both boxes from [sportebois](https://dev.to/sportebois/software-design-red-flags-wisdom-nuggets-from-john-ousterhout-43i2).
- **The body sentence** is the same as the Hard to Pick Name box, per [Hofstetter](https://books.danielhofstetter.com/a-philosophy-of-software-design/).
  - Correction: the wording is "may not have a clean design", not "unclean".
  - The body also says: "The most common problem with names is that they are too generic or vague…"

### 4. Name modules by what they own, not by kind of operation

1. **Go blog, "Package names", Sameer Ajmani, 4 Feb 2015 [PRIMARY]**: "Avoid meaningless package names. Packages named util, common, or misc provide clients with no sense of what the package contains. This makes it harder for clients to use the package and makes it harder for maintainers to keep the package focused."
   - Also: "If you cannot come up with a package name that's a meaningful prefix for the package's contents, the package abstraction boundary may be wrong." This pairs well with Ousterhout's Hard to Pick Name.
   - [go.dev/blog/package-names](https://go.dev/blog/package-names). Go Code Review Comments has a one-line version: "Avoid meaningless package names like util, common, misc, api, types, and interfaces." ([wiki](https://go.dev/wiki/CodeReviewComments))
2. **Dave Cheney, "Avoid package names like base, util, or common" (2019) [PRIMARY]**: "This often leads to a package's name being derived from what the package contains—utilities." … "Name your packages after what they provide, not what they contain."
   - [dave.cheney.net](https://dave.cheney.net/2019/01/08/avoid-package-names-like-base-util-or-common)
3. **Google Go Style Guide, Best Practices, "Util packages" [PRIMARY]**: "Naming a package just util, helper, common or similar is usually a poor choice (it can be used as part of the name though). Uninformative names make the code harder to read…"
   - [google.github.io](https://google.github.io/styleguide/go/best-practices)
4. **matklad, "Design Pattern: Kitchen Sink" (2021) [PRIMARY]**. This is a counterpoint: he argues for an explicit catch-all.
   - "many classifications work best if there is an explicit "can't classify this" category. … Note that for this pattern to be useful, an existence of a good solid design is prerequisite, lest all the code ends up in an utils module."
   - [matklad.github.io](https://matklad.github.io/2021/05/12/design-pattern-dumping-ground.html)
5. **Ousterhout**: use the Vague Name and Hard to Pick Name boxes from 3e [SECONDARY].
6. **Kevlin Henney: [NOT FOUND]**. I found no primary source by him with a quote on utils/helpers/manager.

## Sources
- **Kept**:
  - Parnas 1972 PDF (TU/e): full original text.
  - Page-Jones 1992 CACM scan (archive.org copy of the cfcl.com file): the original paper.
  - connascence.io (home, About, strength, locality, degree): primary for the site.
  - Weirich "Connascence and Java" on archive.org: primary for his taxonomy.
  - Ousterhout's CS190 Winter 2018 lecture notes: his own words.
  - Go blog, Go Code Review Comments, Google Go style guide, Cheney, matklad: primary.
  - Reader notes for Ousterhout: Hofstetter, sportebois, Douban, dev.to, ashishb. Secondary, but they agree with each other.
- **Dropped**:
  - Unauthorized PDF copies of Ousterhout's book (pdfcoffee, milkov.tech, passeidireto, djvu.online, slideshare uploads): copyright. I used their snippets only to cross-check.
  - DZone (403 error).
  - Generic StackOverflow and blog posts on "Manager" naming: not authoritative.

## Gaps
- **Ousterhout page numbers and exact 2nd-edition text**: check against the print book. Classitis and the shallow-module material are in §4.6, leakage in §5.2, temporal decomposition in §5.3, and names in §14.3. Also verify the "Module depth" lead-in and the sentence that follows the temporal-decomposition example.
- **Page-Jones 1995/96 book wording** (definition, static/dynamic kinds, strength/degree/locality): not online. Library access is needed. Until then, attribute the full taxonomy and the three properties to Weirich and connascence.io, not to Page-Jones.
- **Google 2018 talk transcript**: not retrievable here. Watching the talk on YouTube (id `bmSAYlu0NcY`) would give quotable spoken wording.
- **Kevlin Henney**: needs a targeted search of his writing, for example his naming essays or *97 Things*.
