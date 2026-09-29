// Created on: 1995-03-14
// Created by: Modelistation
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
// Rust translation of ModelingData/TKGeomBase/AppCont/AppCont_LeastSquare.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AppCont_LeastSquare`: the Bezier curve of a given degree that fits a continuous
//! function in the least-squares sense over Gauss points, with optional pass or
//! tangency constraints at both ends.
mod border;
mod sampling;
mod solve;

use super::function::Function;
use crate::app_par_curves::{Constraint, MultiCurve, MultiPoint};
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};
use rustycad_tkmath::math::{Matrix, Vector};

/// `PeriodicityInfo`: whether one output coordinate of the function is periodic,
/// and its period (from `AppCont_Function::PeriodInformation`).
///
/// ```
/// use rustycad_tkgeombase::app_cont::PeriodicityInfo;
/// let info = PeriodicityInfo { is_periodic: true, my_period: 6.0 };
/// assert!(info.is_periodic && PeriodicityInfo::default().my_period == 0.0);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PeriodicityInfo {
    /// `isPeriodic`.
    pub is_periodic: bool,
    /// `myPeriod`.
    pub my_period: f64,
}

/// `AppCont_LeastSquare`: the least-squares Bezier fit of degree `Deg` to a
/// [`Function`] over `[U0, U1]`, sampled at `NbPoints` Gauss points.
///
/// The constructor computes the poles; [`value`](Self::value) stores them in a
/// MultiCurve and [`error`](Self::error) measures the fit at the Gauss points.
/// Natively [`is_done`](Self::is_done) turns true only when constrained poles were
/// solved for: the unconstrained fit and the fits without free pole (degree 1 with
/// two pass points, for example) report not done although their poles are set.
///
/// ```ignore
/// let fit = LeastSquare::new(&function, 0.0, 1.0, Constraint::PassPoint, Constraint::PassPoint, 5, 12)?;
/// let (f, max_e3d, max_e2d) = fit.error();
/// ```
pub struct LeastSquare {
    my_scu: MultiCurve,
    my_points: Matrix<'static>,
    my_poles: Matrix<'static>,
    my_param: Vector<'static>,
    my_vb: Matrix<'static>,
    my_per_info: Array1<'static, PeriodicityInfo>,
    my_done: bool,
    my_degre: i32,
    my_nbdiscret: i32,
    my_nb_p: i32,
    my_nb_p2d: i32,
}

// A matrix with the native bounds; bad bounds are Standard_RangeError.
pub(super) fn matrix(
    rows: [i32; 2],
    cols: [i32; 2],
    value: f64,
) -> Result<Matrix<'static>, Failure> {
    Matrix::filled(rows[0], rows[1], cols[0], cols[1], value)
        .map_err(|error| array_failure(error, "AppCont_LeastSquare"))
}

// A 1-based array of `count` entries; a negative count is Standard_RangeError.
fn array<T: Default>(count: i32) -> Result<Array1<'static, T>, Failure> {
    Array1::with_bounds(1, count).map_err(|error| array_failure(error, "AppCont_LeastSquare"))
}

/// The point and derivative arrays the fit hands to the function, each of
/// `max(count, 1)` entries as natively.
pub(super) struct Evaluation {
    pub(super) tab_p: Array1<'static, Point3>,
    pub(super) tab_p2d: Array1<'static, Point2>,
    pub(super) tab_v: Array1<'static, Vector3>,
    pub(super) tab_v2d: Array1<'static, Vector2>,
}

impl Evaluation {
    // The arrays of `nb_p` 3D and `nb_p2d` 2D values.
    fn new(nb_p: i32, nb_p2d: i32) -> Result<Self, Failure> {
        Ok(Self {
            tab_p: array(nb_p.max(1))?,
            tab_p2d: array(nb_p2d.max(1))?,
            tab_v: array(nb_p.max(1))?,
            tab_v2d: array(nb_p2d.max(1))?,
        })
    }
}

