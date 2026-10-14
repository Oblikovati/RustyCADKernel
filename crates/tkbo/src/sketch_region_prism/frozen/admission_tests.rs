// SPDX-License-Identifier: GPL-2.0-only
//! Final admission repeats after callbacks; required legacy diagnostics cannot disappear.
use super::{tests::*, *};
use rustycad_tkernel::{
    message::{AlertHandle, BaseAlert, Gravity, Report},
    standard::Failure,
};
use std::sync::Arc;
struct AppendReportAtCommit {
    report: Arc<Report>,
    visited: bool,
}
impl FreezeProgress for AppendReportAtCommit {
    fn checkpoint(&mut self, phase: FreezePhase) -> Result<bool, Failure> {
        if phase == FreezePhase::Commit {
            let alert: AlertHandle = Arc::new(BaseAlert);
            self.report.add_alert(Gravity::Fail, &alert);
            self.visited = true;
        }
        Ok(false)
    }
}
#[test]
fn report_alert_after_initial_admission_prevents_final_capture() {
    let (source, receipt) = authored_receipt();
    let bindings = authored_bindings(&receipt, &source);
    let mut progress = AppendReportAtCommit {
        report: receipt.input.report.clone(),
        visited: false,
    };
    let result = freeze_prism(&receipt, &bindings, &mut progress);
    assert!(progress.visited);
    assert!(matches!(
        result,
        PrismFreezeOutcome::Incomplete(PrismFreezeFailure {
            phase: FreezePhase::Commit,
            cause: PrismFreezeCause::UnsupportedDiagnostic {
                actual: "present owner-bearing input Report alert",
                ..
            },
            ..
        })
    ));
}
#[test]
fn present_affine_error_cannot_be_discarded_as_unbound_success() {
    use rustycad_tkmath::predicates::roots::metric_bounds::ResidualError;
    let (source, mut receipt) = authored_receipt();
    let bindings = authored_bindings(&receipt, &source);
    receipt.affine_chart.as_mut().unwrap().decision = Err(
        super::super::affine_chart::PrismAffineChartCause::Exact(ResidualError::Domain([2., 1.])),
    );
    let result = freeze_prism(&receipt, &bindings, &mut NamedFreezeProgress);
    assert!(matches!(
        result,
        PrismFreezeOutcome::Incomplete(PrismFreezeFailure {
            phase: FreezePhase::Admission,
            cause: PrismFreezeCause::UnsupportedDiagnostic {
                actual: "present PrismAffineChartObservation error",
                ..
            },
            ..
        })
    ));
}
#[test]
fn context_failures_preserve_native_cause_or_refuse_shape_payload() {
    use super::super::{PrismCause, PrismPhase};
    let failure = PrismCause::Mapped {
        phase: PrismPhase::History,
        failure: Failure::new("owned native cause"),
    };
    assert!(matches!(
        super::bindings::freeze_context_cause(failure),
        PrismFreezeCause::Native(_)
    ));
    let (source, _) = authored_receipt();
    let cycle = PrismCause::CyclicBodyContext { member: source };
    assert!(matches!(
        super::bindings::freeze_context_cause(cycle),
        PrismFreezeCause::UnsupportedDiagnostic {
            actual: "cyclic actual body context requires unbound Shape diagnostic",
            ..
        }
    ));
}
