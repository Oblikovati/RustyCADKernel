//! Regression tests of the native behaviour the CPnts and GCPnts ports preserve
//! (docs/tkgeombase-progress.md); the outcomes were read from the pinned native build
//! (tools/oracle/tkgeombase_probe_gcpnts.cxx).
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::EXPECT_THROW;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCircle, Geom2dLine, Geom2dOffsetCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeombase::cpnts::{self, MyGaussFunction, PntsCurve};
use rustycad_tkgeombase::gcpnts::{
    DistFunction2d, QuasiUniformAbscissa, TDistFunctionMV, TangentialCriteria,
    TangentialDeflection, UniformAbscissa, UniformDeflection,
};
use rustycad_tkmath::gp::{Axis22d, Direction2, Point2, Point3};
use rustycad_tkmath::math::{Function, MultipleVarFunction, Vector};

// The X axis on [first, last].
fn x_axis(first: f64, last: f64) -> Geom2dAdaptorCurve {
    let line = Handle::new(Geom2dLine::from_point(
        Point2::new(0.0, 0.0),
        Direction2::new(1.0, 0.0).unwrap(),
    ));
    Geom2dAdaptorCurve::with_range(&line.cast(), first, last).unwrap()
}

fn unit_circle() -> Geom2dAdaptorCurve {
    let circle = Handle::new(Geom2dCircle::new(Axis22d::default(), 1.0).unwrap());
    Geom2dAdaptorCurve::with_curve(&circle.cast()).unwrap()
}

// The offset of a Bezier curve whose first derivative vanishes at 0.
fn offset_cusp() -> Geom2dAdaptorCurve {
    let mut poles: Array1<'static, Point2> = Array1::with_bounds(1, 4).unwrap();
    poles[1] = Point2::new(0.0, 0.0);
    poles[2] = Point2::new(0.0, 0.0);
    poles[3] = Point2::new(2.0, 1.0);
    poles[4] = Point2::new(3.0, 0.5);
    let bezier = Handle::new(Geom2dBezierCurve::new(&poles).unwrap());
    let offset = Handle::new(Geom2dOffsetCurve::new(&bezier.cast(), 0.3, false).unwrap());
    Geom2dAdaptorCurve::with_curve(&offset.cast()).unwrap()
}

#[test]
fn pnts_curve_lifts_2d_results_to_the_plane_z_0() {
    let circle = unit_circle();
    let curve: PntsCurve = PntsCurve::from(&circle as &dyn Adaptor2dCurve2d);
    let (point, d1, d2) = curve.lifted_d2(0.0).unwrap();
    assert_eq!(point, Point3::new(1.0, 0.0, 0.0));
    assert_eq!((d1.z(), d2.z()), (0.0, 0.0));
    assert_eq!(curve.speed(0.3).unwrap(), 1.0);
    assert_eq!(curve.gauss_order().unwrap(), 10);
}

#[test]
fn an_empty_gauss_function_fails_with_a_null_object() {
    let mut function = MyGaussFunction::new();
    let mut value: f64 = 0.0;
    assert!(!function.value(0.5, &mut value));
    EXPECT_THROW(function.take_failure(), "Standard_NullObject");
}

#[test]
fn a_curve_shorter_than_confusion_gives_u0_done() {
    let point = x_axis(1.0, 1.0);
    let mut computer = cpnts::AbscissaPoint::new();
    EXPECT_THROW(computer.parameter(), "StdFail_NotDone");
    computer.init(&point).unwrap();
    computer.perform(3.0, 0.25, 1e-9).unwrap();
    assert_eq!(computer.parameter().unwrap(), 0.25);
}

#[test]
fn an_empty_uniform_deflection_iterator_has_no_point() {
    let mut iterator = cpnts::UniformDeflection::new();
    assert!(!iterator.more().unwrap());
    EXPECT_THROW(iterator.value(), "StdFail_NotDone");
    EXPECT_THROW(iterator.next(), "Standard_OutOfRange");
}

