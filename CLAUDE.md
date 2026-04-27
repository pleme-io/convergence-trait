# convergence-trait

> **★★★ CSE / Knowable Construction.** This repo operates under **Constructive Substrate Engineering** — canonical specification at [`pleme-io/theory/CONSTRUCTIVE-SUBSTRATE-ENGINEERING.md`](https://github.com/pleme-io/theory/blob/main/CONSTRUCTIVE-SUBSTRATE-ENGINEERING.md). The Compounding Directive (operational rules: solve once, load-bearing fixes only, idiom-first, models stay current, direction beats velocity) is in the org-level pleme-io/CLAUDE.md ★★★ section. Read both before non-trivial changes.


Universal convergence controller trait for the pleme-io platform.

## What This Is

This crate defines the `ConvergenceController` trait -- the pattern every integration must follow. The convergence loop is the operating model for all pleme-io systems:

```
declare -> simulate -> prove -> remediate -> render -> deploy -> verify -> reconverge
```

If a system doesn't implement `ConvergenceController`, it is not part of the convergence platform.

## Architecture

```
src/
  lib.rs          -- Public API re-exports
  types.rs        -- Declaration, Constraint, Violation, Drift, ConvergenceResult, ConvergencePhase
  proof.rs        -- ConvergenceProof, ConvergenceCertificate
  controller.rs   -- The ConvergenceController trait with default converge() method
tests/
  controller_tests.rs -- 12 tests covering the full convergence loop
```

## The Trait

`ConvergenceController` is generic over three associated types:

| Type | Purpose | Examples |
|------|---------|---------|
| `Simulation` | Intermediate representation after simulation | Terraform JSON, Helm values, generated code |
| `Rendering` | Platform-specific output after rendering | Ruby source, YAML manifests, Go source |
| `DeploymentHandle` | Reference to deployed artifact | PID, Helm release name, K8s resource UID |

## Phases

| Phase | Method | Returns |
|-------|--------|---------|
| 1. Declare | `declare()` | `Declaration` |
| 2. Simulate | `simulate()` | `Result<Simulation, Vec<Violation>>` |
| 3. Prove | `prove()` | `Result<ConvergenceProof, Vec<Violation>>` |
| 3b. Remediate | `remediate()` | `Result<Simulation, Vec<Violation>>` |
| 4. Render | `render()` | `Rendering` |
| 5. Deploy | `deploy()` | `Result<DeploymentHandle, Vec<Violation>>` |
| 6. Verify | `verify()` | `Result<(), Vec<Drift>>` |
| 7. Reconverge | `reconverge()` | `Declaration` |

The `converge()` default method runs phases 1-4 automatically, including remediation on proof failure.

## Key Types

- **Declaration** -- what the user wants (name, intent, constraints)
- **Constraint** -- Baseline, Invariant, Platform, ComposesWith, Custom
- **Violation** -- a constraint that was not satisfied (with remediable flag)
- **Drift** -- divergence between proven and deployed state
- **ConvergenceProof** -- proof that constraints are satisfied
- **ConvergenceCertificate** -- proof + rendering target + deterministic hash

## Commands

```bash
cargo test          # Run all tests
cargo clippy        # Lint (pedantic warnings enabled)
cargo doc --open    # Generate and view docs
```

## Rules

- Every new type must derive `Serialize` and `Deserialize`
- The trait must remain generic -- no concrete types in the trait definition
- The `converge()` default method must not be overridden without good reason
- All public items must have doc comments
- Clippy pedantic is enforced
