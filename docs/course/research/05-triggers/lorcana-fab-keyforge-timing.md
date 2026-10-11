# Lorcana, Flesh and Blood, KeyForge: simultaneous removal and state checks

Researcher report, session 05. Lorcana and KeyForge PDFs were read through a question-answering
fetch tool, not as raw text: check exact wording against the PDFs before quoting.

## Summary

Lorcana looks back (cards banished together see each other leave) and runs a game state check after
every effect, including each triggered ability. Flesh and Blood clears allies at 0 life as a game
state action whenever the game enters a new priority state, which happens after each layer
(including each trigger) resolves; it has a look-back rule for a card's own removal, but no explicit
rule for "another ally dies" when several go together. KeyForge has no state check: lethal damage
destroys at once; "Destroyed:" abilities resolve while the card is still in play; "after a creature
is destroyed" abilities on cards destroyed in the same event do not resolve.

## Disney Lorcana

Comprehensive Rules v2.0.1, effective Feb 5 2026
(https://files.disneylorcana.com/Disney-Lorcana-Comprehensive-Rules-020526-EN-Edited.pdf).

1. Simultaneous banish, look-back CONFIRMED (it triggers). 7.4.3: "Whenever 1 or more cards would
   leave play, they first check whether any abilities would trigger from them and/or others leaving
   play with them. The triggered abilities 'see' the other cards leaving the Play zone along with
   the source card of the triggered ability." Example: Lyle Tiberius Rourke ("Whenever one of your
   other characters is banished…") banished by Be Prepared with two others: his ability "occurs
   twice, once for each character banished along with Lyle."
2. Own "when banished" uses the state before leaving PARTLY CONFIRMED. 6.7.6: "If an ability or
   effect needs to reference a card's characteristic from when it was in play but that card isn't in
   play anymore, the ability or effect references the last known value for that characteristic from
   before the card left play." 7.1.6: a card leaving play "becomes a new card".
3. Lethal damage removed at a defined check CONFIRMED. 1.8.1: "The game checks for the listed
   conditions at the end of every step of the Start-of-Turn Phase, after the Challenge Declaration
   and Challenge Damage steps…, after every turn action is completed, after any effect resolves, and
   when the turn ends." 1.8.1.4: "If a character or location has damage equal to or greater than its
   Willpower {W}, that character or location is banished." 1.8.3: "After a game state check is
   completed, the game state check immediately occurs again." (Example B: a cascade through a lost
   Willpower bonus; it cites "1.8.1.5", which doesn't exist, a numbering slip.)
4. Trigger ordering and a check after each trigger CONFIRMED. 7.7.3.1: triggers "added to the bag
   simultaneously". 7.7.4.2: the active player "choose[s] any one of their triggered effects and
   fully resolve[s] it". 7.7.4.3: "After the effect has resolved and the game state check has
   completed, the bag checks…"; the same player continues until their triggers are gone, then the
   bag passes in turn order (7.7.4.4). 6.7.4: "A game state check occurs after every effect
   resolves." 1.8.2: triggers caused by a check "won't resolve until after the game state check or
   checks are completed."

## Flesh and Blood

Comprehensive Rules at rules.fabtcg.com.

1. "Another ally dies" when allies die together PARTLY CONFIRMED. 5.4.6a covers only the card's own
   removal: "If the source of a triggered-static ability ceases to exist from an event that would
   trigger its effect, and the ability was functional immediately before the event occurs, then the
   triggered effect is triggered." (https://rules.fabtcg.com/en/cr/05-layers-cards-abilities/)
   "Destroy all" effects are multi-events: 1.8.6d "the player determines the order of the individual
   events"; 1.9.2a a trigger on a multi-event "does not trigger again for any of the individual
   events." (https://rules.fabtcg.com/en/cr/01-game-concepts/) No rule settles whether a card
   destroyed in the same multi-event sees the others.
2. Own "when destroyed" uses last known information CONFIRMED. 1.2.3: "Last known information about
   an object is a snapshot of the state of an object immediately before it ceased to exist." 1.2.3b:
   it "includes its state, counters, parameters, history, and effects applied."
3. Lethal damage removed by game state actions CONFIRMED. 2.5.3f: "If a living-card's life total is
   reduced to zero, it is cleared as a game state action." 2.5.3g: "If a living-card ceases to
   exist, it is considered to have died." (https://rules.fabtcg.com/en/cr/02-object-properties/)
   1.10.2: "When the game transitions to a new priority state, the following game state actions are
   performed first." 1.10.2b clears "Living-cards that have 0 life total."
4. Trigger ordering and checks after each layer CONFIRMED. 6.6.6b: "the turn-player selects a
   player, then each player in clockwise order adds all pending triggered-layers they control onto
   the stack in an order they choose." (https://rules.fabtcg.com/en/cr/06-effects/) 1.10.2d: new
   triggered layers go on the stack during game state actions. 5.3.7: after a layer resolves, "it is
   cleared, then the turn-player gains priority", a new priority state, so game state actions run
   after every layer. 1.11.5: no priority "during game state actions."

## KeyForge

Master Rulebook v18.0 (https://keyforging.com/wp-content/uploads/2025/02/KeyForge-Rulebook-v18-0.pdf).

1. Simultaneous destruction CONFIRMED (does not trigger). DESTROYED, p. 12: "If an ability resolves
   'after' a card is destroyed, the card with that ability must be in play at the time the destroyed
   card leaves play." FAQ p. 34 (Tolas destroyed by Gateway to Dis): "No. Tolas' ability does not
   trigger until cards are actually put into discard piles, at which point Tolas is already
   destroyed." Stealer of Souls FAQ p. 30: "destroyed (and leave play) simultaneously". FFG v1.2:
   "any non-'Destroyed:' abilities cannot trigger".
2. Own "Destroyed:" resolves while the card is still in play CONFIRMED. "After it is tagged, then
   that card's 'Destroyed:' abilities resolve, and finally the tagged card is placed into its
   owner's discard pile." "A card only resolves 'Destroyed:' abilities that it had at the time it
   was tagged for destruction." Timing Chart p. 52: tag, then "Destroyed" effects, then all destroyed
   cards to discard together, then "after/each time a creature is destroyed" effects.
3. Destruction immediate, no deferred check CONFIRMED. DAMAGE: "If a creature has an amount of
   damage on it equal to or greater than its power, the creature is destroyed… If multiple creatures
   are damaged by a single effect, that damage is dealt simultaneously." p. 22: "all damage dealt by
   a card's ability is dealt simultaneously regardless of where it appears in the ability's text."
4. Ordering CONFIRMED; no state check CONFIRMED by absence. ACTIVE PLAYER p. 7: "Whenever multiple
   effects happen at the same timing point, the active player decides the order in which those
   effects resolve." DESTROYED: "the active player chooses the order in which to resolve the
   'Destroyed:' abilities." "None of the cards that have been tagged for destruction are put into
   their owners' discard piles until all 'Destroyed:' effects have finished resolving."

## Gaps

- FaB question 1 unsettled; Lorcana's Oct-2026 "For Review" CR update not used; KeyForge "Leaves
  Play:" has no dedicated rule in v18.
