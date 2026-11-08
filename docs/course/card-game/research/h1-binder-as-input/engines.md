## Summary
Of the five engines, only Spellsource passes the card database into the game object: `GameContext` holds a `CardCatalogue` field that a caller can set. The other four use process-wide globals. Forge has `StaticData.instance()`, XMage has the `CardRepository.instance` enum, SabberStone has the `static class Cards`, and Fireplace has a module-level `db`. Most tests use real cards. XMage (`CustomTestCard`) and Spellsource (`addTempCard`, plus a `"TEST"` hero class and set) also build test-only cards inline, and those never enter the global catalogue.

## Findings

**1. Forge (MTG, Java)**
- **Database:** a global, though the constructor is public. In [`forge-core/src/main/java/forge/StaticData.java`](https://github.com/Card-Forge/forge/blob/master/forge-core/src/main/java/forge/StaticData.java) the constructor runs `lastInstance = this;` and `public static StaticData instance() { return lastInstance; }`. `new Game(players, rules, match)` takes no database. A test could swap it only by building a new `StaticData`, which replaces the global. The test base comments: *"a timeout interrupt mid-load corrupts the shared StaticData cache"* ([`AITest.java`](https://github.com/Card-Forge/forge/blob/master/forge-gui-desktop/src/test/java/forge/ai/AITest.java)).
- **Tests:** they use real cards by name: `createCard(name, p)` calls `FModel.getMagicDb().getCommonCards().getCard(name)`, and `fillLibrary` uses "Runeclaw Bear". I found no test-only card set (unverified that none exists).
- **Card ID:** the card name, as a `String` key into CardDb. Prints (`PaperCard`) add edition and art. Interning is unverified.

**2. XMage (MTG, Java)**
- **Database:** a global enum singleton, `public enum CardRepository { instance; ... }`, backed by an H2 database ([`Mage/.../cards/repository/CardRepository.java`](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/cards/repository/CardRepository.java)). Tests call `CardScanner.scan(...)` and then `CardRepository.instance.findCard(cardName, true)` ([`CardTestPlayerAPIImpl.java`](https://github.com/magefree/mage/blob/master/Mage.Tests/src/test/java/org/mage/test/serverside/base/impl/CardTestPlayerAPIImpl.java)). There is no injection point.
- **Tests:** mostly real cards by name. Test-only cards are built inline with `addCustomCardWithAbility(...)`, which creates a package-private `class CustomTestCard extends CardImpl` in test sources ([`Mage.Tests/.../MageTestPlayerBase.java`](https://github.com/magefree/mage/blob/master/Mage.Tests/src/test/java/org/mage/test/serverside/base/MageTestPlayerBase.java)). They get the pseudo-set code `needSetCode = "custom"`, and their abilities sit in a static map keyed by card name. They stay out of real play because they live only in test sources and are put straight into zones, never registered in the repository.
- **Card ID:** name `String` + set code + collector number (`CardSetInfo(name, setCode, "123", Rarity.COMMON)`). Each real card is also a Java class.

**3. SabberStone (Hearthstone, C#)**
- **Database:** a global static class. In [`SabberStoneCore/src/Model/Cards.cs`](https://github.com/HearthSim/SabberStone/blob/master/SabberStoneCore/src/Model/Cards.cs), `public static class Cards { private static readonly CardContainer Data; ... static Cards() { ... cardLoader.Load(); ... } }`. It cannot be swapped.
- **Tests:** they use real cards (e.g. `Cards.FromName` / `FromId`). The only test concession found is `GameConfig.UnPredictableCardIDs`, real cards that fill-deck logic skips. No test-only cards were found (unverified that none exist).
- **Hero:** set per player in the config: `public CardClass Player1HeroClass`, `public Card Player1HeroCard` ([`GameConfig.cs`](https://github.com/HearthSim/SabberStone/blob/master/SabberStoneCore/src/Config/GameConfig.cs)). When built from a deckstring, `deck.GetHero()` fills both.
- **Card ID:** `public string Id` (e.g. `"CS2_029"`) plus `public int AssetId` (the dbfId) ([`Card.cs`](https://github.com/HearthSim/SabberStone/blob/master/SabberStoneCore/src/Model/Card.cs)).

**4. Spellsource / Metastone (Java)**
- **Database:** injected, with a global default. In [`GameContext.java`](https://github.com/hiddenswitch/Spellsource/blob/master/spellsource-game/src/main/java/net/demilich/metastone/game/GameContext.java): `private CardCatalogue cardCatalogue = ClasspathCardCatalogue.INSTANCE;`, `public GameContext(@NotNull CardCatalogue cardCatalogue)` and `setCardCatalogue(...)`. `Player` also takes a catalogue. A test can swap it, e.g. `fromTwoRandomDecks(seed, CardCatalogue cardCatalogue)`.
- **Tests:** mostly real JSON cards by id. Real cards live in `spellsource-cards-git/src/main/resources/cards/{collectible,uncollectible}/...`; tests are in `spellsource-cards-git/src/test/java/.../tests/cards/` ([`docs/architecture.md`](https://github.com/hiddenswitch/Spellsource/blob/master/docs/architecture.md)). Test-only cards are built inline as a `CardDesc` and registered per game with `context.addTempCard(card)`. The javadoc says a temporary card *"exists only in this instance and not in the CardCatalogue"*. Examples in [`TestBase.java`](https://github.com/hiddenswitch/Spellsource/blob/master/spellsource-testutils/src/main/java/net/demilich/metastone/tests/util/TestBase.java): `receive(...)` makes a "Test Minion", and `castDamageSpell` uses `setSet(TestCardResources.TEST)`. The default test hero class is `"TEST"` (`getDefaultHeroClass()`). Where `TestCardResources` lives (package `com.hiddenswitch.spellsource.cards.test`) is unverified; it may be in the private submodule. Retired real cards are excluded with `"collectible": false`.
- **Hero:** comes from the deck or a per-player class input. `setDeck` sets the *"deck and hero (by implication)"* via `deck.getHeroCard(catalogue).hero()`. `GameContext(String... heroClasses)` and `new Player(heroClass, catalogue)` look up the class's hero card. CLASS cards carry `"hero": "hero_zara"`.
- **Card ID:** a `String` equal to the JSON filename (`"minion_big_bro"`), compared with `equalsIgnoreCase`. In-test ids come from `context.getLogic().generateCardId()`.

**5. Fireplace (Hearthstone, Python)**
- **Database:** a module global in [`fireplace/cards/__init__.py`](https://github.com/jleclanche/fireplace/blob/master/fireplace/cards/__init__.py): `if "db" not in globals(): db = CardDB()`, where `class CardDB(dict[str, cardxml.CardXML])` merges `CardDefs.xml` with the Python scripts. Tests could only swap it by monkeypatching (unverified).
- **Tests:** they use real card ids. [`tests/utils.py`](https://github.com/jleclanche/fireplace/blob/master/tests/utils.py) defines aliases such as `FIREBALL = "CS2_029"`, `WISP = "CS2_231"`, plus a `BLACKLIST` of non-deterministic real cards. No test-only cards were found.
- **Hero:** a per-player constructor argument: `def __init__(self, name, deck: list[str], hero: str, ...)` ([`fireplace/player.py`](https://github.com/jleclanche/fireplace/blob/master/fireplace/player.py)). Tests pass `class1.default_hero`. `db.filter` excludes `HERO_` ids by default.
- **Card ID:** a `str` card id (`"CS2_029"`), with `db.dbf` mapping int dbfId to that id.

## Sources
- **Kept:** the repo files linked above. They are primary sources, all fetched from the raw master/HEAD branch.
- **Dropped:** the XMage 2019 commit page (it didn't show code, and current `MageTestPlayerBase` supersedes it), Reddit/forum threads (secondhand), and grep.app (blocked by a bot check).

## Gaps
- Where `TestCardResources` lives in Spellsource, and whether its `TEST` set is excluded from formats.
- Whether Forge, SabberStone or Fireplace have any test-only cards. I checked only the test base classes.
- How the original Metastone chose heroes. I believe it used `HeroClass` on the deck, but didn't check.
- Interning: none of the engines intern explicitly as far as I saw. They store ordinary `String`s in maps, apart from Java literal interning.
- Next step: clone the repos and grep for `TestCardResources`, `"set": "TEST"` and `test` under each engine's test resources.