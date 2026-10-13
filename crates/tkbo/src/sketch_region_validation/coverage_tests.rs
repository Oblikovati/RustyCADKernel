// SPDX-License-Identifier: GPL-2.0-only
//! Ordered stage results must distinguish accepted mapped checks from missing proof.
use super::{
    outcome::{BodyKind, StageCoverage, ValidationCause},
    validate_owned_candidate,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

#[test]
fn valid_solid_advances_two_stages_but_not_complete_body() {
    let mut maker = BRepPrimAPIMakeBox::from_sizes(1.0, 1.0, 1.0).unwrap();
    let report = validate_owned_candidate(maker.shape().unwrap(), BodyKind::Solid, &[]);
    assert_eq!(
        report.topology,
        StageCoverage::Passed,
        "{:?}",
        report.findings
    );
    assert_eq!(report.geometry, StageCoverage::Passed);
    assert_eq!(report.self_intersection, StageCoverage::Incomplete);
    assert_eq!(report.tolerance, StageCoverage::Blocked);
    assert!(!report.complete());
}

#[test]
fn null_candidate_retains_failure_and_blocks_later_stages() {
    let report = validate_owned_candidate(&Shape::default(), BodyKind::Solid, &[]);
    assert_eq!(report.topology, StageCoverage::Incomplete);
    assert_eq!(report.geometry, StageCoverage::Blocked);
    assert!(matches!(
        report.findings[0].cause,
        ValidationCause::NativeFailure(_)
    ));
}

#[test]
fn face_region_rejects_solid_kind_without_geometry_scan() {
    let report = validate_owned_candidate(&super::tests::box_face(), BodyKind::Solid, &[]);
    assert_eq!(report.topology, StageCoverage::Failed);
    assert_eq!(report.geometry, StageCoverage::Blocked);
    assert!(matches!(
        report.findings[0].cause,
        ValidationCause::BodyKindMismatch { .. }
    ));
}

#[test]
fn valid_planar_region_advances_mapped_geometry() {
    let report = validate_owned_candidate(&super::tests::box_face(), BodyKind::PlanarRegion, &[]);
    assert_eq!(report.topology, StageCoverage::Passed);
    assert_eq!(report.geometry, StageCoverage::Passed);
    assert!(!report.complete());
}

#[test]
fn spherical_face_cannot_be_a_planar_region() {
    use rustycad_tkbrep::topexp::Explorer;
    use rustycad_tkg3d::top_abs::ShapeEnum;
    use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeSphere;
    let mut maker = BRepPrimAPIMakeSphere::new(1.0).unwrap();
    let face = Explorer::new(maker.shape().unwrap(), ShapeEnum::Face, ShapeEnum::Shape)
        .unwrap()
        .current()
        .clone();
    let report = validate_owned_candidate(&face, BodyKind::PlanarRegion, &[]);
    assert_eq!(report.topology, StageCoverage::Passed);
    assert_eq!(report.geometry, StageCoverage::Failed);
    assert!(matches!(
        report.findings[0].cause,
        ValidationCause::NonPlanarRegion
    ));
}

#[test]
fn success_failure_success_has_no_retained_coverage() {
    let face = super::tests::box_face();
    assert_eq!(
        validate_owned_candidate(&face, BodyKind::PlanarRegion, &[]).geometry,
        StageCoverage::Passed
    );
    assert_eq!(
        validate_owned_candidate(&Shape::default(), BodyKind::PlanarRegion, &[]).geometry,
        StageCoverage::Blocked
    );
    assert_eq!(
        validate_owned_candidate(&face, BodyKind::PlanarRegion, &[]).geometry,
        StageCoverage::Passed
    );
}
