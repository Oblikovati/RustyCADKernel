// SPDX-License-Identifier: GPL-2.0-only
use super::tolerance::inspect_planar_edge_residual;
use super::tolerance_types::PlanarMetricInspection;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Edge, Face},
};
use rustycad_tkernel::{ncollection::Array1, standard::Handle};
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve};
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCurve, GeomPlane, GeomSurface};
use rustycad_tkmath::{
    gp::{Axis3, Point2, Point3},
    predicates::roots::{RootIsolationOptions, metric_bounds::ResidualDecision},
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
fn spatial_cubic(height: f64) -> Handle<dyn GeomCurve> {
    let poles = [
        Point3::new(0., 0., height),
        Point3::new(4., 0., height),
        Point3::new(0., 4., height),
        Point3::new(0., 0., height),
    ];
    Handle::new(GeomBezierCurve::new(&Array1::from_shared(&poles, 1)).unwrap()).cast()
}
fn planar_cubic() -> Handle<dyn Geom2dCurve> {
    let poles = [
        Point2::new(0., 0.),
        Point2::new(4., 0.),
        Point2::new(0., 4.),
        Point2::new(0., 0.),
    ];
    Handle::new(Geom2dBezierCurve::new(&Array1::from_shared(&poles, 1)).unwrap()).cast()
}
pub(super) fn metric_fixture(height: f64) -> (Edge, Face) {
    let space = spatial_cubic(height);
    let edge = BRepBuilderAPIMakeEdge::from_curve(&space)
        .unwrap()
        .edge()
        .unwrap();
    let face = attach_planar_curve(&edge, &planar_cubic());
    (edge, face)
}
pub(super) fn attach_planar_curve(edge: &Edge, curve: &Handle<dyn Geom2dCurve>) -> Face {
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let mut face = Face::default();
    let builder = BRepBuilder::default();
    builder
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    builder
        .update_edge_pcurve_on_face(edge, curve, &face, 1e-7)
        .unwrap();
    builder.range(edge, 0., 1., false).unwrap();
    face
}
pub(super) fn inspect(
    edge: &Edge,
    face: &Face,
    limit: f64,
    cancelled: bool,
) -> PlanarMetricInspection {
    inspect_planar_edge_residual(
        edge,
        face,
        &[],
        limit,
        RootIsolationOptions::new(0.01),
        || cancelled,
    )
}
pub(super) fn assert_within(report: &PlanarMetricInspection) {
    assert!(matches!(
        report.decision,
        Ok(ResidualDecision::Within { .. })
    ));
}
pub(super) fn assert_exceeds(report: &PlanarMetricInspection) {
    assert!(matches!(
        report.decision,
        Ok(ResidualDecision::Exceeds { .. })
    ));
}
