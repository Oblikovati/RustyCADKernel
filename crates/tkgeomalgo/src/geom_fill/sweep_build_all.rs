// Copyright (c) 1997-1999 Matra Datavision
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
// Rust translation of GeomFill_Sweep::BuildAll of
// ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_Sweep.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The approximated sweep: the B-spline surface of `Approx_SweepApproximation` on
//! a `GeomFill_SweepFunction`, its 2d restrictions and traces, and the optional C1
//! re-approximation. The private `BuildProduct` (and its `GeomFill_Sweep_Eval`) is
//! called nowhere natively and is not ported.
use super::sweep::GeomFillSweep;
use super::sweep_function::GeomFillSweepFunction;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomSurface};
use rustycad_tkgeombase::approx::{SweepApproximation, SweepFunction, SweepOptions};
use rustycad_tkgeombase::geom_convert::ApproxSurface;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Direction2, Point2};

/// The 2d curves of a sweep (`myCurve2d`).
type Curves2d = Array1<'static, Handle<dyn Geom2dCurve>>;

/// The vertical 2d line at `u`, trimmed to `[v1, v2]`.
fn iso_restriction(u: f64, v1: f64, v2: f64) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let line: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::from_point(Point2::new(u, 0.0), Direction2::Y)).cast();
    Ok(Handle::new(Geom2dTrimmedCurve::new(&line, v1, v2, true, true)?).cast())
}

fn put_curve(
    curves: &mut Curves2d,
    index: i32,
    curve: Handle<dyn Geom2dCurve>,
) -> Result<(), Failure> {
    rustycad_tkg2d::geom2d::arrays::put(curves, index, curve)
}

fn put_errors(c_error: &mut Array2<'static, f64>, col: i32, u: f64, v: f64) -> Result<(), Failure> {
    for (row, value) in [(1, u), (2, v)] {
        c_error.set_value(row, col, value).map_err(|e| {
            super::sweep_failures::sweep_failure(
                "Standard_OutOfRange",
                &format!("GeomFill_Sweep: CError({row}, {col}): {e}; expected an index in the array bounds"),
            )
        })?;
    }
    Ok(())
}

impl GeomFillSweep {
    /// `BuildAll(Continuity, Degmax, Segmax)`: true when the approximation is done.
    pub(crate) fn build_all(
        &mut self,
        continuity: Shape,
        degmax: i32,
        segmax: i32,
    ) -> Result<bool, Failure> {
        let ratio = (self.s_last - self.s_first) / (self.last - self.first);
        let func: Handle<dyn SweepFunction> = Handle::new(GeomFillSweepFunction::new(
            &self.my_sec,
            &self.my_loc,
            self.first,
            self.s_first,
            ratio,
        ))
        .cast();
        let mut approx = SweepApproximation::new(&func);
        let options = SweepOptions {
            continuity,
            degmax,
            segmax,
            ..SweepOptions::new(self.tol3d, self.bound_tol, self.tol2d, self.tol_angular)
        };
        approx.perform(self.first, self.last, options)?;
        if !approx.is_done() {
            return Ok(false);
        }
        let surface = approx.surface()?;
        let bspline = GeomBSplineSurface::with_weights(
            &surface.poles,
            &surface.weights,
            (&surface.u_knots, &surface.u_mults),
            (&surface.v_knots, &surface.v_mults),
            (approx.u_degree()?, approx.v_degree()?),
            (self.sec()?.is_u_periodic()?, false),
        )?;
        self.my_surface = Handle::new(bspline).cast();
        self.s_error = approx.max_error_on_surf()?;
        if self.my_force_approx_c1 && !self.surface_ref()?.is_cn_v(1)? {
            self.force_c1()?;
        }
        if self.my_curve2d.is_none() {
            self.approximated_curves(&approx, &surface.u_knots)?;
        }
        Ok(true)
    }

    fn surface_ref(&self) -> Result<&dyn GeomSurface, Failure> {
        super::sweep_failures::live(&self.my_surface, "GeomFill_Sweep::BuildAll", "mySurface")
    }

