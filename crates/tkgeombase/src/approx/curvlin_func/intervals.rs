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
// Rust translation of ModelingData/TKGeomBase/Approx/Approx_CurvlinFunc.cxx (NbIntervals, Intervals, Trim, Length)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The continuity breaks in S, the restriction to an S range and the lengths.
use super::{CurvlinFunc, CurvlinState, curve_on_surface, null_handle};
use crate::geom_lib::fuse_intervals;
use crate::native_failure::{array_failure, failure};
use rustycad_tkernel::ncollection::{Array1, LinearVector};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkmath::geom_abs::Shape;

/// `GeomLib::FuseIntervals` default confusion.
const FUSE_CONFUSION: f64 = 1.0e-9;

impl CurvlinFunc {
    /// `NbIntervals(S)`: the spans of continuity `shape` of the curve (the fused
    /// breaks of both curves in case 3); `f.nb_intervals(Shape::C2)?`.
    pub fn nb_intervals(&self, shape: Shape) -> Result<i32, Failure> {
        let st: CurvlinState = self.snapshot();
        match st.case {
            1 => st
                .c3d
                .get()
                .ok_or_else(|| null_handle("curve"))?
                .nb_intervals(shape),
            2 => curve_on_surface(&st.c2d1, &st.surf1)?.nb_intervals(shape),
            3 => Ok(fused_breaks(&st, shape)?.len() as i32 - 1),
            // "POP pour WNT": no other case exists.
            _ => Ok(1),
        }
    }

    /// `Intervals(T, S)`: the breaks as abscissae; `t(1..=Length())` is converted
    /// whatever its lower bound, as natively; `f.intervals(&mut t, Shape::C2)?`.
    pub fn intervals(&self, t: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
        let st: CurvlinState = self.snapshot();
        match st.case {
            1 => st
                .c3d
                .get()
                .ok_or_else(|| null_handle("curve"))?
                .intervals(t, shape)?,
            2 => curve_on_surface(&st.c2d1, &st.surf1)?.intervals(t, shape)?,
            3 => {
                let fusion: LinearVector<f64> = fused_breaks(&st, shape)?;
                for index in 0..fusion.len() {
                    let value: f64 = *fusion
                        .value(index)
                        .map_err(|e| array_failure(e, "Approx_CurvlinFunc::Intervals"))?;
                    *t.change_at(index)
                        .map_err(|e| array_failure(e, "Approx_CurvlinFunc::Intervals"))? = value;
                }
            }
            _ => {}
        }
        for i in 1..=t.length() {
            let u: f64 = *t
                .value(i)
                .map_err(|e| array_failure(e, "Approx_CurvlinFunc::Intervals"))?;
            let s: f64 = self.get_s_parameter(u)?;
            t.set_value(i, s)
                .map_err(|e| array_failure(e, "Approx_CurvlinFunc::Intervals"))?;
        }
        Ok(())
    }

    /// `Trim(First, Last, Tol)`: restricts the curves to the abscissae `[first, last]`
    /// (nothing for a range shorter than `tol`); Standard_OutOfRange outside `[0, 1]`.
    /// In case 3 the second curve is located with the tables of the first (native
    /// quirk); `f.trim(0.2, 0.8, 1e-7)?`.
    pub fn trim(&self, first: f64, last: f64, tol: f64) -> Result<(), Failure> {
        if first < 0.0 || last > 1.0 {
            return Err(failure(
                "Standard_OutOfRange",
                &format!("Approx_CurvlinFunc::Trim: [{first}, {last}]; expected within [0, 1]"),
            ));
        }
        if (last - first) < tol {
            return Ok(());
        }
        let st: CurvlinState = self.snapshot();
        match st.case {
            1 => self.trim_curve(&st, [first, last], tol)?,
            3 => {
                self.trim_on_surface(&st, 2, [first, last], tol)?;
                self.trim_on_surface(&st, 1, [first, last], tol)?;
            }
            2 => self.trim_on_surface(&st, 1, [first, last], tol)?,
            _ => {}
        }
        let mut w = self.state.write();
        (w.first_s, w.last_s) = (first, last);
        Ok(())
    }

    fn trim_curve(
        &self,
        st: &CurvlinState,
        [first, last]: [f64; 2],
        tol: f64,
    ) -> Result<(), Failure> {
        let c3d = st.c3d.get().ok_or_else(|| null_handle("curve"))?;
        let whole: Handle<dyn Adaptor3dCurve> = c3d.trim(st.first_u1, st.last_u1, tol)?;
        self.state.write().c3d = whole.clone();
        let whole: &dyn Adaptor3dCurve = whole.get().ok_or_else(|| null_handle("curve"))?;
        let first_u: f64 = self.get_u_parameter(whole, first, 1)?;
        let last_u: f64 = self.get_u_parameter(whole, last, 1)?;
        self.state.write().c3d = whole.trim(first_u, last_u, tol)?;
        Ok(())
    }

