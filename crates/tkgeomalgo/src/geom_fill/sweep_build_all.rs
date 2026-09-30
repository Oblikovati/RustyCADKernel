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

/// The restrictions and traces of a sweep with their errors (`myCurve2d`, `CError`).
struct RestrictionTable {
    curves: Curves2d,
    c_error: Array2<'static, f64>,
}

impl RestrictionTable {
    /// `n` null curves; the errors are unset natively until written (0 here).
    fn new(n: i32) -> Result<Self, Failure> {
        let curves = rustycad_tkg2d::geom2d::arrays::one_based(n)?;
        let c_error = Array2::with_bounds(1, 2, 1, n).map_err(|e| {
            super::sweep_failures::sweep_failure(
                "Standard_RangeError",
                &format!("NCollection_HArray2(1, 2, 1, {n}): {e}; expected n >= 1"),
            )
        })?;
        Ok(Self { curves, c_error })
    }

    fn last(&self) -> i32 {
        self.curves.len() as i32
    }

    /// Curve `index` with the error `error` in U and V.
    fn set(
        &mut self,
        index: i32,
        curve: Handle<dyn Geom2dCurve>,
        error: f64,
    ) -> Result<(), Failure> {
        rustycad_tkg2d::geom2d::arrays::put(&mut self.curves, index, curve)?;
        for row in [1, 2] {
            self.c_error.set_value(row, index, error).map_err(|e| {
                super::sweep_failures::sweep_failure(
                    "Standard_OutOfRange",
                    &format!("GeomFill_Sweep: CError({row}, {index}): {e}; expected an index in the array bounds"),
                )
            })?;
        }
        Ok(())
    }

    /// Curve `index`: the exact iso line at `u` over `v`.
    fn set_iso(&mut self, index: i32, u: f64, v: (f64, f64)) -> Result<(), Failure> {
        self.set(index, iso_restriction(u, v.0, v.1)?, 0.0)
    }
}

impl GeomFillSweep {
    /// The approximation of the sweep function of the laws over the path domain.
    fn approximation(
        &self,
        continuity: Shape,
        degmax: i32,
        segmax: i32,
    ) -> Result<SweepApproximation, Failure> {
        let ratio = (self.s_last - self.s_first) / (self.last - self.first);
        let func =
            GeomFillSweepFunction::new(&self.my_sec, &self.my_loc, self.first, self.s_first, ratio);
        let func: Handle<dyn SweepFunction> = Handle::new(func).cast();
        let mut approx = SweepApproximation::new(&func);
        let options = SweepOptions {
            continuity,
            degmax,
            segmax,
            ..SweepOptions::new(self.tol3d, self.bound_tol, self.tol2d, self.tol_angular)
        };
        approx.perform(self.first, self.last, options)?;
        Ok(approx)
    }

    /// `BuildAll(Continuity, Degmax, Segmax)`: true when the approximation is done.
    pub(crate) fn build_all(
        &mut self,
        continuity: Shape,
        degmax: i32,
        segmax: i32,
    ) -> Result<bool, Failure> {
        let approx = self.approximation(continuity, degmax, segmax)?;
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
        let force = self.my_force_approx_c1 && !bspline.is_cn_v(1)?;
        self.my_surface = Handle::new(bspline).cast();
        self.s_error = approx.max_error_on_surf()?;
        if force {
            self.force_c1()?;
        }
        if self.my_curve2d.is_none() {
            self.approximated_curves(&approx, &surface.u_knots)?;
        }
        Ok(true)
    }

    /// The C1 approximation of a surface that is not C1 in V, with its two
    /// restrictions at the first and last U knots.
    fn force_c1(&mut self) -> Result<(), Failure> {
        let the_tol = 1.0e-4;
        let shape = ((Shape::C1, Shape::C1), (14, 14));
        let convert =
            ApproxSurface::from_surface(&self.my_surface, the_tol, shape.0, shape.1, 16, 1)?;
        if !convert.has_result() {
            return Ok(());
        }
        let bspl = convert.surface();
        self.my_surface = bspl.clone().cast();
        let b = super::sweep_failures::live(&bspl, "GeomFill_Sweep::BuildAll", "BSplSurf")?;
        let mut table = RestrictionTable::new(2)?;
        let v_last = b.v_knot(b.nb_v_knots())?;
        table.set_iso(1, b.u_knot(1)?, (0.0, v_last))?;
        table.set_iso(table.last(), b.u_knot(b.nb_u_knots())?, (0.0, v_last))?;
        (self.my_curve2d, self.c_error) = (Some(table.curves), Some(table.c_error));
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
        let mut table = RestrictionTable::new(2 + trace_number)?;
        let ideb = if has_first { 1 } else { 2 };
        let ifin = 1 + trace_number + i32::from(has_last);
        for (kk, ii) in (ideb..=ifin).enumerate() {
            let kk = kk as i32 + 1;
            table.set(
                ii,
                approximated_curve(approx, kk)?,
                approx.max_2d_error(kk)?,
            )?;
        }
        let range = (self.first, self.last);
        if !has_first {
            table.set_iso(1, u_knots[u_knots.lower()], range)?;
        }
        if !has_last {
            table.set_iso(table.last(), u_knots[u_knots.upper()], range)?;
        }
        (self.my_curve2d, self.c_error) = (Some(table.curves), Some(table.c_error));
        Ok(())
    }
}

/// The B-spline of the 2d curve `kk` of the approximation.
fn approximated_curve(
    approx: &SweepApproximation,
    kk: i32,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let c = Geom2dBSplineCurve::new(
        approx.curve_2d_poles(kk)?,
        approx.curves_2d_knots()?,
        approx.curves_2d_mults()?,
        approx.curves_2d_degree()?,
        false,
    )?;
    Ok(Handle::new(c).cast())
}
