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

//! Behavior of the AppCont port beyond the upstream suite: the function base, the
//! least-squares fit and its constraints, singular and periodic ends, and the
//! native exceptions (the bits are compared with OCCT by the numeric probe).
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::app_cont::cont_matrices::{
    ibp_matrix_offset, ibt_matrix_offset, m_matrix_offset, v_bernstein,
};
use rustycad_tkgeombase::app_cont::{Function, FunctionBase, LeastSquare, PeriodicityInfo};
use rustycad_tkgeombase::app_par_curves::{Constraint, MultiCurveEval};
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};
use rustycad_tkmath::math::Matrix;
use std::f64::consts::TAU;

/// A fake user function: one 3D cubic `(u, u^2, u^3)` and one 2D point `(1 - u, 2u)`,
/// with switches for a failing D1, a jump at `u = 0` and a first coordinate periodic
/// of `period`, wrapped by one period after `u = 0.9`.
struct CubicPlanarFunction {
    base: FunctionBase,
    d1_ok: bool,
    jump_at_zero: bool,
    period: Option<f64>,
}

impl CubicPlanarFunction {
    fn new() -> Self {
        let mut base = FunctionBase::new();
        base.set_number_of_points(1, 1);
        Self {
            base,
            d1_ok: true,
            jump_at_zero: false,
            period: None,
        }
    }
}

impl Function for CubicPlanarFunction {
    fn base(&self) -> &FunctionBase {
        &self.base
    }
    fn first_parameter(&self) -> f64 {
        0.0
    }
    fn last_parameter(&self) -> f64 {
        1.0
    }
    fn value(&self, u: f64, pnt2d: &mut Array1<'_, Point2>, pnt: &mut Array1<'_, Point3>) -> bool {
        let jump: f64 = if self.jump_at_zero && u == 0.0 {
            5.0
        } else {
            0.0
        };
        let wrap: f64 = match self.period {
            Some(period) if u > 0.9 => -period,
            _ => 0.0,
        };
        pnt[1] = Point3::new(u + jump + wrap, u * u, u * u * u);
        pnt2d[1] = Point2::new(1.0 - u, 2.0 * u);
        true
    }
    fn d1(&self, u: f64, vec2d: &mut Array1<'_, Vector2>, vec: &mut Array1<'_, Vector3>) -> bool {
        vec[1] = Vector3::new(1.0, 2.0 * u, 3.0 * u * u);
        vec2d[1] = Vector2::new(-1.0, 2.0);
        self.d1_ok
    }
    fn period_information(&self, dim_idx: i32) -> (bool, f64) {
        match self.period {
            Some(period) if dim_idx == 1 => (true, period),
            _ => (false, 0.0),
        }
    }
}

fn fit(
    function: &CubicPlanarFunction,
    cons: [Constraint; 2],
    deg: i32,
    nb_points: i32,
) -> Result<LeastSquare, Failure> {
    LeastSquare::new(function, 0.0, 1.0, cons[0], cons[1], deg, nb_points)
}

const PASS: Constraint = Constraint::PassPoint;
const TANGENT: Constraint = Constraint::TangencyPoint;
const FREE: Constraint = Constraint::NoConstraint;

#[test]
fn function_base_starts_at_minus_one_and_is_not_periodic() {
    let base = FunctionBase::default();
    assert_eq!(base.get_number_of_points(), (-1, -1));
    struct Bare(FunctionBase);
    impl Function for Bare {
        fn base(&self) -> &FunctionBase {
            &self.0
        }
        fn first_parameter(&self) -> f64 {
            -1.0
        }
        fn last_parameter(&self) -> f64 {
            2.0
        }
        fn value(&self, _: f64, _: &mut Array1<'_, Point2>, _: &mut Array1<'_, Point3>) -> bool {
            false
        }
        fn d1(&self, _: f64, _: &mut Array1<'_, Vector2>, _: &mut Array1<'_, Vector3>) -> bool {
            false
        }
    }
    let bare = Bare(base);
    assert_eq!(
        (bare.get_nb_of_3d_points(), bare.get_nb_of_2d_points()),
        (-1, -1)
    );
    assert_eq!(bare.period_information(1), (false, 0.0));
    assert_eq!((bare.first_parameter(), bare.last_parameter()), (-1.0, 2.0));
    assert_eq!(
        PeriodicityInfo::default(),
        PeriodicityInfo {
            is_periodic: false,
            my_period: 0.0
        }
    );
}

