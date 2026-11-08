## Summary
Both shapes have solid precedent. Card engines usually split a description type from a runtime type, as in B, but often keep a string ID inside the runtime type and look it up when the effect runs. Spellsource does this. rustc's AST→HIR lowering is the clearest precedent for B that also resolves names. GHC's "Trees That Grow" is the clearest precedent for A: one tree whose name type changes per compiler phase. A default type parameter keeps code that writes `PlayerVerb` in type position compiling. It does nothing in expressions, and it cannot be put on `fn` or `impl` parameters.

## Findings

**1. Card engines (split, and whether references are resolved)**

1. **Spellsource/Metastone: B-shaped, but references stay strings.** `CardDesc` is "the class that card JSON files deserialize (get decoded) into… should only store the data related to the card". The conversion runs from data to runtime: `public Card create() { return new Card(this); }`. However, `SummonSpell.create(String Card)` stores `SpellArg.CARD` as a string, and when the spell is cast it calls `card = context.getCardById(replacementCard)`. So references are looked up at runtime and are not resolved by the conversion. [CardDesc.java](https://raw.githubusercontent.com/hiddenswitch/Spellsource/master/spellsource-game/src/main/java/net/demilich/metastone/game/cards/desc/CardDesc.java), [SummonSpell.java](https://raw.githubusercontent.com/hiddenswitch/Spellsource/master/spellsource-game/src/main/java/net/demilich/metastone/game/spells/SummonSpell.java)
2. **SabberStone: references are resolved when the definition is built.** `public SummonTask(string cardId, …) { Card = Cards.FromId(cardId); … }`. The task holds a `Card` object, and `Process` calls `Entity.FromCard(controller, Card, …)`. Lookup happens when the card definition is constructed, not during play. A bad ID gives null and only a log warning ("Cannot summon … Please Check the implementation"), not a load error. [SummonTask.cs](https://raw.githubusercontent.com/HearthSim/SabberStone/master/SabberStoneCore/src/Tasks/SimpleTasks/SummonTask.cs)
3. **Fireplace: there is no split.** Card classes write `play = Summon(CONTROLLER, "CS2_152")`, and the string ID stays as written. [neutral_common.py](https://raw.githubusercontent.com/jleclanche/fireplace/master/fireplace/cards/classic/neutral_common.py)
4. **Forge: the script is parsed into `CardRules`, and some name references are resolved eagerly with a hard failure.** `CardRules.fromScript(...)` parses the text. Placeholder faces are filled in "after all scripts have been processed", and a missing name throws `NoSuchElementException("Missing placeholder face … Cannot find …")`. That matches the binder idea. *Unverified:* how `CardFactory` turns `CardRules` into `Card`/`CardState`, and whether `TokenScript$` references stay strings until runtime. [CardRules.java](https://raw.githubusercontent.com/Card-Forge/forge/master/forge-core/src/main/java/forge/card/CardRules.java)

**2. Rust precedent for B**

5. **rustc HIR (RFC 1191).** The RFC wants to "separate the interface to the compiler (the AST) from its implementation (the HIR)", including "using resolved names in the HIR (i.e., performing name resolution as part of the AST->HIR lowering)". It names the cost directly: "Tools such as the AST visitor will need to be duplicated." Initially the HIR was "an (almost) identical copy of the AST". [RFC 1191](https://rust-lang.github.io/rfcs/1191-hir.html)
6. **Cargo `TomlManifest` → `Manifest`.** `to_real_manifest(..., original_toml: TomlManifest, normalized_toml: TomlManifest, ...) -> CargoResult<Manifest>`. The schema types were moved into their own crate, `cargo-util-schemas`, so other tools (crates.io) can use them without drifting out of sync. The author admits the cost: "On its own, this PR is a net negative for readability / complexity." [to_real_manifest](https://doc.rust-lang.org/stable/nightly-rustc/cargo/workspace/parser/fn.to_real_manifest.html), [cargo#12911](https://github.com/rust-lang/cargo/pull/12911), [cargo#12801](https://github.com/rust-lang/cargo/issues/12801)
7. **serde `try_from`.** `#[serde(try_from = "FromType")]`: "Deserialize this type by deserializing into FromType, then converting fallibly." This is the standard Rust "raw DTO → validated type" pattern. It does not fit our case directly, because resolving a `DefCode` needs the binder's context and `TryFrom` takes none. [serde container attrs](https://serde.rs/container-attrs)

**3. Rust/Haskell precedent for A**

8. **Trees That Grow (Najd & Peyton Jones).** The paper frames the trade-off directly: "Common practice… is either to define a new separate data type… or to use the same large data type… Both methods are unsatisfactory: the former leads to duplication, both in the definition of data types and in the functions defined over them." GHC's `HsSyn` is indexed by pass (`GhcPs`/`GhcRn`/`GhcTc`). *Not fetched, from memory:* the name type per pass comes through `IdP p` (`RdrName` → `Name` → `Id`). That is exactly A's "same tree, the reference type changes per phase". [arXiv 1610.04799](https://arxiv.org/abs/1610.04799), [GHC commit](https://github.com/TravisWhitaker/ghc/commit/8e6ec0fa7431b0454b09c0011a615f0845df1198)
9. **Derive bounds spread.** `#[derive]` adds `C: Trait` for every type parameter, whether or not it is needed. The issue is still open: "The bounds that `#[derive]` uses aren't conservative, they're just wrong." So `DefCode` and `DefId` must both implement every derived trait. Any bound you write by hand also spreads through every generic helper. [rust#26925](https://github.com/rust-lang/rust/issues/26925)

**4. Default type parameters (concrete Rust behaviour)**

10. **They help only in type position.** "If you use HashMap as a type… rust will implicitly understand [the default]… But if you refer to HashMap as part of an expression… the default isn't 'turned on' – all 3… are free to be inferred." `HashMap::new()` with no later use fails with E0282. [Faultlore](https://faultlore.com/blah/defaults-affect-inference/) For `enum PlayerVerb<C = DefId>` this means:
    - Field types, signatures and `impl PlayerVerb { … }` keep compiling, but they now mean `PlayerVerb<DefId>` only.
    - `PlayerVerb::Summon(x)` infers `C` from `x`.
    - A variant with no `C`, such as `let v = PlayerVerb::Pass;` with nothing constraining `v` later, fails to infer. *This is inferred from the rule above; I did not compile it.*
11. **Defaults are not allowed on `fn` or `impl` parameters.** The deny-by-default `invalid_type_param_default` lint rejects `fn foo<T=i32>`. The `default_type_parameter_fallback` feature (RFC 213) was removed, and PR #127655 "Fixes #27336 by removing the feature gate". Generic helpers must still be written `impl<C> … for PlayerVerb<C>`. [rustc lints](https://doc.rust-lang.org/rustc/lints/listing/deny-by-default.html), [rust#27336](https://github.com/rust-lang/rust/issues/27336), [rust#127655](https://github.com/rust-lang/rust/pull/127655)

## Which fits when

- **B (public written type + private internal type)** fits when:
  - the written form should stay simple and stable for other crates (RFC 1191's "interface vs implementation" argument);
  - the internal form is expected to change shape: desugared, flattened, with precomputed indexes, extra fields, or one variant split into several;
  - you want `DefId` to never appear in public types at all.

  Cost: two parallel enums plus a conversion that must be updated every time a variant is added. The compiler's exhaustive `match` in the conversion catches forgotten variants. Keep the conversion in one place.
- **A (one tree, generic over the reference type)** fits when the two trees would really be the same except for the reference type, so the copy is close to 100% (this is GHC's situation). Cost: derive bounds on both parameters, `<C>` on every generic `impl`, and defaults that do not help in expressions. It also exposes the internal tree's shape publicly, which means public API commitments.
- **Card engines** rarely use A. The split design is common, but the most-cited engine (Spellsource) still keeps strings in the runtime type. SabberStone and Forge show the binder pattern: resolve when the card is built and fail loudly. SabberStone fails only softly (null plus a warning). If the learner picks B, the value comes from the internal type holding `DefId` rather than a string. The split by itself does not give that.

## Sources
- Kept:
  - Spellsource `CardDesc.java` / `SummonSpell.java`: primary source, verified quotes.
  - SabberStone `SummonTask.cs`: primary source.
  - Fireplace `neutral_common.py`: primary source.
  - Forge `CardRules.java`: primary source.
  - RFC 1191: the reasoning for the HIR, from its author.
  - Cargo `to_real_manifest`, PRs #12911 / #12801: real-world B in Rust.
  - serde container attributes: official docs.
  - Trees That Grow paper: the precedent for A.
  - rust#26925, rust#27336, rust#127655, the rustc lint listing: authoritative on defaults and derives.
  - Faultlore post (Aria Desires): clear explanation of the type-vs-expression rule.
- Dropped:
  - DEV.to / rust-skills articles on serde validation: they repeat serde's docs.
  - The Spellsource javadoc site: pages returned 404.
  - The GHC wiki: needs JavaScript, so it could not be fetched.

## Gaps
- Forge `CardFactory` / `CardState` and how token references are resolved: unverified.
- GHC's `IdP` mapping is quoted from memory.
- The `PlayerVerb::Pass` E0282 case is reasoned, not compiled. Write a 10-line playground test to confirm it.
- I found no statement from the cargo or rustc authors measuring "drift" cost beyond the duplication and readability quotes above.