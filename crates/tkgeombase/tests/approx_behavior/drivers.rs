//! The multi-line drivers: options, accessors, errors and the native quirks.
use super::fakes::{CubicFunction, LineScript, ParabolaLine};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::app_par_curves::{
    Constraint, LineTool, MultiCurve, MultiCurveEval, MultiPoint,
};
use rustycad_tkgeombase::approx::{
    BSplComputeLine, ComputeCLine, ComputeCLineOptions, ComputeLine, ComputeLineOptions,
    FitAndDivide, MCurvesToBSpCurve, ParametrizationType, Status,
};
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::math::Vector;

#[test]
fn enumerations_keep_the_native_values() {
    assert_eq!(Status::PointsAdded.code(), 0);
    assert_eq!(Status::from_code(2), Some(Status::NoApproximation));
    assert_eq!(Status::from_code(3), None);
    assert_eq!(ParametrizationType::IsoParametric.code(), 2);
    assert_eq!(ParametrizationType::from_code(-1), None);
    assert_eq!(
        ParametrizationType::default(),
        ParametrizationType::ChordLength
    );
}

#[test]
fn compute_c_line_defaults_are_the_native_ones() {
    let o = ComputeCLineOptions::default();
    assert_eq!((o.degree_min, o.degree_max, o.cutting), (3, 8, false));
    assert_eq!((o.tolerance_3d, o.tolerance_2d), (1.0e-5, 1.0e-5));
    assert_eq!(o.first_constraint, Constraint::TangencyPoint);
    let unperformed = FitAndDivide::new(o);
    assert!(!unperformed.is_all_approximated());
    assert_eq!(unperformed.nb_multi_curves(), 0);
}

#[test]
fn compute_c_line_fits_a_cubic_in_one_piece() -> Result<(), Failure> {
    let fit = ComputeCLine::with_line(
        &CubicFunction::new(0.0, 1.0),
        ComputeCLineOptions::default(),
    )?;
    assert!(fit.is_all_approximated() && fit.is_tolerance_reached());
    assert_eq!(fit.nb_multi_curves(), 1);
    assert_eq!(fit.parameters(1)?, (0.0, 1.0));
    // The inverse order walks down from degreemax and keeps the last degree that
    // fits: degree 3 misses the tolerance with both tangency ends, so degree 4 stays.
    assert_eq!(fit.value(1)?.nb_poles(), 5);
    Ok(())
}

#[test]
fn compute_c_line_out_of_range_pieces_raise() {
    let fit = ComputeCLine::new(ComputeCLineOptions::default());
    for failure in [
        fit.value(1).err(),
        fit.error(0).err(),
        fit.parameters(2).err(),
    ] {
        assert_eq!(failure.unwrap().exception_type(), "Standard_OutOfRange");
    }
}

#[test]
fn compute_c_line_perform_appends_to_the_previous_pieces() -> Result<(), Failure> {
    // Quirk: Perform never clears the sequences, so a second run adds its pieces.
    let line = CubicFunction::new(0.0, 1.0);
    let mut fit = ComputeCLine::new(ComputeCLineOptions::default());
    fit.perform(&line)?;
    fit.perform(&line)?;
    assert_eq!(fit.nb_multi_curves(), 2);
    Ok(())
}

#[test]
fn compute_c_line_stores_the_best_uncut_fit_when_the_tolerance_fails() -> Result<(), Failure> {
    let options = ComputeCLineOptions {
        degree_min: 1,
        degree_max: 2,
        tolerance_3d: 1.0e-12,
        ..ComputeCLineOptions::default()
    };
    let fit = ComputeCLine::with_line(&CubicFunction::new(0.0, 1.0), options)?;
    assert!(!fit.is_all_approximated() && !fit.is_tolerance_reached());
    assert_eq!(fit.nb_multi_curves(), 1);
    assert!(fit.error(1)?.0 > 1.0e-12);
    Ok(())
}

#[test]
fn compute_c_line_cutting_stops_at_the_segment_cap() -> Result<(), Failure> {
    let mut fit = ComputeCLine::new(ComputeCLineOptions {
        degree_min: 1,
        degree_max: 1,
        tolerance_3d: 1.0e-9,
        cutting: true,
        ..ComputeCLineOptions::default()
    });
    fit.set_max_segments(3);
    fit.perform(&CubicFunction::new(0.0, 1.0))?;
    assert!(fit.is_all_approximated());
    assert!(fit.nb_multi_curves() <= 4);
    Ok(())
}

