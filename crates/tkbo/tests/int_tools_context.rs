//! Native Context regressions; differential coverage lives in verify_tkbo_foundations.py.
#[path = "../examples/bo_probe/fclass_faces.rs"]
mod faces;
use rustycad_tkbo::int_tools::{Context, Curve, Tools};
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Edge, Face},
};
use rustycad_tkernel::{ncollection::BaseAllocator, standard::Handle};
use rustycad_tkg3d::{
    geom::{GeomCurve, GeomLine, GeomTrimmedCurve},
    top_abs::{Orientation, State},
};
use rustycad_tkmath::gp::{Axis1, Point2, Point3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeVertex};
use std::sync::Arc;
fn line_edge() -> Edge {
    BRepBuilderAPIMakeEdge::from_points(Point3::default(), Point3::new(0., 0., 2.))
        .unwrap()
        .edge()
        .unwrap()
}
#[test]
fn context_identity_ignores_orientation_and_uses_allocator() {
    let c = Context::with_allocator(BaseAllocator::incremental(4096));
    let f = faces::face(0, 2.).unwrap();
    let r = Face::unchecked(f.oriented(Orientation::Reversed));
    assert!(Arc::ptr_eq(
        &c.fclass2d(&f).unwrap(),
        &c.fclass2d(&r).unwrap()
    ));
    assert!(Arc::ptr_eq(
        &c.surface_adaptor(&f).unwrap(),
        &c.surface_adaptor(&r).unwrap()
    ));
    assert!(Arc::ptr_eq(
        &c.surface_data(&f).unwrap(),
        &c.surface_data(&r).unwrap()
    ));
    assert!(Arc::ptr_eq(
        &c.hatcher(&f).unwrap(),
        &c.hatcher(&r).unwrap()
    ));
}
#[test]
fn context_uv_bounds_and_infinite_face() {
    let c = Context::default();
    let f = faces::face(0, 2.).unwrap();
    assert_eq!(c.uv_bounds(&f).unwrap(), (0., 2., 0., 2.));
    assert!(!c.is_infinite_face(&f).unwrap());
    assert!(c.is_infinite_face(&faces::face(7, 2.).unwrap()).unwrap());
}
#[test]
fn context_projection_invalidation_drops_only_surface_cache() {
    let c = Context::default();
    let f = faces::face(0, 2.).unwrap();
    let e = line_edge();
    let old = c.proj_ps(&f).unwrap();
    let edge = c.proj_pc(&e).unwrap();
    old.write().perform(Point3::new(1., 1., 0.)).unwrap();
    assert!(old.read().is_done());
    c.set_pons_projection_tolerance(1e-12);
    let new = c.proj_ps(&f).unwrap();
    assert!(!Arc::ptr_eq(&old, &new));
    assert!(!new.read().is_done());
    assert!(Arc::ptr_eq(&edge, &c.proj_pc(&e).unwrap()));
}
#[test]
fn context_point_distance_strictness_and_face_boundary() {
    let c = Context::default();
    let f = faces::face(0, 2.).unwrap();
    let p = Point3::new(1., 1., 0.125);
    assert!(!c.is_point3d_in_face(p, &f, 0.125).unwrap());
    assert!(c.is_valid_point_for_face(p, &f, 0.125).unwrap());
    assert!(c.is_valid_point_for_faces(p, &f, &f, 0.125).unwrap());
    assert_eq!(
        c.state_point_face(&f, Point2::new(0., 1.)).unwrap(),
        State::On
    );
    assert!(!c.is_point_in_face(&f, Point2::new(0., 1.)).unwrap());
    assert!(c.is_point_in_on_face(&f, Point2::new(0., 1.)).unwrap());
}
#[test]
fn context_compute_early_failure_preserves_outputs() {
    let c = Context::default();
    let mut e = Edge::default();
    BRepBuilder::default().make_edge(&mut e).unwrap();
    let vertex = BRepBuilderAPIMakeVertex::new(Point3::default())
        .unwrap()
        .vertex()
        .unwrap();
    let (mut t, mut d) = (31., 32.);
    assert_eq!(
        c.compute_pe(Point3::default(), 0., &e, &mut t, &mut d)
            .unwrap(),
        -2
    );
    assert_eq!(c.compute_ve(&vertex, &e, &mut t, &mut d, 0.).unwrap(), -2);
    BRepBuilder::default().degenerated(&e, true).unwrap();
    assert_eq!(c.compute_ve(&vertex, &e, &mut t, &mut d, 0.).unwrap(), -1);
    assert_eq!((t, d), (31., 32.));
}
#[test]
fn context_compute_point_vertex_fallback_and_no_projection() {
    let c = Context::default();
    let e = line_edge();
    let (mut t, mut d) = (31., 32.);
    let p = Point3::new(0., 0., -1e-6);
    assert_eq!(c.compute_pe(p, 1e-5, &e, &mut t, &mut d).unwrap(), 0);
    assert_eq!((t, d), (0., 1e-6));
    let vertex = BRepBuilderAPIMakeVertex::new(p).unwrap().vertex().unwrap();
    (t, d) = (31., 32.);
    assert_eq!(c.compute_ve(&vertex, &e, &mut t, &mut d, 0.).unwrap(), -3);
    assert_eq!((t, d), (31., 32.));
    assert!(!c.project_point_on_edge(p, &e, &mut t).unwrap());
    assert_eq!(t, 31.);
}
#[test]
fn context_compute_face_rejects_on_boundary_after_writing_outputs() {
    let c = Context::default();
    let f = faces::face(0, 2.).unwrap();
    let vertex = BRepBuilderAPIMakeVertex::new(Point3::new(0., 1., 0.))
        .unwrap()
        .vertex()
        .unwrap();
    let (mut u, mut v, mut tolerance) = (31., 32., 33.);
    assert_eq!(
        c.compute_vf(&vertex, &f, &mut u, &mut v, &mut tolerance, 0.)
            .unwrap(),
        -3
    );
    assert_eq!((u, v, tolerance), (0., 1., 1e-7));
}
fn trimmed_line() -> Handle<dyn GeomCurve> {
    let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(Axis1::default())).into_cast();
    let trimmed: Handle<dyn GeomCurve> =
        Handle::new(GeomTrimmedCurve::new(&line, 2., 4., true, true).unwrap()).into_cast();
    trimmed
}
#[test]
fn context_vertex_endpoint_refinement_and_interior_projection() {
    let c = Context::default();
    let trimmed = trimmed_line();
    let mut curve = Curve::default();
    curve.set_curve(&trimmed);
    let mut t = 31.;
    for z in [2., 2.0000002, 3., 3.9999998, 4.] {
        let vertex = BRepBuilderAPIMakeVertex::new(Point3::new(0., 0., z))
            .unwrap()
            .vertex()
            .unwrap();
        assert!(c.is_vertex_on_line(&vertex, &curve, 1e-7, &mut t).unwrap());
        assert_eq!(t, z);
        assert!(
            c.is_vertex_on_line_tolerance(&vertex, 1e-7, &curve, 1e-7, &mut t)
                .unwrap()
        );
    }
}
#[test]
fn context_curve_projectors_retain_curve_identity() {
    let c = Context::default();
    let trimmed = trimmed_line();
    assert!(Arc::ptr_eq(
        &c.proj_pt(&trimmed).unwrap(),
        &c.proj_pt(&trimmed).unwrap()
    ));
}
#[test]
fn intermediate_parameter_uses_pinned_rounded_constant() {
    assert_eq!(Tools::intermediate_point(0., 1.), 0.43213918);
    assert_eq!(Tools::intermediate_point(1., 0.), 1. - 0.43213918);
    assert_eq!(
        Tools::intermediate_point(-1., 4.),
        -(1. - 0.43213918) + 0.43213918 * 4.
    );
    assert!(Tools::intermediate_point(f64::INFINITY, f64::NEG_INFINITY).is_nan());
}
