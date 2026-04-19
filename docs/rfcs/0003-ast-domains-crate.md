# RFC 0003 — Publish ast-domains as a shared crate

**Status:** Draft
**Author:** cross-repo Rust survey, 2026-04-19
**Discussion scope:** arch-synthesizer, 16 *-synthesizer crates, iac-forge

## Summary

Six synthesizer crates hand-roll their own AST node hierarchy:

| Crate | AST root | ~Variants | Module placement |
|---|---|---|---|
| rust-synthesizer | `RustNode` | 26 | Standalone, public |
| helm-synthesizer + yaml-synthesizer | `HelmExpr` + `YamlNode` | 20+ | Embedded, no public docs |
| hcl-synthesizer | `HclNode` | ~15 | Embedded |
| nix-synthesizer | `NixNode` + `NixValue` | ~30 | Embedded |
| go-synthesizer | `GoNode` | ~25 | Embedded |
| dockerfile-synthesizer | `DockerNode` | ~15 | Embedded |

`arch-synthesizer/src/ast_domains.rs` and `catalog.rs` claim a 19-domain
catalog with morphisms between them, but that module is internal — there's
no public crate that the individual synthesizers import from. Each
rebuilds its primitives in isolation.

## Motivation

- Adding a new morphism (e.g. `Dockerfile → NixOSModule` for OCI→nixosSystem
  sharing) today requires importing six unrelated crates and translating
  between six AST shapes.
- The "universal irreducible primitives" claim in arch-synthesizer
  (Literal, Sequence, Mapping, Declaration, Annotation, Escape) is a type
  system waiting to be written — every synthesizer reimplements these.
- Cross-language sexpr + BLAKE3 portability (already proven in iac-forge
  via `tests/cross_lang_vectors.rs`) should apply per-AST-domain too, but
  can't because each synthesizer emits through a different path.

## Proposal

Publish a new crate `ast-domains` (or fold into an existing shared crate —
see questions below) exposing:

```rust
/// The 6 universal irreducible primitives every AST domain reduces to.
pub enum Primitive {
    Literal(LiteralValue),
    Sequence(Vec<Primitive>),
    Mapping(Vec<(Primitive, Primitive)>),
    Declaration { kind: &'static str, body: Vec<Primitive> },
    Annotation { target: Box<Primitive>, meta: Mapping },
    Escape(Box<dyn Fn(&mut dyn io::Write) -> io::Result<()>>),
}

/// A domain is a typed AST whose emission reduces to `Primitive`.
pub trait AstDomain {
    type Node: SynthesizerNode;
    fn name() -> &'static str;
    fn to_primitive(node: &Self::Node) -> Primitive;
}

/// A structure-preserving map between two domains.
pub trait Morphism<Src: AstDomain, Dst: AstDomain> {
    fn apply(src: &Src::Node) -> Dst::Node;
    /// Deterministic + total: required by construction.
}
```

Every existing synthesizer:
1. Moves its AST root behind a public module (same file contents, new path).
2. `impl AstDomain for $Domain { ... }` mapping to `Primitive`.
3. Inherits `ContentHash` + cross-language vector testing for free via
   blanket impls keyed on `AstDomain`.

## Non-goals

- Merging the 16 AST hierarchies into one type. Domains are distinct; the
  unification is at the Primitive layer (emission) and Morphism layer
  (composition).
- Changing any synthesizer's public API beyond adding the `AstDomain`
  impl. All emit functions stay.

## Migration path

1. Bootstrap `ast-domains` crate in pleme-io with the `Primitive` enum
   + `AstDomain` trait + `Morphism` trait + zero other content.
2. Add `impl AstDomain` in one small synthesizer (dockerfile-synthesizer
   is a good pilot — ~15 variants, self-contained).
3. Add one morphism (`Dockerfile → Nix` or similar) implementing `Morphism`.
4. Incrementally add `AstDomain` to the remaining 5-6 hierarchies.
5. Once ≥3 synthesizers adopt, extract `arch-synthesizer/src/catalog.rs`
   to `ast-domains` as well so the catalog is public + consumable.

## Questions to resolve

1. Should this live in a new `pleme-io/ast-domains` crate, or fold into
   an existing shared crate (e.g. `synthesizer-core` which already has
   `SynthesizerNode` + `Artifact`)? Folding minimizes crate sprawl; new
   crate keeps the domain-theory concerns isolated.
2. Is `Primitive::Escape` necessary, or can we cover all emission with
   the other five variants? An escape hatch is honest but invites abuse.
3. Who owns the morphism catalog — `ast-domains`, or each synthesizer
   declares its outgoing morphisms and `ast-domains` aggregates?
