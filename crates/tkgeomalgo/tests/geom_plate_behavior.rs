//! Behaviour tests of GeomPlate. The GeomPlate content of
//! GeomPlate_BuildPlateSurface_Test.cxx, whose A1 and A2 cases build their curves as
//! `BRepAdaptor_Curve` of `BRepBuilderAPI_MakeEdge` edges (TKTopAlgo, not ported):
//! here the same Bezier curves go through `GeomAdaptor_Curve` on their natural range,
//! which an edge of `MakeEdge` evaluates identically.
use rustycad_tkernel::message::{
    ProgressIndicator, ProgressIndicatorHooks, ProgressRange, ProgressScope,
};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCurve, GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeomalgo::geom_plate::{
    GeomPlateBuildPlateSurface, GeomPlateCurveConstraint, GeomPlatePointConstraint,
};
use rustycad_tkmath::gp::{Axis3, Direction3, Point3};
use std::f64::consts::PI;
use std::sync::{Arc, Mutex};

/// The `ProgressObserver` of the native test: one message per show, the rounded
/// percentage followed by the names of the scope chain.
struct ProgressObserver(Arc<Mutex<Vec<String>>>);
impl ProgressIndicatorHooks for ProgressObserver {
    fn show(&self, scope: &ProgressScope<'_>, position: f64, _force: bool) {
        let mut message = format!("{}%", (100.0 * position + 0.5) as i32);
        let mut current: Option<&ProgressScope<'_>> = Some(scope);
        while let Some(s) = current {
            if let Some(name) = s.name() {
                message.push(' ');
                message.push_str(&String::from_utf8_lossy(name));
            }
            current = s.parent();
        }
        self.0.lock().unwrap().push(message);
    }
}

fn expect_plate_progress(messages: &[String]) {
    assert!(!messages.is_empty());
    for text in [
        "0%",
        "100%",
        "Calculating the surface filled",
        "Plate_Plate::SolveTI1()",
        "math_Gauss LU_Decompose",
    ] {
        assert!(
            messages.iter().any(|m| m.contains(text)),
            "missing {text} in {messages:?}"
        );
    }
}

fn bezier(p1: Point3, p2: Point3, p3: Point3) -> Handle<dyn Adaptor3dCurve> {
    let mut poles: Array1<'static, Point3> = Array1::with_bounds(1, 3).unwrap();
    (poles[1], poles[2], poles[3]) = (p1, p2, p3);
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomBezierCurve::new(&poles).unwrap()).cast();
    Handle::new(GeomAdaptorCurve::with_curve(&curve).unwrap()).cast()
}

fn add_curve(builder: &mut GeomPlateBuildPlateSurface, curve: Handle<dyn Adaptor3dCurve>) {
    let constraint = GeomPlateCurveConstraint::new(&curve, 0, 10, 0.0001, 0.01, 0.1).unwrap();
    builder.add_curve(&Handle::new(constraint));
}

fn add_point(builder: &mut GeomPlateBuildPlateSurface, p: Point3) {
    let constraint = GeomPlatePointConstraint::with_point(p, 0, 0.0001).unwrap();
    builder.add_point(&Handle::new(constraint));
}

