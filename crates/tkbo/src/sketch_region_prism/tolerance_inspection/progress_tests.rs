// SPDX-License-Identifier: GPL-2.0-only
//! Mutation and cancellation checkpoints precede a callback-free final binding pass.
use super::{inspection_fixture::*, inspection_tests::assert_zero, *};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkg3d::top_abs::ShapeEnum;

struct CancelAt(ToleranceInspectionPhase);
impl ToleranceInspectionProgress for CancelAt {
    fn cancelled(&mut self, phase: ToleranceInspectionPhase) -> bool {
        phase == self.0
    }
}
#[test]
fn all_checkpoints_cancel_without_promoting_or_mutating_the_diagnostic() {
    for phase in [
        ToleranceInspectionPhase::Capture,
        ToleranceInspectionPhase::Metadata,
        ToleranceInspectionPhase::Endpoints,
        ToleranceInspectionPhase::Commit,
    ] {
        let fixture = InspectionFixture::new();
        let before = fixture.captured(ordinary_request());
        let error = fixture
            .inspect(ordinary_request(), &mut CancelAt(phase))
            .err()
            .unwrap();
        assert_eq!(error.phase, phase);
        assert_cancelled(error.cause);
        assert_zero(&before);
        assert_zero(&fixture.captured(ordinary_request()));
    }
}
fn assert_cancelled(cause: ToleranceInspectionCause) {
    match cause {
        ToleranceInspectionCause::Cancelled => {}
        ToleranceInspectionCause::Binding(cause) => assert!(matches!(
            *cause,
            super::super::region_certificate::current_binding::PrismBindingFailure::Cancelled
        )),
        _ => panic!("expected typed cancellation"),
    }
}
struct ChangeRawAtCommit {
    actual: Shape,
    raw: f64,
}
impl ToleranceInspectionProgress for ChangeRawAtCommit {
    fn cancelled(&mut self, phase: ToleranceInspectionPhase) -> bool {
        if phase == ToleranceInspectionPhase::Commit {
            set_raw(&self.actual, self.raw);
        }
        false
    }
}
#[test]
fn commit_subfloor_metadata_mutation_is_visible_despite_identical_effective_floor() {
    for kind in [ShapeEnum::Vertex, ShapeEnum::Edge, ShapeEnum::Face] {
        check_raw_mutation(kind, 5e-8);
    }
}
fn check_raw_mutation(kind: ShapeEnum, raw: f64) {
    let fixture = InspectionFixture::new();
    let actual = fixture.actual(kind);
    set_raw(&actual, 0.);
    let before = fixture.captured(ordinary_request());
    let error = fixture
        .inspect(ordinary_request(), &mut ChangeRawAtCommit { actual, raw })
        .err()
        .unwrap();
    assert_eq!(error.phase, ToleranceInspectionPhase::Commit);
    assert!(matches!(
        error.cause,
        ToleranceInspectionCause::Stale {
            entity: Some(_),
            quantity: "raw/effective metadata bits"
        }
    ));
    assert_zero(&before);
    assert!(before.metadata.iter().all(|row| row.observation.raw != raw));
}
#[test]
fn invalid_metadata_added_at_commit_keeps_scalar_cause_and_commit_phase() {
    let fixture = InspectionFixture::new();
    let actual = fixture.actual(ShapeEnum::Face);
    let error = fixture
        .inspect(
            ordinary_request(),
            &mut ChangeRawAtCommit {
                actual,
                raw: f64::NAN,
            },
        )
        .err()
        .unwrap();
    assert_eq!(error.phase, ToleranceInspectionPhase::Commit);
    assert!(
        matches!(error.cause, ToleranceInspectionCause::Scalar { cause: ToleranceScalarError { quantity: ToleranceQuantity::RawStoredMetadata, actual }, .. } if actual.is_nan())
    );
}

struct ChangeVertexAtCommit {
    actual: Shape,
}
impl ToleranceInspectionProgress for ChangeVertexAtCommit {
    fn cancelled(&mut self, phase: ToleranceInspectionPhase) -> bool {
        if phase == ToleranceInspectionPhase::Commit {
            rustycad_tkbrep::brep::access::vertex_tshape(&self.actual, "source mutation fixture")
                .unwrap()
                .set_pnt(rustycad_tkmath::gp::Point3::new(0.125, 0., 0.));
        }
        false
    }
}
#[test]
fn original_source_mutation_at_commit_rejects_stale_evidence_without_changing_owned_capture() {
    let fixture = InspectionFixture::new();
    let before = fixture.captured(ordinary_request());
    let actual = crate::sketch_region::equal_occurrences(&fixture.sources[0])
        .unwrap()
        .into_iter()
        .find(|shape| shape.shape_type().ok() == Some(ShapeEnum::Vertex))
        .unwrap();
    let error = fixture
        .inspect(ordinary_request(), &mut ChangeVertexAtCommit { actual })
        .err()
        .unwrap();
    assert_eq!(error.phase, ToleranceInspectionPhase::Commit);
    assert!(matches!(error.cause, ToleranceInspectionCause::Binding(_)));
    assert_zero(&before);
    assert!(before.census.endpoints.iter().all(|row| row.point[0] == 0.));
}
