// Copyright (c) 1995-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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

//! The exceptions and preserved native quirks of the AppParCurves port
//! (docs/tkgeombase-progress.md).
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeombase::app_par_curves::{
    self as apc, Constraint, ConstraintCouple, Constraints, Function, Gradient, LeastSquare,
    LineTool, MultiBSpCurve, MultiCurve, MultiCurveEval, MultiPoint, ResolConstraint,
};
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};
use rustycad_tkmath::math::{Matrix, Vector};

fn kind<T>(result: Result<T, Failure>) -> &'static str {
    result.err().map_or("none", |error| error.exception_type())
}

/// A MultiLine of `nb3d` 3D and `nb2d` 2D points per multipoint on a parabola, with
/// tangents at the listed points (AppDef_MultiLine semantics).
#[derive(Clone)]
struct FakeMultiLine {
    nb3d: i32,
    nb2d: i32,
    n: i32,
    tangents: Vec<i32>,
}

impl FakeMultiLine {
    fn t(&self, index: i32) -> f64 {
        f64::from(index - 1) / f64::from(self.n - 1)
    }
    fn check(&self, index: i32) -> Result<(), Failure> {
        if index < 1 || index > self.n {
            return Err(Failure::out_of_range("AppDef_MultiLine::Value"));
        }
        Ok(())
    }
    fn fill<T: Copy>(tab: &mut Array1<'_, T>, count: i32, value: T) -> Result<(), Failure> {
        let low: i32 = tab.lower();
        for i in 0..count {
            tab.set_value(low + i, value)
                .map_err(|e| Failure::out_of_range(&e.to_string()))?;
        }
        Ok(())
    }
}

impl LineTool for FakeMultiLine {
    fn first_point(&self) -> i32 {
        1
    }
    fn last_point(&self) -> i32 {
        self.n
    }
    fn nb_p2d(&self) -> i32 {
        self.nb2d
    }
    fn nb_p3d(&self) -> i32 {
        self.nb3d
    }
    fn value_3d(&self, index: i32, points: &mut Array1<'_, Point3>) -> Result<(), Failure> {
        self.check(index)?;
        let t: f64 = self.t(index);
        Self::fill(points, self.nb3d, Point3::new(t, t * t, 1.0 - t))
    }
    fn value_2d(&self, index: i32, points2d: &mut Array1<'_, Point2>) -> Result<(), Failure> {
        self.check(index)?;
        let t: f64 = self.t(index);
        Self::fill(points2d, self.nb2d, Point2::new(t, t * t))
    }
    fn value(
        &self,
        index: i32,
        points: &mut Array1<'_, Point3>,
        points2d: &mut Array1<'_, Point2>,
    ) -> Result<(), Failure> {
        self.value_3d(index, points)?;
        self.value_2d(index, points2d)
    }
    fn tangency_3d(&self, index: i32, tangents: &mut Array1<'_, Vector3>) -> Result<bool, Failure> {
        self.check(index)?;
        if !self.tangents.contains(&index) {
            return Ok(false);
        }
        Self::fill(
            tangents,
            self.nb3d,
            Vector3::new(1.0, 2.0 * self.t(index), -1.0),
        )
        .map(|()| true)
    }
    fn tangency_2d(
        &self,
        index: i32,
        tangents2d: &mut Array1<'_, Vector2>,
    ) -> Result<bool, Failure> {
        self.check(index)?;
        if !self.tangents.contains(&index) {
            return Ok(false);
        }
        Self::fill(
            tangents2d,
            self.nb2d,
            Vector2::new(1.0, 2.0 * self.t(index)),
        )
        .map(|()| true)
    }
    fn tangency(
        &self,
        index: i32,
        tangents: &mut Array1<'_, Vector3>,
        tangents2d: &mut Array1<'_, Vector2>,
    ) -> Result<bool, Failure> {
        Ok(self.tangency_3d(index, tangents)? & self.tangency_2d(index, tangents2d)?)
    }
    fn curvature_3d(&self, _: i32, _: &mut Array1<'_, Vector3>) -> Result<bool, Failure> {
        Ok(false)
    }
    fn curvature_2d(&self, _: i32, _: &mut Array1<'_, Vector2>) -> Result<bool, Failure> {
        Ok(false)
    }
    fn curvature(
        &self,
        _: i32,
        _: &mut Array1<'_, Vector3>,
        _: &mut Array1<'_, Vector2>,
    ) -> Result<bool, Failure> {
        Ok(false)
    }
}

fn line(nb3d: i32, nb2d: i32) -> FakeMultiLine {
    FakeMultiLine {
        nb3d,
        nb2d,
        n: 8,
        tangents: vec![1, 3, 8],
    }
}