fn segment_curve(from: Point3, to: Point3, degree: i32) -> Result<MultiCurve, Failure> {
    let curve = MultiCurve::with_nb_poles(degree + 1)?;
    for i in 1..=degree + 1 {
        let t = f64::from(i - 1) / f64::from(degree);
        let pole = MultiPoint::with_counts(1, 0)?;
        pole.set_point(
            1,
            Point3::new(
                from.x() + t * (to.x() - from.x()),
                from.y() + t * (to.y() - from.y()),
                0.0,
            ),
        )?;
        curve.set_value(i, &pole)?;
    }
    Ok(curve)
}

#[test]
fn mcurves_join_raises_on_an_empty_sequence() {
    let failure = MCurvesToBSpCurve::new().perform().err().unwrap();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
}

#[test]
fn mcurves_join_raises_curves_to_the_common_degree() -> Result<(), Failure> {
    let mut join = MCurvesToBSpCurve::new();
    join.append(&segment_curve(
        Point3::new(0., 0., 0.),
        Point3::new(1., 1., 0.),
        1,
    )?);
    join.append(&segment_curve(
        Point3::new(1., 1., 0.),
        Point3::new(3., 0., 0.),
        3,
    )?);
    join.perform()?;
    assert_eq!(join.value().degree()?, 3);
    assert_eq!(join.value().knots()?.length(), 3);
    join.reset();
    join.append(&segment_curve(
        Point3::new(0., 0., 0.),
        Point3::new(1., 0., 0.),
        2,
    )?);
    join.perform()?;
    // One curve keeps its poles over the knots [0, 1].
    assert_eq!(join.change_value().nb_poles(), 3);
    Ok(())
}

