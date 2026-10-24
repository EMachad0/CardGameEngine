# Trait impl placement: a survey of 15 codebases

Local check by the agent, outside a session, for PR #70's doc gaps. Question: where large Rust
codebases put their hand-written trait `impl` blocks, relative to the type and the trait they
name. The two researcher reports beside this file cover what the sources recommend.

## Summary

Of 19,254 trait impls, 76% sit in the module that defines their self type and 12% in the module
that defines their trait. Over half of the second group implement a local trait for a type the
crate doesn't define, a primitive, another crate's type or a generic one. 3% sit with another type the impl names, as `From<String> for Vec<u8>` does in std's
`string.rs`, and 3% in a child file of the type's or the trait's module, split by trait. The 5%
left are feature-gated impls of third-party traits in a module of their own, impls grouped by a
domain decision such as a database backend, and std's `FromStr` impls for the address types in
`core/net/parser.rs`, next to the private parser they call.

## Method

A throwaway binary, its source below, parses every `.rs` file under each package's `src/` with
`syn` 2. It skips `tests/`, `benches/`, `examples/` and fixture directories. For each `impl Trait
for Type` outside a `#[cfg(test)]` module, it compares the last path segment of the trait, the self
type and the trait's type arguments with the structs, enums, unions, type aliases and traits that
the impl's own module defines, and then with those the crate defines anywhere.

Limits:

- Names are matched, not resolved. Two items with one name in different modules can land an impl
  in the wrong column.
- Derives and impls written inside macro bodies aren't counted, since nothing is expanded. A type
  defined inside a macro, as tokio's `pin_project!` structs are, can't be found, so its impls land
  in "unresolved".
- "Subtree" means the impl's file sits under the directory of the module that defines the type or
  the trait.

Revisions scanned: std, core and alloc from the local Rust 1.92.0 sources; rust-analyzer `0c4d746`,
bevy `3a79685` (`crates/` only), tokio `8a2408e`, diesel `dde3b22`, cargo `52574fe`, regex `80b9d14`,
ripgrep `6892bc2`, serde `0db9b3b`, hyper `b073378`, axum `86aa615`, clap `ccc89b8`, chrono
`6adaa52`, indexmap `41a8708`, rayon `ee0a00b`.

## Results

Each row adds up to 100%. "Type" counts impls whose module defines the self type, including those
whose module also defines the trait. "Trait" counts impls whose module defines the trait and not
the self type. "Arg type" counts impls whose module defines a type in the trait's arguments only.

| Crate | Impls | Type | Trait | Arg type | Subtree | Other | Unresolved |
|---|---:|---:|---:|---:|---:|---:|---:|
| std | 4301 | 78% | 13% | 2% | 2% | 4% | 1% |
| rust-analyzer | 3321 | 88% | 7% | 2% | 0% | 2% | 1% |
| bevy | 4915 | 73% | 13% | 6% | 3% | 4% | 1% |
| tokio | 1155 | 68% | 6% | 1% | 0% | 8% | 17% |
| diesel | 2095 | 51% | 23% | 2% | 5% | 19% | 0% |
| cargo | 513 | 92% | 6% | 1% | 0% | 1% | 0% |
| regex | 447 | 89% | 7% | 3% | 0% | 0% | 0% |
| ripgrep | 128 | 88% | 5% | 5% | 0% | 2% | 0% |
| serde | 301 | 66% | 3% | 3% | 28% | 0% | 0% |
| hyper | 168 | 71% | 16% | 1% | 0% | 5% | 7% |
| axum | 387 | 76% | 15% | 1% | 4% | 1% | 4% |
| clap | 383 | 72% | 19% | 6% | 0% | 3% | 0% |
| chrono | 260 | 89% | 1% | 1% | 8% | 1% | 0% |
| indexmap | 307 | 78% | 5% | 2% | 2% | 11% | 3% |
| rayon | 573 | 82% | 4% | 0% | 8% | 7% | 0% |
| All | 19254 | 76% | 12% | 3% | 3% | 5% | 2% |

