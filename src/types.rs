use serde::{Deserialize, Serialize};

/// A convergence declaration — what the user wants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Declaration {
    pub name: String,
    pub intent: String,
    pub constraints: Vec<Constraint>,
}

/// A constraint on the declared system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Constraint {
    /// Must satisfy a compliance baseline
    Baseline(String),
    /// Must satisfy a specific invariant
    Invariant(String),
    /// Must render to a specific platform
    Platform(String),
    /// Must compose with another declaration
    ComposesWith(String),
    /// Custom constraint
    Custom { name: String, value: serde_json::Value },
}

/// A violation found during proving.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub source: String,
    pub constraint: String,
    pub message: String,
    pub remediable: bool,
}

/// Drift detected between proven state and deployed state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Drift {
    pub resource: String,
    pub expected: serde_json::Value,
    pub actual: serde_json::Value,
    pub severity: DriftSeverity,
}

/// How severe is the drift.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DriftSeverity {
    /// Cosmetic — doesn't affect invariants
    Low,
    /// Functional — may affect behavior but not compliance
    Medium,
    /// Compliance — violates an invariant or control
    High,
    /// Critical — security invariant violated
    Critical,
}

/// Result of a convergence cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceResult {
    pub declaration_name: String,
    pub phase: ConvergencePhase,
    pub success: bool,
    pub violations: Vec<Violation>,
    pub certificate_hash: Option<String>,
    pub message: String,
}

/// Which phase of the convergence loop we're in.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConvergencePhase {
    Declared,
    Simulated,
    Proven,
    Remediated,
    Rendered,
    Deployed,
    Verified,
    Reconverging,
}

impl std::fmt::Display for ConvergencePhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
