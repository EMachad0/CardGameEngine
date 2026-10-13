# Triage labels

The skills name five triage roles. In this repo each role is written as-is on an issue's `Status:`
line (see `docs/agents/issue-tracker.md`).

| Role in mattpocock/skills | String on the `Status:` line | Meaning                                  |
| ------------------------- | ---------------------------- | ---------------------------------------- |
| `needs-triage`            | `needs-triage`               | Maintainer needs to evaluate this issue  |
| `needs-info`              | `needs-info`                 | Waiting on reporter for more information |
| `ready-for-agent`         | `ready-for-agent`            | Fully specified, ready for an AFK agent  |
| `ready-for-human`         | `ready-for-human`            | Requires human implementation            |
| `wontfix`                 | `wontfix`                    | Will not be actioned                     |
