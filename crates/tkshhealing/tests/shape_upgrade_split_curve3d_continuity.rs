//! Native oracle rows at tools/oracle/tkshhealing_split_curve3d_continuity.expected.
use rustycad_tkernel::{ncollection::Array1, standard::Handle};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomCurve, GeomLine, GeomOffsetCurve, GeomTrimmedCurve,
};
use rustycad_tkmath::{
    geom_abs::Shape,
    gp::{Axis1, Direction3, Point3},
};
use rustycad_tkshhealing::shape_upgrade::{
    ShapeUpgradeSplitCurve3d, ShapeUpgradeSplitCurve3dContinuity, SplitCurve,
};

fn polyline_point(corner: bool, index: usize) -> Point3 {
    if corner && index > 1 {
        return Point3::new(1., (index - 1) as f64, 0.);
    }
    Point3::new(index as f64, 0., 0.)
}
fn polyline(corner: bool, intervals: usize) -> Handle<dyn GeomCurve> {
    let points: Vec<Point3> = (0..=intervals).map(|i| polyline_point(corner, i)).collect();
    let knots: Vec<f64> = (0..=intervals).map(|i| i as f64).collect();
    let mults: Vec<i32> = (0..=intervals)
        .map(|i| if i == 0 || i == intervals { 2 } else { 1 })
        .collect();
    let points = Array1::from_shared(&points, 1);
    let knots = Array1::from_shared(&knots, 1);
    let mults = Array1::from_shared(&mults, 1);
    Handle::new(GeomBSplineCurve::new(&points, &knots, &mults, 1, false).unwrap()).cast()
}
fn oracle_row(
    label: &str,
    curve: &Handle<dyn GeomCurve>,
    criterion: Shape,
    tolerance: f64,
    split: Option<f64>,
) -> String {
    let fixer = ShapeUpgradeSplitCurve3dContinuity::new();
    fixer.init_curve_range(curve, 0., 2.).unwrap();
    fixer.set_criterion(criterion);
    fixer.set_tolerance(tolerance);
    let shared = fixer.split_values();
    if let Some(split) = split {
        shared.change_sequence(|s| {
            s.insert_before(2, split).unwrap();
        });
    }
    fixer.compute().unwrap();
    assert_eq!(
        shared.object_address(),
        fixer.split_values().object_address()
    );
    render_row(label, &fixer)
}
fn render_row(label: &str, fixer: &ShapeUpgradeSplitCurve3dContinuity) -> String {
    let shared = fixer.split_values();
    let spline = fixer.curve().cast::<GeomBSplineCurve>();
    let knots = spline.get().map_or(-1, GeomBSplineCurve::nb_knots);
    let status = fixer.split_curve_data().with(|f| f.status);
    assert_eq!(
        shared.object_address(),
        fixer.split_values().object_address()
    );
    let values = shared.sequence(|s| {
        s.iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ")
    });
    format!("{label} {status} 1 {knots} {values}\n")
}
fn assert_oracle(
    label: &str,
    curve: &Handle<dyn GeomCurve>,
    criterion: Shape,
    tolerance: f64,
    split: Option<f64>,
) {
    let actual = oracle_row(label, curve, criterion, tolerance, split);
    let expected =
        include_str!("../../../tools/oracle/tkshhealing_split_curve3d_continuity.expected");
    let expected = expected
        .lines()
        .find(|line| line.starts_with(&format!("{label} ")))
        .unwrap();
    assert_eq!(actual.trim_end(), expected);
}
#[test]
fn native_polyline_continuity_oracle_rows_match() {
    for (label, corner, criterion, tolerance) in simple_cases() {
        assert_oracle(label, &polyline(corner, 2), criterion, tolerance, None);
    }
    assert_oracle(
        "existing-split",
        &polyline(false, 2),
        Shape::C1,
        1e-7,
        Some(0.5),
    );
    assert_oracle(
        "consecutive-removal",
        &polyline(false, 4),
        Shape::CN,
        1e-7,
        None,
    );
}
#[test]
fn native_wrapper_continuity_oracle_rows_match() {
    let trimmed =
        Handle::new(GeomTrimmedCurve::new(&polyline(false, 2), 0., 2., true, true).unwrap()).cast();
    assert_oracle("trimmed", &trimmed, Shape::C1, 1e-7, None);
    let direction = Direction3::new(0., 0., 1.).unwrap();
    let offset =
        Handle::new(GeomOffsetCurve::new(&polyline(false, 2), 0.2, direction, true).unwrap())
            .cast();
    assert_oracle("offset", &offset, Shape::C0, 1e-7, None);
    let line = Handle::new(GeomLine::new(Axis1::default())).cast();
    assert_oracle("line", &line, Shape::CN, 1e-7, None);
}
fn simple_cases() -> [(&'static str, bool, Shape, f64); 8] {
    [
        ("corner-c0", true, Shape::C0, 1e-7),
        ("corner-c1", true, Shape::C1, 1e-7),
        ("straight-c1", false, Shape::C1, 1e-7),
        ("straight-cn", false, Shape::CN, 1e-7),
        ("corner-large", true, Shape::C1, 2.),
        ("corner-negative", true, Shape::C1, -1.),
        ("corner-g1", true, Shape::G1, 1e-7),
        ("corner-g2", true, Shape::G2, 1e-7),
    ]
}
#[test]
fn corrected_curve_is_private_and_parent_projection_shares_state() {
    let original = polyline(false, 2);
    let splitter = Handle::new(ShapeUpgradeSplitCurve3dContinuity::new());
    assert!(splitter.curve().is_null());
    assert!(splitter.curves().is_null());
    splitter.init_curve(&original).unwrap();
    let base = splitter.cast::<ShapeUpgradeSplitCurve3d>();
    let dispatched = splitter.cast::<dyn SplitCurve>();
    dispatched.compute().unwrap();
    assert_eq!(original.cast::<GeomBSplineCurve>().nb_knots(), 3);
    assert_eq!(splitter.curve().cast::<GeomBSplineCurve>().nb_knots(), 2);
    assert_eq!(
        splitter.split_values().object_address(),
        base.split_values().object_address()
    );
    dispatched.build(true).unwrap();
    assert_eq!(splitter.curves().array1().len(), 1);
    assert!(!splitter.curves().array1().value(1).unwrap().is_null());
}
