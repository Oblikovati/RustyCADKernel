//! Native oracle rows at tools/oracle/tkshhealing_split_curve2d_continuity.expected.
use rustycad_tkernel::{ncollection::Array1, standard::Handle};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dCurve, Geom2dLine, Geom2dOffsetCurve, Geom2dTrimmedCurve,
};
use rustycad_tkmath::{
    geom_abs::Shape,
    gp::{Axis2d, Point2},
};
use rustycad_tkshhealing::shape_upgrade::{
    ShapeUpgradeSplitCurve2d, ShapeUpgradeSplitCurve2dContinuity, SplitCurve,
};

fn polyline_point(corner: bool, index: usize) -> Point2 {
    if corner && index > 1 {
        return Point2::new(1.0, (index - 1) as f64);
    }
    Point2::new(index as f64, 0.0)
}

fn polyline(corner: bool, intervals: usize) -> Handle<dyn Geom2dCurve> {
    let points: Vec<Point2> = (0..=intervals).map(|i| polyline_point(corner, i)).collect();
    let knots: Vec<f64> = (0..=intervals).map(|i| i as f64).collect();
    let mults: Vec<i32> = (0..=intervals)
        .map(|i| if i == 0 || i == intervals { 2 } else { 1 })
        .collect();
    let (points, knots, mults) = (
        Array1::from_shared(&points, 1),
        Array1::from_shared(&knots, 1),
        Array1::from_shared(&mults, 1),
    );
    Handle::new(Geom2dBSplineCurve::new(&points, &knots, &mults, 1, false).unwrap()).cast()
}

fn near_last_knot() -> Handle<dyn Geom2dCurve> {
    let points = [
        Point2::new(0., 0.),
        Point2::new(1., 0.),
        Point2::new(2., 0.),
    ];
    let knots = [0., 1. - rustycad_tkernel::precision::p_confusion(), 2.];
    let mults = [2, 1, 2];
    let points = Array1::from_shared(&points, 1);
    let knots = Array1::from_shared(&knots, 1);
    let mults = Array1::from_shared(&mults, 1);
    Handle::new(Geom2dBSplineCurve::new(&points, &knots, &mults, 1, false).unwrap()).cast()
}

fn oracle_row(
    label: &str,
    curve: &Handle<dyn Geom2dCurve>,
    criterion: Shape,
    tolerance: f64,
    range: (f64, f64),
    split: Option<f64>,
) -> String {
    let fixer = ShapeUpgradeSplitCurve2dContinuity::new();
    fixer.init_curve_range(curve, range.0, range.1).unwrap();
    fixer.set_criterion(criterion);
    fixer.set_tolerance(tolerance);
    set_existing_split(&fixer, split);
    fixer.compute().unwrap();
    render_row(label, &fixer)
}

fn set_existing_split(fixer: &ShapeUpgradeSplitCurve2dContinuity, split: Option<f64>) {
    if let Some(split) = split {
        fixer.split_values().change_sequence(|values| {
            values.insert_before(2, split).unwrap();
        });
    }
}

fn render_row(label: &str, fixer: &ShapeUpgradeSplitCurve2dContinuity) -> String {
    let shared = fixer.split_values();
    let spline = fixer.curve().cast::<Geom2dBSplineCurve>();
    let knots = spline.get().map_or(-1, Geom2dBSplineCurve::nb_knots);
    let status = fixer.split_curve_data().with(|fields| fields.status);
    let curves = fixer.split_curve_data().with(|fields| fields.nb_curves);
    let values = shared.sequence(|sequence| {
        sequence
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ")
    });
    format!("{label} {status} 1 {knots} {values} {curves}\n")
}

fn assert_oracle(
    label: &str,
    curve: &Handle<dyn Geom2dCurve>,
    criterion: Shape,
    tolerance: f64,
    range: (f64, f64),
    split: Option<f64>,
) {
    let actual = oracle_row(label, curve, criterion, tolerance, range, split);
    let expected =
        include_str!("../../../tools/oracle/tkshhealing_split_curve2d_continuity.expected");
    let expected = expected
        .lines()
        .find(|line| line.starts_with(&format!("{label} ")))
        .unwrap();
    assert_eq!(actual.trim_end(), expected);
}

#[test]
fn native_spline_continuity_rows_match() {
    assert_oracle(
        "corner-c1",
        &polyline(true, 2),
        Shape::C1,
        1e-7,
        (0., 2.),
        None,
    );
    assert_oracle(
        "corner-g1",
        &polyline(true, 2),
        Shape::G1,
        1e-7,
        (0., 2.),
        None,
    );
    assert_oracle(
        "corner-g2",
        &polyline(true, 2),
        Shape::G2,
        1e-7,
        (0., 2.),
        None,
    );
    assert_oracle(
        "straight-c1",
        &polyline(false, 2),
        Shape::C1,
        1e-7,
        (0., 2.),
        None,
    );
    assert_oracle(
        "straight-cn",
        &polyline(false, 3),
        Shape::CN,
        1e-7,
        (0., 3.),
        None,
    );
    assert_oracle(
        "existing-split",
        &polyline(false, 3),
        Shape::C1,
        1e-7,
        (0., 3.),
        Some(1.5),
    );
    assert_oracle(
        "near-last-boundary",
        &near_last_knot(),
        Shape::C1,
        1e-7,
        (0., 1.),
        None,
    );
}

#[test]
fn native_wrapper_and_non_spline_rows_match() {
    let basis = polyline(false, 2);
    let trimmed = Handle::new(Geom2dTrimmedCurve::new(&basis, 0., 2., true, true).unwrap()).cast();
    assert_oracle("trimmed", &trimmed, Shape::C1, 1e-7, (0., 2.), None);
    let offset = Handle::new(Geom2dOffsetCurve::new(&basis, 0.2, true).unwrap()).cast();
    assert_oracle("offset", &offset, Shape::C0, 1e-7, (0., 2.), None);
    let line = Handle::new(Geom2dLine::new(Axis2d::default())).cast();
    assert_oracle("line", &line, Shape::CN, 1e-7, (0., 2.), None);
}

#[test]
fn continuity_dispatch_uses_the_private_curve_and_shared_base_state() {
    let original = polyline(false, 2);
    let splitter = Handle::new(ShapeUpgradeSplitCurve2dContinuity::new());
    assert!(splitter.curve().is_null());
    assert!(splitter.curves().is_null());
    splitter.init_curve(&original).unwrap();
    let base = splitter.cast::<ShapeUpgradeSplitCurve2d>();
    let dispatched = splitter.cast::<dyn SplitCurve>();
    dispatched.compute().unwrap();
    assert_eq!(original.cast::<Geom2dBSplineCurve>().nb_knots(), 3);
    assert_eq!(splitter.curve().cast::<Geom2dBSplineCurve>().nb_knots(), 2);
    assert_eq!(
        splitter.split_values().object_address(),
        base.split_values().object_address()
    );
    dispatched.build(true).unwrap();
    assert_eq!(splitter.curves().array1().len(), 1);
    assert!(!splitter.curves().array1().value(1).unwrap().is_null());
}
