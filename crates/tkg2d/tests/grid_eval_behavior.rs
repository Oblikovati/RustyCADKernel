// Copyright (c) 2025 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! Behaviour and preserved quirks of the Geom2dGridEval port that the legacy suites do
//! not reach; tools/tkg2d_grid_corpus.py compares the same paths with OCCT bit for bit.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::adaptor2d::{Adaptor2dCurve2dBase, Adaptor2dLine2d};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dLine,
    Geom2dOffsetCurve, Geom2dParabola, Geom2dTrimmedCurve,
};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg2d::geom2d_eval::{Geom2dEvalSineWaveCurve, RepCurveDescFull};
use rustycad_tkg2d::geom2d_grid_eval::{
    Geom2dGridEvalBSplineCurve, Geom2dGridEvalBezierCurve, Geom2dGridEvalCurve,
    Geom2dGridEvalEvaluator, Geom2dGridEvalLine, Geom2dGridEvalOffsetCurve,
    Geom2dGridEvalOtherCurve, Geom2dGridEvalParabola,
};
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2, Vector2};

fn params(lower: i32, values: &[f64]) -> Array1<'static, f64> {
    let mut array: Array1<'static, f64> =
        Array1::with_bounds(lower, lower + values.len() as i32 - 1).unwrap();
    array.as_mut_slice().copy_from_slice(values);
    array
}

fn x_axis() -> Axis2d {
    Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0).unwrap())
}

fn points(values: &[(f64, f64)]) -> Array1<'static, Point2> {
    let mut array: Array1<'static, Point2> = Array1::with_bounds(1, values.len() as i32).unwrap();
    for (slot, &(x, y)) in array.iter_mut().zip(values) {
        *slot = Point2::new(x, y);
    }
    array
}

fn reals<T: Copy + Default>(values: &[T]) -> Array1<'static, T> {
    let mut array: Array1<'static, T> = Array1::with_bounds(1, values.len() as i32).unwrap();
    array.as_mut_slice().copy_from_slice(values);
    array
}

// Degree 1 through (i, i^2) on knots 0, 0.3, 0.5, 0.6, 1 (end multiplicities 2).
fn polyline() -> Handle<Geom2dBSplineCurve> {
    let poles = points(&[(1.0, 1.0), (2.0, 4.0), (3.0, 9.0), (4.0, 16.0), (5.0, 25.0)]);
    let knots = reals(&[0.0, 0.3, 0.5, 0.6, 1.0]);
    let mults = reals(&[2, 1, 1, 1, 2]);
    Handle::new(Geom2dBSplineCurve::new(&poles, &knots, &mults, 1, false).unwrap())
}

#[test]
fn results_are_one_based_whatever_the_parameter_bounds() {
    let line = Handle::new(Geom2dLine::from_point(
        Point2::new(1.0, 2.0),
        Direction2::new(0.0, 1.0).unwrap(),
    ));
    let grid = Geom2dGridEvalLine::new(line).evaluate_grid(&params(-3, &[0.0, 2.0]));
    assert_eq!((grid.lower(), grid.len()), (1, 2));
    assert_eq!(grid[2], Point2::new(1.0, 4.0));
}

