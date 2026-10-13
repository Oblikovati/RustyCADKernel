// SPDX-License-Identifier: GPL-2.0-only
//! The internal stage is not a healthy-body API; these tests retain that distinction.
use super::{arrangement::*, copy_inputs::*, outcome::*};
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp,
    topods::{Shape, edge},
    toptools::IndexedMapOfShape,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::{geom::GeomBezierCurve, top_abs::ShapeEnum};
use rustycad_tkmath::gp::Point3;

#[path = "../../tests/support/closed_analytic_cubic.rs"]
mod cubic_fixture;
pub(super) fn closed_cubic() -> Shape {
    // Both diagnostic/native bridge and stage admission use the same authored geometry.
    cubic_fixture::closed_analytic_cubic(1., false)
}

pub(super) fn sources(shape: &Shape) -> [ArrangementSource<'_>; 1] {
    [ArrangementSource {
        identity: RegionSourceId(42),
        shape,
    }]
}
pub(super) fn arranged(shape: &Shape) -> OwnedArrangement {
    match arrange_owned_sources(&sources(shape), &mut |_| false) {
        ArrangementCompletion::Arranged(output) => output,
        _ => panic!("owned arrangement: expected completed stage, not a final valid body"),
    }
}
#[test]
fn closed_analytic_curve_uses_general_pipeline_without_changing_default_admission() {
    let source = closed_cubic();
    let output = arranged(&source);
    assert!(!output.arrangement.is_null());
    assert!(output.achieved_error.is_none() && !output.incidences_certified);
    let copied = copy_sources(&sources(&source)).unwrap_or_else(|_| panic!("copy failed"));
    let mut mapped = crate::bop_algo::BOPAlgoBuilder::new();
    mapped.set_arguments(&copied.arguments);
    mapped.perform();
    assert!(mapped.options().has_errors());
}
#[test]
fn empty_null_and_duplicate_identity_are_explicit_declines() {
    assert!(matches!(
        arrange_owned_sources(&[], &mut |_| false),
        ArrangementCompletion::Declined(ArrangementCause::Empty)
    ));
    let null = Shape::default();
    assert!(matches!(
        arrange_owned_sources(&sources(&null), &mut |_| false),
        ArrangementCompletion::Declined(ArrangementCause::NullSource(RegionSourceId(42)))
    ));
}
#[test]
fn duplicate_source_identity_is_declined_before_copy() {
    let source = closed_cubic();
    let repeated = [
        ArrangementSource {
            identity: RegionSourceId(42),
            shape: &source,
        },
        ArrangementSource {
            identity: RegionSourceId(42),
            shape: &source,
        },
    ];
    assert!(matches!(
        arrange_owned_sources(&repeated, &mut |_| false),
        ArrangementCompletion::Declined(ArrangementCause::DuplicateIdentity(RegionSourceId(42)))
    ));
}
pub(super) fn curve_of(shape: &Shape) -> Handle<GeomBezierCurve> {
    let mut edges = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(shape, ShapeEnum::Edge, &mut edges).unwrap();
    let edge = edge(edges.find_key(1).unwrap()).unwrap();
    BRepTool::curve(&edge).unwrap().0.cast()
}
#[test]
fn copied_output_survives_later_source_geometry_and_topology_mutation() {
    let source = closed_cubic();
    let original_free = source.free().unwrap();
    let output = arranged(&source);
    assert_eq!(source.free().unwrap(), original_free);
    curve_of(&source)
        .set_pole(2, Point3::new(8., 0., 0.))
        .unwrap();
    let output_closed = output.arrangement.closed().unwrap();
    source.set_closed(false).unwrap();
    assert_eq!(
        curve_of(&output.arrangement).pole(2).unwrap(),
        Point3::new(4., 0., 0.)
    );
    assert!(!curve_of(&source).is_null());
    assert_eq!(output.arrangement.closed().unwrap(), output_closed);
}
pub(super) struct FailingMappedStage;
impl ArrangementExecutor for FailingMappedStage {
    fn execute(
        &mut self,
        copied: &CopiedSources,
    ) -> Result<crate::bop_algo::BOPAlgoBuilder, ArrangementCause> {
        curve_of(
            copied
                .arguments
                .first()
                .ok_or_else(|| ArrangementCause::Empty)?,
        )
        .set_pole(2, Point3::new(9., 0., 0.))
        .unwrap();
        Err(ArrangementCause::Failure {
            phase: ArrangementPhase::Interference,
            cause: Failure::new("named controlled interference failure"),
        })
    }
}
struct MissingMappedHistory;
impl ArrangementExecutor for MissingMappedHistory {
    fn execute(
        &mut self,
        _: &CopiedSources,
    ) -> Result<crate::bop_algo::BOPAlgoBuilder, ArrangementCause> {
        let mut builder = crate::bop_algo::BOPAlgoBuilder::new();
        builder.set_to_fill_history(false);
        Ok(builder)
    }
}
#[test]
fn failing_and_missing_history_stages_preserve_caller_geometry() {
    let source = closed_cubic();
    let result = arrange_with_executor(&sources(&source), &mut |_| false, &mut FailingMappedStage);
    assert!(matches!(
        result,
        ArrangementCompletion::Failed(ArrangementCause::Failure {
            phase: ArrangementPhase::Interference,
            ..
        })
    ));
    assert_eq!(curve_of(&source).pole(2).unwrap(), Point3::new(4., 0., 0.));
    let result =
        arrange_with_executor(&sources(&source), &mut |_| false, &mut MissingMappedHistory);
    assert!(matches!(
        result,
        ArrangementCompletion::Incomplete(ArrangementCause::MissingHistory { source: None })
    ));
}
#[test]
fn cancellation_at_each_boundary_leaves_source_unchanged() {
    let source = closed_cubic();
    for phase in [
        ArrangementPhase::Copy,
        ArrangementPhase::Interference,
        ArrangementPhase::History,
    ] {
        let result = arrange_owned_sources(&sources(&source), &mut |current| current == phase);
        assert!(matches!(result, ArrangementCompletion::Cancelled(found) if found==phase));
        assert_eq!(curve_of(&source).pole(2).unwrap(), Point3::new(4., 0., 0.));
    }
}