fn parameters(n: i32) -> Vector<'static> {
    let mut u = Vector::filled(1, n, 0.).unwrap();
    (1..=n).for_each(|i| u[i] = f64::from(i - 1) / f64::from(n - 1));
    u
}

fn constraints(pairs: &[(i32, Constraint)]) -> Constraints {
    let couples: Vec<ConstraintCouple> = pairs
        .iter()
        .map(|(i, c)| ConstraintCouple::new(*i, *c))
        .collect();
    Handle::new(HArray1::from_array(&Array1::from_shared(&couples, 1)))
}

#[test]
fn multi_point_dimension_zero_is_3d_and_copies_share_points() {
    let point = MultiPoint::with_counts(1, 1).unwrap();
    assert_eq!(point.dimension(0).unwrap(), 3);
    assert_eq!(kind(point.dimension(-1)), "Standard_OutOfRange");
    assert_eq!(kind(point.point2d(1)), "Standard_OutOfRange");
    assert_eq!(kind(point.transform(2, [0.0; 6])), "Standard_OutOfRange");
    point
        .clone()
        .set_point(1, Point3::new(1.0, 2.0, 3.0))
        .unwrap();
    assert_eq!(point.point(1).unwrap(), Point3::new(1.0, 2.0, 3.0));
    let mut text = String::new();
    point.dump(&mut text).unwrap();
    assert!(text.starts_with("AppParCurves_MultiPoint dump:\nIt contains 1 3d points and 1 2d points.\n3D-Point #1\n Pole x = 1\n"));
}

#[test]
fn multi_curve_pole_checks_both_indexes_with_and() {
    let pole = MultiPoint::with_counts(1, 0).unwrap();
    let curve = MultiCurve::from_multi_points(&Array1::from_shared(&[pole.clone(), pole], 1));
    // `(CuIndex <= 0) && (Nieme <= 0)`: a single bad index reaches Point(0).
    assert_eq!(kind(curve.pole(0, 1)), "Standard_OutOfRange");
    assert_eq!(kind(curve.pole(0, 0)), "Standard_OutOfRange");
    assert_eq!(MultiCurve::new().nb_poles(), 0);
    assert_eq!(kind(MultiCurve::new().degree()), "Standard_NullObject");
    assert_eq!(
        kind(curve.transform2d(1, [0.0, 1.0, 0.0, 1.0])),
        "Standard_OutOfRange"
    );
}

#[test]
fn multi_bsp_curve_set_nb_poles_keeps_the_degree() {
    let mut curve = MultiBSpCurve::with_nb_poles(3).unwrap();
    assert_eq!(kind(curve.knots()), "Standard_NullObject");
    curve.set_multiplicities(&Array1::from_shared(&[3, 3], 1));
    assert_eq!(MultiCurveEval::degree(&curve).unwrap(), 2);
    curve.set_nb_poles(5).unwrap();
    assert_eq!(
        (MultiCurveEval::degree(&curve).unwrap(), curve.nb_poles()),
        (2, 5)
    );
}

#[test]
fn bernstein_needs_two_poles() {
    let u = Vector::filled(1, 2, 0.5).unwrap();
    let mut a = Matrix::filled(1, 2, 1, 1, 0.).unwrap();
    assert_eq!(
        kind(apc::bernstein_matrix(1, &u, &mut a)),
        "Standard_OutOfRange"
    );
    let mut dda = Vector::filled(1, 3, 0.).unwrap();
    apc::second_derivative_bernstein(0.25, &mut dda).unwrap();
    assert_eq!((dda[1], dda[2], dda[3]), (2.0, -4.0, 2.0));
}

#[test]
fn least_square_errors_and_distance_quirk() {
    let (line, u) = (line(1, 0), parameters(8));
    let unperformed = LeastSquare::without_parameters(
        &line,
        1,
        8,
        Constraint::PassPoint,
        Constraint::PassPoint,
        4,
    )
    .unwrap();
    assert_eq!(kind(unperformed.function_matrix()), "StdFail_NotDone");
    let not_ready = LeastSquare::new(
        &line,
        1,
        8,
        Constraint::TangencyPoint,
        Constraint::TangencyPoint,
        &u,
        4,
    )
    .unwrap();
    assert!(!not_ready.is_done());
    let knots = Array1::from_shared(&[0.0, 1.0], 1);
    let mults = Array1::from_shared(&[4, 4], 1);
    let mut bspline = LeastSquare::with_knots(
        &line,
        &knots,
        &mults,
        1,
        8,
        Constraint::PassPoint,
        Constraint::PassPoint,
        &u,
        4,
    )
    .unwrap();
    assert_eq!(kind(bspline.bezier_value()), "Standard_NoSuchObject");
    let mut fit = LeastSquare::new(
        &line,
        1,
        8,
        Constraint::NoConstraint,
        Constraint::NoConstraint,
        &u,
        3,
    )
    .unwrap();
    let (_, max3d, _) = fit.error().unwrap();
    let distance: f64 = fit.distance()[(2, 1)];
    let mut grad = Vector::filled(1, 8, 0.).unwrap();
    fit.error_gradient(&mut grad).unwrap();
    // Distance() takes the square roots once per Perform: after ErrorGradient it
    // returns the squared distances.
    assert_eq!(fit.distance()[(2, 1)], distance * distance);
    assert!(max3d >= distance);
}