Files that hold many impls away from both the type and the trait:

- Split by trait inside the type's or the trait's subtree: std's `alloc/boxed/convert.rs` and
  `alloc/vec/partial_eq.rs`, serde's `ser/impls.rs` and `de/impls.rs`, rayon's `iter/extend.rs`,
  diesel's `select_statement/dsl_impls.rs`.
- Feature-gated third-party traits in their own module: indexmap's `serde.rs`, `sval.rs`,
  `arbitrary.rs` and `rayon/`.
- Grouped by a domain decision: diesel's per-backend `pg/`, `mysql_like/` and `sqlite/` type
  impls, bevy_mesh's `primitives/dim2.rs` (meshing for bevy_math's shapes), bevy_reflect's
  `impls/`, rust-analyzer's `hir/display.rs`.
- Next to private helpers the body calls: std's `core/net/parser.rs`, which holds `FromStr` for
  `IpAddr`, `Ipv4Addr`, `Ipv6Addr` and the socket address types beside its private `Parser`.

## This crate

On main at `95dea0c`, the scan finds 11 trait impls outside test modules in `crates/rules/src`.
Five sit with their self type, and `impl Observer for ()` sits with the trait. Four are `From<A> for
B` in the module of `A`, which `B`'s module can't name: `Play`, `Death`, `Ending` and
`LegalAction` into `Event` and `Action`. One sits in a module that defines neither its type nor its
trait: `impl Default for CardDef` in `cards/binder/sets/base.rs`.

## Scanner source

Built against `syn` 2 with the `full`, `visit` and `extra-traits` features, and `walkdir` 2. Run as
`implscan <name> <src dir>...`, with `VERBOSE=1` to list the impls outside the type's module.

