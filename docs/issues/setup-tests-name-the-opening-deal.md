# Setup's spec tests say "opening" and "deal" where the glossary says Setup and Draw

Status: needs-triage
Category: enhancement

The spec fixtures `opening()` in `crates/rules/tests/spec/setup.rs` and
`crates/rules/tests/spec/views.rs` build a game that has finished its setup, and
`setup_deals_three_cards_each_from_the_top` names what setup draws a deal. The glossary has
**Setup** and **Draw**, and neither "opening" nor "deal". `docs/agents/domain.md` asks test names to
use the glossary's terms.

Rename the fixtures, such as `after_setup()`, and the test to
`setup_draws_three_cards_each_from_the_top`. The same words appear in prose: the module doc and
assertion messages of `tests/spec/setup.rs`, a message in `tests/spec/events.rs`, a docstring in
`tests/spec/support.rs`, and `Game::new`'s docstring.
