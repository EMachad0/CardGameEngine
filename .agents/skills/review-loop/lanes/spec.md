# Spec

Does the change do what the spec asks, and only that?

The spec is the ticket or the request the task gives. Read it whole, with the ADRs and glossary
terms it cites. Then read the titles in `docs/adr/` and open each record whose subject the range
touches.

Report:

- a requirement the range leaves out or only partly does
- a requirement that looks done but isn't, such as a "Done when" item the code doesn't meet
- a change the spec doesn't ask for, including anything under "Leave alone"
- code that contradicts an accepted ADR

Not this lane: defects in what the spec asked for (Behaviour), and how the code is organized
(Locality, Names and placement).
