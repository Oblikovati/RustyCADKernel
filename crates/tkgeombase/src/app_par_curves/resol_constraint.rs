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
// Rust translation of ModelingData/TKGeomBase/AppParCurves/AppParCurves_ResolConstraint.gxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AppParCurves_ResolConstraint` (AppParCurves_ResolConstraint.gxx): the poles of
//! a MultiCurve moved by the Uzawa method onto the pass-point and tangency
//! constraints of a MultiLine.
mod derivative;
mod ratio;
mod rows;

use super::Constraints;
use super::constraint::{Constraint, ConstraintCouple};
use super::line_tool::LineTool;
use super::multi_curve::{MultiCurve, MultiCurveEval};
use super::multi_point::MultiPoint;
use crate::native_failure::{array_failure, failure};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkmath::math::{Matrix, Uzawa, Vector};

/// `AppParCurves_ResolConstraint`: the native generic over `MultiLine`/`ToolLine`; the
/// line is read by the constructor and `ConstraintDerivative` only, so the struct is
/// not generic.
///
/// The native header declares `Error()` without defining it (a link error if
/// called); the port has no counterpart. Without pass points the constructor stops
/// with IsDone true and leaves CTCinv and Vardua uninitialized natively (0 here).
///
/// ```ignore
/// let resol = ResolConstraint::new(&line, &mut curve, 1, 10, &constraints, &a, &da, 1e-10)?;
/// assert!(resol.is_done());
/// ```
pub struct ResolConstraint {
    done: bool,
    cont: Matrix<'static>,
    de_cont: Matrix<'static>,
    secont: Vector<'static>,
    ctc_inv: Matrix<'static>,
    vardua: Vector<'static>,
    inc_pass: i32,
    inc_tan: i32,
    inc_curv: i32,
    i_pas: Vec<i32>,
    i_tan: Vec<i32>,
    i_curv: Vec<i32>,
}

// The couples of a constraint handle; Standard_NullObject for a null handle.
pub(crate) fn couples(constraints: &Constraints) -> Result<Vec<ConstraintCouple>, Failure> {
    let array = constraints
        .get()
        .ok_or_else(|| failure("Standard_NullObject", "AppParCurves: null constraint array"))?;
    let couples: Vec<ConstraintCouple> = array.array1().iter().copied().collect();
    Ok(couples)
}

fn matrix(rows: i32, cols: i32) -> Result<Matrix<'static>, Failure> {
    Matrix::filled(1, rows, 1, cols, 0.)
        .map_err(|error| array_failure(error, "AppParCurves_ResolConstraint"))
}

// IPas(i) = value with the checked native access (an array of LastPoint-FirstPoint+1).
fn push(list: &mut Vec<i32>, capacity: i32, value: i32) -> Result<(), Failure> {
    if list.len() as i32 >= capacity {
        return Err(Failure::out_of_range(&format!(
            "AppParCurves_ResolConstraint: more than {capacity} constrained points"
        )));
    }
    list.push(value);
    Ok(())
}

impl ResolConstraint {
    /// `NbConstraints(SSP, FirstPoint, LastPoint, TheConstraints)` (protected natively):
    /// `CCol * IncPass + IncTan * (CCol - 1) + 3 * IncCurv`;
    /// `ResolConstraint::nb_constraints(&line, &constraints)?`.
    pub fn nb_constraints(ssp: &impl LineTool, constraints: &Constraints) -> Result<i32, Failure> {
        let (mut pass, mut tan, mut curv) = (0, 0, 0);
        for couple in couples(constraints)? {
            let code: i32 = couple.constraint().code();
            pass += i32::from(code >= 1);
            tan += i32::from(code >= 2);
            curv += i32::from(code == 3);
        }
        let ccol: i32 = ssp.nb_p3d() * 3 + ssp.nb_p2d() * 2;
        Ok(ccol * pass + tan * (ccol - 1) + 3 * curv)
    }

    /// `NbColumns(SSP, Deg)` (protected natively): `CCol * (Deg + 1)`;
    /// `ResolConstraint::nb_columns(&line, 3)`.
    pub fn nb_columns(ssp: &impl LineTool, deg: i32) -> i32 {
        (ssp.nb_p3d() * 3 + ssp.nb_p2d() * 2) * (deg + 1)
    }

