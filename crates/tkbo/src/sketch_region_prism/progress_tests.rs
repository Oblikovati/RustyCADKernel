// SPDX-License-Identifier: GPL-2.0-only
use super::{tests::*, *};
use crate::sketch_region_candidates::{CandidateCause, tests::candidates};
use rustycad_tkernel::standard::Failure;
struct NamedSolverProgress {
    selection_polls: usize,
    fail_at: usize,
}
impl PrismProgress for NamedSolverProgress {
    fn checkpoint(&mut self, phase: PrismPhase) -> Result<bool, Failure> {
        if phase != PrismPhase::Selection {
            return Ok(false);
        }
        self.selection_polls += 1;
        if self.selection_polls != self.fail_at {
            return Ok(false);
        }
        Err(Failure::with_message(
            "Standard_DomainError",
            "selection poll 5: expected successful callback",
        ))
    }
}
#[test]
fn actual_solver_progress_failure_keeps_original_failure_and_query_prefix() {
    let source = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let mut observer = NamedSolverProgress {
        selection_polls: 0,
        fail_at: 5,
    };
    let result = retain_selected_prism(candidates(&[source]), request([1., 1.]), &mut observer);
    let PrismCompletion::Failed(PrismCause::ClassificationProgress { failure, query }) =
        result.completion
    else {
        panic!("expected original progress failure");
    };
    assert_eq!(failure.exception_type(), "Standard_DomainError");
    assert_eq!(observer.selection_polls, 5);
    assert!(matches!(query, CandidateCause::CapturedOperation(_)));
    assert!(result.diagnostic.is_none());
}
#[test]
fn invalid_limits_retain_separate_offending_quantities() {
    for quantity in ["nonnegative spatial limit", "nonnegative parameter limit"] {
        let source = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
        let mut options = request([1., 1.]);
        if quantity.contains("spatial") {
            options.spatial_limit = -1.;
        } else {
            options.parameter_limit = -1.;
        }
        let result =
            retain_selected_prism(candidates(&[source]), options, &mut progress(None, None));
        assert_invalid_limit(result, quantity);
    }
}

fn assert_invalid_limit(result: PrismReceipt, quantity: &str) {
    let PrismCompletion::Failed(PrismCause::InvalidRequest {
        quantity: actual,
        actual: values,
    }) = result.completion
    else {
        panic!("expected typed negative-limit decline");
    };
    assert_eq!(actual, quantity);
    assert_eq!(values, vec![-1.]);
}
