use super::*;
use rustycad_tkg3d::geom::{
    GeomCircle, GeomConicalSurface, GeomCurve, GeomLine, GeomPlane, GeomTrimmedCurve,
};
use rustycad_tkmath::gp::{Axis1, Axis2, Axis3, Direction3, Point3};
#[test]
fn initial_state_and_unguarded_points_match_native() {
    let mut ff = FaceFace::default();
    assert!(!ff.is_done());
    assert!(!ff.tangent_faces());
    assert!(ff.face1().is_null() && ff.face2().is_null());
    assert!(ff.context().is_null());
    assert!(ff.points().is_empty());
    assert_eq!(
        ff.lines().err().unwrap().exception_type(),
        "StdFail_NotDone"
    );
    assert_eq!(ff.fuzzy_value(), precision::confusion());
    ff.prepare_lines_3d(true).unwrap();
    ff.done = true;
    assert!(ff.lines().unwrap().is_empty());
}
#[test]
fn setters_preserve_copy_identity() {
    let mut ff = FaceFace::default();
    ff.set_parameters(false, true, false, -3.0);
    assert_eq!(ff.approximation, [false, true, false]);
    assert_eq!(ff.approximation_tolerance, -3.0);
    let mut seeds = vec![IntSurfPntOn2S::default()];
    ff.set_list(&seeds);
    seeds.clear();
    assert_eq!(ff.seeds.len(), 1);
    let context = Handle::new(Context::default());
    ff.set_context(&context);
    assert!(ff.context() == &context);
}
#[test]
fn fuzzy_preserves_native_nan_order() {
    let mut ff = FaceFace::default();
    for (input, expected) in [
        (-1.0, 1e-7),
        (0.0, 1e-7),
        (1e-6, 1e-6),
        (f64::INFINITY, f64::INFINITY),
    ] {
        ff.set_fuzzy_value(input);
        assert_eq!(ff.fuzzy_value(), expected);
    }
    ff.set_fuzzy_value(f64::NAN);
    assert!(ff.fuzzy_value().is_nan());
}
fn descriptor(geometry: Handle<dyn GeomCurve>) -> Curve {
    Curve::new(
        &geometry,
        &Handle::default(),
        &Handle::default(),
        2e-7,
        3e-7,
    )
}
fn closed_circle() -> Curve {
    let curve = Handle::new(GeomCircle::new(Axis2::default(), 2.0).unwrap()).cast();
    descriptor(
        Handle::new(GeomTrimmedCurve::new(&curve, 0.0, std::f64::consts::TAU, true, true).unwrap())
            .cast(),
    )
}
#[test]
fn preparation_splits_closed_curves_only_when_requested() {
    let mut ff = FaceFace::default();
    ff.curves.push(closed_circle());
    ff.prepare_lines_3d(false).unwrap();
    assert_eq!(ff.curves.len(), 1);
    ff.prepare_lines_3d(true).unwrap();
    assert_eq!(ff.curves.len(), 2);
    assert_eq!(ff.curves[0].curve().last_parameter(), std::f64::consts::PI);
    assert_eq!(ff.curves[1].curve().first_parameter(), std::f64::consts::PI);
    assert!(
        ff.curves
            .iter()
            .all(|curve| curve.tolerance() == 2e-7 && curve.tangential_tolerance() == 3e-7)
    );
}
fn plane_cone() -> FaceFace {
    let ff = FaceFace::default();
    let plane = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let cone = Handle::new(GeomConicalSurface::new(Axis3::default(), 0.3, 2.0).unwrap()).cast();
    ff.surfaces[0].load(&plane).unwrap();
    ff.surfaces[1].load(&cone).unwrap();
    ff
}
#[test]
fn preparation_reduces_only_four_line_plane_cone_case() {
    let mut ff = plane_cone();
    let line = descriptor(Handle::new(GeomLine::new(Axis1::default())).cast());
    ff.curves = vec![line.clone(); 3];
    ff.prepare_lines_3d(false).unwrap();
    assert_eq!(ff.curves.len(), 3);
    ff.curves.push(line);
    ff.curves[2] =
        descriptor(Handle::new(GeomLine::new(Axis1::new(Point3::default(), Direction3::X))).cast());
    ff.prepare_lines_3d(false).unwrap();
    assert_eq!(ff.curves.len(), 2);
    ff.curves = vec![closed_circle(); 4];
    ff.prepare_lines_3d(false).unwrap();
    assert_eq!(ff.curves.len(), 4);
}