fn p(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

/// Performs with a progress observer and returns its messages.
fn perform_observed(builder: &mut GeomPlateBuildPlateSurface) -> Vec<String> {
    let messages = Arc::new(Mutex::new(Vec::new()));
    let indicator = ProgressIndicator::new(Box::new(ProgressObserver(Arc::clone(&messages))));
    builder.perform(&indicator.start()).unwrap();
    drop(indicator);
    Arc::try_unwrap(messages).unwrap().into_inner().unwrap()
}

fn default_builder() -> GeomPlateBuildPlateSurface {
    GeomPlateBuildPlateSurface::new(3, 15, 2, 0.00001, 0.0001, 0.01, 0.1, false).unwrap()
}

#[test]
fn occ525_perform_without_constraints() {
    let mut builder =
        GeomPlateBuildPlateSurface::new(3, 10, 3, 0.00001, 0.0001, 0.01, 0.1, false).unwrap();
    builder.perform(&ProgressRange::default()).unwrap();
    // IsDone() is true because Plate_Plate::Init() sets OK = true, as the native test notes.
    assert!(builder.is_done());
    assert!(builder.surface().is_null());
}

#[test]
fn perform_clears_stale_result() {
    let mut builder =
        GeomPlateBuildPlateSurface::new(3, 10, 3, 1.0e-5, 1.0e-4, 0.01, 0.1, false).unwrap();
    for q in [
        p(0.0, 0.0, 0.0),
        p(1.0, 0.0, 0.0),
        p(0.0, 1.0, 0.0),
        p(1.0, 1.0, 0.1),
    ] {
        add_point(&mut builder, q);
    }
    builder.perform(&ProgressRange::default()).unwrap();
    assert!(builder.surface().is_some());
    builder.init();
    builder.perform(&ProgressRange::default()).unwrap();
    assert!(builder.is_done());
    assert!(builder.surface().is_null());
}

#[test]
fn progress_a1_four_bezier_edges_with_plane() {
    let mut builder = default_builder();
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(
        Axis3::from_normal(p(0.0, 0.0, 0.0), Direction3::new(0.0, 0.0, 1.0).unwrap()).unwrap(),
    ))
    .cast();
    builder.load_init_surface(&plane);
    add_curve(
        &mut builder,
        bezier(p(0., 0., 0.), p(1., 0., 1.), p(2., 0., 0.)),
    );
    add_curve(
        &mut builder,
        bezier(p(0., 2., 0.), p(1., 2., 1.), p(2., 2., 0.)),
    );
    add_curve(
        &mut builder,
        bezier(p(0., 0., 0.), p(0., 1., 1.), p(0., 2., 0.)),
    );
    add_curve(
        &mut builder,
        bezier(p(2., 0., 0.), p(2., 1., 1.), p(2., 2., 0.)),
    );
    let messages = perform_observed(&mut builder);
    assert!(builder.is_done());
    assert!(builder.surface().is_some());
    expect_plate_progress(&messages);
}

#[test]
fn progress_a2_four_bezier_edges_without_plane() {
    let mut builder = default_builder();
    add_curve(
        &mut builder,
        bezier(p(0., 0., 0.), p(1., 0., 1.), p(2., 0., 0.)),
    );
    add_curve(
        &mut builder,
        bezier(p(0., 2., 0.), p(1., 2., 1.), p(2., 2., 0.)),
    );
    add_curve(
        &mut builder,
        bezier(p(0., 0., 0.), p(0., 1., 2.), p(0., 2., 0.)),
    );
    add_curve(
        &mut builder,
        bezier(p(2., 0., 0.), p(2., 1., 2.), p(2., 2., 0.)),
    );
    let messages = perform_observed(&mut builder);
    assert!(builder.is_done());
    assert!(builder.surface().is_some());
    expect_plate_progress(&messages);
}

#[test]
fn progress_a3_large_point_contour() {
    let mut builder = default_builder();
    (0..300).for_each(|i| add_point(&mut builder, p(f64::from(i), 0., 0.)));
    (1..300).for_each(|i| add_point(&mut builder, p(0., f64::from(i), 0.)));
    (0..300).for_each(|i| add_point(&mut builder, p(300., f64::from(i), 0.)));
    (0..=300).for_each(|i| add_point(&mut builder, p(f64::from(i), 300., 0.)));
    let messages = perform_observed(&mut builder);
    assert!(builder.is_done());
    assert!(builder.surface().is_some());
    expect_plate_progress(&messages);
}

#[test]
fn progress_a4_one_hundred_point_contour() {
    let mut builder = default_builder();
    for i in 0..100 {
        let t: f64 = 2. * PI * f64::from(i) / 100.;
        add_point(
            &mut builder,
            p(t.sin(), t.cos(), (1. - 2. * f64::from(i) / 100.).abs()),
        );
    }
    let messages = perform_observed(&mut builder);
    assert!(builder.is_done());
    assert!(builder.surface().is_some());
    expect_plate_progress(&messages);
}
