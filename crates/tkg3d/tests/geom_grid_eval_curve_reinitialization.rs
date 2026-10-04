//! Supplemental protected-hook reinitialization checks, separate from legacy tests.
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dIsoCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomCircle, GeomCurve, GeomLine, GeomPlane, GeomSurface, GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkg3d::geom_grid_eval::{GeomGridEvalCurve, GeomGridEvalCurveEvaluator};
use rustycad_tkmath::geom_abs::{CurveType, IsoType};
use rustycad_tkmath::gp::{Axis2, Axis3, Direction3, Point3};

#[test]
fn curve_handle_reinitialization_strips_trim_and_null_clears_evaluator() {
    let line = line_curve();
    let circle = circle_curve();
    let mut eval = GeomGridEvalCurve::from_curve(&line);
    assert_initial_line(&eval);
    assert_circle_replacement(&mut eval, &circle);
    assert_trimmed_replacement(&mut eval, &circle);
    assert_null_replacement(&mut eval);
}

fn assert_initial_line(eval: &GeomGridEvalCurve<'_>) {
    assert_eq!(eval.get_type(), CurveType::Line);
    assert_point(grid_point(eval, 0.25), Point3::new(1.25, 2.0, 3.0));
}

fn assert_circle_replacement(eval: &mut GeomGridEvalCurve<'_>, circle: &Handle<dyn GeomCurve>) {
    eval.initialization_curve(circle);
    assert_eq!(eval.get_type(), CurveType::Circle);
    let expected = Point3::new(2.0 * 0.5f64.cos(), 2.0 * 0.5f64.sin(), 0.0);
    assert_point(grid_point(eval, 0.5), expected);
}

fn assert_trimmed_replacement(eval: &mut GeomGridEvalCurve<'_>, circle: &Handle<dyn GeomCurve>) {
    eval.initialization_curve(&trimmed_circle(circle));
    assert_eq!(eval.get_type(), CurveType::Circle);
    let expected = Point3::new(2.0 * 2.0f64.cos(), 2.0 * 2.0f64.sin(), 0.0);
    assert_point(grid_point(eval, 2.0), expected);
}

fn assert_null_replacement(eval: &mut GeomGridEvalCurve<'_>) {
    eval.initialization_curve(&Handle::default());
    assert_eq!(eval.get_type(), CurveType::OtherCurve);
    assert_eq!(eval.evaluate_grid(&params(&[0.5])).unwrap().len(), 0);
}

#[test]
fn concrete_geom_adaptor_initialization_ignores_its_parameter_range() {
    let line = line_curve();
    let curve: Handle<dyn GeomCurve> = line.cast();
    let adaptor = GeomAdaptorCurve::with_range(&curve, 4.0, 5.0).unwrap();
    let mut eval = GeomGridEvalCurve::from_curve(&curve);
    eval.initialization_adaptor(&adaptor).unwrap();
    assert_eq!(eval.get_type(), CurveType::Line);
    assert!((eval.evaluate_grid(&params(&[0.5])).unwrap()[1].x() - 1.5).abs() < 1e-12);
}

#[test]
fn fallback_borrows_adaptor_and_failed_get_type_preserves_prior_curve() {
    let line = line_curve();
    let mut eval = GeomGridEvalCurve::from_curve(&line);
    let surface = plane_adaptor();
    let iso = Adaptor3dIsoCurve::with_iso(&surface, IsoType::IsoU, 0.5).unwrap();
    eval.initialization_adaptor(&iso).unwrap();
    assert!(
        matches!(eval.evaluator(), GeomGridEvalCurveEvaluator::Other(other) if std::ptr::eq(other.curve(), &iso as &dyn Adaptor3dCurve))
    );
    assert!((eval.evaluate_grid(&params(&[0.25])).unwrap()[1].z()).abs() < 1e-12);
    let failing = FailingGetTypeAdaptor::default();
    assert_eq!(
        eval.initialization_adaptor(&failing)
            .unwrap_err()
            .exception_type(),
        "Standard_ConstructionError"
    );
    assert_eq!(eval.get_type(), CurveType::Line);
    assert_point(grid_point(&eval, 0.25), Point3::new(0.5, 0.25, 0.0));
}

fn line_curve() -> Handle<dyn GeomCurve> {
    let direction = Direction3::new(1.0, 0.0, 0.0).unwrap();
    Handle::new(GeomLine::from_point(Point3::new(1.0, 2.0, 3.0), direction)).cast()
}

fn circle_curve() -> Handle<dyn GeomCurve> {
    let axis = Axis2::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    Handle::new(GeomCircle::new(axis, 2.0).unwrap()).cast()
}

fn trimmed_circle(circle: &Handle<dyn GeomCurve>) -> Handle<dyn GeomCurve> {
    Handle::new(GeomTrimmedCurve::new(circle, 0.0, 1.0, true, true).unwrap()).cast()
}

fn plane_adaptor() -> Handle<dyn Adaptor3dSurface> {
    let frame = Axis3::from_normal(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(frame)).cast();
    Handle::new(GeomAdaptorSurface::with_surface(&plane).unwrap()).cast()
}

fn params(values: &[f64]) -> Array1<'static, f64> {
    let mut result = Array1::with_bounds(1, values.len() as i32).unwrap();
    for (index, value) in values.iter().enumerate() {
        result[index as i32 + 1] = *value;
    }
    result
}

fn grid_point(eval: &GeomGridEvalCurve<'_>, parameter: f64) -> Point3 {
    eval.evaluate_grid(&params(&[parameter])).unwrap()[1]
}

fn assert_point(actual: Point3, expected: Point3) {
    assert!(
        actual.distance(&expected) < 1e-12,
        "{actual:?} != {expected:?}"
    );
}

#[derive(Default)]
struct FailingGetTypeAdaptor {
    parent: StandardTransient,
}
impl_transient!(
    FailingGetTypeAdaptor,
    "FailingGetTypeAdaptor",
    dyn Adaptor3dCurve,
    parent,
    interfaces: [dyn Adaptor3dCurve]
);
impl Adaptor3dCurve for FailingGetTypeAdaptor {
    fn get_type(&self) -> Result<CurveType, Failure> {
        Err(Failure::with_message(
            "Standard_ConstructionError",
            "probe GetType failure",
        ))
    }
}
