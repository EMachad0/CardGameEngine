# Rename draft and choice to targeting and target

Status: needs-triage
Category: enhancement

The learner settled these names while reviewing PR #40 (untangle-game ticket 02). "Draft" clashes
with the draft game mode of other card games. "Choice" and "choose" are near-synonyms of "pick",
which the glossary has to keep apart under Flagged ambiguities. "Targeting" and "target" are what
card games call naming what an effect acts on before the card resolves.

## Names

| Today | New |
|---|---|
| Draft (glossary), a card a player has started to play with the choices made so far | Targeting |
| Choose (glossary) | Target |
| `Action::Draft { object_id }` | `Action::StartTargeting { object_id }` |
| `Action::Choose { choice_id, object_id }` | `Action::Target { target_id, object_id }` |
| `ChoiceId` | `TargetId` |
| `CharacterChoice` | `CharacterTarget` |
| `CharacterSelector::Chosen(ChoiceId)` | `CharacterSelector::Targeted(TargetId)` |
| `CardDef.choices`, `CardDef::choice` | `CardDef.targets`, `CardDef::target` |
| `DraftView`, `PlayerView.draft` | `TargetingView`, `PlayerView.targeting` |
| `Awaiting::Draft(Draft)` in `game/player_input.rs` | `Awaiting::Targets(Targeting)` |
| `PlayerInput::draft`, `open_draft`, `choose`, `commit_draft` | `targeting`, `start_targeting`, `target`, `commit_targeting` |

"Pick", `PickId`, `Action::Pick` and "pending pick" keep their names.

## Where else the old names sit

- `CardDefError::DuplicateChoice`, `ChoiceNotFound` and `ChoiceInTrigger`, and the
  `validate_choice_*` functions and `resolves_with_draft` in `cards/binder/validity.rs`.
- `game/condition.rs`: `scan_for_valid_choices`, `check_choice_count_fulfilled`,
  `can_finish_choices`, and the `chosen` lists threaded through `legal_actions` in `game/mod.rs`.
- `choice.rs`, whose name no longer covers what it holds once its main type is `CharacterTarget`.
  Its new name has to pass against its siblings, and it also holds `PickId`, `EffectAmount` and
  `AmountBound`.
- The card text in `cards/binder/sets/base.rs`.
- `tests/spec/drafts.rs`, the `draft` and `choose` helpers in `tests/spec/support.rs`, and test
  names and messages that say draft or choice.
- `benches/game.rs`: `drafting_deck`, `drafting_stages`, `legal_actions_drafting` and
  `playout/drafting_seeds_0_to_9`. Renaming a bench ID starts a new CodSpeed history for it.

## Docs

- The glossary: replace Draft and Choose with Targeting and Target, with "draft" and "choose" under
  `_Avoid_`, and drop the Choose and Pick pair from Flagged ambiguities if the two no longer get
  confused.
- ADRs 0007, 0009 and 0010 keep their wording. `docs/adr/README.md` forbids rewriting a record, and
  the decision itself doesn't change.

## When

After the untangle-game effort. Tickets 09, 11 and 12 rework the draft walk, `apply_action` and the
`Chosen` selector, so renaming first would conflict with each of them.

The rename changes the public API in `prelude.rs` (`Action`, `ChoiceId`, `DraftView`,
`PlayerView`), so its PR says so under its behavior notes.