#[test]
fn compute_line_defaults_and_empty_accessors() {
    let o = ComputeLineOptions::default();
    assert_eq!((o.degree_min, o.degree_max, o.nb_iterations), (4, 8, 5));
    assert!(o.cutting && !o.squares);
    let compute = ComputeLine::new(o);
    // tolreached is uninitialized natively; the port starts it false.
    assert!(!compute.is_tolerance_reached());
    assert_eq!(
        compute.value(1).err().unwrap().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(
        compute.parameters(1).err().unwrap().exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn compute_line_fits_a_parabola_line() -> Result<(), Failure> {
    let line = ParabolaLine::uniform(12, LineScript::AppDef);
    let mut compute = ComputeLine::with_line(&line, ComputeLineOptions::default())?;
    assert!(compute.is_all_approximated() && compute.is_tolerance_reached());
    assert!(compute.nb_multi_curves() >= 1);
    let params = compute.parameters(1)?;
    assert_eq!((params.lower(), params[params.lower()]), (1, 0.0));
    assert!(compute.spline_value()?.nb_poles() >= 3);
    compute.change_value(1)?;
    Ok(())
}

#[test]
fn compute_line_given_parameters_start_the_first_piece() -> Result<(), Failure> {
    let line = ParabolaLine::uniform(9, LineScript::AppDef);
    let mut u = Vector::with_bounds(1, 9)?;
    (1..=9).for_each(|i| u[i] = f64::from(i - 1) / 8.0);
    let compute = ComputeLine::with_line_and_parameters(&line, &u, ComputeLineOptions::default())?;
    assert_eq!(
        compute.parametrization(),
        ParametrizationType::IsoParametric
    );
    assert!(compute.nb_multi_curves() >= 1);
    Ok(())
}

#[test]
fn compute_line_refuses_lines_marked_no_approximation() -> Result<(), Failure> {
    let line = ParabolaLine::uniform(6, LineScript::Refuse);
    let compute = ComputeLine::with_line(&line, ComputeLineOptions::default())?;
    assert!(compute.is_all_approximated());
    assert_eq!(compute.nb_multi_curves(), 0);
    Ok(())
}

#[test]
fn compute_line_recurses_on_refined_lines() -> Result<(), Failure> {
    let line = ParabolaLine::uniform(5, LineScript::Refine { cap: 30 });
    let compute = ComputeLine::with_line(&line, ComputeLineOptions::default())?;
    assert!(compute.is_all_approximated());
    // A range of at most degreemax + 5 points asks WhatStatus and refines through
    // MakeMLBetween: 5, then 9, then 17 points, which is past the short-range bound.
    assert_eq!(compute.parameters(1)?.length(), 17);
    Ok(())
}

#[test]
fn compute_line_squares_skips_the_parameter_optimization() -> Result<(), Failure> {
    let line = ParabolaLine::uniform(12, LineScript::AppDef);
    let options = ComputeLineOptions {
        squares: true,
        ..ComputeLineOptions::default()
    };
    let compute = ComputeLine::with_line(&line, options)?;
    assert!(compute.is_all_approximated() && compute.is_tolerance_reached());
    // The chord-length parameters stay normalized to [0, 1].
    let params = compute.parameters(1)?;
    assert_eq!((params[params.lower()], params[params.upper()]), (0.0, 1.0));
    Ok(())
}

#[test]
fn bspl_compute_line_parameters_before_perform_raise() {
    let compute = BSplComputeLine::new(ComputeLineOptions::default());
    assert_eq!(
        compute.parameters().err().unwrap().exception_type(),
        "Standard_NullObject"
    );
    assert_eq!(compute.error(), (f64::MAX, f64::MAX));
}

#[test]
fn bspl_compute_line_interpolates_two_points_as_a_segment() -> Result<(), Failure> {
    let line = ParabolaLine::uniform(2, LineScript::AppDef);
    let mut compute = BSplComputeLine::new(ComputeLineOptions::default());
    compute.interpol(&line)?;
    let spline = compute.value();
    assert_eq!((spline.degree()?, spline.nb_poles()), (1, 2));
    assert_eq!(compute.parameters()?.length(), 2);
    Ok(())
}

#[test]
fn bspl_compute_line_curvature_request_keeps_tangent_ends() -> Result<(), Failure> {
    // Quirk: FindRealConstraints asks the tangents twice for a curvature end, so a
    // line with tangents and no curvatures still gets curvature constraints.
    let mut line = ParabolaLine::uniform(15, LineScript::AppDef);
    line.tangents = true;
    let mut compute = BSplComputeLine::new(ComputeLineOptions {
        degree_min: 3,
        degree_max: 6,
        ..ComputeLineOptions::default()
    });
    compute.set_constraints(Constraint::CurvaturePoint, Constraint::CurvaturePoint);
    compute.perform(&line)?;
    assert!(compute.value().nb_poles() >= 4);
    Ok(())
}

#[test]
fn bspl_compute_line_uses_the_given_knots() -> Result<(), Failure> {
    let line = ParabolaLine::uniform(12, LineScript::AppDef);
    let mut compute = BSplComputeLine::new(ComputeLineOptions {
        degree_min: 3,
        degree_max: 3,
        cutting: false,
        ..ComputeLineOptions::default()
    });
    let mut knots = rustycad_tkernel::ncollection::Array1::with_bounds(1, 3).unwrap();
    (knots[1], knots[2], knots[3]) = (0.0, 0.5, 1.0);
    let mut mults = rustycad_tkernel::ncollection::Array1::with_bounds(1, 3).unwrap();
    (mults[1], mults[2], mults[3]) = (4, 1, 4);
    compute.set_knots_and_multiplicities(&knots, &mults);
    compute.set_constraints(Constraint::PassPoint, Constraint::PassPoint);
    compute.perform(&line)?;
    assert_eq!(compute.value().knots()?.length(), 3);
    assert!(compute.is_tolerance_reached());
    Ok(())
}

#[test]
fn line_tool_answers_of_the_appdef_semantics() -> Result<(), Failure> {
    let line = ParabolaLine::uniform(4, LineScript::AppDef);
    assert_eq!(line.what_status(1, 4)?, Status::NoPointsAdded);
    assert_eq!(line.make_ml_between(1, 4, 3)?.params.len(), 4);
    assert!(line.make_ml_one_more_point(1, 4, 2)?.is_none());
    Ok(())
}
