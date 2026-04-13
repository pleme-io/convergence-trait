use convergence_trait::*;
use convergence_trait::controller::ConvergenceController;
use convergence_trait::proof::ConvergenceProof;

/// A mock controller that implements the full convergence loop.
/// Behavior is controlled by the `fail_at` and `remediable` fields.
struct MockController {
    /// If set, the controller will fail at this phase.
    fail_at: Option<ConvergencePhase>,
    /// Whether violations are remediable.
    remediable: bool,
}

impl MockController {
    fn new() -> Self {
        Self {
            fail_at: None,
            remediable: true,
        }
    }

    fn failing_at(phase: ConvergencePhase) -> Self {
        Self {
            fail_at: Some(phase),
            remediable: true,
        }
    }

    fn unfixable_at(phase: ConvergencePhase) -> Self {
        Self {
            fail_at: Some(phase),
            remediable: false,
        }
    }
}

impl ConvergenceController for MockController {
    type Simulation = String;
    type Rendering = String;
    type DeploymentHandle = String;

    fn declare(&self, intent: &str, constraints: Vec<Constraint>) -> Declaration {
        Declaration {
            name: format!("mock-{intent}"),
            intent: intent.to_string(),
            constraints,
        }
    }

    fn simulate(&self, declaration: &Declaration) -> Result<String, Vec<Violation>> {
        if self.fail_at == Some(ConvergencePhase::Simulated) {
            return Err(vec![Violation {
                source: "mock".into(),
                constraint: "simulation".into(),
                message: "Simulation failed".into(),
                remediable: self.remediable,
            }]);
        }
        Ok(format!("simulated:{}", declaration.name))
    }

    fn prove(
        &self,
        _simulation: &String,
        constraints: &[Constraint],
    ) -> Result<ConvergenceProof, Vec<Violation>> {
        if self.fail_at == Some(ConvergencePhase::Proven) {
            return Err(vec![Violation {
                source: "mock".into(),
                constraint: "proof".into(),
                message: "Proof failed".into(),
                remediable: self.remediable,
            }]);
        }
        Ok(ConvergenceProof {
            declaration_name: "mock".into(),
            constraints_checked: constraints.len(),
            constraints_satisfied: constraints.len(),
            invariants_proven: vec!["test-invariant".into()],
            baselines_verified: vec!["test-baseline".into()],
            proof_hash: "abc123".into(),
            timestamp: "2026-04-13T00:00:00Z".into(),
        })
    }

    fn remediate(
        &self,
        simulation: &String,
        violations: &[Violation],
    ) -> Result<String, Vec<Violation>> {
        if violations.iter().any(|v| !v.remediable) {
            return Err(violations.to_vec());
        }
        Ok(format!("remediated:{simulation}"))
    }

    fn render(&self, simulation: &String, _proof: &ConvergenceProof) -> String {
        format!("rendered:{simulation}")
    }

    fn deploy(&self, rendering: &String) -> Result<String, Vec<Violation>> {
        if self.fail_at == Some(ConvergencePhase::Deployed) {
            return Err(vec![Violation {
                source: "mock".into(),
                constraint: "deploy".into(),
                message: "Deploy failed".into(),
                remediable: false,
            }]);
        }
        Ok(format!("handle:{rendering}"))
    }

    fn verify(&self, _handle: &String) -> Result<(), Vec<Drift>> {
        if self.fail_at == Some(ConvergencePhase::Verified) {
            return Err(vec![Drift {
                resource: "test-resource".into(),
                expected: serde_json::json!("expected"),
                actual: serde_json::json!("actual"),
                severity: DriftSeverity::High,
            }]);
        }
        Ok(())
    }

    fn reconverge(&self, drift: &[Drift]) -> Declaration {
        Declaration {
            name: "reconverge".into(),
            intent: format!("fix {} drifts", drift.len()),
            constraints: vec![],
        }
    }
}

// ─── Test 1: Full convergence loop succeeds ──────────────────────────────

