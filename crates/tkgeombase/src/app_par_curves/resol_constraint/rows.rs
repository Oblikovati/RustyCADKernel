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

//! The constraint rows of `AppParCurves_ResolConstraint`: the pass points, the
//! tangency collinearity rows of each curve, and the rows that give every curve the
//! same tangency ratio, with the right-hand side and the Uzawa start point.
use super::ResolConstraint;
use crate::app_par_curves::arrays::scratch;
use crate::app_par_curves::line_tool::LineTool;
use crate::app_par_curves::multi_curve::{MultiCurve, MultiCurveEval};
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};
use rustycad_tkmath::math::{Matrix, Vector};

/// The function-level `T1`, `T2`, `T3` and `IP` of the native constructor: their
/// values carry over from one loop to the next where a case does not assign them.
#[derive(Clone, Copy, Default)]
pub(super) struct Tangent {
    pub(super) t1: f64,
    pub(super) t2: f64,
    pub(super) t3: f64,
    pub(super) ip: i32,
}

/// `Ibont(k, i)`: 1, 2 or 3, the largest coordinate of tangent i of curve k.
pub(super) fn largest(t: &Tangent, dimension3: bool) -> (i32, f64) {
    let (mut code, mut tmax) = (1, t.t1.abs());
    if t.t2.abs() > tmax {
        (code, tmax) = (2, t.t2.abs());
    }
    if dimension3 && t.t3.abs() > tmax {
        (code, tmax) = (3, t.t3.abs());
    }
    (code, tmax)
}

/// The three rows of one 3D collinearity pair `(Inc3, Inc3 + 1)`, as the columns
/// offsets `(row, block)` and the numerators, for `Ibont` 1, 2 or 3.
pub(super) fn collinearity_cells(code: i32, t: &Tangent) -> [(i32, i32, f64); 4] {
    match code {
        3 => [(0, 1, t.t3), (0, 2, -t.t2), (1, 0, t.t3), (1, 2, -t.t1)],
        1 => [(0, 0, t.t3), (0, 2, -t.t1), (1, 0, t.t2), (1, 1, -t.t1)],
        _ => [(0, 1, t.t3), (0, 2, -t.t2), (1, 0, t.t2), (1, 1, -t.t1)],
    }
}

// Cont(row, col) = value.
pub(super) fn put(m: &mut Matrix<'_>, row: i32, col: i32, value: f64) -> Result<(), Failure> {
    *m.change_value(row, col)? = value;
    Ok(())
}

