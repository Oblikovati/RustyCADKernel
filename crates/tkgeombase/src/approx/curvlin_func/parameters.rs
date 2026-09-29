// Copyright (c) 1998-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/Approx/Approx_CurvlinFunc.cxx (Init, Length, GetUParameter, GetSParameter, cubic, findfourpoints)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The tables of the curvilinear parametrization and the conversions between the
//! curve parameter U and the abscissa S.
use super::{CurvlinFunc, CurvlinState, curve_on_surface, null_handle, reload};
use crate::gcpnts::AbscissaPoint;
use crate::native_failure::{array_failure, failure};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurveOnSurface;
use rustycad_tkmath::geom_abs::Shape;

/// `NbInt`: the samples of each C3 span in the tables.
const SAMPLES_PER_SPAN: i32 = 10;

/// `cubic(X, Xi, Yi)`: the Newton form of the cubic through the four points.
fn cubic(x: f64, xi: &[f64; 4], yi: &[f64; 4]) -> f64 {
    let i1: f64 = (yi[0] - yi[1]) / (xi[0] - xi[1]);
    let i2: f64 = (yi[1] - yi[2]) / (xi[1] - xi[2]);
    let i3: f64 = (yi[2] - yi[3]) / (xi[2] - xi[3]);
    let i21: f64 = (i1 - i2) / (xi[0] - xi[2]);
    let i22: f64 = (i2 - i3) / (xi[1] - xi[3]);
    let i31: f64 = (i21 - i22) / (xi[0] - xi[3]);
    yi[0] + (x - xi[0]) * (i1 + (x - xi[1]) * (i21 + (x - xi[2]) * i31))
}

fn table_value(table: &Array1<'static, f64>, index: i32) -> Result<f64, Failure> {
    table
        .value(index)
        .copied()
        .map_err(|e| array_failure(e, "Approx_CurvlinFunc"))
}

