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
// Rust translation of ModelingData/TKGeomBase/AppParCurves/AppParCurves_Function.gxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The constrained branch of `AppParCurves_Function`: the Bezier fit corrected by
//! `AppParCurves_ResolConstraint`, and F on the corrected poles.
use super::Function;
use crate::app_par_curves::line_tool::LineTool;
use crate::app_par_curves::multi_curve::MultiCurve;
use crate::app_par_curves::resol_constraint::ResolConstraint;
use crate::native_failure::array_failure;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{Matrix, Vector};

/// The native default tolerance of the ResolCons constructor.
const RESOL_TOLERANCE: f64 = 1.0e-10;

/// The poles of the curves, one matrix per coordinate: `(1..=Npol, 1..=NbCu)` (Z is
/// 0 for a 2D curve): PTCX/PTCY/PTCZ (or PTCOX... before the correction).
pub(super) type PoleColumns = [Matrix<'static>; 3];

impl<L: LineTool + Clone> Function<L> {
    // The poles of `curve`, one column per curve.
    pub(super) fn pole_columns(&self, curve: &MultiCurve) -> Result<PoleColumns, Failure> {
        let npol: i32 = self.degre + 1;
        let new = || {
            Matrix::filled(1, npol, 1, self.nb_cu, 0.)
                .map_err(|e| array_failure(e, "AppParCurves_Function"))
        };
        let mut columns: PoleColumns = [new()?, new()?, new()?];
        for ci in 1..=self.nb_cu {
            for j in 1..=npol {
                let pole = curve.value(j)?;
                let xyz: [f64; 3] = if self.is_3d(ci) {
                    let p = pole.point(ci)?;
                    [p.x(), p.y(), p.z()]
                } else {
                    let p = pole.point2d(ci)?;
                    [p.x(), p.y(), 0.]
                };
                (0..3).for_each(|c| columns[c][(j, ci)] = xyz[c]);
            }
        }
        Ok(columns)
    }

    pub(super) fn is_3d(&self, ci: i32) -> bool {
        self.targets.tabdim[(ci - 1) as usize] == 3
    }

    // ResolCons on MyMultiCurve (which shares its poles with the fit's curve, so both
    // are corrected); None when Uzawa fails.
    pub(super) fn resolve(&mut self, da: &Matrix<'_>) -> Result<Option<ResolConstraint>, Failure> {
        let resol = ResolConstraint::new(
            &self.multi_line,
            &mut self.multi_curve,
            self.first_p,
            self.last_p,
            &self.constraints,
            &self.a,
            da,
            RESOL_TOLERANCE,
        )?;
        if !resol.is_done() {
            self.done = false;
            return Ok(None);
        }
        Ok(Some(resol))
    }

    /// The constrained `Value`: F and the largest distances on the corrected poles.
    pub(super) fn constrained_value(&mut self) -> Result<bool, Failure> {
        self.err3d = 0.;
        self.err2d = 0.;
        self.multi_curve = self.least_square.bezier_value()?;
        self.a.initialized(self.least_square.function_matrix()?)?;
        let da: Matrix<'static> = self.least_square.derivative_function_matrix()?.clone();
        if self.resolve(&da)?.is_none() {
            return Ok(false);
        }
        self.fval = 0.;
        let poles: PoleColumns = self.pole_columns(&self.multi_curve.clone())?;
        for ci in 1..=self.nb_cu {
            for i in self.targets.adeb..=self.targets.afin {
                self.fval += self.point_distance(&poles, i, ci, None)?.0;
            }
        }
        Ok(true)
    }

    /// MyF(i, Ci) and the largest distances; with `da`, the unconstrained gradient
    /// term `2 (dC . (C - P))` too. Returns `(Fi, gradient term)`.
    pub(super) fn point_distance(
        &mut self,
        poles: &PoleColumns,
        i: i32,
        ci: i32,
        da: Option<&Matrix<'_>>,
    ) -> Result<(f64, f64), Failure> {
        let three: bool = self.is_3d(ci);
        let (mut value, mut derivative) = ([0.; 3], [0.; 3]);
        for j in 1..=self.degre + 1 {
            let aij: f64 = *self.a.value(i, j)?;
            let daij: f64 = match da {
                Some(da) => *da.value(i, j)?,
                None => 0.,
            };
            for c in 0..if three { 3 } else { 2 } {
                value[c] += aij * poles[c][(j, ci)];
                derivative[c] += daij * poles[c][(j, ci)];
            }
        }
        let fx: f64 = value[0] - self.targets.ptl[0][(i, ci)];
        let fy: f64 = value[1] - self.targets.ptl[1][(i, ci)];
        *self.my_f.change_value(i, ci)? = fx * fx + fy * fy;
        let mut gradient: f64 = 2.0 * (derivative[0] * fx + derivative[1] * fy);
        if three {
            let fz: f64 = value[2] - self.targets.ptl[2][(i, ci)];
            self.my_f[(i, ci)] += fz * fz;
            gradient += 2.0 * derivative[2] * fz;
        }
        let fi: f64 = self.my_f[(i, ci)];
        let maximum: &mut f64 = if three {
            &mut self.err3d
        } else {
            &mut self.err2d
        };
        if fi.sqrt() > *maximum {
            *maximum = fi.sqrt();
        }
        Ok((fi, gradient))
    }

    /// The constrained `Perform`: the gradient of F with the derivative of the
    /// Uzawa correction.
    pub(super) fn constrained_gradient(&mut self, x: &Vector<'_>) -> Result<(), Failure> {
        self.err3d = 0.;
        self.err2d = 0.;
        self.multi_curve = self.least_square.bezier_value()?;
        let original: PoleColumns = self.pole_columns(&self.multi_curve.clone())?;
        self.a.initialized(self.least_square.function_matrix()?)?;
        self.da
            .initialized(self.least_square.derivative_function_matrix()?)?;
        let da: Matrix<'static> = self.da.clone();
        let Some(mut resol) = self.resolve(&da)? else {
            return Ok(());
        };
        self.fval = 0.;
        for j in self.first_p..=self.last_p {
            *self.val_grad_f.change_value(j)? = 0.;
        }
        let dk: Matrix<'static> = resol
            .constraint_derivative(&self.multi_line, x, self.degre, &da)?
            .clone();
        let corrected: PoleColumns = self.pole_columns(&self.multi_curve.clone())?;
        for ci in 1..=self.nb_cu {
            for i in self.targets.adeb..=self.targets.afin {
                let (fi, gradient) = self.point_distance(&corrected, i, ci, Some(&da))?;
                self.fval += fi;
                *self.val_grad_f.change_value(i)? += gradient;
            }
        }
        self.correction_gradient(&resol, &dk, [&original, &corrected])
    }
}
