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
// Rust translation of ModelingData/TKGeomBase/ProjLib/ProjLib_ComputeApprox.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ProjLib_ComputeApprox`: the 2D curve of a 3D curve projected on an analytic
//! surface, by a Bezier fit of the surface parameters of the curve points, or by
//! the projection of the poles for a Bezier or B-spline curve on a plane.
mod cone_bounds;
pub(super) mod function;
mod home;
mod plane_poles;
mod sphere_bounds;
mod uv_bounds;

use super::statics::is_ana_surf;
use crate::app_par_curves::{Constraint, MultiCurveEval};
use crate::approx::{ComputeCLineOptions, FitAndDivide2d};
use function::{Projection, ProjectionFunction, live_curve, live_surface};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dBezierCurve};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::convert::CompBezierCurves2dToBSplineCurve2d;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::Point2;

/// `ProjLib_ComputeApprox`: the projected 2D B-spline (or Bezier) and the reached
/// tolerance.
///
/// ```ignore
/// let mut approx = ComputeApprox::new();
/// approx.set_tolerance(1e-6);
/// approx.perform(&curve, &surface)?;
/// let pcurve = approx.bspline();
/// ```
#[derive(Clone)]
pub struct ComputeApprox {
    tolerance: f64,
    bspline: Handle<Geom2dBSplineCurve>,
    bezier: Handle<Geom2dBezierCurve>,
    deg_min: i32,
    deg_max: i32,
    max_segments: i32,
    bnd_pnt: Constraint,
}

impl Default for ComputeApprox {
    fn default() -> Self {
        Self {
            tolerance: precision::p_approximation(),
            bspline: Handle::default(),
            bezier: Handle::default(),
            deg_min: -1,
            deg_max: -1,
            max_segments: -1,
            bnd_pnt: Constraint::TangencyPoint,
        }
    }
}

impl ComputeApprox {
    /// `ProjLib_ComputeApprox()`: tolerance PApproximation and default options;
    /// `ComputeApprox::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `ProjLib_ComputeApprox(C, S, Tol)`: performs with the tolerance
    /// `max(tol, PApproximation)`; `ComputeApprox::with_curve(&curve, &surface, 1e-6)?`.
    pub fn with_curve(
        curve: &Handle<dyn Adaptor3dCurve>,
        surface: &Handle<dyn Adaptor3dSurface>,
        tol: f64,
    ) -> Result<Self, Failure> {
        let mut approx: Self = Self {
            tolerance: tol.max(precision::p_approximation()),
            ..Self::default()
        };
        approx.perform(curve, surface)?;
        Ok(approx)
    }
    /// `SetTolerance(theTolerance)`; `approx.set_tolerance(1e-6)`.
    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.tolerance = tolerance;
    }
    /// `SetDegree(theDegMin, theDegMax)`: a negative degree keeps the default of the
    /// curve and surface kinds; `approx.set_degree(2, 8)`.
    pub fn set_degree(&mut self, deg_min: i32, deg_max: i32) {
        (self.deg_min, self.deg_max) = (deg_min, deg_max);
    }
    /// `SetMaxSegments(theMaxSegments)`: a negative count keeps 1000;
    /// `approx.set_max_segments(100)`.
    pub fn set_max_segments(&mut self, max_segments: i32) {
        self.max_segments = max_segments;
    }
    /// `SetBndPnt(theBndPnt)`; `approx.set_bnd_pnt(Constraint::PassPoint)`.
    pub fn set_bnd_pnt(&mut self, bnd_pnt: Constraint) {
        self.bnd_pnt = bnd_pnt;
    }
    /// `BSpline()`: the projected B-spline, null when not computed; `approx.bspline()`.
    pub fn bspline(&self) -> Handle<Geom2dBSplineCurve> {
        self.bspline.clone()
    }
    /// `Bezier()`: the projected Bezier (a Bezier curve on a plane), null otherwise;
    /// `approx.bezier()`.
    pub fn bezier(&self) -> Handle<Geom2dBezierCurve> {
        self.bezier.clone()
    }
    /// `Tolerance()`: the reached tolerance; `approx.tolerance()`.
    pub fn tolerance(&self) -> f64 {
        self.tolerance
    }

    /// `Perform(C, S)`: the projection with the current options. The poles of a
    /// Bezier or B-spline curve on a plane are projected; other cases are fitted. The
    /// results of a previous Perform stay when this one does not replace them, as
    /// natively; `approx.perform(&curve, &surface)?`.
    pub fn perform(
        &mut self,
        curve: &Handle<dyn Adaptor3dCurve>,
        surface: &Handle<dyn Adaptor3dSurface>,
    ) -> Result<(), Failure> {
        let (c, s) = (live_curve(curve)?, live_surface(surface)?);
        let (c_type, s_type) = (c.get_type()?, s.get_type()?);
        let simplecase: bool = simple_case(curve, surface, c_type)?;
        if c_type == CurveType::BSplineCurve && s_type == SurfaceType::Plane {
            self.bspline = plane_poles::bspline_on_plane(c, &s.plane()?)?;
            return Ok(());
        }
        if c_type == CurveType::BezierCurve && s_type == SurfaceType::Plane {
            self.bezier = plane_poles::bezier_on_plane(c, &s.plane()?)?;
            return Ok(());
        }
        self.fit(Projection { curve: c, surface: s }, simplecase)
    }
}

