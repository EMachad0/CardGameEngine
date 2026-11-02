# A summon is still called entering the board

Status: needs-triage
Category: enhancement

The glossary's Summon lists "enter" and "board entry" as words to avoid. `Event::Summoned`,
`Report::Summoned` and `HistoryKind::MinionSummoned` follow it, but older names and text don't:

- `Zones::enter_board` in `crates/rules/src/zones.rs`, the write `zone_move::summon` makes, and
  its unit tests in the same file. Its siblings `bury` and `entomb` are named for the glossary's
  verbs.
- Test names and messages in `crates/rules/tests/spec/` that say a minion "enters" or "entered"
  the board when it is summoned, such as `"no Squire entered"` in `triggers.rs`,
  `"a summoned Squire enters the board without being played"` in `events.rs`, and
  `a_captain_buffs_friendly_minions_whenever_they_entered` in `derived.rs`.
  `rg -n "enter" crates/rules/tests` finds the rest.

Rename `enter_board` to `summon`, and reword the test text in the glossary's terms. Text where
"entered" means entering any zone, as in `zones.rs`'s order-of-play stamp, stays.
