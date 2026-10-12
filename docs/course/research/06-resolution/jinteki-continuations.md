# Jinteki.net: continuations as closures keyed by eid

Researcher report, session 06. Source: github.com/mtgred/netrunner `master` at fetch time, plus its
wiki pages. Line numbers not given.

## Summary

Jinteki.net has no explicit continuation stack. An eid is a map with an integer `:eid` id taken
from a counter in the game state. The work after an async step is a Clojure closure that `wait-for`
stores in the game-state atom under `[:effect-completed <int>]`. `effect-completed` looks it up,
calls it and removes it. Prompts are a per-side vector at `[side :prompt]`, each entry holding its
own `:effect` closure, with the first mirrored into `[side :prompt-state]`. Commands are gated by
the acting side's own `:prompt-state`; there is no global "whose decision is it". The live state
holds closures and is never serialized: undo snapshots the whole value in memory, rejoin sends a
stripped projection, and restoring from a replay rebuilds only the board, without prompts or
continuations. The rules checkpoint is async and called by primitives such as `gain-credits` and
`pay`, so it runs between the steps of one ability. Sequencing is nested `wait-for` ending in
`continue-ability`, which the maintainers' own wiki calls "callback hell".

## 1. The eid system

1. CONFIRMED, `src/clj/game/core/eid.clj`:
   ```clojure
   (defn make-eid
     ([state] (make-eid state nil))
     ([state existing-eid]
      (assoc existing-eid :eid (:eid (swap! state update :eid inc)))))
   ```
   The child keeps the parent's keys (`:source`, `:source-type`, `:cost-paid`, `:result`, ...).
   The counter starts at `:eid 0` in `new-state` (`state.clj`).
2. CONFIRMED, continuations in the state atom:
   ```clojure
   (defn register-effect-completed
     [state eid effect]
     (if (get-in @state [:effect-completed (:eid eid)])
       (throw (ex-info "Eid has already been registered" eid))
       (swap! state assoc-in [:effect-completed (:eid eid)] effect)))

   (defn effect-completed
     [state _ eid]
     (doseq [side [:corp :runner]]
       (clear-eid-wait-prompt state side eid))
     (when-let [handler (get-in @state [:effect-completed (:eid eid)])]
       (handler eid)
       (swap! state update :effect-completed dissoc (:eid eid)))
     nil)
   ```
   Results ride on the eid: `(defn make-result [eid result] (assoc eid :result result))`.
3. CONFIRMED, `src/clj/game/macros.clj`: `wait-for` wraps its body in a named `fn`, registers it
   under a new or given eid, then calls the action with that eid. Docstring: "Wrap `body` in a
   callback, execute `action` with the assumption that `action` will call `effect-completed` on an
   `eid`, which will execute the callback."
4. CONFIRMED: `continue-ability` resolves a new ability map on the current `eid`, handing over the
   duty to complete it.
5. CONFIRMED, `engine.clj`: `resolve-ability` makes an eid if none was given and dispatches.
   Non-async abilities are completed automatically (`(when-not (:async ability) (effect-completed state side eid))`).
   Doc comment: "`:async` -- boolean. Mark the ability as "async", meaning the :effect function must
   call effect-completed itself... This part of the engine is really dumb and complicated, so ask
   someone on slack about it."
6. CONFIRMED, wiki "Brain dump about `wait for` and how to solve our 'async' problem": "Internally,
   we're keeping a pointer to these callback functions on state, and as things are finished, we use
   the provided `eid` to find and call the callback function... we're lugging around a lot of
   anonymous functions/closures, which I'm sure has a memory cost and 2) all of our code has the
   awful Callback Hell look." A ringteki-style step pipeline was considered and rejected: "that's
   sort of cool but is fundamentally not how jnet is built". Main failure mode: "we already make
   sure to put so-called async functions in the tail position... it's the single biggest issue with
   dropped and incorrect effects."
   https://github.com/mtgred/netrunner/wiki/Brain-dump-about-%60wait-for%60-and-how-to-solve-our-%22async%22-problem

## 2. Prompts

7. CONFIRMED, `prompt_state.clj`: `add-to-prompt-queue` conses the prompt onto `[side :prompt]`
   and `set-prompt-state` mirrors the first entry into `[side :prompt-state]`. New prompts go to the
   front, so the queue acts as a stack.
8. CONFIRMED, `prompts.clj` `show-prompt` ("All prompts flow through this method"): each entry is
   `{:eid eid :msg prompt :choices choices :effect f :card card ... :cancel cancel :end-effect end-effect}`.
   With `waiting-prompt` set, a `:prompt-type :waiting` entry goes on the other side's queue and is
   cleared by `effect-completed`. In `engine.clj`, `prompt!` builds the resume closure:
   `(let [f #(resolve-ability state side ability card [%])] (show-prompt ...))`.