#[test]
fn offsets_follow_the_block_layout() {
    assert_eq!(
        (m_matrix_offset(2), m_matrix_offset(3), m_matrix_offset(4)),
        (0, 4, 13)
    );
    assert_eq!((ibp_matrix_offset(3), ibp_matrix_offset(5)), (0, 5));
    assert_eq!((ibt_matrix_offset(5), ibt_matrix_offset(7)), (0, 5));
}

#[test]
fn v_bernstein_at_classe_one_is_refused_instead_of_reading_before_the_table() {
    let mut vb = Matrix::with_bounds(1, 1, 1, 4).unwrap();
    let error = v_bernstein(1, 4, &mut vb).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    assert!(v_bernstein(0, 4, &mut vb).is_ok());
}

#[test]
fn free_fit_reproduces_a_cubic_but_reports_not_done() {
    let function = CubicPlanarFunction::new();
    let mut ls = fit(&function, [FREE, FREE], 3, 8).unwrap();
    assert!(!ls.is_done());
    let (f, e3, e2) = ls.error();
    assert!(f < 1e-24 && e3 < 1e-12 && e2 < 1e-12, "{f} {e3} {e2}");
    let curve = ls.value().unwrap();
    assert_eq!(curve.nb_poles(), 4);
    let middle = curve.value_3d(1, 0.5).unwrap();
    assert!(middle.distance(&Point3::new(0.5, 0.25, 0.125)) < 1e-12);
    assert!(
        curve
            .value_2d(2, 0.5)
            .unwrap()
            .distance(Point2::new(0.5, 1.0))
            < 1e-12
    );
}

#[test]
fn pass_points_fix_the_end_poles_and_mark_the_fit_done() {
    let function = CubicPlanarFunction::new();
    let mut ls = fit(&function, [PASS, PASS], 2, 10).unwrap();
    assert!(ls.is_done());
    let curve = ls.value().unwrap();
    assert_eq!(curve.pole(1, 1).unwrap(), Point3::new(0.0, 0.0, 0.0));
    assert_eq!(curve.pole(1, 3).unwrap(), Point3::new(1.0, 1.0, 1.0));
    assert_eq!(curve.pole2d(2, 3).unwrap(), Point2::new(0.0, 2.0));
    assert!(ls.error().1 > 1e-3);
}

#[test]
fn tangency_points_set_the_second_poles_from_the_derivatives() {
    let function = CubicPlanarFunction::new();
    let mut ls = fit(&function, [TANGENT, TANGENT], 5, 12).unwrap();
    assert!(ls.is_done());
    let curve = ls.value().unwrap();
    assert_eq!(curve.pole(1, 2).unwrap(), Point3::new(0.2, 0.0, 0.0));
    assert_eq!(
        curve.pole(1, 5).unwrap(),
        Point3::new(1.0 - 0.2, 1.0 - 0.4, 1.0 - 0.6000000000000001)
    );
    assert!(ls.error().0 < 1e-24);
}

#[test]
fn a_failing_derivative_turns_tangency_into_pass() {
    let mut function = CubicPlanarFunction::new();
    function.d1_ok = false;
    let tangent = fit(&function, [TANGENT, TANGENT], 4, 9).unwrap();
    let pass = fit(&function, [PASS, PASS], 4, 9).unwrap();
    assert_eq!(tangent.error(), pass.error());
}

#[test]
fn mixed_constraints_invert_the_normal_matrix() {
    let function = CubicPlanarFunction::new();
    let mut ls = fit(&function, [PASS, TANGENT], 4, 9).unwrap();
    assert!(ls.is_done());
    assert!(ls.error().0 < 1e-24);
    assert_eq!(
        ls.value().unwrap().pole(1, 1).unwrap(),
        Point3::new(0.0, 0.0, 0.0)
    );
}

