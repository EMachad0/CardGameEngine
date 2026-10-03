# Dependent sequencing and heroes as objects: verified facts (session 04)

Researcher report, 2026-10-03.

## Yu-Gi-Oh PSCT conjunctions
Source: YGOrganization, Demystifying Rulings Part 5, https://ygorganization.com/learnrulingspart5/

| | A required for B | A not required for B |
|---|---|---|
| Simultaneous | "and if you do" | "also" |
| Sequential | "then" | "also, after that" |

- "B is never required to do A - if you can't do B when the effect resolves, you still do A (ie. you resolve as much as you can)."
- "and" (no qualifier): "you have to be able to do both A and B at resolution, otherwise you do nothing." Some older PSCT cards print "and" meaning "and if you do".
- "Conjunctions do not tell us anything about activation legality!"
- "then" makes A not one of the last things to happen, so a "When ... you can" trigger on A misses the timing (Soul Taker destroying Lightpulsar Dragon). With "and if you do" (Blackship of Corn) it can activate.
- Yugipedia's PSCT table (read from search snippets, direct fetch was 403): "Sequentially | also, after that | then", "Simultaneously | also | and if you do | and".
- Konami's own post (https://yugiohblog.konami.com/articles/?p=4514) not fetched.

## MTG
Source: https://yawgatog.com/resources/magic-rules/ (effective 2026-09-25)
- 608.2c: "The controller of the spell or ability follows its instructions in the order written. However, replacement effects may modify these actions."
- 118.12, "If you do": "The action [do something] is a cost, paid when the spell or ability resolves. The 'If [a player] [does, doesn't, or can't]' clause checks whether the player chose to pay an optional cost or started to pay a mandatory cost, regardless of what events actually occurred." The dependency is on the player's act, not on the outcome.
- "Then" has no CR definition. 101.3: "Any part of an instruction that's impossible to perform is ignored." With 608.2c this reads as sequence with no dependency (inference). 701.70a uses "Draw a card, then discard a card." Gatherer, Teferi's Protégé: "Nothing can happen between the two."
- 608.2b: "Illegal targets, if any, won't be affected by parts of a resolving spell's effect for which they're illegal. Other parts of the effect for which those targets are not illegal may still affect them."
- 608.2e: in multiplayer, choices for each action are made in APNAP order, then that action is processed simultaneously, then the next.

## Legends of Runeterra
- Deny is a spell card ("Stop a Fast spell, Slow spell, or Skill."), not a keyword.
- No official "if you do" convention found. LoR uses "X to Y" (old Rummage: "Discard 2 to draw 2"), reworded to "To play, discard up to 2 cards. Draw 1 for each card you discarded."

## Hearthstone: hero vs player
- https://hearthsim.info/docs/gamestate-protocol/: entity types `Game` (ID 1), `Player` (IDs 2 and 3), `Card` ("Everything that isn't a Game or a Player is a Card"). Card type `HERO` covers the starting heroes and the Death Knight heroes. "Player 1's initial Hero and Hero Power is created." "the game immediately ends when at least one player has no hero in `PLAY`."
- The Player entity's `HERO_ENTITY` tag points at the hero card entity.
- Hero cards (https://hearthstone.wiki.gg/wiki/Hero_card): "Upon being played, a hero card will replace the player's hero"; health carries over, armor is gained, the Hero Power is replaced.
- Lord Jaraxxus (Classic): summoned as a minion, then his Battlecry transforms him into a hero who "takes over from the playing hero". "any modifiers affecting the previous hero will not be transferred." Health set to 15.
- Not verified: how the real Power.log reports a hero replacement.

## SabberStone
`SabberStoneCore/src/Model/Entities/`:
- `Character : Playable, ICharacter`; `Hero : Character` ("The entity representing the player."); `Minion : Character`.
- `Controller : Entity` ("Instance that represents a player"), with `Hero Hero` and `HeroId` (tag `HERO_ENTITY`). Not a `Playable`.
- `Controller.AddHeroAndPower` moves the old hero and power to `SetasideZone` and builds a new `Hero` with `FromCard`, keeping the weapon.

## MTG: players are not objects
- 109.1: "An object is an ability on the stack, a card, a copy of a card, a token, a spell, a permanent, or an emblem." 102.1: "A player is one of the people in the game."
- 120.3a: damage to a player causes life loss. 120.3e: damage to a creature is marked on it.

## Hearthstone tokens
- https://hearthstone.wiki.gg/wiki/Token: "Token is an unofficial term for most uncollectible minions that are summoned directly into play by other cards."
- HearthSim cards docs: CardIDs and DBF IDs exist for all cards, including heroes, hero powers and enchantments. Tokens having ordinary card definitions is an inference.
