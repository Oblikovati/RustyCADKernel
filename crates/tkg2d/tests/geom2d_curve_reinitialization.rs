//! Supplemental protected initialization transitions, compared with pinned OCCT.
use rustycad_tkernel::{ncollection::Array1, standard::Handle};
use rustycad_tkg2d::{
    adaptor2d::{Adaptor2dCurve2dBase, Adaptor2dLine2d},
    geom2d::{Geom2dCircle, Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve},
    geom2d_adaptor::Geom2dAdaptorCurve,
    geom2d_grid_eval::Geom2dGridEvalCurve,
};
use rustycad_tkmath::gp::{Axis22d, Direction2, Point2};
use std::fmt::Write;

fn append_state(output: &mut String, evaluator: &Geom2dGridEvalCurve<'_>) {
    let mut parameters: Array1<'_, f64> = Array1::with_bounds(-1, 0).unwrap();
    parameters[-1] = 0.0;
    parameters[0] = 1.0;
    let points = evaluator.evaluate_grid(&parameters).unwrap();
    write!(
        output,
        "{} {} {}",
        evaluator.get_type() as i32,
        points.lower(),
        points.upper()
    )
    .unwrap();
    for point in points.iter() {
        write!(output, " {} {}", point.x(), point.y()).unwrap();
    }
    output.push('\n');
}

fn line_and_circle() -> (Handle<dyn Geom2dCurve>, Handle<dyn Geom2dCurve>) {
    let direction = Direction2::new(1.0, 0.0).unwrap();
    let line = Handle::new(Geom2dLine::from_point(Point2::new(3.0, 4.0), direction));
    let axis = Axis22d::new(Point2::new(0.0, 0.0), direction, true).unwrap();
    let circle = Handle::new(Geom2dCircle::new(axis, 2.0).unwrap());
    (line.cast(), circle.cast())
}

#[test]
fn curve_and_adaptor_reinitialization_matches_native_transitions() {
    let (line, circle) = line_and_circle();
    let adaptor = Geom2dAdaptorCurve::with_range(&line, 4.0, 5.0).unwrap();
    let throwing = Adaptor2dCurve2dBase::default();
    let fallback = Adaptor2dLine2d::new(
        Point2::new(6.0, 7.0),
        Direction2::new(0.0, 1.0).unwrap(),
        4.0,
        5.0,
    );
    let mut evaluator = Geom2dGridEvalCurve::from_curve(&line);
    let mut output = String::new();
    check_curve_changes(&mut evaluator, &line, &circle, &mut output);
    check_clear_restore_failure(&mut evaluator, &adaptor, &throwing, &mut output);
    evaluator.initialization_adaptor(&fallback).unwrap();
    append_state(&mut output, &evaluator);
    assert_eq!(
        output,
        include_str!("../../../tools/oracle/tkg2d_curve_reinitialization.expected")
    );
}

fn check_curve_changes(
    evaluator: &mut Geom2dGridEvalCurve<'_>,
    line: &Handle<dyn Geom2dCurve>,
    circle: &Handle<dyn Geom2dCurve>,
    output: &mut String,
) {
    append_state(output, evaluator);
    evaluator.initialization_curve(circle);
    let origin: Array1<'_, f64> = Array1::with_bounds(1, 1).unwrap();
    let point = evaluator.evaluate_grid(&origin).unwrap()[1];
    writeln!(
        output,
        "{} {} {}",
        evaluator.get_type() as i32,
        point.x(),
        point.y()
    )
    .unwrap();
    let trimmed = Handle::new(Geom2dTrimmedCurve::new(line, 4.0, 5.0, true, true).unwrap());
    evaluator.initialization_curve(&trimmed.cast());
    append_state(output, evaluator);
}

fn check_clear_restore_failure<'a>(
    evaluator: &mut Geom2dGridEvalCurve<'a>,
    adaptor: &'a Geom2dAdaptorCurve,
    throwing: &'a Adaptor2dCurve2dBase,
    output: &mut String,
) {
    evaluator.initialization_curve(&Handle::default());
    append_state(output, evaluator);
    evaluator.initialization_adaptor(adaptor).unwrap();
    append_state(output, evaluator);
    let failure = evaluator.initialization_adaptor(throwing).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_NotImplemented");
    output.push_str("GetType raised\n");
    append_state(output, evaluator);
}

#[test]
fn clearing_curve_clears_every_derivative_grid() {
    let (line, _) = line_and_circle();
    let mut evaluator = Geom2dGridEvalCurve::from_curve(&line);
    evaluator.initialization_curve(&Handle::default());
    let parameters: Array1<'_, f64> = Array1::with_bounds(-2, 2).unwrap();
    assert!(evaluator.evaluate_grid(&parameters).unwrap().is_empty());
    assert!(evaluator.evaluate_grid_d1(&parameters).unwrap().is_empty());
    assert!(evaluator.evaluate_grid_d2(&parameters).unwrap().is_empty());
    assert!(evaluator.evaluate_grid_d3(&parameters).unwrap().is_empty());
    assert!(
        evaluator
            .evaluate_grid_dn(&parameters, 2)
            .unwrap()
            .is_empty()
    );
}