impl LeastSquare {
    /// `AppCont_LeastSquare(SSP, U0, U1, FirstCons, LastCons, Deg, NbPoints)`: fits a
    /// Bezier curve of degree `deg` to `ssp` over `[u0, u1]` at `nb_points` Gauss
    /// points. A tangency constraint whose D1 fails becomes a pass constraint.
    ///
    /// Standard_DimensionError for `nb_points > 24` (VBernstein) or `deg > 23` (the
    /// mass matrices); Standard_RangeError for negative point or pole counts (`deg < -1`
    /// or `nb_points < 0`, which overrun the stack buffer natively), while the empty
    /// ranges (`deg == -1`, `nb_points == 0`, no coordinate) fit over empty tables;
    /// Standard_OutOfRange when a fixed end needs a pole row that degree -1 lacks;
    /// math_SingularMatrix if a mixed-constraint normal matrix is singular.
    /// `LeastSquare::new(&f, 0.0, 1.0, Constraint::NoConstraint, Constraint::NoConstraint, 4, 10)?`
    pub fn new(
        ssp: &dyn Function,
        u0: f64,
        u1: f64,
        first_cons: Constraint,
        last_cons: Constraint,
        deg: i32,
        nb_points: i32,
    ) -> Result<Self, Failure> {
        let mut fit: Self = Self::allocate(ssp, deg, nb_points)?;
        fit.perform(ssp, [u0, u1], [first_cons, last_cons])?;
        Ok(fit)
    }

    // The member initializers, in the native order.
    fn allocate(ssp: &dyn Function, deg: i32, nb_points: i32) -> Result<Self, Failure> {
        let nbcol: i32 = 3 * ssp.get_nb_of_3d_points() + 2 * ssp.get_nb_of_2d_points();
        Ok(Self {
            my_scu: MultiCurve::with_nb_poles(deg + 1)?,
            my_points: matrix([1, nb_points], [1, nbcol], 0.0)?,
            my_poles: matrix([1, deg + 1], [1, nbcol], 0.0)?,
            my_param: Vector::with_bounds(1, nb_points)?,
            my_vb: matrix([1, deg + 1], [1, nb_points], 0.0)?,
            my_per_info: array(nbcol)?,
            my_done: false,
            my_degre: deg,
            my_nbdiscret: nb_points,
            my_nb_p: 0,
            my_nb_p2d: 0,
        })
    }

    // The constructor body: sampling, right-hand side, then the free or constrained
    // resolution.
    fn perform(
        &mut self,
        ssp: &dyn Function,
        u: [f64; 2],
        cons: [Constraint; 2],
    ) -> Result<(), Failure> {
        let classe: i32 = self.my_degre + 1;
        let b: Matrix<'static> = matrix([1, classe], [1, self.nbcol()], 0.0)?;
        (self.my_nb_p, self.my_nb_p2d) = ssp.get_number_of_points();
        let mut arrays: Evaluation = Evaluation::new(self.my_nb_p, self.my_nb_p2d)?;
        self.read_period_information(ssp)?;
        let cons: [Constraint; 2] = check_tangencies(ssp, u, cons, &mut arrays);
        let weights: Vector<'static> = self.sample(ssp, u, &mut arrays)?;
        self.fix_period_jumps()?;
        super::cont_matrices::v_bernstein(classe, self.my_nbdiscret, &mut self.my_vb)?;
        let b: Matrix<'static> = self.right_hand_side(&weights, b);
        if cons == [Constraint::NoConstraint; 2] {
            return self.solve_free(&b);
        }
        self.solve_constrained(ssp, u, cons, b, &mut arrays)
    }

    // `nbcol`: the coordinates per sample, `3 NbPnt + 2 NbPnt2d`.
    fn nbcol(&self) -> i32 {
        self.my_points.upper_col()
    }

    // `SSP.PeriodInformation` for every output coordinate.
    fn read_period_information(&mut self, ssp: &dyn Function) -> Result<(), Failure> {
        for dim_idx in 1..=self.my_nb_p * 3 + self.my_nb_p2d * 2 {
            let (is_periodic, my_period) = ssp.period_information(dim_idx);
            *self.per_info_mut(dim_idx)? = PeriodicityInfo {
                is_periodic,
                my_period,
            };
        }
        Ok(())
    }

    // `myPerInfo(aDimIdx)`; outside the array (undefined behavior natively, only for
    // inconsistent point counts) Standard_OutOfRange.
    fn per_info(&self, dim_idx: i32) -> Result<PeriodicityInfo, Failure> {
        self.my_per_info
            .value(dim_idx)
            .copied()
            .map_err(|error| array_failure(error, "AppCont_LeastSquare::myPerInfo"))
    }

