# convergence-trait

Universal convergence controller trait for the pleme-io platform.

Every computing problem follows the same loop:

```
declare -> simulate -> prove -> remediate -> render -> deploy -> verify -> reconverge
```

This crate defines the Rust trait that every controller implements.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
convergence-trait = { git = "https://github.com/pleme-io/convergence-trait" }
```

## Implementing the Trait

```rust
use convergence_trait::*;
use convergence_trait::controller::ConvergenceController;
use convergence_trait::proof::{ConvergenceProof, ConvergenceCertificate};

struct MyController;

impl ConvergenceController for MyController {
    // What your simulation produces (e.g., Terraform JSON)
    type Simulation = serde_json::Value;
    // What your renderer outputs (e.g., YAML manifests)
    type Rendering = String;
    // How you reference a deployment (e.g., Helm release name)
    type DeploymentHandle = String;

    fn declare(&self, intent: &str, constraints: Vec<Constraint>) -> Declaration {
        Declaration {
            name: format!("my-{intent}"),
            intent: intent.to_string(),
            constraints,
        }
    }

    fn simulate(&self, declaration: &Declaration) -> Result<Self::Simulation, Vec<Violation>> {
        // Generate your intermediate representation
        Ok(serde_json::json!({ "resources": [] }))
    }

    fn prove(
        &self,
        simulation: &Self::Simulation,
        constraints: &[Constraint],
    ) -> Result<ConvergenceProof, Vec<Violation>> {
        // Verify all constraints are satisfied
        Ok(ConvergenceProof {
            declaration_name: "my-controller".into(),
            constraints_checked: constraints.len(),
            constraints_satisfied: constraints.len(),
            invariants_proven: vec![],
            baselines_verified: vec![],
            proof_hash: "abc".into(),
            timestamp: "2026-04-13T00:00:00Z".into(),
        })
    }

    fn remediate(
        &self,
        simulation: &Self::Simulation,
        violations: &[Violation],
    ) -> Result<Self::Simulation, Vec<Violation>> {
        // Fix what you can, return errors for what you can't
        Err(violations.to_vec())
    }

    fn render(&self, simulation: &Self::Simulation, proof: &ConvergenceProof) -> Self::Rendering {
        // Convert simulation to platform-specific output
        serde_json::to_string_pretty(simulation).unwrap()
    }

    fn deploy(&self, rendering: &Self::Rendering) -> Result<Self::DeploymentHandle, Vec<Violation>> {
        // Deploy the rendered output
        Ok("release-1".into())
    }

    fn verify(&self, handle: &Self::DeploymentHandle) -> Result<(), Vec<Drift>> {
        // Check deployed state matches proven state
        Ok(())
    }

    fn reconverge(&self, drift: &[Drift]) -> Declaration {
        // Produce a new declaration to fix drift
        Declaration {
            name: "reconverge".into(),
            intent: format!("fix {} drifts", drift.len()),
            constraints: vec![],
        }
    }
}

fn main() {
    let ctrl = MyController;
    let constraints = vec![
        Constraint::Baseline("soc2".into()),
        Constraint::Invariant("encryption-at-rest".into()),
    ];

    match ctrl.converge("my-service", constraints) {
        Ok(cert) => println!("Converged: {}", cert.certificate_hash),
        Err(result) => println!("Failed at {}: {}", result.phase, result.message),
    }
}
```

## The Convergence Loop

| Phase | Method | What Happens |
|-------|--------|-------------|
| 1. Declare | `declare()` | Accept user intent and constraints |
| 2. Simulate | `simulate()` | Generate intermediate representation |
| 3. Prove | `prove()` | Verify all constraints are satisfied |
| 3b. Remediate | `remediate()` | Fix violations (automatic on proof failure) |
| 4. Render | `render()` | Convert to platform-specific output |
| 5. Deploy | `deploy()` | Deploy the rendered output |
| 6. Verify | `verify()` | Check deployed state matches proven state |
| 7. Reconverge | `reconverge()` | Given drift, produce a new declaration |

The `converge()` default method runs phases 1-4 with automatic remediation. Deploy, verify, and reconverge are called separately by the orchestrator since they involve side effects.

## License

MIT
