// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region_candidates::{SeedSelection, tests::candidates};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::{gp::Vector3, predicates::roots::RootIsolationOptions};
pub(super) struct NamedPrismProgress {
    stop: Option<PrismPhase>,
    fail: Option<PrismPhase>,
    visited: Vec<PrismPhase>,
}
impl PrismProgress for NamedPrismProgress {
    fn checkpoint(&mut self, phase: PrismPhase) -> Result<bool, Failure> {
        self.visited.push(phase);
        if self.fail == Some(phase) {
            return Err(Failure::new(&format!(
                "injected phase {phase:?}: expected uninterrupted execution"
            )));
        }
        Ok(self.stop == Some(phase))
    }
}
pub(super) fn progress(stop: Option<PrismPhase>, fail: Option<PrismPhase>) -> NamedPrismProgress {
    NamedPrismProgress {
        stop,
        fail,
        visited: Vec::new(),
    }
}
pub(super) fn request(seed: [f64; 2]) -> PrismRequest {
    PrismRequest {
        seed,
        extrusion: Vector3::new(0., 0., 2.),
        roots: RootIsolationOptions::new(0.001),
        spatial_limit: 1e-8,
        parameter_limit: 1e-8,
    }
}
pub(super) fn cubic_receipt(seed: [f64; 2], observer: &mut NamedPrismProgress) -> PrismReceipt {
    let source = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let batch = candidates(&[source]);
    retain_selected_prism(batch, request(seed), observer)
}
#[test]
fn cubic_prism_retains_history_but_never_claims_completed_body() {
    let result = cubic_receipt([1., 1.], &mut progress(None, None));
    assert!(matches!(result.completion, PrismCompletion::Incomplete));
    let diagnostic = result.diagnostic.unwrap();
    assert!(!diagnostic.relations.is_empty());
    assert!(
        diagnostic
            .relations
            .iter()
            .all(|row| !row.generated.is_empty() || !row.modified.is_empty())
    );
    assert!(result.limits.achieved_error.is_none());
    assert!(!diagnostic.coverage.unwrap().complete());
}
#[test]
fn nonunique_selection_has_no_body_and_is_not_native_deletion() {
    for (seed, boundary) in [([-1., 1.], false), ([0., 0.], true)] {
        let result = cubic_receipt(seed, &mut progress(None, None));
        match result.completion {
            PrismCompletion::NoBody(SeedSelection::Empty) => assert!(!boundary),
            PrismCompletion::NoBody(SeedSelection::Boundary { .. }) => assert!(boundary),
            _ => panic!("expected explicit empty/boundary disposition"),
        }
        assert!(result.diagnostic.is_none());
        assert!(result.input.input_deleted.is_empty());
    }
}
#[test]
fn each_boundary_preserves_cancellation_and_original_typed_failure() {
    for phase in [
        PrismPhase::Admission,
        PrismPhase::Selection,
        PrismPhase::Copy,
        PrismPhase::Sweep,
        PrismPhase::History,
        PrismPhase::Validation,
    ] {
        let cancelled = cubic_receipt([1., 1.], &mut progress(Some(phase), None));
        assert!(
            matches!(cancelled.completion, PrismCompletion::Cancelled(actual) if actual==phase)
        );
        let failed = cubic_receipt([1., 1.], &mut progress(None, Some(phase)));
        assert!(
            matches!(failed.completion, PrismCompletion::Failed(PrismCause::Mapped {phase:actual,..}) if actual==phase)
        );
        assert_eq!(failed.phase, phase);
    }
}
#[test]
fn invalid_extrusion_preserves_offending_value_without_sweep() {
    for vector in [Vector3::new(0., 0., 0.), Vector3::new(f64::NAN, 0., 2.)] {
        let source = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
        let mut options = request([1., 1.]);
        options.extrusion = vector;
        let result =
            retain_selected_prism(candidates(&[source]), options, &mut progress(None, None));
        assert!(matches!(
            result.completion,
            PrismCompletion::Failed(PrismCause::InvalidRequest { .. })
        ));
        assert!(result.diagnostic.is_none());
    }
}
