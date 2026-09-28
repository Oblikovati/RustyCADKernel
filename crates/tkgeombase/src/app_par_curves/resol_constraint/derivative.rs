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

//! `AppParCurves_ResolConstraint::ConstraintDerivative`: the derivative of the
//! constraint matrix along the parameters.
use super::ResolConstraint;
use super::rows::{Tangent, collinearity_cells, largest, out_of_range, put, scaled};
use crate::app_par_curves::arrays::scratch;
use crate::app_par_curves::bernstein::second_derivative_bernstein;
use crate::app_par_curves::line_tool::LineTool;
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Vector2, Vector3};
use rustycad_tkmath::math::{Matrix, Vector};

impl ResolConstraint {
    /// `ConstraintDerivative(SSP, Parameters, Deg, DA)`: DeCont, from `da` on the pass
    /// points and the second Bernstein derivatives on the tangency points.
    ///
    /// As natively, the 2D tangents are read at the curve index (not the index minus
    /// the 3D curves), the 2D curves do not advance the column offset, and the
    /// tangency-ratio rows are written into the *constraint* matrix (Cont), which
    /// ConstraintMatrix() then returns; `resol.constraint_derivative(&line, &u, 3, &da)?`.
    pub fn constraint_derivative(
        &mut self,
        ssp: &impl LineTool,
        parameters: &Vector<'_>,
        deg: i32,
        da: &Matrix<'_>,
    ) -> Result<&Matrix<'static>, Failure> {
        let (nb3d, nb2d) = (ssp.nb_p3d(), ssp.nb_p2d());
        let (npol, ccol) = (deg + 1, nb3d * 3 + nb2d * 2);
        let mut ibont: Array2<'static, i32> = Array2::with_bounds(1, nb3d + nb2d, 1, self.inc_tan)
            .map_err(|error| array_failure(error, "AppParCurves_ResolConstraint"))?;
        let mut dda: Vector<'static> = Vector::filled(1, npol, 0.)?;
        let (mut tab_v, mut tab_v2d) = (scratch::<Vector3>(nb3d)?, scratch::<Vector2>(nb2d)?);
        self.de_cont.init(0.);
        self.pass_derivatives(da, ccol, npol)?;
        let mut t: Tangent = Tangent::default();
        let mut inc3: i32 = ccol * self.inc_pass + 1;
        let mut inc_col: i32 = 0;
        for k in 1..=nb3d + nb2d {
            let three: bool = k <= nb3d;
            for i in 1..=self.inc_tan {
                let npt: i32 = self.i_tan[(i - 1) as usize];
                if three {
                    ssp.tangency_3d(npt, &mut tab_v)?;
                    let v: Vector3 = *tab_v.value(k).map_err(out_of_range)?;
                    (t.t1, t.t2, t.t3) = (v.x(), v.y(), v.z());
                    let (code, tmax) = largest(&t, true);
                    ibont[(k, i)] = code;
                    second_derivative_bernstein(*parameters.value(npt)?, &mut dda)?;
                    for j in 1..=npol {
                        for (row, block, numerator) in collinearity_cells(code, &t) {
                            put(
                                &mut self.de_cont,
                                inc3 + row,
                                j + block * npol + inc_col,
                                scaled(dda[j], numerator, tmax),
                            )?;
                        }
                    }
                    inc3 += 2;
                } else {
                    second_derivative_bernstein(*parameters.value(npt)?, &mut dda)?;
                    ssp.tangency_2d(npt, &mut tab_v2d)?;
                    let v: Vector2 = *tab_v2d.value(k).map_err(out_of_range)?;
                    (t.t1, t.t2) = (v.x(), v.y());
                    ibont[(k, i)] = largest(&t, false).0;
                    for j in 1..=npol {
                        put(&mut self.de_cont, inc3, j + inc_col, dda[j] * t.t2)?;
                        put(&mut self.de_cont, inc3, j + npol + inc_col, -dda[j] * t.t1)?;
                    }
                    inc3 += 1;
                }
            }
            if three {
                inc_col += 3 * npol;
            }
        }
        let mut tables = (tab_v, tab_v2d);
        self.ratio_derivatives(ssp, parameters, &ibont, [nb3d, npol, inc3 - 1], &mut tables)?;
        Ok(&self.de_cont)
    }

    // DeCont's pass-point rows: DA on the pass points, once per coordinate.
    fn pass_derivatives(&mut self, da: &Matrix<'_>, ccol: i32, npol: i32) -> Result<(), Failure> {
        let mut init: Matrix<'static> = Matrix::filled(1, self.inc_pass, 1, npol, 0.)
            .map_err(|error| array_failure(error, "AppParCurves_ResolConstraint"))?;
        for (i, npt) in (1..).zip(self.i_pas.clone()) {
            for j in 1..=npol {
                init[(i, j)] = *da.value(npt, j)?;
            }
        }
        let pass: i32 = self.inc_pass;
        for i in 1..=ccol {
            self.de_cont.set(
                pass * (i - 1) + 1,
                pass * i,
                npol * (i - 1) + 1,
                npol * i,
                &init,
            )?;
        }
        Ok(())
    }

    // The tangency-ratio rows with the second derivatives, written into Cont.
    fn ratio_derivatives(
        &mut self,
        ssp: &impl LineTool,
        parameters: &Vector<'_>,
        ibont: &Array2<'_, i32>,
        [nb3d, npol, mut inc3]: [i32; 3],
        (tab_v, tab_v2d): &mut (Array1<'static, Vector3>, Array1<'static, Vector2>),
    ) -> Result<(), Failure> {
        let nb_cu: i32 = ssp.nb_p3d() + ssp.nb_p2d();
        let mut dda: Vector<'static> = Vector::filled(1, npol, 0.)?;
        for i in 1..=self.inc_tan {
            let npt: i32 = self.i_tan[(i - 1) as usize];
            second_derivative_bernstein(*parameters.value(npt)?, &mut dda)?;
            let mut inc_col: i32 = 0;
            for k in 1..nb_cu {
                inc3 += 1;
                let (code1, code2) = (ibont[(k, i)], ibont[(k + 1, i)]);
                // The 2D tangents are read at index k (and k + 1), as natively.
                let read = |tab_v: &mut Array1<'static, Vector3>,
                            tab_v2d: &mut Array1<'static, Vector2>,
                            curve: i32,
                            code: i32|
                 -> Result<f64, Failure> {
                    if curve <= nb3d || code == 3 {
                        ssp.tangency_3d(npt, tab_v)?;
                        let v: Vector3 = *tab_v.value(curve).map_err(out_of_range)?;
                        return Ok([v.x(), v.y(), v.z()][(code - 1) as usize]);
                    }
                    ssp.tangency_2d(npt, tab_v2d)?;
                    let v: Vector2 = *tab_v2d.value(curve).map_err(out_of_range)?;
                    Ok([v.x(), v.y()][(code - 1) as usize])
                };
                let t1: f64 = read(tab_v, tab_v2d, k, code1)?;
                let ip: i32 = if k <= nb3d || code1 == 3 {
                    3 * npol
                } else {
                    2 * npol
                };
                let t2: f64 = read(tab_v, tab_v2d, k + 1, code2)?;
                let (add1, add2) = ((code1 - 1) * npol, (code2 - 1) * npol);
                for j in 1..=npol {
                    put(&mut self.cont, inc3, j + add1 + inc_col, dda[j] * t2)?;
                    put(&mut self.cont, inc3, j + ip + add2 + inc_col, -dda[j] * t1)?;
                }
                inc_col += ip;
            }
        }
        Ok(())
    }
}