#[test]
fn empty_parameters_null_curves_and_low_orders_give_empty_grids() {
    let line = Handle::new(Geom2dLine::from_point(
        Point2::new(0.0, 0.0),
        Direction2::new(1.0, 0.0).unwrap(),
    ));
    let evaluator = Geom2dGridEvalLine::new(line);
    assert!(evaluator.evaluate_grid(&Array1::default()).is_empty());
    assert!(evaluator.evaluate_grid_dn(&params(1, &[0.5]), 0).is_empty());
    let null = Geom2dGridEvalOffsetCurve::new(Handle::default());
    assert!(null.geometry().is_null());
    assert!(
        null.evaluate_grid_d3(&params(1, &[0.5]))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn parabola_grid_ignores_an_indirect_frame() {
    let parabola = Handle::new(Geom2dParabola::from_axis(x_axis(), 1.0, false).unwrap());
    let grid = Geom2dGridEvalParabola::new(parabola.clone())
        .evaluate_grid(&params(1, &[1.5]))
        .unwrap();
    assert_eq!(grid[1], Point2::new(0.5625, 1.5));
    assert_eq!(parabola.value(1.5).unwrap(), Point2::new(0.5625, -1.5));
}

#[test]
fn zero_focal_parabola_grid_divides_by_zero() {
    let parabola = Handle::new(Geom2dParabola::from_axis(x_axis(), 0.0, true).unwrap());
    let grid = Geom2dGridEvalParabola::new(parabola)
        .evaluate_grid(&params(1, &[1.5]))
        .unwrap();
    assert!(grid[1].x().is_infinite() && grid[1].y().is_nan());
}

#[test]
fn rational_bezier_orders_above_degree_are_zero() {
    let poles = points(&[(1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]);
    let bezier =
        Handle::new(Geom2dBezierCurve::with_weights(&poles, &reals(&[1.0, 0.5, 1.0])).unwrap());
    let grid = Geom2dGridEvalBezierCurve::new(bezier.clone())
        .evaluate_grid_dn(&params(1, &[0.3]), 3)
        .unwrap();
    assert_eq!(grid[1], Vector2::new(0.0, 0.0));
    assert!(bezier.dn(0.3, 3).unwrap().magnitude() > 1.0);
}

#[test]
fn bspline_dn_uses_the_flat_span_index_as_a_knot_index() {
    let evaluator = Geom2dGridEvalBSplineCurve::new(polyline());
    let d1 = |u: f64| evaluator.evaluate_grid_dn(&params(1, &[u]), 1);
    // Flat span 2 ([0, 0.3)) is read as knot span 2, [0.3, 0.5], and flat span 3
    // ([0.3, 0.5)) as knot span 3, [0.5, 0.6]: each gives the slope of the next span.
    let shifted: Vector2 = d1(0.1).unwrap()[1];
    assert!((shifted.x() - 5.0).abs() < 1e-12 && (shifted.y() - 25.0).abs() < 1e-12);
    assert!((polyline().dn(0.1, 1).unwrap().x() - 1.0 / 0.3).abs() < 1e-12);
    let wrong: Vector2 = d1(0.4).unwrap()[1];
    assert!((wrong.x() - 10.0).abs() < 1e-12 && (wrong.y() - 70.0).abs() < 1e-12);
    // Flat span 5 is the last knot index: natively BuildKnots reads past the arrays.
    assert_eq!(d1(0.9).unwrap_err().exception_type(), "Standard_OutOfRange");
}

#[test]
fn bspline_cached_and_direct_runs_agree_with_the_curve() {
    let curve = polyline();
    let values: Vec<f64> = (0..10).map(|i| 0.6 + 0.04 * f64::from(i)).collect();
    let grid = Geom2dGridEvalBSplineCurve::new(curve.clone())
        .evaluate_grid(&params(1, &values))
        .unwrap();
    for (point, &u) in grid.iter().zip(&values) {
        assert!(point.distance(curve.value(u).unwrap()) < 1e-12);
    }
}

#[test]
fn dispatch_strips_trims_and_leaves_other_classes_uninitialized() {
    let circle: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dCircle::from_axis(x_axis(), 2.0, true).unwrap()).cast();
    let trimmed: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dTrimmedCurve::new(&circle, 0.5, 1.0, true, true).unwrap()).cast();
    let evaluator = Geom2dGridEvalCurve::from_curve(&trimmed);
    assert_eq!(evaluator.get_type(), CurveType::Circle);
    let grid = evaluator.evaluate_grid(&params(1, &[3.0])).unwrap();
    assert!(grid[1].distance(circle.value(3.0).unwrap()) < 1e-15);
    let sine: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dEvalSineWaveCurve::new(x_axis(), 2.0, 3.0, 0.5).unwrap()).cast();
    let uninitialized = Geom2dGridEvalCurve::from_curve(&sine);
    assert!(matches!(
        uninitialized.evaluator(),
        Geom2dGridEvalEvaluator::Uninitialized
    ));
    assert_eq!(uninitialized.get_type(), CurveType::OtherCurve);
    assert!(
        uninitialized
            .evaluate_grid(&params(1, &[0.5]))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn adaptors_other_than_geom2d_adaptor_use_the_fallback() {
    let line = Adaptor2dLine2d::new(
        Point2::new(0.0, 0.0),
        Direction2::new(1.0, 0.0).unwrap(),
        0.0,
        1.0,
    );
    let evaluator = Geom2dGridEvalCurve::from_adaptor(&line).unwrap();
    assert_eq!(evaluator.get_type(), CurveType::Line);
    assert!(matches!(
        evaluator.evaluator(),
        Geom2dGridEvalEvaluator::Other(_)
    ));
    let base = Adaptor2dCurve2dBase::default();
    let refused = Geom2dGridEvalCurve::from_adaptor(&base).err().unwrap();
    assert_eq!(refused.exception_type(), "Standard_NotImplemented");
    let fallback = Geom2dGridEvalOtherCurve::new(&base);
    assert_eq!(
        fallback
            .evaluate_grid(&params(1, &[0.5]))
            .unwrap_err()
            .exception_type(),
        "Standard_NotImplemented"
    );
}

#[test]
fn grid_evaluation_ignores_the_evaluation_representation() {
    let bezier = Handle::new(
        Geom2dBezierCurve::new(&points(&[(0.0, 0.0), (1.0, 2.0), (2.0, 0.0)])).unwrap(),
    );
    let rep: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(
        Point2::new(5.0, 5.0),
        Direction2::new(1.0, 0.0).unwrap(),
    ))
    .cast();
    bezier
        .set_eval_representation(Handle::new(RepCurveDescFull::new(rep)).cast())
        .unwrap();
    let adaptor = Geom2dAdaptorCurve::with_curve(&bezier.cast()).unwrap();
    let grid = Geom2dGridEvalCurve::from_adaptor(&adaptor)
        .unwrap()
        .evaluate_grid(&params(1, &[0.5]))
        .unwrap();
    assert_eq!(grid[1], Point2::new(1.0, 1.0));
    assert_eq!(bezier.value(0.5).unwrap(), Point2::new(5.5, 5.0));
}

#[test]
fn offset_grid_is_empty_at_a_degenerate_normal() {
    let cusp: Handle<dyn Geom2dCurve> = Handle::new(
        Geom2dBezierCurve::new(&points(&[(0.0, 0.0), (0.0, 0.0), (1.0, 1.0), (2.0, 0.0)])).unwrap(),
    )
    .cast();
    let offset = Handle::new(Geom2dOffsetCurve::new(&cusp, 0.25, false).unwrap());
    let evaluator = Geom2dGridEvalOffsetCurve::new(offset);
    assert!(
        evaluator
            .evaluate_grid(&params(1, &[0.5, 0.0]))
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        evaluator
            .evaluate_grid_d2(&params(1, &[0.5, 0.0]))
            .unwrap()
            .len(),
        2
    );
}
