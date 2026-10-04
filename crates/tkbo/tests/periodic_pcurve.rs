//! Periodic pcurve translation, identity and cached-adaptor differential tests.
use rustycad_tkbo::{bop_tools::algo_tools2d, int_tools::Context};
use rustycad_tkbrep::{brep_adaptor::BRepAdaptorSurface, topods::Face};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomCylindricalSurface, GeomPlane, GeomSurface, GeomToroidalSurface};
use rustycad_tkmath::gp::{Axis2d, Axis3, Direction2, Point2};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;
use std::f64::consts::PI;
#[test]
fn native_periodic_adjustment_matches_with_and_without_context() {
    let faces: Vec<Face> = (0..5).map(fixture_face).collect();
    let context = Context::default();
    for line in include_str!("../../../tools/oracle/tkbo_periodic_pcurve.expected").lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        let kind = fields[0].parse::<usize>().unwrap();
        replay(&fields, &faces[kind], &context);
    }
}
fn fixture_face(kind: usize) -> Face {
    let surface: Handle<dyn GeomSurface> = match kind {
        0 => Handle::new(GeomPlane::new(Axis3::default())).cast(),
        1 | 3 => Handle::new(GeomCylindricalSurface::new(Axis3::default(), 2.).unwrap()).cast(),
        _ => Handle::new(GeomToroidalSurface::new(Axis3::default(), 5., 2.).unwrap()).cast(),
    };
    let umax = if kind >= 3 { 4. * PI } else { 2. };
    let vmin = if kind == 4 { 0. } else { -1. };
    let vmax = if kind == 4 { 4. * PI } else { 1. };
    BRepBuilderAPIMakeFace::from_surface_bounds(&surface, (0., umax, vmin, vmax), 1e-7)
        .unwrap()
        .face()
        .unwrap()
}
fn replay(fields: &[&str], face: &Face, context: &Context) {
    let us = [-13., -0.1, 0., 1., 2., 6.3, 13.];
    let vs = [-7., 0., 1., 7.];
    let point = Point2::new(
        us[fields[1].parse::<usize>().unwrap()],
        vs[fields[2].parse::<usize>().unwrap()],
    );
    let curve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::new(
        Point2::new(point.x() - 0.5, point.y()),
        Direction2::new(1., 0.).unwrap(),
    )))
    .cast();
    let surface = BRepAdaptorSurface::with_face(face, true).unwrap();
    let direct = algo_tools2d::adjust_pcurve_on_surface(&surface, 0., 1., &curve).unwrap();
    compare(fields, &curve, &direct);
    for cache in [None, Some(context)] {
        let result = algo_tools2d::adjust_pcurve_on_face(face, 0., 1., &curve, cache).unwrap();
        compare(fields, &curve, &result);
    }
    assert!(curve.value(0.5).unwrap().distance(point) < 1e-14);
}
fn compare(fields: &[&str], curve: &Handle<dyn Geom2dCurve>, result: &Handle<dyn Geom2dCurve>) {
    assert_eq!(result == curve, fields[3] == "1", "{fields:?}");
    let point = result.value(0.5).unwrap();
    let expected = Point2::new(fields[4].parse().unwrap(), fields[5].parse().unwrap());
    assert!(point.distance(expected) < 1e-12, "{fields:?}: {point:?}");
}
