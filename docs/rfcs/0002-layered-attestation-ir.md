# RFC 0002 — LayeredAttestation as unified IR

**Status:** Draft
**Author:** cross-repo Rust survey, 2026-04-19
**Discussion scope:** tameshi, sekiban, kensa, arch-synthesizer, convergence-trait

## Summary

Three types compose layered BLAKE3 hashes under different names:

| Name | Home | Structure |
|---|---|---|
| `MasterSignature` | tameshi | `{ untested_hash, compliance_hash, secure_hash }` |
| `RenderAttestation` | arch-synthesizer | Merkle tree of workspace hashes + compliance hash |
| `ComplianceResult` | kensa | Assessment result (pre-hash, but destined for same chain) |

Plus seven peripheral types (`LayerSignature`, `WorkspaceAttestation`,
`CertificationArtifact`, `HeartbeatChain`, `SignatureGate`,
`BlastRadiusReport`, `Outcome::before_hash/after_hash`) that all compose
ordered BLAKE3 hashes with ad-hoc field names.

This RFC unifies them under a single parametric type.

## Motivation

- When `tameshi::MasterSignature` adds a fourth layer (e.g. "policy_hash"),
  `arch-synthesizer::RenderAttestation` has to add the same field with a
  compatible serde shape — or consumers that bridge the two break.
- `kensa::ComplianceResult` ends up hashed into `MasterSignature.compliance_hash`,
  but the two types don't share a trait — the hash composition is open-coded
  in one place and must be kept in sync by reviewers.
- Cross-language vectors (`iac-forge/tests/cross_lang_vectors.rs`) already
  prove that canonical sexpr + BLAKE3 is language-portable. The same
  portability should apply to layered attestation, but can't because each
  crate picks a different field order.

## Proposal

```rust
/// Ordered composition of typed layer hashes into a single BLAKE3 root.
pub struct LayeredAttestation<Layers> {
    pub layers: Layers,
    pub root: Blake3Hash,
}

/// Implemented by any ordered tuple / struct of layers whose ordering is
/// canonical. `compute_root` hashes `layer.0.hash() ++ layer.1.hash() ++ …`
/// with length framing.
pub trait AttestationLayers {
    fn layer_hashes(&self) -> Vec<Blake3Hash>;
    fn compute_root(&self) -> Blake3Hash {
        blake3_merkle(&self.layer_hashes())
    }
}
```

Existing types become parameterizations:

- `MasterSignature` = `LayeredAttestation<(Untested, Compliance, Secure)>`
- `RenderAttestation` = `LayeredAttestation<(WorkspaceHashes, Compliance)>`
- `CertificationArtifact` = `LayeredAttestation<(Artifact, Control, Intent)>`

The `AttestationChain` concept from `HeartbeatChain` + `SignatureGate`
becomes `Vec<LayeredAttestation<_>>` with per-entry timestamps.

## Migration path

1. Land `LayeredAttestation<L>` + `AttestationLayers` trait in `tameshi`
   (current home of `MasterSignature`, matching memory of that decision).
2. Add blanket impls / re-exports so old names still work.
3. Add cross-language test vectors for `LayeredAttestation<(A, B, C)>`
   spanning several layer types.
4. Deprecate the ad-hoc types with a 6-month window.

## Non-goals

- Redefining the `LayerCollector` trait — that's a separate concern
  (producing a `Layer`), handled by RFC 0001.
- Cross-hash algorithms — BLAKE3 stays the only option.

## Questions to resolve

1. Is `Layers` a tuple (flexible arity, poor discoverability) or a named
   struct type (discoverable, requires new type per composition)?
2. How is Merkle ordering defined — position in tuple, or declared via
   associated const?
3. Should `AttestationChain` live here or in `sekiban`?
