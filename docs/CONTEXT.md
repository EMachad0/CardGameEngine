# Card game rules

The language of the rules core: the cards, the state of a game, the actions that change it and the
events it reports.

## Language

### Players

**Player**:
Someone taking part in a game, with an identity that stays the same all game.
_Avoid_: controller

**Seat**:
A place in the turn order.
_Avoid_: position

**Active player**:
The player whose turn it is.
_Avoid_: current player, turn player

**Opponent**:
Another player, from one player's point of view.
_Avoid_: other player

**Caster**:
The player who played the card being resolved.

**Owner**:
The player an object belongs to.

**Side**:
Whether a player or object is friendly or enemy, from one player's point of view.

**Friendly**:
The player whose point of view is taken, or an object that player owns.

**Enemy**:
An opponent, or an object an opponent owns.

**Hero**:
The piece that represents a player in the game.
_Avoid_: avatar

### Cards

**Card definition**:
A card as printed: its name, cost, kind, stats and text. Every copy of the card shares it.
_Avoid_: template, prototype

**Printed**:
What a card definition states, static, independent of context.
_Avoid_: base, original

**Binder**:
The card definitions a game can use, each found by its ID.
_Avoid_: card table, card database, catalog

**Set**:
A group of card definitions released together, such as the base set.
_Avoid_: expansion, edition

**Object**:
One card in one game. Two copies of the same card are two objects.
_Avoid_: instance, entity

**Minion**:
A card that stays on its owner's board once played.
_Avoid_: monster, creature

**Spell**:
A card that resolves when played and is then gone.

**Character**:
A minion or a hero.

**Cost**:
The mana a card in hand takes to play right now, which can differ from its printed cost.
_Avoid_: price

**Health**:
What a hero or a card on the board has left before it dies.
_Avoid_: life, hit points

**Damage**:
Health that has been lost.

**Fatigue**:
The damage a hero takes for a draw from an empty deck.

**Buff**:
A raise to a card's attack or health that comes from another card.
_Avoid_: enchantment

**Aura**:
A buff that a card on the board gives other cards for as long as it stays there.

### Zones

**Zone**:
A place an object can be, such as a deck, a hand or a board.
_Avoid_: area, location

**Deck**:
A player's cards not yet drawn, in order from the top.
_Avoid_: library, draw pile

**Hand**:
The cards a player holds and can play.

**Board**:
The cards a player has in play, in order from left to right.
_Avoid_: battlefield, field, play area

**Graveyard**:
A zone where a player puts his dead minions.
_Avoid_: discard pile, cemetery

### Turns

**Turn**:
A stretch of the game in which the active player acts.

**Mana**:
What a player spends to play cards.

**Max mana**:
The amount a player's mana refills to when their turn starts.

**Draw**:
Moving the top card of a player's deck into their hand.

### Actions and resolution

**Action**:
A decision a player sends to the game, such as playing a card, picking or ending the turn.
_Avoid_: move, command

**Legal action**:
An action the game accepts from a given player right now.
_Avoid_: valid move

**Play**:
Paying a card's cost, taking it from the hand and resolving it.

**Targeting**:
A card a player has started to play but hasn't played yet, with what they have targeted for it so
far.
_Avoid_: draft

**Target**:
To name what an effect will act on while playing the card, before it resolves. As a noun, one
thing a card declares the player must name. The objects named for it are targeted, not targets.
_Avoid_: choose

**Resolve**:
To carry out a card's text.

**Effect**:
One instruction in a card's text.
_Avoid_: ability, entry

**Selector**:
The part of an effect that says what it acts on.
_Avoid_: targeteer

**Restore**:
Giving back health that has been lost.
_Avoid_: heal

**Reveal**:
Taking cards from the top of a deck for a player to pick from.

**Pending pick**:
A pick the game waits on from a player in the middle of resolving a card.
_Avoid_: picker

**Awaiting**:
What the game waits on from one player before it goes on: targeting or a pending pick.
_Avoid_: question, prompt, request

**Player input**:
What the game awaits from each player, and what each player targeted and picked for a card.
_Avoid_: interaction, answers

**Bury**:
Putting a card on the bottom of its owner's deck.

**Trigger**:
Card text that reacts to an event. It matches when the event happens and resolves later.
_Avoid_: listener, hook

**Matched trigger**:
A trigger an event has matched, with the card it belongs to.

**Death rattle**:
A trigger on its own card's death.
_Avoid_: deathrattle, own death trigger

**Pending effect**:
Text waiting to resolve, with the card it came from: a played card's effects or a matched trigger.
_Avoid_: unit

**Effect stack**:
The pending effects waiting to resolve. The newest resolves first.
_Avoid_: queue

**Resolving effect**:
The pending effect being carried out now, with its place in the text. It can pause on a pending pick.
_Avoid_: unit, current trigger

**State check**:
The pass that removes dead cards from the board and decides whether the game is over.
_Avoid_: death check, state-based actions

**Death**:
A character dying, as a state check finds it.

**Outcome**:
How a finished game ended: one player won, or it was a draw.
_Avoid_: result, winner

**Standings**:
Which players are still playing, and the outcome once the game is over.
_Avoid_: survivors

**Aftermath**:
What an action or a resolving effect leaves for the state check to settle: the dead, the standings
and whether the game is over.
_Avoid_: cleanup

### Reports

**Event**:
The game's report of one thing that happened while it resolved an action.
_Avoid_: message, notification

**Report**:
To make an event known as it happens.
_Avoid_: emit

**Checkpoint**:
A moment during an action when each player's view can be read.

**View**:
Everything one player may see of the game at one moment.
_Avoid_: snapshot

**Viewer**:
The player a view is for.

**Face**:
The part of a card that can be hidden from a player: its definition and cost.

## Flagged ambiguities

- **Player** and **Seat**: a player is who plays, a seat is a place in the turn order. A player's
  identity doesn't change with their seat.
- **Player** and **Side**: a side is friendly or enemy, from one player's point of view. It is
  never another word for a player.
- **Player** and **Hero**: a player makes decisions, a hero is the piece that represents them and
  takes damage.
- **Card definition** and **Object**: the definition is the printed card, shared by every copy. An
  object is one copy in one game.
- **Action** and **Event**: actions go into the game, events come out of it. A player sends an
  action, and the game reports events while it resolves.
- **Target** and **Pick**: a player targets while playing a card, before it resolves, from what is
  already in play. A player picks while a card resolves, from the cards it revealed.
- **Pending effect** and **Resolving effect**: a pending effect waits on the effect stack. The
  resolving effect has been taken off it and is being carried out, one effect at a time.
- **Checkpoint** and **State check**: a checkpoint lets observers read views and changes nothing. A
  state check changes the game: it removes dead cards from the board and can end the game.
- **Aftermath** and **State check**: the aftermath is what an action or an effect leaves to settle.
  The state check is the pass that settles it.
- **Standings** and **Outcome**: the standings say who is still playing at any moment. The outcome
  exists only once the game is over, and the standings hold it from then on.
- **Draw** a card and a **draw** as an outcome: the first moves a card from deck to hand, the second
  is a finished game with no winner.
- **View** and **game state**: a view is what one player may see. The game state is all of it,
  hidden cards included.