    /// The native constructor (`Tolerance` defaults to 1e-10 natively): builds the
    /// constraint matrix from `bern` and `derivative_bern` (rows indexed by point),
    /// solves by Uzawa from the poles of `scurv` and writes the constrained poles
    /// back into `scurv` (whose pole array is shared with its copies).
    #[allow(clippy::too_many_arguments)] // Mirrors the native eight-argument constructor.
    pub fn new(
        ssp: &impl LineTool,
        scurv: &mut MultiCurve,
        first_point: i32,
        last_point: i32,
        constraints: &Constraints,
        bern: &Matrix<'_>,
        derivative_bern: &Matrix<'_>,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let (rows, columns) = (
            Self::nb_constraints(ssp, constraints)?,
            Self::nb_columns(ssp, scurv.nb_poles() - 1),
        );
        let mut resol: Self = Self {
            done: false,
            cont: matrix(rows, columns)?,
            de_cont: matrix(rows, columns)?,
            secont: Vector::filled(1, rows, 0.)?,
            ctc_inv: matrix(rows, rows)?,
            vardua: Vector::filled(1, rows, 0.)?,
            inc_pass: 0,
            inc_tan: 0,
            inc_curv: 0,
            i_pas: Vec::new(),
            i_tan: Vec::new(),
            i_curv: Vec::new(),
        };
        let [fc, lc] = resol.count_constraints(constraints, first_point, last_point)?;
        if resol.inc_pass == 0 {
            resol.done = true;
            return Ok(resol);
        }
        let start: Vector<'static> = resol.fill_system(ssp, scurv, bern, derivative_bern)?;
        resol.solve(scurv, &start, tolerance, ssp, [fc, lc])?;
        Ok(resol)
    }

    // IPas/ITan/ICurv from the couples; the constraints of the first and last points.
    fn count_constraints(
        &mut self,
        constraints: &Constraints,
        first_point: i32,
        last_point: i32,
    ) -> Result<[Constraint; 2], Failure> {
        let capacity: i32 = last_point - first_point + 1;
        let (mut fc, mut lc) = (Constraint::NoConstraint, Constraint::NoConstraint);
        for couple in couples(constraints)? {
            let (index, cons) = (couple.index(), couple.constraint());
            if index == first_point {
                fc = cons;
            }
            if index == last_point {
                lc = cons;
            }
            if cons >= Constraint::PassPoint {
                push(&mut self.i_pas, capacity, index)?;
            }
            if cons >= Constraint::TangencyPoint {
                push(&mut self.i_tan, capacity, index)?;
            }
            if cons == Constraint::CurvaturePoint {
                push(&mut self.i_curv, capacity, index)?;
            }
        }
        (self.inc_pass, self.inc_tan, self.inc_curv) = (
            self.i_pas.len() as i32,
            self.i_tan.len() as i32,
            self.i_curv.len() as i32,
        );
        Ok([fc, lc])
    }

    // Uzawa from `start`; on success CTCinv (symmetrized from its lower triangle),
    // Vardua and the constrained poles.
    fn solve(
        &mut self,
        scurv: &mut MultiCurve,
        start: &Vector<'_>,
        tolerance: f64,
        ssp: &impl LineTool,
        [fc, lc]: [Constraint; 2],
    ) -> Result<(), Failure> {
        let uzawa: Uzawa =
            Uzawa::with_options(&self.cont, &self.secont, start, tolerance, 1.0e-6, 500)?;
        if !uzawa.is_done() {
            self.done = false;
            return Ok(());
        }
        self.ctc_inv.initialized(uzawa.inverse_constraints()?)?;
        uzawa.copy_dual(&mut self.vardua)?;
        for i in 1..=self.ctc_inv.row_number() {
            for j in i..=self.ctc_inv.row_number() {
                self.ctc_inv[(i, j)] = self.ctc_inv[(j, i)];
            }
        }
        self.done = true;
        let npol: i32 = scurv.nb_poles();
        let poles: &Vector<'static> = uzawa.value()?;
        let polinit: i32 = if fc >= Constraint::PassPoint { 2 } else { 1 };
        let polfin: i32 = if lc >= Constraint::PassPoint {
            npol - 1
        } else {
            npol
        };
        for i in polinit..=polfin {
            scurv.set_value(i, &Self::pole(ssp, poles, npol, i)?)?;
        }
        Ok(())
    }

    // The MultiPoint of pole i in the Uzawa solution (X, Y[, Z] blocks per curve).
    fn pole(
        ssp: &impl LineTool,
        poles: &Vector<'_>,
        npol: i32,
        i: i32,
    ) -> Result<MultiPoint, Failure> {
        let (nb3d, nb2d) = (ssp.nb_p3d(), ssp.nb_p2d());
        let point: MultiPoint = MultiPoint::with_counts(nb3d, nb2d)?;
        let mut inc_col: i32 = 0;
        for k in 1..=nb3d + nb2d {
            let at = |offset: i32| poles.value(inc_col + offset * npol + i).copied();
            if k <= nb3d {
                point.set_point(k, Point3::new(at(0)?, at(1)?, at(2)?))?;
                inc_col += 3 * npol;
            } else {
                point.set_point2d(k, Point2::new(at(0)?, at(1)?))?;
                inc_col += 2 * npol;
            }
        }
        Ok(point)
    }

    /// `IsDone()`; `resol.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// `ConstraintMatrix()`: the constraint matrix (changed by
    /// `ConstraintDerivative`, see there); `resol.constraint_matrix()`.
    pub fn constraint_matrix(&self) -> &Matrix<'static> {
        &self.cont
    }

    /// `InverseMatrix()`: the inverse of C tC from Uzawa; `resol.inverse_matrix()`.
    pub fn inverse_matrix(&self) -> &Matrix<'static> {
        &self.ctc_inv
    }

    /// `Duale()`: the dual variables of Uzawa; `resol.duale()`.
    pub fn duale(&self) -> &Vector<'static> {
        &self.vardua
    }
}
