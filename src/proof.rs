use serde::{Deserialize, Serialize};

/// A proof that a declaration satisfies its constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceProof {
    pub declaration_name: String,
    pub constraints_checked: usize,
    pub constraints_satisfied: usize,
    pub invariants_proven: Vec<String>,
    pub baselines_verified: Vec<String>,
    pub proof_hash: String,
    pub timestamp: String,
}

impl ConvergenceProof {
    /// Returns true if every checked constraint was satisfied.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.constraints_checked == self.constraints_satisfied
    }
}

/// A certificate proving a complete convergence cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceCertificate {
    pub proof: ConvergenceProof,
    pub rendering_target: String,
    pub certificate_hash: String,
}