// Both kinds analytic; a Bezier or B-spline curve (never analytic) still reads its
// knot count, as natively.
fn simple_case(
    curve: &Handle<dyn Adaptor3dCurve>,
    surface: &Handle<dyn Adaptor3dSurface>,
    c_type: CurveType,
) -> Result<bool, Failure> {
    let surf_is_anal: bool = is_ana_surf(surface)?;
    let curv_is_anal: bool = !matches!(
        c_type,
        CurveType::BSplineCurve | CurveType::BezierCurve | CurveType::OffsetCurve | CurveType::OtherCurve
    );
    let mut simplecase: bool = surf_is_anal && curv_is_anal;
    if matches!(c_type, CurveType::BSplineCurve | CurveType::BezierCurve) {
        let c: &dyn Adaptor3dCurve = live_curve(curve)?;
        let nb_knots: i32 = if c_type == CurveType::BSplineCurve { c.nb_knots()? } else { 1 };
        simplecase = simplecase && c.degree()? <= 2 && nb_knots <= 2;
    }
    Ok(simplecase)
}

/// `ComputeTolU` and `ComputeTolV` of ProjLib: the parametric resolutions of
/// `tolerance`, at most 1% of a period; `compute_tol_uv(surface, 1e-6)?`.
pub(in crate::proj_lib) fn compute_tol_uv(
    surface: &dyn Adaptor3dSurface,
    tolerance: f64,
) -> Result<(f64, f64), Failure> {
    let mut tol_u: f64 = surface.u_resolution(tolerance)?;
    if surface.is_u_periodic()? {
        tol_u = tol_u.min(0.01 * surface.u_period()?);
    }
    let mut tol_v: f64 = surface.v_resolution(tolerance)?;
    if surface.is_v_periodic()? {
        tol_v = tol_v.min(0.01 * surface.v_period()?);
    }
    Ok((tol_u, tol_v))
}

impl ComputeApprox {
    // The fit options of the general case.
    fn fit_options(&self, simplecase: bool, tol_2d: f64) -> ComputeCLineOptions {
        let deg1: i32 = if self.deg_min > 0 { self.deg_min } else { 5 };
        let default_deg2: i32 = if simplecase { 8 } else { 10 };
        let deg2: i32 = if self.deg_max > 0 { self.deg_max } else { default_deg2 };
        // Both bounds take myBndPnt (TangencyPoint by default).
        let bound: Constraint = self.bnd_pnt;
        ComputeCLineOptions {
            degree_min: deg1,
            degree_max: deg2,
            tolerance_3d: self.tolerance,
            tolerance_2d: tol_2d,
            cutting: true,
            first_constraint: bound,
            last_constraint: bound,
        }
    }