#[test]
fn full_convergence_loop_succeeds() {
    let ctrl = MockController::new();
    let constraints = vec![
        Constraint::Baseline("soc2".into()),
        Constraint::Invariant("encryption-at-rest".into()),
    ];
    let result = ctrl.converge("test-service", constraints);
    assert!(result.is_ok());
    let cert = result.unwrap();
    assert_eq!(cert.proof.declaration_name, "mock");
    assert!(cert.proof.is_complete());
    assert!(!cert.certificate_hash.is_empty());
}

// ─── Test 2: Simulation failure stops at phase 2 ────────────────────────

#[test]
fn simulation_failure_stops_at_phase_2() {
    let ctrl = MockController::failing_at(ConvergencePhase::Simulated);
    let result = ctrl.converge("test-service", vec![]);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.phase, ConvergencePhase::Simulated);
    assert!(!err.success);
    assert_eq!(err.violations.len(), 1);
    assert_eq!(err.message, "Simulation failed");
}

// ─── Test 3: Proof failure triggers remediation ─────────────────────────

#[test]
fn proof_failure_triggers_remediation() {
    // Build a controller that fails prove the first time but succeeds after remediation.
    // We'll use a custom controller for this.
    struct RemediatingController {
        prove_calls: std::cell::Cell<usize>,
    }

    impl ConvergenceController for RemediatingController {
        type Simulation = String;
        type Rendering = String;
        type DeploymentHandle = String;

        fn declare(&self, intent: &str, constraints: Vec<Constraint>) -> Declaration {
            Declaration {
                name: intent.into(),
                intent: intent.into(),
                constraints,
            }
        }

        fn simulate(&self, declaration: &Declaration) -> Result<String, Vec<Violation>> {
            Ok(format!("sim:{}", declaration.name))
        }

        fn prove(
            &self,
            _simulation: &String,
            constraints: &[Constraint],
        ) -> Result<ConvergenceProof, Vec<Violation>> {
            let call = self.prove_calls.get();
            self.prove_calls.set(call + 1);
            if call == 0 {
                // First call: fail
                return Err(vec![Violation {
                    source: "test".into(),
                    constraint: "needs-fix".into(),
                    message: "fixable".into(),
                    remediable: true,
                }]);
            }
            // Second call (after remediation): succeed
            Ok(ConvergenceProof {
                declaration_name: "test".into(),
                constraints_checked: constraints.len(),
                constraints_satisfied: constraints.len(),
                invariants_proven: vec![],
                baselines_verified: vec![],
                proof_hash: "fixed".into(),
                timestamp: "2026-04-13T00:00:00Z".into(),
            })
        }

        fn remediate(
            &self,
            simulation: &String,
            _violations: &[Violation],
        ) -> Result<String, Vec<Violation>> {
            Ok(format!("remediated:{simulation}"))
        }

        fn render(&self, simulation: &String, _proof: &ConvergenceProof) -> String {
            format!("rendered:{simulation}")
        }

        fn deploy(&self, rendering: &String) -> Result<String, Vec<Violation>> {
            Ok(format!("handle:{rendering}"))
        }

        fn verify(&self, _handle: &String) -> Result<(), Vec<Drift>> {
            Ok(())
        }

        fn reconverge(&self, _drift: &[Drift]) -> Declaration {
            Declaration {
                name: "reconverge".into(),
                intent: "fix".into(),
                constraints: vec![],
            }
        }
    }

    let ctrl = RemediatingController {
        prove_calls: std::cell::Cell::new(0),
    };
    let result = ctrl.converge("test", vec![Constraint::Invariant("x".into())]);
    assert!(result.is_ok(), "remediation should fix the violation");
    assert_eq!(ctrl.prove_calls.get(), 2, "prove should be called twice");
}

// ─── Test 4: Remediation fixes violations and re-proves ─────────────────

#[test]
fn remediation_produces_corrected_simulation() {
    let ctrl = MockController::new();
    let violations = vec![Violation {
        source: "test".into(),
        constraint: "fixable".into(),
        message: "can fix".into(),
        remediable: true,
    }];
    let result = ctrl.remediate(&"original".to_string(), &violations);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "remediated:original");
}