    fn per_info_mut(&mut self, dim_idx: i32) -> Result<&mut PeriodicityInfo, Failure> {
        self.my_per_info
            .change_value(dim_idx)
            .map_err(|error| array_failure(error, "AppCont_LeastSquare::myPerInfo"))
    }

    /// `Value()`: the fitted poles as a MultiCurve of `Deg + 1` poles (3D points first,
    /// then 2D points, as the function orders them); `fit.value()?`.
    pub fn value(&mut self) -> Result<&MultiCurve, Failure> {
        for i in 1..=self.my_degre + 1 {
            let pole: MultiPoint = self.pole(i)?;
            self.my_scu.set_value(i, &pole)?;
        }
        Ok(&self.my_scu)
    }

    // The MultiPoint of pole row `i`.
    fn pole(&self, i: i32) -> Result<MultiPoint, Failure> {
        let m_pole: MultiPoint = MultiPoint::with_counts(self.my_nb_p, self.my_nb_p2d)?;
        let row = |j2: i32| self.my_poles[(i, j2)];
        let mut j2: i32 = 1;
        for j in 1..=self.my_nb_p {
            m_pole.set_point(j, Point3::new(row(j2), row(j2 + 1), row(j2 + 2)))?;
            j2 += 3;
        }
        for j in self.my_nb_p + 1..=self.my_nb_p + self.my_nb_p2d {
            m_pole.set_point2d(j, Point2::new(row(j2), row(j2 + 1)))?;
            j2 += 2;
        }
        Ok(m_pole)
    }

    /// `Error(F, MaxE3d, MaxE2d)`: at the Gauss points, the sum of the squared
    /// distances between the function and the fit, and the largest 3D and 2D
    /// distances; `let (f, max_e3d, max_e2d) = fit.error();`.
    pub fn error(&self) -> (f64, f64, f64) {
        let residuals: Matrix<'static> = self.residuals();
        let (mut f, mut max_e3d, mut max_e2d) = (0.0, 0.0, 0.0);
        for i in 1..=self.my_nbdiscret {
            let mut i2: i32 = 1;
            for _ in 1..=self.my_nb_p {
                let e = |k: i32| residuals[(i, i2 + k)];
                let err3d: f64 = e(0) * e(0) + e(1) * e(1) + e(2) * e(2);
                max_e3d = f64::max(max_e3d, err3d);
                f += err3d;
                i2 += 3;
            }
            for _ in 1..=self.my_nb_p2d {
                let e = |k: i32| residuals[(i, i2 + k)];
                let err2d: f64 = e(0) * e(0) + e(1) * e(1);
                max_e2d = f64::max(max_e2d, err2d);
                f += err2d;
                i2 += 2;
            }
        }
        (f, max_e3d.sqrt(), max_e2d.sqrt())
    }

    // The sampled points minus the fit, pole by pole in the native order.
    fn residuals(&self) -> Matrix<'static> {
        let mut residuals: Matrix<'static> = self.my_points.clone();
        let ncol: i32 = self.nbcol();
        for c in 1..=self.my_degre + 1 {
            for i in 1..=self.my_nbdiscret {
                let coeff: f64 = self.my_vb[(c, i)];
                for j in 1..=ncol {
                    residuals[(i, j)] -= self.my_poles[(c, j)] * coeff;
                }
            }
        }
        residuals
    }

    /// `IsDone()`: true once constrained poles were solved for (see the type
    /// documentation); `fit.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_done
    }
}

// A tangency constraint whose D1 fails at its end becomes a pass constraint.
fn check_tangencies(
    ssp: &dyn Function,
    u: [f64; 2],
    mut cons: [Constraint; 2],
    arrays: &mut Evaluation,
) -> [Constraint; 2] {
    for (constraint, parameter) in cons.iter_mut().zip(u) {
        if *constraint == Constraint::TangencyPoint
            && !ssp.d1(parameter, &mut arrays.tab_v2d, &mut arrays.tab_v)
        {
            *constraint = Constraint::PassPoint;
        }
    }
    cons
}
