# What a dead hero or a lost player's objects can still hear

Researcher report, session 05a. Question: when a player's hero dies or the player loses, what can
that hero or the player's objects still trigger on, and does a character dying at the same time as
others hear their deaths?

## Summary

None of the four games lets a dead hero or a lost player's objects hear anything after the loss,
and the game never resolves their pending effects either. MTG, in multiplayer, removes the losing
player's objects at once and drops their pending triggers (800.4a, 800.4d). Hearthstone kills the
hero in the same Death Creation Step as the minions, and anything removed from play "cannot
trigger". LoR ends the game the moment a Nexus hits 0, but only community sources say so. In
Hearthstone and MTG, a character that dies at the same moment as others does not hear their deaths.

## 1. MTG comprehensive rules

Source: [yawgatog CR, rules effective Sep 25 2026](https://yawgatog.com/resources/magic-rules/).

1. **800.4a**: "When a player leaves the game, all objects (see rule 109) owned by that player
   leave the game and any effects which give that player control of any objects or players end.
   Then, if that player controlled any objects on the stack not represented by cards, those objects
   cease to exist. Then, if there are any objects still controlled by that player, those objects
   are exiled. This is not a state-based action. It happens as soon as the player leaves the game."
2. **800.4d** covers abilities waiting to go on the stack: "If a triggered ability that would be
   controlled by a player who has left the game would be put onto the stack, it isn't put on the
   stack." The rule's example is a delayed Astral Slide trigger that "triggers, but it isn't put on
   the stack." So a lost player's abilities can still trigger, but they never resolve.
3. **800.4b**: an object that "would be put onto the battlefield or onto the stack under the
   control of a player who has left the game, … remains in its current zone." No tokens are created
   for that player.
4. **800.4i**: effects that need information about a player who left use "last known information
   about that player before they left the game."
5. **800.4j**: "If a player leaves the game during their turn, that turn continues to its
   completion without an active player." **800.4k**: the turn of a player who has left "doesn't
   begin."
6. **800.4**: "Unlike two-player games, multiplayer games can continue after one or more players
   have left the game."
7. **104.3a**: "A player who concedes leaves the game immediately." **104.3b**: "If a player's life
   total is 0 or less, that player loses the game the next time a player would receive priority.
   (This is a state-based action.)"
8. **704.3**: the game "performs all applicable state-based actions simultaneously as a single
   event." So players who lose to the same check lose together. **104.4a**: "If all the players
   remaining in a game lose simultaneously, the game is a draw."
9. "Loses the game" triggers exist, and other players' objects hear them. Blood Tyrant reads
   "Whenever a player loses the game, put five +1/+1 counters on this creature." Its ruling says it
   "will trigger no matter how a player loses the game: due to a state-based action…, a concession,
   or a game loss awarded by a judge."
   ([Scryfall](https://scryfall.com/card/con/99/blood-tyrant))
   - Inference, not a quoted rule: if the Tyrant's own controller is the one who lost,
     800.4a and 800.4d mean that trigger never reaches the stack.
10. **603.10a**: leaves-the-battlefield abilities "look back in time." No CR rule found on whether
    leaving the game under 800.4a fires leaves-the-battlefield triggers. Unverified.

## 2. Hearthstone

Source: [Advanced rulebook](https://hearthstone.wiki.gg/wiki/Advanced_rulebook).

1. Heroes die in the same step as minions. Rule 4a: the Death Creation Step "Looks for all mortally
   wounded … / pending destroy … Entities and kills them, removing them from play simultaneously …
   Entities that have been removed from play cannot trigger, be triggered, or emit auras."
   - The step "in order of play, sends them from play to the graveyard and creates Death Events
     for each." On order: "Your Heroes are older than every minion."
   - The list of things that go to the graveyard includes "dead heroes". The hero leaves play.
2. Destroying a hero sets a flag, not an immediate loss: "Hero in Play Zone: Set pending destroy.
   When the Hero actually dies, your Player Entity's PLAYSTATE tag is set to LOSING, and the next
   time Hearthstone checks for win/loss/draw you lose the game."
3. Simultaneous deaths cannot save the hero: "You and the Zombie Chow die and are removed from play
   - it is too late for the Zombie Chow's Deathrattle to save you, and you lose the game."
4. A dead minion does not hear deaths in the same batch: "The Cult Master cannot trigger on the
   Bloodfen Raptor's Death because it is removed from play at the same time."
5. The game ends only at the end of the Sequence, so the rest of the Sequence still resolves.
   - Each sequence ends with "Hearthstone checks for win/loss/draw."
   - Hero Power example: "the game ends in a tie instead of a win, because Hearthstone checks for
     win/loss/draw only at the end of the Sequence."
   - Hellfire example: "the Knife Juggler kills your opponent. However, the Sequence continues, and
     you kill yourself … the game ends in a draw."
   - Exception: a check also runs after the pre-attack step, so in the Explosive Trap example "The
     Stormwind Knight's attack never finishes, and you lose."
6. Whether a dead hero's own Secrets, Quests or hero-card enchantments fire on its death is not
   stated. Unverified. Rule 4a's "removed from play" suggests triggers on the hero entity itself
   cannot fire.

## 3. Legends of Runeterra

1. [LoR wiki, Nexus](https://wiki.leagueoflegends.com/en-us/LoR:Nexus): the Nexus "is destroyed
   when its health is depleted or the player otherwise loses the game … at which point the game
   ends." Also: "If both Nexuses are destroyed by effects that occur simultaneously, the game ends
   in a Tie game."
2. Community sources only, unverified against Riot:
   - "the game instantly ends once a nexus reaches zero health, even if this happens in the middle
     of a stack or combat phase"
     ([reddit](https://www.reddit.com/r/LegendsOfRuneterra/comments/gim4du/vi_judgment_tie_ends_in_defeat/)).
   - On lifesteal later in the same combat: "Immediately when lethal is dealt"
     ([reddit](https://www.reddit.com/r/LegendsOfRuneterra/comments/r1vlea/exactly_when_is_the_match_lost_or_won/)).
   - So pending effects, including Last Breath from the same moment, appear not to resolve.
3. Whether "when X dies" effects of units dying in the same moment fire before the game ends: not
   found.

## 4. Engines

1. XMage, `GameImpl.leave()`
   ([source](https://raw.githubusercontent.com/magefree/mage/master/Mage/src/main/java/mage/game/GameImpl.java)),
   in this order:
   - Returns early `if (checkIfGameIsOver())`, commented "no need to remove objects if only one
     player is left so the game is over".
   - Otherwise moves owned permanents to `Zone.OUTSIDE`, commented "needed to send event that
     permanent leaves the battlefield to allow non stack effects to execute". XMage emits
     zone-change events.
   - Runs the leaver's non-stack triggers right away.
   - Then `getStack().removeIf(object -> object.isControlledBy(playerId))`.
2. Fireplace
   ([actions.py](https://raw.githubusercontent.com/jleclanche/fireplace/master/fireplace/actions.py),
   [card.py](https://raw.githubusercontent.com/jleclanche/fireplace/master/fireplace/card.py)):
   - `Death.do` treats every dead card the same way:
     `card.zone = Zone.GRAVEYARD; source.game.check_for_end_game()`.
   - `Hero._set_zone`: `elif value == Zone.GRAVEYARD: if self.controller.hero is self:
     self.controller.playstate = PlayState.LOSING`.
   - `check_for_end_game` gives `TIED` when both players have the same playstate.
   - The hero is removed from play on death.
3. Forge has a `LosesGame` trigger: "Goes off when a player loses the game. ValidPlayer…"
   ([wiki](https://github.com/Card-Forge/forge/wiki/Triggers)). Its code for removing a lost
   player's permanents was not checked. Unverified.
4. Sabberstone and Metastone: not checked.

## Gaps

- No official Riot source on Nexus death in the middle of resolution, or on Last Breath in the same
  moment.
- No CR text on whether leaving the game under 800.4a fires leaves-the-battlefield triggers. XMage
  fires the events, but that is the engine's choice.
- For Hearthstone, nothing on triggers attached to the hero itself (Secrets, Quests, enchantments)
  at the moment it dies.
- Not read: Forge `GameAction.checkGameOverCondition` / `onPlayerLost` and Sabberstone
  `Game.DeathProcessingAndAuraUpdate`.