    /// The C1 approximation of a surface that is not C1 in V, with its two
    /// restrictions at the first and last U knots.
    fn force_c1(&mut self) -> Result<(), Failure> {
        let the_tol = 1.0e-4;
        let convert = ApproxSurface::from_surface(
            &self.my_surface,
            the_tol,
            (Shape::C1, Shape::C1),
            (14, 14),
            16,
            1,
        )?;
        if !convert.has_result() {
            return Ok(());
        }
        let bspl = convert.surface();
        self.my_surface = bspl.clone().cast();
        let b = super::sweep_failures::live(&bspl, "GeomFill_Sweep::BuildAll", "BSplSurf")?;
        let mut curves: Curves2d = rustycad_tkg2d::geom2d::arrays::one_based(2)?;
        let mut c_error = error_table(2)?;
        let v_last = b.v_knot(b.nb_v_knots())?;
        put_curve(&mut curves, 1, iso_restriction(b.u_knot(1)?, 0.0, v_last)?)?;
        put_errors(&mut c_error, 1, 0.0, 0.0)?;
        let length = curves.len() as i32;
        put_curve(
            &mut curves,
            length,
            iso_restriction(b.u_knot(b.nb_u_knots())?, 0.0, v_last)?,
        )?;
        put_errors(&mut c_error, length, 0.0, 0.0)?;
        (self.my_curve2d, self.c_error) = (Some(curves), Some(c_error));
        self.s_error = the_tol;
        Ok(())
    }

    /// The traces and restrictions of the approximation, and the iso lines at the
    /// first and last U knots where the law has no restriction.
    fn approximated_curves(
        &mut self,
        approx: &SweepApproximation,
        u_knots: &Array1<'static, f64>,
    ) -> Result<(), Failure> {
        let loc = self.loc()?;
        let trace_number = loc.trace_number();
        let (has_first, has_last) = (loc.has_first_restriction(), loc.has_last_restriction());
        let mut curves: Curves2d = rustycad_tkg2d::geom2d::arrays::one_based(2 + trace_number)?;
        let mut c_error = error_table(2 + trace_number)?;
        let ideb = if has_first { 1 } else { 2 };
        let ifin = 1 + trace_number + i32::from(has_last);
        for (kk, ii) in (ideb..=ifin).enumerate() {
            let kk = kk as i32 + 1;
            let c = Geom2dBSplineCurve::new(
                approx.curve_2d_poles(kk)?,
                approx.curves_2d_knots()?,
                approx.curves_2d_mults()?,
                approx.curves_2d_degree()?,
                false,
            )?;
            put_curve(&mut curves, ii, Handle::new(c).cast())?;
            let error = approx.max_2d_error(kk)?;
            put_errors(&mut c_error, ii, error, error)?;
        }
        let (first, last) = (self.first, self.last);
        if !has_first {
            put_curve(
                &mut curves,
                1,
                iso_restriction(u_knots[u_knots.lower()], first, last)?,
            )?;
            put_errors(&mut c_error, 1, 0.0, 0.0)?;
        }
        if !has_last {
            let length = curves.len() as i32;
            put_curve(
                &mut curves,
                length,
                iso_restriction(u_knots[u_knots.upper()], first, last)?,
            )?;
            put_errors(&mut c_error, length, 0.0, 0.0)?;
        }
        (self.my_curve2d, self.c_error) = (Some(curves), Some(c_error));
        Ok(())
    }
}

/// `new NCollection_HArray2<double>(1, 2, 1, cols)` (its values are unset natively
/// until written; here 0).
fn error_table(cols: i32) -> Result<Array2<'static, f64>, Failure> {
    Array2::with_bounds(1, 2, 1, cols).map_err(|e| {
        super::sweep_failures::sweep_failure(
            "Standard_RangeError",
            &format!("NCollection_HArray2(1, 2, 1, {cols}): {e}; expected cols >= 1"),
        )
    })
}
