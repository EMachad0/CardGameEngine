---
status: accepted
---

# A pick reads its options in place, and the resolving effect holds the picking until its verbs run

When resolution reaches a `Picked` effect with at least one option, it opens a picking and stops.
The options stay in their zones. The picking records their ids, the cards in the window that the
kind filter excluded, and the effect's choice and verbs, and the resolving effect holds it while its
cursor moves past the effect. Each `Pick` action adds one object. Once the pick is complete, the
effect's verbs run on the objects picked, then the verbs in `others` run on everything else the pick
covered: the options left and the excluded window cards. The picking is then dropped, and no answer
outlives its effect. The source's owner answers. The chooser sees the options face up, and the
other players see only how many there are. A pick with no options is skipped.

Cards stay in place because looking moves nothing in the games this engine follows: MTG 701.20b
says revealing a card doesn't move it, and the YGO rulebook treats excavated cards as still in the
Deck. XMage and Forge keep looked-at cards in the library. Cards nobody moves then stay where they
were, in order, with no stored position, looking reports no zone change, and every reader sees the
true deck. Nothing can move them while the pick is open, since only the pick's answers are legal and
the loop doesn't step. The picking lives with the resolving effect because a pick can't outlast its
resolution, and two resolutions from one source then can't share an answer (MTG 113.7a).

This removes `Reveal`, `ResolvePick`, `PickId`, `ZoneMove`, the stored `Picks` and the load check
that paired each reveal with its resolve.

## Considered options

- Taking the cards out of every zone while the player looks, as the code did. Every card then needs
  a destination, and a load check made sure each had one.
- A set-aside zone, as SabberStone uses for Tracking. Cards not taken that stay in the deck would
  need their old positions stored, and looking would count as two moves.
- Answers stored by source and pick id in player input. Two resolutions from one source collide.
- The picking beside the cursor, with the cursor left on the effect. A picking could then sit next
  to an effect that picks nothing.

## Consequences

- Whether a resolving effect is done counts the picking: a list whose last effect picks isn't done
  until the pick is answered.
- Reading options in place relies on nothing happening during a pick. A feature that lets a player
  act while a pick is open must revisit it.
- Whether a pick opens can tell the other players that a hidden zone holds no option.
- What the chooser saw is not remembered once the pick closes.
- A chooser other than the source's owner, the order of the cards in `others`, and `others` on
  `Random` are not designed.
