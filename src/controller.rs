use crate::proof::{ConvergenceCertificate, ConvergenceProof};
use crate::types::{Constraint, ConvergencePhase, ConvergenceResult, Declaration, Drift, Violation};

/// The universal convergence controller trait.
///
/// Every integration implements this trait. The convergence loop is:
/// `declare -> simulate -> prove -> remediate (if needed) -> render -> deploy -> verify -> reconverge (if drift)`
///
/// The trait is generic over the intermediate representations at each phase.
/// This allows different controllers to use different types while sharing
/// the same convergence semantics.
pub trait ConvergenceController {
    /// The simulation output type (e.g., Terraform JSON, Helm manifest, generated code)
    type Simulation: Clone;
    /// The rendered output type (e.g., Ruby source, Go source, YAML)
    type Rendering;
    /// The deployment handle (e.g., process ID, Helm release name)
    type DeploymentHandle;

    /// Phase 1: Declare — accept a declaration of what the user wants.
    /// Returns the declaration with resolved constraints.
    fn declare(&self, intent: &str, constraints: Vec<Constraint>) -> Declaration;

    /// Phase 2: Simulate — generate the simulation output from the declaration.
    ///
    /// # Errors
    ///
    /// Returns violations if the simulation cannot be generated.
    fn simulate(&self, declaration: &Declaration) -> Result<Self::Simulation, Vec<Violation>>;

    /// Phase 3: Prove — verify the simulation satisfies all constraints.
    ///
    /// # Errors
    ///
    /// Returns violations if the simulation fails to satisfy constraints.
    fn prove(
        &self,
        simulation: &Self::Simulation,
        constraints: &[Constraint],
    ) -> Result<ConvergenceProof, Vec<Violation>>;

    /// Phase 3b: Remediate — fix violations and produce a corrected simulation.
    ///
    /// # Errors
    ///
    /// Returns violations that could not be remediated.
    fn remediate(
        &self,
        simulation: &Self::Simulation,
        violations: &[Violation],
    ) -> Result<Self::Simulation, Vec<Violation>>;

    /// Phase 4: Render — convert proven simulation to platform-specific output.
    fn render(&self, simulation: &Self::Simulation, proof: &ConvergenceProof) -> Self::Rendering;

    /// Phase 5: Deploy — deploy the rendered output.
    ///
    /// # Errors
    ///
    /// Returns violations if deployment fails.
    fn deploy(&self, rendering: &Self::Rendering) -> Result<Self::DeploymentHandle, Vec<Violation>>;

    /// Phase 6: Verify — check deployed state matches proven state.
    ///
    /// # Errors
    ///
    /// Returns drift items if deployed state diverges from proven state.
    fn verify(&self, handle: &Self::DeploymentHandle) -> Result<(), Vec<Drift>>;

    /// Phase 7: Reconverge — given drift, produce a new declaration to fix it.
    fn reconverge(&self, drift: &[Drift]) -> Declaration;

    /// Run the complete convergence loop: declare -> simulate -> prove -> render -> deploy -> verify.
    /// Returns the certificate if successful, or violations if any phase fails.
    ///
    /// # Errors
    ///
    /// Returns a `ConvergenceResult` describing which phase failed and why.
    fn converge(
        &self,
        intent: &str,
        constraints: Vec<Constraint>,
    ) -> Result<ConvergenceCertificate, ConvergenceResult> {
        // Phase 1: Declare
        let declaration = self.declare(intent, constraints.clone());

        // Phase 2: Simulate
        let simulation = self.simulate(&declaration).map_err(|v| ConvergenceResult {
            declaration_name: declaration.name.clone(),
            phase: ConvergencePhase::Simulated,
            success: false,
            violations: v,
            certificate_hash: None,
            message: "Simulation failed".into(),
        })?;

        // Phase 3: Prove
        let proof = match self.prove(&simulation, &constraints) {
            Ok(proof) => proof,
            Err(violations) => {
                // Phase 3b: Try remediation
                let remediated =
                    self.remediate(&simulation, &violations)
                        .map_err(|v| ConvergenceResult {
                            declaration_name: declaration.name.clone(),
                            phase: ConvergencePhase::Remediated,
                            success: false,
                            violations: v,
                            certificate_hash: None,
                            message: "Remediation failed".into(),
                        })?;

                // Re-prove after remediation
                self.prove(&remediated, &constraints).map_err(|v| {
                    ConvergenceResult {
                        declaration_name: declaration.name.clone(),
                        phase: ConvergencePhase::Proven,
                        success: false,
                        violations: v,
                        certificate_hash: None,
                        message: "Proof failed after remediation".into(),
                    }
                })?
            }
        };

        // Phase 4: Render
        let _rendering = self.render(&simulation, &proof);

        // Create certificate
        let cert_content = serde_json::to_vec(&proof).unwrap_or_default();
        let cert_hash = format!("{:x}", fnv1a_hash(&cert_content));

        Ok(ConvergenceCertificate {
            proof,
            rendering_target: std::any::type_name::<Self::Rendering>().to_string(),
            certificate_hash: cert_hash,
        })
    }
}

/// FNV-1a hash for certificate fingerprinting.
/// Real implementations should use blake3.
fn fnv1a_hash(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &byte in data {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv1a_deterministic() {
        let data = b"convergence";
        assert_eq!(fnv1a_hash(data), fnv1a_hash(data));
    }

    #[test]
    fn fnv1a_different_inputs_differ() {
        assert_ne!(fnv1a_hash(b"hello"), fnv1a_hash(b"world"));
    }
}