#[test]
fn point_constraint_falls_back_to_the_last_couple() {
    let cons = constraints(&[(3, Constraint::PassPoint), (5, Constraint::TangencyPoint)]);
    assert_eq!(
        apc::point_constraint(&cons, 1).unwrap(),
        Constraint::TangencyPoint
    );
    assert_eq!(
        apc::point_constraint(&cons, 3).unwrap(),
        Constraint::PassPoint
    );
}

#[test]
fn resol_constraint_reads_2d_pass_points_at_the_pass_index() {
    let (line, u) = (line(1, 1), parameters(8));
    let mut fit = LeastSquare::new(
        &line,
        1,
        8,
        Constraint::PassPoint,
        Constraint::PassPoint,
        &u,
        5,
    )
    .unwrap();
    let mut curve = fit.bezier_value().unwrap();
    let (a, da) = (
        fit.function_matrix().unwrap().clone(),
        fit.derivative_function_matrix().unwrap().clone(),
    );
    let cons = constraints(&[
        (1, Constraint::PassPoint),
        (4, Constraint::PassPoint),
        (8, Constraint::PassPoint),
    ]);
    // tabP2d(i - nb3d) with i = 1: index 0.
    assert_eq!(
        kind(ResolConstraint::new(
            &line, &mut curve, 1, 8, &cons, &a, &da, 1e-10
        )),
        "Standard_OutOfRange"
    );
}

#[test]
fn resol_constraint_derivative_writes_the_ratio_rows_into_cont() {
    let (line, u) = (line(2, 0), parameters(8));
    let mut fit = LeastSquare::new(
        &line,
        1,
        8,
        Constraint::PassPoint,
        Constraint::PassPoint,
        &u,
        6,
    )
    .unwrap();
    let mut curve = fit.bezier_value().unwrap();
    let (a, da) = (
        fit.function_matrix().unwrap().clone(),
        fit.derivative_function_matrix().unwrap().clone(),
    );
    let cons = constraints(&[
        (1, Constraint::PassPoint),
        (3, Constraint::TangencyPoint),
        (8, Constraint::PassPoint),
    ]);
    let mut resol = ResolConstraint::new(&line, &mut curve, 1, 8, &cons, &a, &da, 1e-10).unwrap();
    assert!(resol.is_done());
    let before: Matrix<'static> = resol.constraint_matrix().clone();
    resol.constraint_derivative(&line, &u, 5, &da).unwrap();
    let last: i32 = before.row_number();
    let changed: bool = (1..=before.col_number())
        .any(|j| before[(last, j)] != resol.constraint_matrix()[(last, j)]);
    assert!(changed);
}

#[test]
fn function_error_reads_the_constrained_branch_only() {
    let (line3d, u) = (line(1, 0), parameters(8));
    let cons = constraints(&[(1, Constraint::PassPoint), (8, Constraint::PassPoint)]);
    let mut f = Function::new(&line3d, 1, 8, &cons, &u, 3).unwrap();
    let mut value = 0.0;
    assert!(f.evaluate(&u, &mut value).unwrap());
    assert_eq!(f.error(4, 1).unwrap(), 0.0);
    let inner = constraints(&[
        (1, Constraint::PassPoint),
        (4, Constraint::PassPoint),
        (8, Constraint::PassPoint),
    ]);
    // PTLX from TabP2d(j) at the curve index: the mixed line has one 2D point.
    assert_eq!(
        kind(Function::new(&line(1, 1), 1, 8, &inner, &u, 3)),
        "Standard_OutOfRange"
    );
}

#[test]
fn gradient_without_inner_constraints_reports_null_errors() {
    let (line, mut u) = (line(1, 0), parameters(8));
    let cons = constraints(&[(1, Constraint::PassPoint), (8, Constraint::PassPoint)]);
    let gradient = Gradient::new(&line, 1, 8, &cons, &mut u, 3, 1e-3, 1e-3, 0).unwrap();
    assert_eq!(
        (gradient.error(4).unwrap(), gradient.average_error()),
        (0.0, 0.0)
    );
    assert_eq!(gradient.value().nb_poles(), 4);
}