9. CONFIRMED, `actions.clj` `resolve-prompt`: removes the answering side's entry and calls its
   `:effect`. `side` comes from the websocket player (`web/game.clj`).
10. CONFIRMED, `actions.clj`:
    ```clojure
    (defn- no-blocking-prompt? [state side]
      (let [prompt-type (get-in @state [side :prompt-state :prompt-type])]
        (or (= nil prompt-type) (= :run prompt-type) (= :prevent prompt-type))))
    ```
    `play` requires `(not (get-in @state [side :prompt-state :prompt-type]))`.
11. UNVERIFIED (inference from 8 and 10): the opponent is blocked only through the `:waiting`
    prompt, and only when the ability sets `:waiting-prompt`. No central lock found.
12. CONFIRMED, `commands.clj`: `/close-prompt` pops the prompt and force-completes its eid. A
    comment in `clear-eid-wait-prompt`: "or else the game will need to be salvaged via close-prompt
    commands".

## 3. Serialization, cloning, reconnect and replay

13. CONFIRMED: closures live in `:effect-completed` and in each prompt's `:effect`, `:cancel` and
    `:end-effect`. `diffs.clj` whitelists prompt fields (`prompt-keys`) for clients, without
    `:effect`. No code serializes the full state (UNVERIFIED that none exists).
14. CONFIRMED: undo stores `(without-history @state)` in `:click-states` (last 4) and restores with
    `(reset! state last-click-state)`; each web action rolls back with `(reset! state old-state)` on
    an exception. UNVERIFIED inference: the snapshots carry the closures, which capture the atom, so
    rollback mid-prompt works in memory.
15. CONFIRMED: rejoin and resync send `diffs/public-states` of the in-memory atom; the engine isn't
    recreated. PR #8758 added a `:sequence` number to detect dropped diffs.
16. CONFIRMED, `web/replay_restore.clj` `setup-state-from-replay`: starts a fresh game, moves cards
    into zones, restores counters and turn keys, re-runs `card-init` with `{:resolve-effect false}`.
    Nothing restores `:prompt`, `:effect-completed` or `:run`.
17. UNVERIFIED: in-progress games are probably lost on a server restart (Jinteki.net Guide wiki:
    "The game server periodically gets restarted").
18. CONFIRMED: wiki "Sequential Abilities brain dump" proposes a `:sequence` wrapper with a shared
    context atom and notes "we're almost always nesting targeted abilities, and this misses out on
    passing that information around". `actions.clj`: `;; TODO - resolve-prompt does some evil things
    with eids, maybe we can fix it later - nbk, 2025`. Issue #6324: "couldn't use /close-prompt.
    game pretty much locked up."

## 4. Checkpoint

19. CONFIRMED, `engine.clj` `checkpoint` follows CR 10.3 as an async chain: queued events to
    handlers, expired durations, disabled cards, agenda win, uniqueness, restrictions, empty
    remotes, then `trigger-pending-abilities` (active player first). Events are batched by
    `queue-event` and fire only at a checkpoint.
20. CONFIRMED: primitives call it. `gaining.clj` `gain-credits`:
    `(if suppress-checkpoint (effect-completed state nil eid) (checkpoint state eid))`. `pay` runs a
    checkpoint after paying, before the effect. Cards opt out for intermediate steps
    (`clearance`: `(wait-for (gain-credits state side creds {:suppress-checkpoint true}) (draw state side eid cards))`).
21. CONFIRMED: `checkpoint.clj` `fake-checkpoint` recomputes derived values (ice, MU, hand size,
    tags) up to 10 times after every command (`process-action` calls `checkpoint+clean-up`), even
    while resolution waits on a prompt.

## 5. "Do A, then if X, do B"

22. CONFIRMED, `cards/operations.clj` Big Deal: `:async true`, `:effect` runs `wait-for` on A, then
    `continue-ability` with an `:optional` ability or `nil` (which completes at once). Loops are
    recursion (Ad Blitz, Audacity).
23. CONFIRMED: values from earlier steps survive only by lexical capture (`(let [card-to-score target] ...)`).
    The shared-context proposal was apparently never adopted (UNVERIFIED).

## Gaps

- Code read from `master`, not pinned to a commit.
- Whether a lint enforces async usage (`^:ignore-async-check` suggests one).
- Game persistence across server restarts.
- Exhaustive issue search for dropped or double-completed eids.