impl ResolConstraint {
    // Cont, Secont and the start point Start (returned).
    pub(super) fn fill_system(
        &mut self,
        ssp: &impl LineTool,
        scurv: &MultiCurve,
        bern: &Matrix<'_>,
        derivative_bern: &Matrix<'_>,
    ) -> Result<Vector<'static>, Failure> {
        let (nb3d, nb2d) = (ssp.nb_p3d(), ssp.nb_p2d());
        let npol: i32 = scurv.nb_poles();
        let ccol: i32 = nb3d * 3 + nb2d * 2;
        self.pass_rows(bern, ccol, npol)?;
        let mut start: Vector<'static> = Vector::filled(1, ccol * npol, 0.)?;
        let mut ibont: Array2<'static, i32> = Array2::with_bounds(1, nb3d + nb2d, 1, self.inc_tan)
            .map_err(|error| {
                crate::native_failure::array_failure(error, "AppParCurves_ResolConstraint")
            })?;
        let mut tangent: Tangent = Tangent::default();
        let mut inc3: i32 = ccol * self.inc_pass + 1;
        let (mut inc_col, mut inc_sec) = (0, 0);
        let (mut tab_v, mut tab_v2d) = (scratch::<Vector3>(nb3d)?, scratch::<Vector2>(nb2d)?);
        for k in 1..=nb3d + nb2d {
            let three: bool = k <= nb3d;
            for i in 1..=self.inc_tan {
                let npt: i32 = self.i_tan[(i - 1) as usize];
                if three {
                    ssp.tangency_3d(npt, &mut tab_v)?;
                    let v: Vector3 = *tab_v.value(k).map_err(out_of_range)?;
                    (tangent.t1, tangent.t2, tangent.t3) = (v.x(), v.y(), v.z());
                } else {
                    ssp.tangency_2d(npt, &mut tab_v2d)?;
                    let v: Vector2 = *tab_v2d.value(k - nb3d).map_err(out_of_range)?;
                    (tangent.t1, tangent.t2) = (v.x(), v.y());
                }
                let (code, tmax) = largest(&tangent, three);
                ibont[(k, i)] = code;
                inc3 = self.tangency_rows(
                    derivative_bern,
                    [npt, npol, inc3, inc_col],
                    &tangent,
                    [code, three as i32],
                    tmax,
                )?;
            }
            self.second_member(ssp, k, three, &mut inc_sec)?;
            self.start_point(scurv, k, three, &mut start, inc_col)?;
            inc_col += if three { 3 * npol } else { 2 * npol };
        }
        let mut tabs = super::ratio::TangentTables { tab_v, tab_v2d };
        self.ratio_rows(
            ssp,
            derivative_bern,
            &ibont,
            [nb3d, npol, inc3 - 1],
            &mut tangent,
            &mut tabs,
        )?;
        Ok(start)
    }

    // The pass-point rows: the Bernstein rows of the pass points, once per coordinate.
    fn pass_rows(&mut self, bern: &Matrix<'_>, ccol: i32, npol: i32) -> Result<(), Failure> {
        let mut cont_init: Matrix<'static> = Matrix::filled(1, self.inc_pass, 1, npol, 0.)
            .map_err(|error| {
                crate::native_failure::array_failure(error, "AppParCurves_ResolConstraint")
            })?;
        for (i, npt) in (1..).zip(self.i_pas.clone()) {
            for j in 1..=npol {
                cont_init[(i, j)] = *bern.value(npt, j)?;
            }
        }
        let pass: i32 = self.inc_pass;
        for i in 1..=ccol {
            self.cont.set(
                pass * (i - 1) + 1,
                pass * i,
                npol * (i - 1) + 1,
                npol * i,
                &cont_init,
            )?;
        }
        Ok(())
    }

    // The collinearity rows of one tangent: two for a 3D curve (divided by Tmax), one
    // for a 2D curve; returns the next row.
    fn tangency_rows(
        &mut self,
        derivative_bern: &Matrix<'_>,
        [npt, npol, inc3, inc_col]: [i32; 4],
        t: &Tangent,
        [code, three]: [i32; 2],
        tmax: f64,
    ) -> Result<i32, Failure> {
        for j in 1..=npol {
            let daij: f64 = *derivative_bern.value(npt, j)?;
            if three == 1 {
                for (row, block, numerator) in collinearity_cells(code, t) {
                    put(
                        &mut self.cont,
                        inc3 + row,
                        j + block * npol + inc_col,
                        scaled(daij, numerator, tmax),
                    )?;
                }
            } else {
                put(&mut self.cont, inc3, j + inc_col, daij * t.t2)?;
                put(&mut self.cont, inc3, j + npol + inc_col, -daij * t.t1)?;
            }
        }
        Ok(inc3 + if three == 1 { 2 } else { 1 })
    }

    // The right-hand side: the coordinates of the pass points of curve k.
    fn second_member(
        &mut self,
        ssp: &impl LineTool,
        k: i32,
        three: bool,
        inc_sec: &mut i32,
    ) -> Result<(), Failure> {
        let (nb3d, nb2d, pass) = (ssp.nb_p3d(), ssp.nb_p2d(), self.inc_pass);
        let (mut tab_p, mut tab_p2d) = (scratch::<Point3>(nb3d)?, scratch::<Point2>(nb2d)?);
        for i in 1..=pass {
            let npt: i32 = self.i_pas[(i - 1) as usize];
            if three {
                ssp.value_3d(npt, &mut tab_p)?;
                let p: Point3 = *tab_p.value(k).map_err(out_of_range)?;
                self.secont[i + *inc_sec] = p.x();
                self.secont[i + pass + *inc_sec] = p.y();
                self.secont[i + 2 * pass + *inc_sec] = p.z();
            } else {
                ssp.value_2d(npt, &mut tab_p2d)?;
                // The native index is the pass-point index minus nb3d, not the curve.
                let p: Point2 = *tab_p2d.value(i - nb3d).map_err(out_of_range)?;
                self.secont[i + *inc_sec] = p.x();
                self.secont[i + pass + *inc_sec] = p.y();
            }
        }
        *inc_sec += if three { 3 * pass } else { 2 * pass };
        Ok(())
    }

    // The Uzawa start point: the poles of curve k in `scurv`.
    fn start_point(
        &self,
        scurv: &MultiCurve,
        k: i32,
        three: bool,
        start: &mut Vector<'_>,
        inc_col: i32,
    ) -> Result<(), Failure> {
        let npol: i32 = scurv.nb_poles();
        for j in 1..=npol {
            if three {
                let p: Point3 = scurv.value(j)?.point(k)?;
                (
                    start[j + inc_col],
                    start[j + npol + inc_col],
                    start[j + 2 * npol + inc_col],
                ) = (p.x(), p.y(), p.z());
            } else {
                let p: Point2 = scurv.value(j)?.point2d(k)?;
                (start[j + inc_col], start[j + npol + inc_col]) = (p.x(), p.y());
            }
        }
        Ok(())
    }
}

// `Daij * T / Tmax` (a negative numerator is `-Daij * T / Tmax` natively: the sign
// goes on Daij, which gives the same IEEE result).
pub(super) fn scaled(daij: f64, numerator: f64, tmax: f64) -> f64 {
    daij * numerator / tmax
}

pub(super) fn out_of_range(error: rustycad_tkernel::ncollection::ArrayError) -> Failure {
    crate::native_failure::array_failure(error, "AppParCurves_ResolConstraint")
}