    // The curve `number` on its surface trimmed to its whole range, then to the
    // parameters of the abscissae (located with the tables of curve 1).
    fn trim_on_surface(
        &self,
        st: &CurvlinState,
        number: i32,
        [first, last]: [f64; 2],
        tol: f64,
    ) -> Result<(), Failure> {
        let (c2d, surf, range) = if number == 1 {
            (&st.c2d1, &st.surf1, [st.first_u1, st.last_u1])
        } else {
            (&st.c2d2, &st.surf2, [st.first_u2, st.last_u2])
        };
        let (c2d, surf) = trimmed_parts(&curve_on_surface(c2d, surf)?, range, tol)?;
        self.store_curve(number, (c2d.clone(), surf.clone()));
        let cons: Adaptor3dCurveOnSurface = curve_on_surface(&c2d, &surf)?;
        let first_u: f64 = self.get_u_parameter(&cons, first, 1)?;
        let last_u: f64 = self.get_u_parameter(&cons, last, 1)?;
        self.store_curve(number, trimmed_parts(&cons, [first_u, last_u], tol)?);
        Ok(())
    }

    fn store_curve(
        &self,
        number: i32,
        (c2d, surf): (Handle<dyn Adaptor2dCurve2d>, Handle<dyn Adaptor3dSurface>),
    ) {
        let mut w = self.state.write();
        if number == 1 {
            (w.c2d1, w.surf1) = (c2d, surf);
        } else {
            (w.c2d2, w.surf2) = (c2d, surf);
        }
    }

    /// `Length()`: recomputes the length (both lengths and their mean in case 3);
    /// `f.length()?`.
    pub fn length(&self) -> Result<(), Failure> {
        let st: CurvlinState = self.snapshot();
        let (length, length1, length2) = match st.case {
            1 => {
                let c3d = st.c3d.get().ok_or_else(|| null_handle("curve"))?;
                (self.whole_length(c3d)?, 0.0, 0.0)
            }
            2 => (
                self.whole_length(&curve_on_surface(&st.c2d1, &st.surf1)?)?,
                0.0,
                0.0,
            ),
            3 => {
                let l1: f64 = self.whole_length(&curve_on_surface(&st.c2d1, &st.surf1)?)?;
                let l2: f64 = self.whole_length(&curve_on_surface(&st.c2d2, &st.surf2)?)?;
                ((l1 + l2) / 2., l1, l2)
            }
            _ => (st.length, st.length1, st.length2),
        };
        let mut w = self.state.write();
        (w.length, w.length1, w.length2) = (length, length1, length2);
        Ok(())
    }

    fn whole_length(&self, c: &dyn Adaptor3dCurve) -> Result<f64, Failure> {
        let (first, last) = (c.first_parameter()?, c.last_parameter()?);
        self.length_of(c, first, last)
    }
}

// The 2D curve and surface of `cons.Trim(first, last, tol)` (a curve on surface).
fn trimmed_parts(
    cons: &Adaptor3dCurveOnSurface,
    [first, last]: [f64; 2],
    tol: f64,
) -> Result<(Handle<dyn Adaptor2dCurve2d>, Handle<dyn Adaptor3dSurface>), Failure> {
    let trimmed: Handle<Adaptor3dCurveOnSurface> = cons.trim(first, last, tol)?.cast();
    let trimmed = trimmed
        .get()
        .ok_or_else(|| null_handle("trimmed curve on surface"))?;
    Ok((trimmed.get_curve(), trimmed.get_surface()))
}

// The fused C-shape breaks of both curves on surfaces (case 3).
fn fused_breaks(st: &CurvlinState, shape: Shape) -> Result<LinearVector<f64>, Failure> {
    let breaks = |c2d: &Handle<dyn Adaptor2dCurve2d>, surf: &Handle<dyn Adaptor3dSurface>| {
        let cons = curve_on_surface(c2d, surf)?;
        let nb: i32 = cons.nb_intervals(shape)?;
        let mut t: Array1<'static, f64> = Array1::with_bounds(1, nb + 1)
            .map_err(|e| array_failure(e, "Approx_CurvlinFunc::NbIntervals"))?;
        cons.intervals(&mut t, shape)?;
        Ok::<_, Failure>(t)
    };
    let t1 = breaks(&st.c2d1, &st.surf1)?;
    let t2 = breaks(&st.c2d2, &st.surf2)?;
    let mut fusion: LinearVector<f64> = LinearVector::default();
    fuse_intervals(&t1, &t2, &mut fusion, FUSE_CONFUSION, false);
    Ok(fusion)
}
