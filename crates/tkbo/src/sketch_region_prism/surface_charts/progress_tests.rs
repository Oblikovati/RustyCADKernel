// SPDX-License-Identifier: GPL-2.0-only
use super::super::{tests::*, *};
use super::{tests::*, *};
use crate::sketch_region_candidates::tests::candidates;
use rustycad_tkernel::standard::Failure;

struct NamedChartStop {
    history_calls: usize,
    stop_at: usize,
    inject_failure: bool,
}
impl PrismProgress for NamedChartStop {
    fn checkpoint(&mut self, phase: PrismPhase) -> Result<bool, Failure> {
        self.history_calls += usize::from(phase == PrismPhase::History);
        if phase != PrismPhase::History || self.history_calls != self.stop_at {
            return Ok(false);
        }
        if self.inject_failure {
            return Err(Failure::new(
                "chart history poll=2; expected uninterrupted exact proof",
            ));
        }
        Ok(true)
    }
}
#[test]
fn chart_cancellation_and_failure_retain_captured_prefix_and_do_not_poison_reuse() {
    for inject_failure in [false, true] {
        let source = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
        let mut observer = NamedChartStop {
            history_calls: 0,
            stop_at: 2,
            inject_failure,
        };
        let receipt =
            retain_selected_prism(candidates(&[source]), request([1., 1.]), &mut observer);
        assert_stopped_chart(&receipt, inject_failure);
    }
    assert_cubic_collection(inspection(&cubic_receipt(
        [1., 1.],
        &mut progress(None, None),
    )));
}
fn assert_stopped_chart(receipt: &PrismReceipt, failed: bool) {
    assert_history_stop(&receipt.completion, failed);
    let observed = inspection(receipt);
    assert!(!observed.scanned_all_faces);
    assert_eq!(observed.entries.len(), 1);
    assert!(observed.entries[0].captured.is_some());
    assert!(observed.entries[0].decision.is_err());
    assert!(receipt.diagnostic.as_ref().unwrap().coverage.is_none());
}
fn assert_history_stop(completion: &PrismCompletion, failed: bool) {
    if failed {
        assert!(matches!(
            completion,
            PrismCompletion::Failed(PrismCause::Mapped {
                phase: PrismPhase::History,
                ..
            })
        ));
    } else {
        assert!(matches!(
            completion,
            PrismCompletion::Cancelled(PrismPhase::History)
        ));
    }
}
#[test]
fn caller_geometry_is_isolated_after_completed_cancelled_and_failed_chart_attempts() {
    for (stop_at, inject_failure) in [(usize::MAX, false), (2, false), (2, true)] {
        assert_input_isolated(stop_at, inject_failure);
    }
}
fn assert_input_isolated(stop_at: usize, inject_failure: bool) {
    let source = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let edge = rustycad_tkbrep::topods::edge(&source).unwrap();
    let raw = rustycad_tkbrep::brep::BRepTool::curve_with_location(&edge).unwrap();
    let original = raw.curve.cast::<rustycad_tkg3d::geom::GeomBezierCurve>();
    let mut observer = stop_observer(stop_at, inject_failure);
    let mut receipt =
        retain_selected_prism(candidates(&[source]), request([1., 1.]), &mut observer);
    assert_eq!(
        original.pole(2).unwrap(),
        rustycad_tkmath::gp::Point3::new(4., 0., 0.)
    );
    original
        .set_pole(2, rustycad_tkmath::gp::Point3::new(7., 0., 0.))
        .unwrap();
    fresh_inspection(&mut receipt);
    assert_cubic_collection(inspection(&receipt));
}

fn stop_observer(stop_at: usize, inject_failure: bool) -> NamedChartStop {
    NamedChartStop {
        history_calls: 0,
        stop_at,
        inject_failure,
    }
    // The named observer keeps failure injection independent of retained geometry.
}
