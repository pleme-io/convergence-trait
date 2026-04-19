# RFC 0001 — Unified ConvergentComputation trait

**Status:** Draft
**Author:** cross-repo Rust survey, 2026-04-19
**Discussion scope:** convergence-trait, arch-synthesizer, tameshi, sekiban, kensa, inshou, 16 synthesizers

## Summary

Three trait hierarchies in the pleme-io Rust tree encode the same mathematical
pattern under different names:

| Family | Entry trait | Renders |
|---|---|---|
| Convergence | `ConvergenceController` (convergence-trait) | Any platform |
| Synthesizer | `RenderBackend` (arch-synthesizer) + `SynthesizerNode` (16 crates) | Language artifacts |
| Attestation | `LayerCollector` + `ComplianceRunner` + `MerkleRootSigner` (tameshi) | BLAKE3 Merkle tree |

All three are **"transform typed input through IR → deterministic output
artifact"** with BLAKE3 content addressing. Different trait names + associated
types hide the shared shape. This RFC proposes one trait that all three
families refine.

## Motivation

Maintenance cost of three parallel hierarchies:

- Cross-family code can't share proofs — a new compliance check must be
  ported to `ConvergenceController`, `RenderBackend`, AND `LayerCollector`.
- The four *Proof* types (`ConvergenceProof`, `ComplianceProof`,
  `Attested<T>`, `CertificationArtifact`) encode the same concept. Code
  reviewers must learn four names.
- Seven *Signature* / *Attestation* types (`LayerSignature`, `MasterSignature`,
  `ComplianceResult`, `SignatureGate`, `GateDecision`, `RenderAttestation`,
  `WorkspaceAttestation`) compose BLAKE3 hashes of ordered layers but with
  divergent fields and serialization.

## Proposal

Introduce one trait parametric on the constraint domain:

```rust
pub trait ConvergentComputation<C: Constraint> {
    type Input;
    type Simulation;
    type Proof: ProofSystem<Constraint = C>;
    type Output;

    /// Phase 1: declare desired state (Rust types, CRD, OpenAPI, Nix module)
    fn declare() -> Self::Input;

    /// Phase 2: simulate at zero cost (pangea-sim, ruby-synthesizer)
    fn simulate(input: &Self::Input) -> Result<Self::Simulation, Vec<Violation<C>>>;

    /// Phase 3: prove invariants (proptest, RSpec, kensa)
    fn prove(sim: &Self::Simulation) -> Result<Self::Proof, Vec<Violation<C>>>;

    /// Phase 4: render (Backend impls, Helm, Nix)
    fn render(proof: &Self::Proof) -> Result<Self::Output, RenderError>;

    /// Phase 5: verify (InSpec, tameshi, health probes)
    fn verify(output: &Self::Output) -> Result<(), Vec<Drift>>;
}
```

Each existing family implements this:

- `ConvergenceController` impls pass through with `Simulation = ()` for
  cluster lifecycle
- `RenderBackend` impls set `Input = IacResource`, `Output = Vec<GeneratedArtifact>`,
  `Proof = ComplianceProof`
- `LayerCollector` composes many collectors into one `ConvergentComputation`
  whose `Output` is a `LayeredAttestation`

## Non-goals

- Forcing every existing trait to migrate this week. Blanket impls let us
  layer this on top of today's code; existing APIs stay.
- Redefining `Constraint` independently. See RFC 0003 for that.

## Questions to resolve before acceptance

1. Where does this trait live? Options: `convergence-trait` (natural fit,
   current home of `ConvergenceController`), a new `pleme-proof` crate
   (cleaner but adds another dep), `arch-synthesizer` (already has the
   `ProvenMorphism` pattern).
2. How do we stage the migration? Proposed path: (a) land trait + blanket
   impls on existing traits, (b) convert convergence-controller's 12 MCP
   tools to speak the new vocabulary, (c) convert one Backend impl as
   proof-of-concept, (d) incremental adoption thereafter.
3. Error type: unified `ConvergenceError` vs per-family errors that all
   implement a shared `ConvergenceError` trait?

## Status

Design stub. This file is the anchor for the next design session — it
names the problem, sketches the shape, and lists the decisions blocking
implementation. No code changes land until the questions are resolved.
