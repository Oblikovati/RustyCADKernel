//! 1:1 translation of src/ModelingAlgorithms/TKGeomAlgo/GTests/GeomPlate_BuildPlateSurface_Test.cxx.
#![allow(non_snake_case)]

use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::message::{
    ProgressIndicator, ProgressIndicatorHooks, ProgressRange, ProgressScope,
};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCurve, GeomPlane, GeomSurface};
use rustycad_tkgeomalgo::geom_plate::{
    GeomPlateBuildPlateSurface, GeomPlateCurveConstraint, GeomPlatePointConstraint,
};
use rustycad_tkmath::gp::{Axis3, Direction3, Point3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
use std::f64::consts::PI;
use std::sync::{Arc, Mutex};

/// The `ProgressObserver` of the native test: one message per Show, the rounded
/// percentage followed by the names of the scope chain.
struct ProgressObserver(Arc<Mutex<Vec<String>>>);

impl ProgressIndicatorHooks for ProgressObserver {
    fn show(&self, the_scope: &ProgressScope<'_>, position: f64, _force: bool) {
        let mut a_message = format!("{}%", (100.0 * position + 0.5) as i32);
        let mut a_scope: Option<&ProgressScope<'_>> = Some(the_scope);
        while let Some(scope) = a_scope {
            if let Some(name) = scope.name() {
                a_message.push(' ');
                a_message.push_str(&String::from_utf8_lossy(name));
            }
            a_scope = scope.parent();
        }
        self.0.lock().unwrap().push(a_message);
    }
}

fn has_progress_text(the_messages: &[String], the_text: &str) -> bool {
    the_messages.iter().any(|m| m.contains(the_text))
}

fn create_bezier_curve(the_p1: Point3, the_p2: Point3, the_p3: Point3) -> Handle<dyn GeomCurve> {
    let mut a_poles: Array1<'static, Point3> = Array1::with_bounds(1, 3).unwrap();
    a_poles[1] = the_p1;
    a_poles[2] = the_p2;
    a_poles[3] = the_p3;
    Handle::new(GeomBezierCurve::new(&a_poles).unwrap()).cast()
}

fn create_bezier_edge(the_p1: Point3, the_p2: Point3, the_p3: Point3) -> Edge {
    BRepBuilderAPIMakeEdge::from_curve(&create_bezier_curve(the_p1, the_p2, the_p3))
        .unwrap()
        .edge()
        .unwrap()
}

fn add_curve_constraint(the_builder: &mut GeomPlateBuildPlateSurface, the_edge: &Edge) {
    let a_curve: Handle<dyn Adaptor3dCurve> =
        Handle::new(BRepAdaptorCurve::with_edge(the_edge).unwrap()).cast();
    // GeomPlate_CurveConstraint(aCurveAdaptor, 0) with the native defaults NPt = 10,
    // TolDist = 0.0001, TolAng = 0.01 and TolCurv = 0.1.
    let constraint = GeomPlateCurveConstraint::new(&a_curve, 0, 10, 0.0001, 0.01, 0.1).unwrap();
    the_builder.add_curve(&Handle::new(constraint));
}

/// `aBuilder.Add(new GeomPlate_PointConstraint(P, 0))` (the native TolDist = 0.0001).
fn add_point_constraint(the_builder: &mut GeomPlateBuildPlateSurface, the_point: Point3) {
    let constraint = GeomPlatePointConstraint::with_point(the_point, 0, 0.0001).unwrap();
    the_builder.add_point(&Handle::new(constraint));
}

fn expect_plate_progress(the_messages: &[String]) {
    assert!(!the_messages.is_empty());
    assert!(has_progress_text(the_messages, "0%"));
    assert!(has_progress_text(the_messages, "100%"));
    assert!(has_progress_text(
        the_messages,
        "Calculating the surface filled"
    ));
    assert!(has_progress_text(the_messages, "Plate_Plate::SolveTI1()"));
    assert!(has_progress_text(the_messages, "math_Gauss LU_Decompose"));
}

/// `aBuilder.Perform(Message_ProgressIndicator::Start(aProgress))`, with the messages of
/// the observer.
fn perform_observed(the_builder: &mut GeomPlateBuildPlateSurface) -> Vec<String> {
    let a_messages = Arc::new(Mutex::new(Vec::new()));
    let a_progress = ProgressIndicator::new(Box::new(ProgressObserver(Arc::clone(&a_messages))));
    the_builder.perform(&a_progress.start()).unwrap();
    drop(a_progress);
    Arc::try_unwrap(a_messages).unwrap().into_inner().unwrap()
}

/// `GeomPlate_BuildPlateSurface(Degree, NbPtsOnCur, NbIter)` with the native defaults of
/// the other arguments.
fn builder(degree: i32, nb_pts_on_cur: i32, nb_iter: i32) -> GeomPlateBuildPlateSurface {
    GeomPlateBuildPlateSurface::new(
        degree,
        nb_pts_on_cur,
        nb_iter,
        0.00001,
        0.0001,
        0.01,
        0.1,
        false,
    )
    .unwrap()
}

fn p(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

#[test]
fn GeomPlate_BuildPlateSurface__OCC525_PerformWithoutConstraints() {
    // GeomPlate_BuildPlateSurface aBuilder; (the default constructor: 3, 10, 3)
    let mut a_builder = builder(3, 10, 3);
    a_builder.perform(&ProgressRange::default()).unwrap();

    // TODO: IsDone() returns true due to Plate_Plate::Init() setting OK=true,
    // which is semantically wrong. Fixing this (finding #30) caused regressions
    // in blend/filling DRAW tests. Needs investigation.
    assert!(a_builder.is_done());
    assert!(
        a_builder.surface().is_null(),
        "Surface should be null when Perform() is called without constraints"
    );
}

// Regression test for bug #19: Surface() must be null after failed recomputation.
// After Init() + empty Perform(), stale surface must not be observable.
#[test]
fn GeomPlate_BuildPlateSurface__Perform_ClearsStaleResult() {
    let mut a_builder =
        GeomPlateBuildPlateSurface::new(3, 10, 3, 1.0e-5, 1.0e-4, 0.01, 0.1, false).unwrap();

    // Add point constraints and solve.
    add_point_constraint(&mut a_builder, p(0.0, 0.0, 0.0));
    add_point_constraint(&mut a_builder, p(1.0, 0.0, 0.0));
    add_point_constraint(&mut a_builder, p(0.0, 1.0, 0.0));
    add_point_constraint(&mut a_builder, p(1.0, 1.0, 0.1));
    a_builder.perform(&ProgressRange::default()).unwrap();

    // Init clears constraints, then Perform with no constraints must
    // produce null surface (not stale result from the previous solve).
    a_builder.init();
    a_builder.perform(&ProgressRange::default()).unwrap();
    assert!(a_builder.is_done());
    assert!(
        a_builder.surface().is_null(),
        "Surface must be null after Init() + empty Perform()"
    );
}

// Migrated from tests/lowalgos/progress/A1.  Validate the progress scopes
// emitted by the same GeomPlate operation rather than testing DRAW output.
#[test]
fn GeomPlate_BuildPlateSurface__Progress_A1_FourBezierEdgesWithPlane() {
    let mut a_builder = builder(3, 15, 2);
    let a_plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(
        Axis3::from_normal(p(0., 0., 0.), Direction3::new(0., 0., 1.).unwrap()).unwrap(),
    ))
    .cast();
    a_builder.load_init_surface(&a_plane);

    let an_edge1 = create_bezier_edge(p(0., 0., 0.), p(1., 0., 1.), p(2., 0., 0.));
    let an_edge2 = create_bezier_edge(p(0., 2., 0.), p(1., 2., 1.), p(2., 2., 0.));
    let an_edge3 = create_bezier_edge(p(0., 0., 0.), p(0., 1., 1.), p(0., 2., 0.));
    let an_edge4 = create_bezier_edge(p(2., 0., 0.), p(2., 1., 1.), p(2., 2., 0.));
    add_curve_constraint(&mut a_builder, &an_edge1);
    add_curve_constraint(&mut a_builder, &an_edge2);
    add_curve_constraint(&mut a_builder, &an_edge3);
    add_curve_constraint(&mut a_builder, &an_edge4);

    let a_progress = perform_observed(&mut a_builder);

    assert!(a_builder.is_done());
    assert!(!a_builder.surface().is_null());
    expect_plate_progress(&a_progress);
}

// Migrated from tests/lowalgos/progress/A2.
#[test]
fn GeomPlate_BuildPlateSurface__Progress_A2_FourBezierEdgesWithoutPlane() {
    let mut a_builder = builder(3, 15, 2);
    add_curve_constraint(
        &mut a_builder,
        &create_bezier_edge(p(0., 0., 0.), p(1., 0., 1.), p(2., 0., 0.)),
    );
    add_curve_constraint(
        &mut a_builder,
        &create_bezier_edge(p(0., 2., 0.), p(1., 2., 1.), p(2., 2., 0.)),
    );
    add_curve_constraint(
        &mut a_builder,
        &create_bezier_edge(p(0., 0., 0.), p(0., 1., 2.), p(0., 2., 0.)),
    );
    add_curve_constraint(
        &mut a_builder,
        &create_bezier_edge(p(2., 0., 0.), p(2., 1., 2.), p(2., 2., 0.)),
    );

    let a_progress = perform_observed(&mut a_builder);

    assert!(a_builder.is_done());
    assert!(!a_builder.surface().is_null());
    expect_plate_progress(&a_progress);
}

// Migrated from tests/lowalgos/progress/A3.
#[test]
fn GeomPlate_BuildPlateSurface__Progress_A3_LargePointContour() {
    let mut a_builder = builder(3, 15, 2);
    for an_index in 0..300 {
        add_point_constraint(&mut a_builder, p(f64::from(an_index), 0., 0.));
    }
    for an_index in 1..300 {
        add_point_constraint(&mut a_builder, p(0., f64::from(an_index), 0.));
    }
    for an_index in 0..300 {
        add_point_constraint(&mut a_builder, p(300., f64::from(an_index), 0.));
    }
    for an_index in 0..=300 {
        add_point_constraint(&mut a_builder, p(f64::from(an_index), 300., 0.));
    }

    let a_progress = perform_observed(&mut a_builder);

    assert!(a_builder.is_done());
    assert!(!a_builder.surface().is_null());
    expect_plate_progress(&a_progress);
}

// Migrated from tests/lowalgos/progress/A4.
#[test]
fn GeomPlate_BuildPlateSurface__Progress_A4_OneHundredPointContour() {
    let mut a_builder = builder(3, 15, 2);
    for an_index in 0..100 {
        let a_parameter: f64 = 2. * PI * f64::from(an_index) / 100.;
        add_point_constraint(
            &mut a_builder,
            p(
                a_parameter.sin(),
                a_parameter.cos(),
                (1. - 2. * f64::from(an_index) / 100.).abs(),
            ),
        );
    }

    let a_progress = perform_observed(&mut a_builder);

    assert!(a_builder.is_done());
    assert!(!a_builder.surface().is_null());
    expect_plate_progress(&a_progress);
}
