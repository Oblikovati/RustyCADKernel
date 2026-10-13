// SPDX-License-Identifier: GPL-2.0-only
//! Real mismatched pcurves must fail geometry after conclusive topology.
use super::{
    outcome::{BodyKind, SourceRelation, StageCoverage, ValidationCause},
    validate_owned_candidate,
};
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topexp::Explorer,
    topods::{Shape, edge, face},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Direction2, Point2};
use rustycad_tktopalgo::brep_check::BRepCheckStatus;

fn inconsistent_face() -> (Shape, Shape) {
    let candidate = super::tests::box_face();
    let edge_shape = Explorer::new(&candidate, ShapeEnum::Edge, ShapeEnum::Shape)
        .unwrap()
        .current()
        .clone();
    let curve = displaced_pcurve();
    BRepBuilder::default()
        .update_edge_pcurve_on_face(
            &edge(&edge_shape).unwrap(),
            &curve,
            &face(&candidate).unwrap(),
            1e-7,
        )
        .unwrap();
    (candidate, edge_shape)
}

#[test]
fn inconsistent_geometry_retains_edge_face_and_all_known_source_parents() {
    let (candidate, edge) = inconsistent_face();
    let sources = [
        SourceRelation {
            output: edge.clone(),
            parents: vec![5, 9],
        },
        SourceRelation {
            output: candidate.clone(),
            parents: vec![5, 9, 17],
        },
    ];
    let report = validate_owned_candidate(&candidate, BodyKind::PlanarRegion, &sources);
    assert_eq!(report.topology, StageCoverage::Passed);
    assert_eq!(report.geometry, StageCoverage::Failed);
    assert_eq!(report.self_intersection, StageCoverage::Blocked);
    check_edge_attribution(&report, &edge);
}

fn displaced_pcurve() -> Handle<dyn Geom2dCurve> {
    let origin = Point2::new(100.0, 100.0);
    let direction = Direction2::new(1.0, 0.0).unwrap();
    Handle::new(Geom2dLine::from_point(origin, direction)).cast()
}

fn check_edge_attribution(report: &super::outcome::OrderedCoverage, edge: &Shape) {
    let finding = report
        .findings
        .iter()
        .find(|finding| finding.affected.shape.is_equal(edge))
        .unwrap();
    let ValidationCause::NativeStatuses(statuses) = &finding.cause else {
        panic!("expected native pcurve defects")
    };
    assert!(statuses.contains(&BRepCheckStatus::InvalidCurveOnSurface));
    assert_eq!(finding.affected.source_parents, Some(vec![5, 9]));
    assert_eq!(
        finding.context.as_ref().unwrap().source_parents,
        Some(vec![5, 9, 17])
    );
}
