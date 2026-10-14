# Parnas 1972: follow-ups, critiques, relevance

Researcher report, session 01a. Kept in full.

## Summary
Most of the eight claims check out. Three need correcting or qualifying: the 1979 paper first appeared at ICSE 1978, PCW 1984 should not be credited with *introducing* "secret", and Sullivan et al. support Parnas rather than offer a separate verdict. I found no credible source that claims to supersede or refute the 1972 criterion. The critiques I found qualify it.

## Findings

**1. Parnas's follow-ups: CONFIRMED, with two corrections.**
- **Program families.** "On the Design and Development of Program Families", *IEEE TSE* SE-2(1):1-9, January 1976. [ACM DL](https://dl.acm.org/doi/10.1109/TSE.1976.233797) [PRIMARY]
- **Extension and contraction.** *IEEE TSE* SE-5(2):128-138, March 1979. **Correction:** it first appeared in *Proc. 3rd ICSE*, May 1978, pp. 264-277. The abstract says: "The most critical step is the design of a software structure called the 'uses' relation." [MSU copy of TSE paper](https://cse.msu.edu/~cse870/Materials/Parnas/extension-contraction-TSE-March-1979.pdf) [PRIMARY]. Both venues are listed in Parnas's own reference list. [Springer](https://link.springer.com/chapter/10.1007/978-3-642-59412-0_25) [PRIMARY]
- **Modular structure.** Parnas, Clements and Weiss: *Proc. 7th ICSE*, March 1984, pp. 408-417, and *IEEE TSE* SE-11(3):259-266, 1985. Both are confirmed by the same Springer reference list.
  - On the module guide: "information hiding or abstraction can be supplemented by a hierarchically-structured document, which we call a module guide."
  - On "secret": "system details that are likely to change independently should be the secrets of separate modules." The paper also uses "primary secret" and "secondary secret". [ICSE 1984 PDF](https://cse.msu.edu/~cse870/Input/SS2002/MiniProject/Sources/parnas84-mod-structure.pdf) [PRIMARY]
  - **Qualification:** "secret" is confirmed as used there, but not as *introduced* there. The paper builds on the earlier NRL A-7E module guide (Britton and Parnas, NRL Memo Report 4702, 1981), which I could not read. Teach it as "used and made central", not "coined".

**2. "The Secret History of Information Hiding": CONFIRMED.**
- It is in *Software Pioneers*, Springer 2002, pp. 398-409. The abstract says that "most industrial software developers do not apply the idea and many consider it unrealistic," and that the paper "speculates on why it has not been completely successful." [Springer](https://link.springer.com/chapter/10.1007/978-3-642-59412-0_25) [PRIMARY]
- He does not withdraw the criterion. He admits a flaw in his own KWIC design: "Only once has anyone noticed that it contains a flaw caused by a failure to hide an important assumption. Every module in the system was written with the knowledge that the data comprised strings of strings." Then: "While the basic idea is to hide information that is likely to change, one can often profit by hiding other information as well because it allows the re-use of algorithms or the use of more efficient algorithms." [Embedded Artistry quoting the chapter](https://embeddedartistry.com/fieldmanual-terms-tags/encapsulation/) [SECONDARY]
- So what he would change is to hide more than "what is likely to change".
- His talk slides for the same event say: "The principle is now clear. It is still hard to apply." [Parnas slides, Canterbury-hosted](https://oowisdom.csse.canterbury.ac.nz/oowisdom.csse.canterbury.ac.nz/images/1/11/Parnas.pdf) [PRIMARY: his own text, hosted by a third party]

**3. Efficiency section in the 1972 paper: CONFIRMED.**
- The section is titled "Efficiency and Implementation". It says: "If we are not careful the second decomposition will prove to be much less efficient than the first. If each of the functions is actually implemented as a procedure with an elaborate calling sequence there will be a great deal of such calling due to the repeated switching between modules."
- His remedy: "To save the procedure call overhead... we must implement these modules in an unusual way. In many cases the routines will be best inserted into the code by an assembler."
- The conclusion repeats the point: "we must abandon the assumption that a module is one or more subroutines." [ACM PDF](https://dl.acm.org/doi/pdf/10.1145/361598.361623) [PRIMARY]
- The TU/e link downloaded, but my tools could not pull its text. These quotes come from the ACM PDF and match the identical [Colorado State copy](https://www.cs.colostate.edu/~france/CS314/Readings/Parnas-decomposition.pdf).

**4. Ousterhout: CONFIRMED.**
- Book, chapter 5, "Information Hiding (and Leakage)": "The most important technique for achieving deep modules is information hiding. This technique was first described by David Parnas." A footnote cites CACM, December 1972. [Fan-translation site that carries the English text](https://yingang.github.io/aposd2e-zh/en/ch05.html) [SECONDARY]
- CS190 notes: "First proposed by David Parnas in a classic paper... More than 40 years old, but still one of the most important papers in all of systems." [CS190 Winter 2018](https://web.stanford.edu/~ouster/cgi-bin/cs190-winter18/lecture.php?topic=modularDesign) [PRIMARY]
- The Spring 2016 notes add: "some parts dated (e.g. predates classes)". [CS190 Spring 2016](https://web.stanford.edu/~ouster/cgi-bin/cs190-spring16/lecture.php?topic=modularDesign) [PRIMARY]

**5. Martin's SRP post (8 May 2014): CONFIRMED.**
- He cites the paper by title, CACM December 1972, vol. 15 no. 12, and quotes its conclusion with "which are likely to change" italicized.
- His reading: "Parnas' conclusion was that modules should be separated based, at lease [sic] in part, on the way that they might change."
- He links SRP to it: "each software module should have one and only one reason to change. This sounds good, and seems to align with Parnas' formulation."
- [Clean Coder Blog](https://blog.cleancoder.com/uncle-bob/2014/05/08/SingleReponsibilityPrinciple.html) [PRIMARY]

**6. Baldwin and Clark, and Sullivan et al. 2001: CONFIRMED, conclusion corrected in emphasis.**
- *Design Rules, Vol. 1: The Power of Modularity*, MIT Press, 2000, ISBN 0-262-02466-7. [AbeBooks listing](https://www.abebooks.com/9780262024662/Design-Rules-Vol-Power-Modularity-0262024667/plp) [SECONDARY]
- Sullivan et al., ESEC/FSE 2001, abstract: "The theory uses design structure matrices to model designs and real options techniques to value them. To test the potential utility of the theory for software we apply it to Parnas's KWIC designs... we show that the options results are consistent with Parnas's conclusions." [ACM DL](https://dl.acm.org/doi/10.1145/503271.503224) [PRIMARY]
- On the information-hiding design: "this design has more hidden information... than the earlier designs... this permits more complex innovation on each of the major system components, increasing the net options value of the design." [Academia.edu, Griswold upload](https://www.academia.edu/60173466/The_structure_and_value_of_modularity_in_software_design) [SECONDARY host]
- **Correction:** the paper does not claim to prove Parnas right. It says its results "suggest that such a theory does have potential to help inform software design."

**7. AOP and MDSOC: CONFIRMED.**
- **Kiczales et al., ECOOP 1997 (LNCS 1241).** "We call the properties these decisions address aspects, and show that the reason they have been hard to capture is that they cross-cut the system's basic functionality." [Springer](https://link.springer.com/chapter/10.1007/BFb0053381) [PRIMARY]
- **Tarr, Ossher, Harrison and Sutton, ICSE 1999, p. 107.** "existing formalisms at all lifecycle phases provide only small, restricted sets of decomposition and composition mechanisms, and these typically support only a single, 'dominant' dimension of separation at a time. We call this 'tyranny of the dominant decomposition.'" [UBC copy of paper](https://www.cs.ubc.ca/~gregor/teaching/papers/p107-tarr.pdf) [PRIMARY]
- Tarr's example ties directly to Parnas: "a decomposition may be chosen to limit the impact of some changes, but traceability may thereby be sacrificed (or, indeed, the ability to limit the impact of other changes)."

**8. Superseding or refuting claims: NONE FOUND from a credible source.**
- Critiques qualify the criterion rather than replace it:
  - AOP and MDSOC (item 7).
  - Steimann, "Fatal Abstraction" (Onward! 2018). It targets abstraction in general and invokes Spolsky's "All non-trivial abstractions, to some degree, are leaky." [ACM](https://dl.acm.org/doi/pdf/10.1145/3276954.3276966) [PRIMARY]
  - Kell (2020), Clark and Basman (2017), and Petricek all argue that predicting what to hide is hard. [Petricek](https://tomasp.net/architecture/) [SECONDARY]
- The best-known reversal went the other way. *The Mythical Man-Month* anniversary edition (1995) has a section titled "Parnas Was Right, and I Was Wrong about Information Hiding". [Pearson sample pages](https://ptgmedia.pearsoncmg.com/images/9780201835953/samplepages/0201835959.pdf) [PRIMARY]
- The fuller Brooks quote, "I dismissed Parnas's concept as a 'recipe for disaster' in Chapter 7", is relayed by [Webster](https://bfwa.com/2012/06/04/rise-the-mythical-man-month-frederick-p-brooks-jr-19751995/) [SECONDARY].

## Sources
- **Kept:** ACM DL, Springer, MSU, UBC and Colorado State copies of the papers (primary text); Stanford CS190 notes; Clean Coder Blog; Pearson sample pages.
- **Dropped:**
  - A Charleston-hosted full PDF of *The Mythical Man-Month*: an unauthorized book copy.
  - Exa/AMiner/gen.wtf summaries: they only restate the papers.
  - A Go textbook relaying a Parnas quote: weak provenance.

## Gaps
- Whether "secret" first appears in PCW 1984 or in the earlier A-7E module guide (1981 NRL report). I did not read the 1981 report.
- The chapter quotes for "Secret History" come through Embedded Artistry. Check them against the Springer chapter (pp. 398-409) before teaching them verbatim.
- The Ousterhout book wording comes from a fan-translation site carrying the English text. Check it against a print copy, chapter 5.1.
- The TU/e PDF could not be machine-read. The item 3 quotes come from the ACM and Colorado State copies.
- The 1985 TSE version was not checked separately from the 1984 ICSE text.