/// `findfourpoints`: four table points around interval `n_interval` (clamped to
/// `1..=NbInt-2`), with `(prevS, prevU)` inserted when it falls strictly inside.
fn four_points(
    mut n_interval: i32,
    [si, ui]: [&Array1<'static, f64>; 2],
    (prev_s, prev_u): (f64, f64),
) -> Result<([f64; 4], [f64; 4]), Failure> {
    let nb_int: i32 = si.length() - 1;
    if nb_int < 3 {
        return Err(failure(
            "Standard_ConstructionError",
            &format!("Approx_CurvlinFunc::GetUParameter: {nb_int} intervals; expected at least 3"),
        ));
    }
    n_interval = n_interval.max(1).min(nb_int - 2);
    let (mut xi, mut yi) = ([0.0; 4], [0.0; 4]);
    for i in 0..4 {
        xi[i] = table_value(si, n_interval - 1 + i as i32)?;
        yi[i] = table_value(ui, n_interval - 1 + i as i32)?;
    }
    for i in 0..3 {
        if xi[i] < prev_s && prev_s < xi[i + 1] {
            for j in 0..i {
                (xi[j], yi[j]) = (xi[j + 1], yi[j + 1]);
            }
            (xi[i], yi[i]) = (prev_s, prev_u);
            break;
        }
    }
    Ok((xi, yi))
}

/// `Length(C, FirstU, LastU)` with the stored tolerance.
fn curve_length(c: &dyn Adaptor3dCurve, [u1, u2]: [f64; 2], tol: f64) -> Result<f64, Failure> {
    AbscissaPoint::length_between_with_tolerance(c, u1, u2, tol)
}

impl CurvlinFunc {
    /// The private `Init()`: the tables of each curve, then Length().
    pub(super) fn init(&self) -> Result<(), Failure> {
        let st: CurvlinState = self.snapshot();
        match st.case {
            1 => {
                let c3d = st.c3d.get().ok_or_else(|| null_handle("curve"))?;
                self.init_tables(c3d, 1)?;
                let mut w = self.state.write();
                (w.first_u1, w.last_u1) = (c3d.first_parameter()?, c3d.last_parameter()?);
                (w.first_u2, w.last_u2) = (0.0, 0.0);
            }
            2 => {
                let cons = curve_on_surface(&st.c2d1, &st.surf1)?;
                let first = self.init_curve_on_surface(&cons, 1)?;
                let mut w = self.state.write();
                (w.first_u1, w.last_u1) = first;
                (w.first_u2, w.last_u2) = (0.0, 0.0);
            }
            3 => {
                // One adaptor for both curves, as natively: its interval cache is not
                // reset by Load, so the second curve may read the breaks of the first.
                let cons = curve_on_surface(&st.c2d1, &st.surf1)?;
                let first = self.init_curve_on_surface(&cons, 1)?;
                reload(&cons, &st.c2d2, &st.surf2)?;
                let second = self.init_curve_on_surface(&cons, 2)?;
                let mut w = self.state.write();
                ((w.first_u1, w.last_u1), (w.first_u2, w.last_u2)) = (first, second);
            }
            _ => {}
        }
        self.length()
    }

    fn init_curve_on_surface(
        &self,
        cons: &Adaptor3dCurveOnSurface,
        number: i32,
    ) -> Result<(f64, f64), Failure> {
        self.init_tables(cons, number)?;
        Ok((cons.first_parameter()?, cons.last_parameter()?))
    }

    /// `Init(C, Si, Ui)`: ten equal parameter steps on each C3 span of `c`, with the
    /// normalized length at each; the last (S, U) becomes the start of the curve.
    fn init_tables(&self, c: &dyn Adaptor3dCurve, number: i32) -> Result<(), Failure> {
        let (first_u, last_u) = (c.first_parameter()?, c.last_parameter()?);
        let tol: f64 = self.state.read().tol_len;
        let nb_int_c3: i32 = c.nb_intervals(Shape::C3)?;
        let mut disc: Array1<'static, f64> = Array1::with_bounds(1, nb_int_c3 + 1)
            .map_err(|e| array_failure(e, "Approx_CurvlinFunc::Init"))?;
        if nb_int_c3 > 1 {
            c.intervals(&mut disc, Shape::C3)?;
        } else {
            disc.set_value(1, first_u)
                .map_err(|e| array_failure(e, "Approx_CurvlinFunc::Init"))?;
            disc.set_value(2, last_u)
                .map_err(|e| array_failure(e, "Approx_CurvlinFunc::Init"))?;
        }
        let upper: i32 = nb_int_c3 * SAMPLES_PER_SPAN;
        let alloc = || {
            Array1::with_bounds(0, upper).map_err(|e| array_failure(e, "Approx_CurvlinFunc::Init"))
        };
        let (mut ui, mut si): (Array1<'static, f64>, Array1<'static, f64>) = (alloc()?, alloc()?);
        (ui[0], si[0]) = (first_u, 0.0);
        let mut i: i32 = 1;
        for j in 1..=nb_int_c3 {
            let step: f64 = (disc[j + 1] - disc[j]) / f64::from(SAMPLES_PER_SPAN);
            for _ in 1..=SAMPLES_PER_SPAN {
                ui[i] = ui[i - 1] + step;
                si[i] = si[i - 1] + curve_length(c, [ui[i - 1], ui[i]], tol)?;
                i += 1;
            }
        }
        let len: f64 = si[upper];
        for k in 0..=upper {
            si[k] /= len;
        }
        let mut w = self.state.write();
        if number == 1 {
            (w.ui_1, w.si_1) = (Some(ui), Some(si));
        } else {
            (w.ui_2, w.si_2) = (Some(ui), Some(si));
        }
        (w.prev_s, w.prev_u) = (w.first_s, first_u);
        Ok(())
    }

    /// `Length(C, FirstU, LastU)`: the length of `c` between two parameters, within
    /// the stored tolerance; `f.length_of(&*adaptor, 0.0, 1.0)?`.
    pub fn length_of(
        &self,
        c: &dyn Adaptor3dCurve,
        first_u: f64,
        last_u: f64,
    ) -> Result<f64, Failure> {
        curve_length(c, [first_u, last_u], self.state.read().tol_len)
    }

    /// `GetUParameter(C, S, NumberOfCurve)`: the parameter of `c` at the abscissa
    /// `s`, found by GCPnts_AbscissaPoint from the table point before it, started at a
    /// cubic guess through four table points (and the last point found). Uses the
    /// tables of curve `number_of_curve` whatever `c` is; Standard_ConstructionError
    /// outside `[0, 1]` or with fewer than three table intervals;
    /// `f.get_u_parameter(&*adaptor, 0.5, 1)?`.
    pub fn get_u_parameter(
        &self,
        c: &dyn Adaptor3dCurve,
        s: f64,
        number_of_curve: i32,
    ) -> Result<f64, Failure> {
        if s < 0.0 || s > 1.0 {
            return Err(failure(
                "Standard_ConstructionError",
                &format!("Approx_CurvlinFunc::GetUParameter: S = {s}; expected 0 <= S <= 1"),
            ));
        }
        let st: CurvlinState = self.snapshot();
        let (ui, si, length) = if number_of_curve == 1 {
            let length: f64 = if st.case == 3 { st.length1 } else { st.length };
            (&st.ui_1, &st.si_1, length)
        } else {
            (&st.ui_2, &st.si_2, st.length2)
        };
        let ui: &Array1<'static, f64> = ui.as_ref().ok_or_else(|| null_handle("Ui table"))?;
        let si: &Array1<'static, f64> = si.as_ref().ok_or_else(|| null_handle("Si table"))?;
        let n_interval: i32 = interval_of(si, s)?;
        if s == table_value(si, n_interval)? {
            return table_value(ui, n_interval);
        }
        if s == table_value(si, n_interval + 1)? {
            return table_value(ui, n_interval + 1);
        }
        let base: f64 = table_value(ui, n_interval)?;
        let delta_s: f64 = (s - table_value(si, n_interval)?) * length;
        let (xi, yi) = four_points(n_interval, [si, ui], (st.prev_s, st.prev_u))?;
        let guess: f64 = cubic(s, &xi, &yi);
        let u: f64 = AbscissaPoint::from_start_with_tolerance(c, delta_s, base, guess, st.tol_len)?
            .parameter()?;
        let mut w = self.state.write();
        (w.prev_s, w.prev_u) = (s, u);
        Ok(u)
    }

    /// `GetSParameter(U)`: the abscissa of the parameter U (the mean of both curves in
    /// case 3); `f.get_s_parameter(0.3)?`.
    pub fn get_s_parameter(&self, u: f64) -> Result<f64, Failure> {
        let st: CurvlinState = self.snapshot();
        match st.case {
            1 => {
                let c3d = st.c3d.get().ok_or_else(|| null_handle("curve"))?;
                self.s_parameter_of(c3d, u, st.length)
            }
            2 => self.s_parameter_of(&curve_on_surface(&st.c2d1, &st.surf1)?, u, st.length),
            3 => {
                let cons = curve_on_surface(&st.c2d1, &st.surf1)?;
                let s1: f64 = self.s_parameter_of(&cons, u, st.length1)?;
                reload(&cons, &st.c2d2, &st.surf2)?;
                let s2: f64 = self.s_parameter_of(&cons, u, st.length2)?;
                Ok((s1 + s2) / 2.)
            }
            _ => Ok(0.0),
        }
    }

    /// The private `GetSParameter(C, U, Len)`: `FirstS + Length(C, First, U) / Len`.
    fn s_parameter_of(&self, c: &dyn Adaptor3dCurve, u: f64, len: f64) -> Result<f64, Failure> {
        let origin: f64 = c.first_parameter()?;
        Ok(self.first_parameter() + self.length_of(c, origin, u)? / len)
    }
}

// The table interval `[Si(i), Si(i+1))` of `s` (the last one for S = 1, NbInt when
// no interval holds `s`).
fn interval_of(si: &Array1<'static, f64>, s: f64) -> Result<i32, Failure> {
    let nb_int: i32 = si.length() - 1;
    if s == 1.0 {
        return Ok(nb_int - 1);
    }
    let mut i: i32 = 0;
    while i < nb_int {
        if table_value(si, i)? <= s && s < table_value(si, i + 1)? {
            break;
        }
        i += 1;
    }
    Ok(i)
}
