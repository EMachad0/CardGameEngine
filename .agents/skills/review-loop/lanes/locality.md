# Locality

Does each decision the change makes live in one place, kept by its owner or keeper?

Read `docs/conventions/file-structure.md`. Under "General", read every section except "Names
exclude". Under "Rust", read "Making knowledge formal", "Checks before writes" and "Restricting
callers". Judge the code in the review range against them, and the code whose coupling the range
changes.

Not this lane: names and where items sit (Names and placement), and whether the code works
(Behaviour).