// ─── Test 5: Unfixable violations stop the loop ─────────────────────────

#[test]
fn unfixable_violations_stop_the_loop() {
    let ctrl = MockController::unfixable_at(ConvergencePhase::Proven);
    let result = ctrl.converge("test-service", vec![]);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.phase, ConvergencePhase::Remediated);
    assert!(!err.success);
    assert_eq!(err.message, "Remediation failed");
}

// ─── Test 6: ConvergencePhase display works ─────────────────────────────

#[test]
fn convergence_phase_display() {
    assert_eq!(ConvergencePhase::Declared.to_string(), "Declared");
    assert_eq!(ConvergencePhase::Simulated.to_string(), "Simulated");
    assert_eq!(ConvergencePhase::Proven.to_string(), "Proven");
    assert_eq!(ConvergencePhase::Remediated.to_string(), "Remediated");
    assert_eq!(ConvergencePhase::Rendered.to_string(), "Rendered");
    assert_eq!(ConvergencePhase::Deployed.to_string(), "Deployed");
    assert_eq!(ConvergencePhase::Verified.to_string(), "Verified");
    assert_eq!(ConvergencePhase::Reconverging.to_string(), "Reconverging");
}

// ─── Test 7: ConvergenceProof is_complete ───────────────────────────────

#[test]
fn convergence_proof_is_complete() {
    let complete = ConvergenceProof {
        declaration_name: "test".into(),
        constraints_checked: 5,
        constraints_satisfied: 5,
        invariants_proven: vec![],
        baselines_verified: vec![],
        proof_hash: "hash".into(),
        timestamp: "now".into(),
    };
    assert!(complete.is_complete());

    let incomplete = ConvergenceProof {
        declaration_name: "test".into(),
        constraints_checked: 5,
        constraints_satisfied: 3,
        invariants_proven: vec![],
        baselines_verified: vec![],
        proof_hash: "hash".into(),
        timestamp: "now".into(),
    };
    assert!(!incomplete.is_complete());
}

// ─── Test 8: DriftSeverity ordering ─────────────────────────────────────

#[test]
fn drift_severity_equality() {
    assert_eq!(DriftSeverity::Low, DriftSeverity::Low);
    assert_eq!(DriftSeverity::Medium, DriftSeverity::Medium);
    assert_eq!(DriftSeverity::High, DriftSeverity::High);
    assert_eq!(DriftSeverity::Critical, DriftSeverity::Critical);
    assert_ne!(DriftSeverity::Low, DriftSeverity::Critical);
    assert_ne!(DriftSeverity::Medium, DriftSeverity::High);
}

// ─── Test 9: Constraint variants serialize/deserialize ──────────────────

#[test]
fn constraint_variants_roundtrip() {
    let constraints = vec![
        Constraint::Baseline("soc2".into()),
        Constraint::Invariant("encryption".into()),
        Constraint::Platform("kubernetes".into()),
        Constraint::ComposesWith("other-service".into()),
        Constraint::Custom {
            name: "max-replicas".into(),
            value: serde_json::json!(10),
        },
    ];

    for constraint in &constraints {
        let json = serde_json::to_string(constraint).expect("serialize");
        let roundtripped: Constraint = serde_json::from_str(&json).expect("deserialize");
        // Compare serialized forms since Constraint doesn't impl PartialEq
        let json2 = serde_json::to_string(&roundtripped).expect("re-serialize");
        assert_eq!(json, json2, "roundtrip failed for {json}");
    }
}

// ─── Test 10: ConvergenceResult serialization roundtrip ─────────────────

