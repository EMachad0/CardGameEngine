# Card game rules

The language of the rules core: the cards, the state of a game, the actions that change it and the
events it reports.

## Language

### Players

**Player**:
Someone taking part in a game, with an identity that stays the same all game.
_Avoid_: controller, side

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

**Hero**:
The piece that represents a player in the game.
_Avoid_: avatar

### Cards

**Card definition**:
A card as printed: its name, cost, kind, stats and text. Every copy of the card shares it.
_Avoid_: template, prototype

**Object**:
One card in one game. Two copies of the same card are two objects.
_Avoid_: instance, entity

**Minion**:
A card that stays on its owner's board once played.
_Avoid_: monster, creature

**Spell**:
A card that resolves when played and is then gone.

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
A choice the game waits on from a player in the middle of resolving a card.
_Avoid_: picker

**Bury**:
Putting a card on the bottom of its owner's deck.

**State check**:
The pass that removes dead cards from the board and decides whether the game is over.
_Avoid_: death check, state-based actions

**Outcome**:
How a finished game ended: one player won, or it was a draw.
_Avoid_: result, winner

### Reports

**Event**:
The game's report of one thing that happened while it resolved an action.
_Avoid_: message, notification

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
- **Player** and **Hero**: a player makes decisions, a hero is the piece that represents them and
  takes damage.
- **Card definition** and **Object**: the definition is the printed card, shared by every copy. An
  object is one copy in one game.
- **Action** and **Event**: actions go into the game, events come out of it. A player sends an
  action, and the game reports events while it resolves.
- **Checkpoint** and **State check**: a checkpoint lets observers read views and changes nothing. A
  state check changes the game: it removes dead cards from the board and can end the game.
- **Draw** a card and a **draw** as an outcome: the first moves a card from deck to hand, the second
  is a finished game with no winner.
- **View** and **game state**: a view is what one player may see. The game state is all of it,
  hidden cards included.