```rust
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use syn::{Attribute, GenericArgument, Item, PathArguments, Type};

#[derive(Default)]
struct Crate {
    // module id -> names defined there (types and traits)
    types: BTreeMap<String, BTreeSet<String>>,
    traits: BTreeMap<String, BTreeSet<String>>,
    all_types: BTreeSet<String>,
    all_traits: BTreeSet<String>,
    impls: Vec<Impl>,
    parse_failures: usize,
}

struct Impl {
    module: String,
    trait_name: String,
    trait_args: Vec<String>,
    self_name: Option<String>,
    self_qualified: bool,
    blanket: bool,
    in_test: bool,
    cfg_feature: bool,
}

fn has_cfg_test(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path().is_ident("cfg") && {
            let s = quote_attr(a);
            s.contains("test") && !s.contains("not(test")
        }
    })
}

fn has_cfg_feature(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|a| (a.path().is_ident("cfg") || a.path().is_ident("cfg_attr")) && quote_attr(a).contains("feature"))
}

fn quote_attr(a: &Attribute) -> String {
    match &a.meta {
        syn::Meta::List(l) => l.tokens.to_string(),
        _ => String::new(),
    }
}

fn type_head(ty: &Type) -> (Option<String>, bool) {
    match ty {
        Type::Path(p) => {
            let segs = &p.path.segments;
            let qualified = segs.len() > 1 && segs.first().map(|s| s.ident != "Self" && s.ident != "self").unwrap_or(false);
            (segs.last().map(|s| s.ident.to_string()), qualified)
        }
        Type::Reference(r) => type_head(&r.elem),
        Type::Paren(p) => type_head(&p.elem),
        Type::Group(g) => type_head(&g.elem),
        _ => (None, false),
    }
}

fn walk(items: &[Item], module: &str, in_test: bool, cfg_feature: bool, c: &mut Crate) {
    for item in items {
        match item {
            Item::Struct(s) => { c.types.entry(module.into()).or_default().insert(s.ident.to_string()); }
            Item::Enum(s) => { c.types.entry(module.into()).or_default().insert(s.ident.to_string()); }
            Item::Union(s) => { c.types.entry(module.into()).or_default().insert(s.ident.to_string()); }
            Item::Type(s) => { c.types.entry(module.into()).or_default().insert(s.ident.to_string()); }
            Item::Trait(s) => { c.traits.entry(module.into()).or_default().insert(s.ident.to_string()); }
            Item::Mod(m) => {
                if let Some((_, items)) = &m.content {
                    let sub = format!("{module}::{}", m.ident);
                    walk(items, &sub, in_test || has_cfg_test(&m.attrs), cfg_feature || has_cfg_feature(&m.attrs), c);
                }
            }
            Item::Impl(i) => {
                if let Some((_, path, _)) = &i.trait_ {
                    let last = path.segments.last().unwrap();
                    let mut args = Vec::new();
                    if let PathArguments::AngleBracketed(ab) = &last.arguments {
                        for a in &ab.args {
                            if let GenericArgument::Type(t) = a {
                                if let (Some(n), _) = type_head(t) { args.push(n); }
                            }
                        }
                    }
                    let (self_name, self_qualified) = type_head(&i.self_ty);
                    let blanket = self_name.as_ref().map(|n| i.generics.type_params().any(|tp| tp.ident == n)).unwrap_or(false);
                    c.impls.push(Impl {
                        module: module.into(),
                        trait_name: last.ident.to_string(),
                        trait_args: args,
                        self_name,
                        self_qualified,
                        blanket,
                        in_test: in_test || has_cfg_test(&i.attrs),
                        cfg_feature: cfg_feature || has_cfg_feature(&i.attrs),
                    });
                }
            }
            _ => {}
        }
    }
}

fn parent_prefixes(def_module: &str) -> Option<Vec<String>> {
    // returns prefixes such that an impl module starting with one is a descendant of def_module
    let (file, inner) = match def_module.split_once("::") { Some((f, i)) => (f, Some(i)), None => (def_module, None) };
    if let Some(i) = inner { return Some(vec![format!("{file}::{i}::")]); }
    let name = file.rsplit('/').next().unwrap();
    if name == "lib.rs" || name == "main.rs" { if !file.contains('/') { return None; } }
    let dir = if name == "mod.rs" || name == "lib.rs" || name == "main.rs" { file[..file.len() - name.len()].to_string() } else { format!("{}/", &file[..file.len() - 3]) };
    Some(vec![dir, format!("{file}::")])
}

fn in_subtree(module: &str, defs: &BTreeMap<String, BTreeSet<String>>, name: &str) -> bool {
    defs.iter().filter(|(_, v)| v.contains(name)).any(|(m, _)| parent_prefixes(m).map(|ps| ps.iter().any(|p| module.starts_with(p.as_str()))).unwrap_or(false))
}

fn same_dir(module: &str, defs: &BTreeMap<String, BTreeSet<String>>, name: &str) -> bool {
    let dir = |m: &str| { let f = m.split("::").next().unwrap(); f.rsplit_once('/').map(|(d, _)| d.to_string()).unwrap_or_default() };
    defs.iter().filter(|(_, v)| v.contains(name)).any(|(m, _)| dir(m) == dir(module))
}

fn scan(root: &Path) -> Crate {
    let mut c = Crate::default();
    for e in walkdir::WalkDir::new(root).into_iter().filter_map(Result::ok) {
        let p = e.path();
        if p.extension().map(|x| x != "rs").unwrap_or(true) { continue; }
        let s = p.to_string_lossy();
        if s.contains("/tests/") || s.contains("/benches/") || s.contains("/examples/") || s.contains("/fixtures/") || s.contains("/test_data/") || s.contains("/test-data/") { continue; }
        let Ok(src) = std::fs::read_to_string(p) else { continue };
        let Ok(file) = syn::parse_file(&src) else { c.parse_failures += 1; continue };
        let module = p.strip_prefix(root).unwrap().to_string_lossy().to_string();
        let test_file = has_cfg_test(&file.attrs);
        let feat_file = has_cfg_feature(&file.attrs);
        walk(&file.items, &module, test_file, feat_file, &mut c);
    }
    for v in c.types.values() { c.all_types.extend(v.iter().cloned()); }
    for v in c.traits.values() { c.all_traits.extend(v.iter().cloned()); }
    c
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let name = &args[0];
    let verbose = std::env::var("VERBOSE").is_ok();
    let mut totals: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut elsewhere: Vec<String> = Vec::new();
    let mut failures = 0;
    for root in &args[1..] {
        let c = scan(&PathBuf::from(root));
        failures += c.parse_failures;
        let empty = BTreeSet::new();
        for i in &c.impls {
            if i.in_test { *totals.entry("z_test_module").or_default() += 1; continue; }
            let types = c.types.get(&i.module).unwrap_or(&empty);
            let traits = c.traits.get(&i.module).unwrap_or(&empty);
            let self_here = i.self_name.as_ref().map(|n| !i.self_qualified && types.contains(n)).unwrap_or(false);
            let trait_here = traits.contains(&i.trait_name);
            let arg_here = i.trait_args.iter().any(|a| types.contains(a));
            let self_in_crate = i.self_name.as_ref().map(|n| c.all_types.contains(n)).unwrap_or(false);
            let trait_in_crate = c.all_traits.contains(&i.trait_name);
            let cat: &'static str = if self_here && trait_here { "a_type_and_trait" }
                else if self_here { "b_type" }
                else if trait_here { if self_in_crate { "c_trait_local_type_elsewhere" } else { "d_trait_foreign_type" } }
                else if arg_here { "e_trait_arg_type" }
                else if self_in_crate { if trait_in_crate { "f_elsewhere_local_trait_local_type" } else { "g_elsewhere_foreign_trait_local_type" } }
                else if i.blanket { "i_blanket_elsewhere" }
                else if trait_in_crate { "h1_elsewhere_local_trait_foreign_type" }
                else { "h2_elsewhere_unresolved" };
            *totals.entry(cat).or_default() += 1;
            if cat.starts_with('f') || cat.starts_with('g') {
                let n = i.self_name.as_deref().unwrap();
                let rel = if in_subtree(&i.module, &c.types, n) { "s_fg_child_of_type_module" } else if in_subtree(&i.module, &c.traits, &i.trait_name) { "s_fg_child_of_trait_module" } else if i.trait_args.iter().any(|a| in_subtree(&i.module, &c.types, a)) { "s_fg_child_of_arg_type_module" } else if same_dir(&i.module, &c.types, n) { "s_fg_sibling_of_type_module" } else { "s_fg_unrelated_module" };
                *totals.entry(rel).or_default() += 1;
            }
            if cat.starts_with("h1") || cat.starts_with('i') {
                let rel = if in_subtree(&i.module, &c.traits, &i.trait_name) { "t_h1_child_of_trait_module" } else if same_dir(&i.module, &c.traits, &i.trait_name) { "t_h1_sibling_of_trait_module" } else { "t_h1_unrelated_module" };
                *totals.entry(rel).or_default() += 1;
            }
            if i.cfg_feature { *totals.entry(match cat { c if c.starts_with('f') || c.starts_with('g') || c.starts_with('h') || c.starts_with('i') => "y_feature_gated_elsewhere", _ => "x_feature_gated_beside" }).or_default() += 1; }
            if verbose && (cat.starts_with('f') || cat.starts_with('g') || cat.starts_with('h') || cat.starts_with('c') || cat.starts_with('e')) {
                elsewhere.push(format!("{cat}\t{}/{}\timpl {}<{}> for {}{}", root, i.module, i.trait_name, i.trait_args.join(","), i.self_name.clone().unwrap_or("?".into()), if i.cfg_feature {"\t[cfg feature]"} else {""}));
            }
        }
    }
    let total: usize = totals.iter().filter(|(k, _)| !k.starts_with('x') && !k.starts_with('y') && !k.starts_with('z') && !k.starts_with('s') && !k.starts_with('t')).map(|(_, v)| v).sum();
    println!("== {name}: {total} trait impls outside test modules (parse failures: {failures})");
    for (k, v) in &totals { println!("  {k:40} {v:6} {:5.1}%", 100.0 * *v as f64 / total.max(1) as f64); }
    for e in elsewhere { println!("{e}"); }
}
```