#[test]
fn typed_report_snapshot_survives_original_container_reuse_without_guessing_source() {
    use rustycad_tkernel::message::{AlertHandle, Gravity, Report};
    let original = Report::new();
    let alert: AlertHandle = std::sync::Arc::new(crate::bop_algo::AlertIntersectionFailed);
    original.add_alert(Gravity::Fail, &alert);
    let captured = snapshot_report(&original);
    original.clear();
    let retained = captured.get_alerts(Gravity::Fail);
    assert_eq!(retained.len(), 1);
    assert!(std::sync::Arc::ptr_eq(&retained[0], &alert));
    assert!(original.get_alerts(Gravity::Fail).is_empty());
}

struct ReportedMappedFailure(std::sync::Arc<rustycad_tkernel::message::Report>);
impl ArrangementExecutor for ReportedMappedFailure {
    fn execute(
        &mut self,
        _: &CopiedSources,
    ) -> Result<crate::bop_algo::BOPAlgoBuilder, ArrangementCause> {
        let builder = crate::bop_algo::BOPAlgoBuilder::new();
        builder.options().report().merge(&self.0);
        Ok(builder)
    }
}
#[test]
fn actual_mapped_alert_failure_keeps_typed_payload_and_unknown_affected_source() {
    use rustycad_tkernel::message::{AlertHandle, Gravity, Report};
    let original = std::sync::Arc::new(Report::new());
    let alert: AlertHandle = std::sync::Arc::new(crate::bop_algo::AlertIntersectionFailed);
    original.add_alert(Gravity::Fail, &alert);
    let source = closed_cubic();
    let result = arrange_with_executor(
        &sources(&source),
        &mut |_| false,
        &mut ReportedMappedFailure(original.clone()),
    );
    original.clear();
    assert_reported_failure(result, &alert);
}

fn assert_reported_failure(
    result: ArrangementCompletion,
    alert: &rustycad_tkernel::message::AlertHandle,
) {
    let ArrangementCompletion::Failed(ArrangementCause::MappedReport {
        report,
        affected_source,
        phase,
    }) = result
    else {
        panic!("expected typedreport failure")
    };
    assert!(affected_source.is_none() && phase == ArrangementPhase::Interference);
    assert!(std::sync::Arc::ptr_eq(
        &report.get_alerts(rustycad_tkernel::message::Gravity::Fail)[0],
        alert
    ));
}