    // The general case: fit, join the pieces, restore the 3D tolerance from the 2D
    // one, and bring the curve home.
    fn fit(&mut self, projection: Projection<'_>, simplecase: bool) -> Result<(), Failure> {
        let function: ProjectionFunction<'_> = ProjectionFunction::new(projection)?;
        let (tol_u, tol_v) = compute_tol_uv(projection.surface, self.tolerance)?;
        let tol_2d: f64 = (tol_u * tol_u + tol_v * tol_v).sqrt().max(precision::p_confusion());
        let options: ComputeCLineOptions = self.fit_options(simplecase, tol_2d);
        let mut fit: FitAndDivide2d = FitAndDivide2d::new(options);
        fit.set_max_segments(if self.max_segments > 0 { self.max_segments } else { 1000 });
        if simplecase {
            fit.set_hang_checking(false);
        }
        let fitted = fit.perform(&function);
        function.take_failure()?;
        fitted?;
        let mut new_tol_2d: f64 = 0.;
        if fit.is_all_approximated() {
            let Some(tol) = self.join(&fit, &options, projection.curve)? else {
                return Ok(());
            };
            new_tol_2d = tol;
        } else if fit.nb_multi_curves() != 0 {
            new_tol_2d = fit.error(fit.nb_multi_curves())?.1;
        }
        // tol3d(new) / tol3d(old) = tol2d(new) / tol2d(old), assuming the U/V
        // tolerance ratio is kept.
        self.tolerance *= new_tol_2d / tol_2d;
        home::bring_home(self, &function, projection)
    }

    // The Bezier pieces joined into one B-spline on the curve range (at least C1
    // where possible), smoothed when a bound is a pass point; None when the joined
    // curve has an invalid pole or knot count. Returns the largest 2D error.
    fn join(
        &mut self,
        fit: &FitAndDivide2d,
        options: &ComputeCLineOptions,
        curve: &dyn Adaptor3dCurve,
    ) -> Result<Option<f64>, Failure> {
        let mut conv: CompBezierCurves2dToBSplineCurve2d = CompBezierCurves2dToBSplineCurve2d::new(1.0e-4);
        let mut new_tol_2d: f64 = 0.;
        for i in 1..=fit.nb_multi_curves() {
            new_tol_2d = new_tol_2d.max(fit.error(i)?.1);
            let mc = fit.value(i)?;
            let mut poles: Array1<'static, Point2> = super::new_array(mc.degree()? + 1)?;
            mc.curve_2d(1, &mut poles)?;
            conv.add_curve(&poles);
        }
        conv.perform()?;
        let (nb_poles, nb_knots) = (conv.nb_poles(), conv.nb_knots());
        if nb_poles <= 0 || nb_poles > 100000 || nb_knots <= 0 || nb_knots > 100000 {
            return Ok(None);
        }
        let mut poles: Array1<'static, Point2> = super::new_array(nb_poles)?;
        let mut knots: Array1<'static, f64> = super::new_array(nb_knots)?;
        let mut mults: Array1<'static, i32> = super::new_array(nb_knots)?;
        conv.knots_and_mults(&mut knots, &mut mults)?;
        conv.poles(&mut poles)?;
        bsplclib::reparametrize(curve.first_parameter()?, curve.last_parameter()?, &mut knots)?;
        // The last knot is exactly the curve's last parameter, for a later trim.
        knots[nb_knots] = curve.last_parameter()?;
        let spline = Geom2dBSplineCurve::new(&poles, &knots, &mults, conv.degree(), false)?;
        if options.first_constraint == Constraint::PassPoint
            || options.last_constraint == Constraint::PassPoint
        {
            smooth_c1(&spline, nb_knots, new_tol_2d)?;
        }
        self.bspline = Handle::new(spline);
        Ok(Some(new_tol_2d))
    }
}

/// "Try to smooth the curve to C1": lowers each interior knot to degree - 1 within
/// `max(Confusion, tol)`, stopping at the first refusal (`OK = OK && ...`).
pub(in crate::proj_lib) fn smooth_c1(
    spline: &Geom2dBSplineCurve,
    nb_knots: i32,
    tol: f64,
) -> Result<(), Failure> {
    let degree: i32 = spline.degree();
    let smooth_tol: f64 = precision::confusion().max(tol);
    for ij in 2..nb_knots {
        if !spline.remove_knot(ij, degree - 1, smooth_tol)? {
            break;
        }
    }
    Ok(())
}