#[test]
fn adaptor_failures_in_the_root_function_propagate() {
    let offset = offset_cusp();
    // The Gauss nodes avoid the cusp, so the length is computed...
    assert!(cpnts::AbscissaPoint::length(&offset).is_ok());
    // ...but the root search starts on it, where the offset derivative is undefined.
    EXPECT_THROW(
        cpnts::AbscissaPoint::with_start(&offset, 0.1, 0.5, 0.0, 1e-9),
        "Geom2d_UndefinedDerivative",
    );
}

#[test]
fn uniform_abscissa_reuses_its_parameter_array() {
    let line = x_axis(0.0, 10.0);
    let mut algo = UniformAbscissa::with_abscissa(&line, 1.0, -1.0).unwrap();
    assert_eq!(algo.nb_points().unwrap(), 11);
    algo.initialize_with_abscissa(&line, 4.0, -1.0).unwrap();
    assert_eq!(algo.nb_points().unwrap(), 4);
    assert_eq!(algo.parameter(4).unwrap(), 10.0);
    // Index 5 still holds the fifth parameter of the first run.
    assert_eq!(algo.parameter(5).unwrap(), 4.0);
}

#[test]
fn quasi_uniform_abscissa_raises_when_the_uniform_abscissa_fails() {
    let point = x_axis(1.0, 1.0);
    EXPECT_THROW(
        QuasiUniformAbscissa::with_nb_points(&point, 5),
        "StdFail_NotDone",
    );
}

#[test]
fn a_null_deflection_on_a_circle_is_done_without_points() {
    let circle = unit_circle();
    let algo = UniformDeflection::with_curve(&circle, 0.0, true).unwrap();
    assert!(algo.is_done());
    assert_eq!(algo.nb_points().unwrap(), 0);
}

#[test]
fn tangential_deflection_refuses_a_zero_deflection_and_keeps_its_points() {
    let line = x_axis(0.0, 10.0);
    let mut algo =
        TangentialDeflection::with_curve(&line, TangentialCriteria::new(0.5, 0.01)).unwrap();
    assert_eq!(algo.nb_points(), 2);
    let zero = TangentialCriteria::new(0.5, 0.0);
    EXPECT_THROW(algo.initialize(&line, zero), "Standard_ConstructionError");
    assert_eq!(algo.nb_points(), 2);
    EXPECT_THROW(
        TangentialDeflection::arc_angular_step(-1.0, 0.1, 0.1, 0.0),
        "Standard_ConstructionError",
    );
}

#[test]
fn add_point_inserts_in_order_and_replaces_within_p_confusion() {
    let line = x_axis(0.0, 10.0);
    let mut algo =
        TangentialDeflection::with_curve(&line, TangentialCriteria::new(0.5, 0.01)).unwrap();
    let point = Point3::new(1.0, 2.0, 3.0);
    assert_eq!(algo.add_point(point, 5.0, true).unwrap(), 2);
    assert_eq!(
        algo.add_point(Point3::new(0.0, 0.0, 0.0), 5.0 + 1e-10, false)
            .unwrap(),
        2
    );
    assert_eq!(algo.value(2).unwrap(), point);
    assert_eq!(algo.add_point(point, 20.0, true).unwrap(), 4);
    assert_eq!(algo.parameter(4).unwrap(), 20.0);
}

#[test]
fn dist_function_copies_and_rejects_parameters_outside_its_range() {
    let circle = unit_circle();
    let curve: &dyn Adaptor2dCurve2d = &circle;
    let mut function = DistFunction2d::new(curve, 0.0, 1.0).unwrap();
    let (mut value, mut copied) = (0.0, 0.0);
    assert!(!function.value(1.5, &mut value));
    assert!(function.value(0.5, &mut value));
    assert!(function.clone().value(0.5, &mut copied));
    assert_eq!(value.to_bits(), copied.to_bits());
    let mut view = TDistFunctionMV::new(&mut function);
    assert_eq!(view.variables(), 1);
    let mut x: Vector<'static> = Vector::with_bounds(1, 1).unwrap();
    x[1] = 0.5;
    assert!(view.value(&x, &mut copied));
    assert_eq!(value.to_bits(), copied.to_bits());
}