#[test]
fn degree_one_with_two_pass_points_has_no_free_pole() {
    let function = CubicPlanarFunction::new();
    let ls = fit(&function, [PASS, PASS], 1, 6).unwrap();
    assert!(!ls.is_done());
}

#[test]
fn a_singular_end_takes_the_limit_from_inside() {
    let mut function = CubicPlanarFunction::new();
    function.jump_at_zero = true;
    let mut ls = fit(&function, [PASS, PASS], 3, 8).unwrap();
    let first = ls.value().unwrap().pole(1, 1).unwrap();
    assert!(first.x().abs() < 1e-3, "{first:?}");
}

#[test]
fn a_periodic_end_pole_is_brought_back_next_to_the_samples() {
    let mut function = CubicPlanarFunction::new();
    function.period = Some(5.0);
    let mut ls = fit(&function, [FREE, PASS], 3, 8).unwrap();
    let last = ls.value().unwrap().pole(1, 4).unwrap();
    assert_eq!(last.x(), 1.0);
    // The wrapped samples were unwrapped too, so the fit stays exact.
    assert!(ls.error().0 < 1e-20, "{:?}", ls.error());
}

#[test]
fn native_exceptions_become_failures() {
    let function = CubicPlanarFunction::new();
    let kind = |result: Result<LeastSquare, Failure>| result.err().unwrap().exception_type();
    assert_eq!(
        kind(fit(&function, [FREE, FREE], 3, 25)),
        "Standard_DimensionError"
    );
    assert_eq!(
        kind(fit(&function, [FREE, FREE], 3, 70)),
        "Standard_DimensionError"
    );
    assert_eq!(
        kind(fit(&function, [PASS, PASS], 24, 8)),
        "Standard_DimensionError"
    );
    assert_eq!(
        kind(fit(&function, [FREE, FREE], 0, 8)),
        "Standard_OutOfRange"
    );
    assert_eq!(
        kind(fit(&function, [FREE, FREE], 3, 0)),
        "Standard_RangeError"
    );
    assert_eq!(
        kind(fit(&function, [FREE, FREE], -3, 4)),
        "Standard_RangeError"
    );
}

/// A fake 2D function whose first coordinate `5.9 + u` is reduced into [0, 2 pi): with
/// 3 Gauss points on [0, 1], only the first sample lies before the wrap.
struct WrappedLineFunction(FunctionBase);

impl Function for WrappedLineFunction {
    fn base(&self) -> &FunctionBase {
        &self.0
    }
    fn first_parameter(&self) -> f64 {
        0.0
    }
    fn last_parameter(&self) -> f64 {
        1.0
    }
    fn value(&self, u: f64, pnt2d: &mut Array1<'_, Point2>, _: &mut Array1<'_, Point3>) -> bool {
        let x: f64 = 5.9 + u;
        pnt2d[1] = Point2::new(x - TAU * (x / TAU).floor(), u);
        true
    }
    fn d1(&self, _: f64, vec2d: &mut Array1<'_, Vector2>, _: &mut Array1<'_, Vector3>) -> bool {
        vec2d[1] = Vector2::new(1.0, 1.0);
        true
    }
    fn period_information(&self, dim_idx: i32) -> (bool, f64) {
        if dim_idx == 1 {
            (true, TAU)
        } else {
            (false, 0.0)
        }
    }
}

#[test]
fn a_first_sample_alone_before_a_wrap_is_moved_by_one_period() {
    let mut base = FunctionBase::new();
    base.set_number_of_points(0, 1);
    let function = WrappedLineFunction(base);
    let mut ls = LeastSquare::new(&function, 0.0, 1.0, FREE, FREE, 1, 3).unwrap();
    // Unwrapped, the samples lie on one line, so the linear fit is exact.
    assert!(ls.error().0 < 1e-24, "{:?}", ls.error());
    let first = ls.value().unwrap().pole2d(1, 1).unwrap();
    assert!((first.x() - (5.9 - TAU)).abs() < 1e-12, "{first:?}");
}