#[test]
fn convergence_result_serialization_roundtrip() {
    let result = ConvergenceResult {
        declaration_name: "test-decl".into(),
        phase: ConvergencePhase::Proven,
        success: false,
        violations: vec![Violation {
            source: "prover".into(),
            constraint: "encryption-at-rest".into(),
            message: "missing KMS key".into(),
            remediable: true,
        }],
        certificate_hash: None,
        message: "proof failed".into(),
    };

    let json = serde_json::to_string(&result).expect("serialize");
    let roundtripped: ConvergenceResult = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(roundtripped.declaration_name, "test-decl");
    assert_eq!(roundtripped.phase, ConvergencePhase::Proven);
    assert!(!roundtripped.success);
    assert_eq!(roundtripped.violations.len(), 1);
    assert_eq!(roundtripped.violations[0].source, "prover");
    assert!(roundtripped.certificate_hash.is_none());
    assert_eq!(roundtripped.message, "proof failed");
}

// ─── Test 11: converge() default method runs all phases in order ────────

#[test]
fn converge_default_method_runs_all_phases() {
    use std::cell::RefCell;

    struct TracingController {
        phases: RefCell<Vec<String>>,
    }

    impl ConvergenceController for TracingController {
        type Simulation = String;
        type Rendering = String;
        type DeploymentHandle = String;

        fn declare(&self, intent: &str, constraints: Vec<Constraint>) -> Declaration {
            self.phases.borrow_mut().push("declare".into());
            Declaration {
                name: intent.into(),
                intent: intent.into(),
                constraints,
            }
        }

        fn simulate(&self, declaration: &Declaration) -> Result<String, Vec<Violation>> {
            self.phases.borrow_mut().push("simulate".into());
            Ok(format!("sim:{}", declaration.name))
        }

        fn prove(
            &self,
            _simulation: &String,
            constraints: &[Constraint],
        ) -> Result<ConvergenceProof, Vec<Violation>> {
            self.phases.borrow_mut().push("prove".into());
            Ok(ConvergenceProof {
                declaration_name: "test".into(),
                constraints_checked: constraints.len(),
                constraints_satisfied: constraints.len(),
                invariants_proven: vec![],
                baselines_verified: vec![],
                proof_hash: "hash".into(),
                timestamp: "now".into(),
            })
        }

        fn remediate(
            &self,
            _simulation: &String,
            _violations: &[Violation],
        ) -> Result<String, Vec<Violation>> {
            self.phases.borrow_mut().push("remediate".into());
            Ok("remediated".into())
        }

        fn render(&self, simulation: &String, _proof: &ConvergenceProof) -> String {
            self.phases.borrow_mut().push("render".into());
            format!("rendered:{simulation}")
        }

        fn deploy(&self, rendering: &String) -> Result<String, Vec<Violation>> {
            self.phases.borrow_mut().push("deploy".into());
            Ok(format!("handle:{rendering}"))
        }

        fn verify(&self, _handle: &String) -> Result<(), Vec<Drift>> {
            self.phases.borrow_mut().push("verify".into());
            Ok(())
        }

        fn reconverge(&self, _drift: &[Drift]) -> Declaration {
            self.phases.borrow_mut().push("reconverge".into());
            Declaration {
                name: "reconverge".into(),
                intent: "fix".into(),
                constraints: vec![],
            }
        }
    }

    let ctrl = TracingController {
        phases: RefCell::new(vec![]),
    };
    let result = ctrl.converge("test", vec![]);
    assert!(result.is_ok());

    let phases = ctrl.phases.borrow();
    // converge() calls: declare, simulate, prove, render
    // (deploy and verify are NOT called by converge() — it stops at rendering)
    assert_eq!(phases[0], "declare");
    assert_eq!(phases[1], "simulate");
    assert_eq!(phases[2], "prove");
    assert_eq!(phases[3], "render");
}

// ─── Test 12: Certificate hash is deterministic ─────────────────────────

#[test]
fn certificate_hash_is_deterministic() {
    let ctrl = MockController::new();
    let constraints = vec![Constraint::Invariant("test".into())];

    let cert1 = ctrl.converge("determinism", constraints.clone()).unwrap();
    let cert2 = ctrl.converge("determinism", constraints).unwrap();

    assert_eq!(
        cert1.certificate_hash, cert2.certificate_hash,
        "same inputs must produce same certificate hash"
    );
    assert!(!cert1.certificate_hash.is_empty());
}
