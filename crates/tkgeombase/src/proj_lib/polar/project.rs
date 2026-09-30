// Rust translation of ProjLib_ComputeApproxOnPolarSurface::ProjectUsingInitialCurve2d
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ProjectUsingInitialCurve2d`: the poles of a Bezier or B-spline curve projected on
//! a plane or on a bilinear patch, otherwise a Bezier fit of [`PolarFunction`] joined
//! into one B-spline.
use super::ComputeApproxOnPolarSurface;
use super::function::PolarFunction;
use crate::app_par_curves::{Constraint, MultiCurveEval};
use crate::approx::{ComputeCLineOptions, FitAndDivide2d};
use crate::native_failure::failure;
use crate::proj_lib::new_array;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dBSplineCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::Point2;

/// The tolerances of one projection.
pub(super) struct Tolerances {
    pub(super) tol3d: f64,
    pub(super) dist_tol3d2: f64,
    pub(super) tol_u: f64,
    pub(super) tol_v: f64,
    pub(super) tol2d: f64,
}

impl ComputeApproxOnPolarSurface {
    fn tolerances(&self, surf: &dyn Adaptor3dSurface) -> Result<Tolerances, Failure> {
        let tol3d: f64 = self.tolerance;
        let dist_tol3d: f64 = if self.max_dist > 0. {
            self.max_dist
        } else {
            100.0 * tol3d
        };
        let (tol_u, tol_v) = (surf.u_resolution(tol3d)?, surf.v_resolution(tol3d)?);
        let tol2d: f64 = (tol_u * tol_u + tol_v * tol_v)
            .sqrt()
            .max(precision::p_confusion());
        Ok(Tolerances {
            tol3d,
            dist_tol3d2: dist_tol3d * dist_tol3d,
            tol_u,
            tol_v,
            tol2d,
        })
    }

    /// `ProjectUsingInitialCurve2d(Curve, S, InitCurve2d)`: the projected 2D
    /// B-spline, or a null handle when the fit fails;
    /// `polar.project_using_initial_curve2d(&curve, &surface, &init)?`.
    pub fn project_using_initial_curve2d(
        &mut self,
        curve: &Handle<dyn Adaptor3dCurve>,
        surface: &Handle<dyn Adaptor3dSurface>,
        init_curve2d: &Handle<dyn Adaptor2dCurve2d>,
    ) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
        let (c, s) = (super::live_curve(curve)?, super::live_surface(surface)?);
        let tols: Tolerances = self.tolerances(s)?;
        let (s_type, c_type) = (s.get_type()?, c.get_type()?);
        if s_type == SurfaceType::Plane
            && let Some(spline) = super::poles::poles_on_plane(self, c, &s.plane()?, c_type)?
        {
            return Ok(spline);
        }
        if matches!(
            s_type,
            SurfaceType::BSplineSurface | SurfaceType::BezierSurface
        ) && let Some(spline) = super::poles::poles_on_bilinear(self, c, s, s_type, &tols)?
        {
            return Ok(spline);
        }
        let init: &dyn Adaptor2dCurve2d = init_curve2d.get().ok_or_else(|| {
            failure(
                "Standard_NullObject",
                "ProjLib_ComputeApproxOnPolarSurface: null initial 2D curve",
            )
        })?;
        self.fit_polar(c, s, init, tols)
    }

    // The fit options: degrees 2 to 8 by default; pass points at both ends when the
    // initial curve is far from the surface.
    fn polar_options(&self, tols: &Tolerances) -> ComputeCLineOptions {
        let deg1: i32 = if self.deg_min > 0 { self.deg_min } else { 2 };
        let deg2: i32 = if self.deg_max > 0 { self.deg_max } else { 8 };
        let mut bound: Constraint = self.bnd_pnt;
        if self.dist > 10. * tols.tol3d {
            bound = Constraint::PassPoint;
        }
        ComputeCLineOptions {
            degree_min: deg1,
            degree_max: deg2,
            tolerance_3d: tols.tol3d,
            tolerance_2d: tols.tol2d,
            cutting: true,
            first_constraint: bound,
            last_constraint: bound,
        }
    }

    // The Bezier fit of the polar function, joined into one B-spline.
    fn fit_polar(
        &mut self,
        c: &dyn Adaptor3dCurve,
        s: &dyn Adaptor3dSurface,
        init: &dyn Adaptor2dCurve2d,
        tols: Tolerances,
    ) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
        let function = PolarFunction::new(c, s, init, tols.tol3d)?;
        let mut fit: FitAndDivide2d = FitAndDivide2d::new(self.polar_options(&tols));
        fit.set_max_segments(if self.max_segments > 0 {
            self.max_segments
        } else {
            1000
        });
        if init.get_type()? == CurveType::Line {
            fit.set_inv_order(false);
        }
        let fitted = fit.perform(&function);
        function.take_failure()?;
        fitted?;
        if !fit.is_all_approximated() {
            return Ok(Handle::default());
        }
        self.join_polar(&fit, tols.tol2d)
    }

    // All pieces raised to the highest degree and joined with interior multiplicity
    // MaxDeg, then smoothed to C1 where the error allows (ten times more with pass
    // points), stopping at the first refused knot.
    fn join_polar(
        &mut self,
        fit: &FitAndDivide2d,
        old_tol2d: f64,
    ) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
        let nb_curves: i32 = fit.nb_multi_curves();
        let (mut max_deg, mut new_tol2d): (i32, f64) = (0, 0.);
        for j in 1..=nb_curves {
            max_deg = max_deg.max(fit.value(j)?.degree()?);
            new_tol2d = new_tol2d.max(fit.error(j)?.1);
        }
        self.tol_reached = self
            .tol_reached
            .max(self.tolerance * (new_tol2d / old_tol2d));
        let mut poles: Array1<'static, Point2> = new_array(max_deg * nb_curves + 1)?;
        let mut knots: Array1<'static, f64> = new_array(nb_curves + 1)?;
        let mut compt: i32 = 1;
        for i in 1..=nb_curves {
            (knots[i], knots[i + 1]) = fit.parameters(i)?;
            let piece: Array1<'static, Point2> = raised_piece(fit, i, max_deg)?;
            for k in 1..=max_deg + 1 {
                poles[compt] = piece[k];
                compt += 1;
            }
            compt -= 1;
        }
        let mut mults: Array1<'static, i32> = new_array(nb_curves + 1)?;
        for i in 1..=nb_curves + 1 {
            mults[i] = max_deg;
        }
        (mults[1], mults[nb_curves + 1]) = (max_deg + 1, max_deg + 1);
        self.proj_is_done = true;
        let dummy = Geom2dBSplineCurve::new(&poles, &knots, &mults, max_deg, false)?;
        let mut smooth_tol: f64 = precision::confusion().max(new_tol2d);
        if self.bnd_pnt == Constraint::PassPoint {
            smooth_tol *= 10.;
        }
        for ij in 2..nb_curves + 1 {
            if !dummy.remove_knot(ij, max_deg - 1, smooth_tol)? {
                break;
            }
        }
        Ok(Handle::new(dummy))
    }
}

// The poles of piece `i`, raised to `max_deg` when its degree is lower.
fn raised_piece(
    fit: &FitAndDivide2d,
    i: i32,
    max_deg: i32,
) -> Result<Array1<'static, Point2>, Failure> {
    let mc = fit.value(i)?;
    let degree: i32 = mc.degree()?;
    let mut poles: Array1<'static, Point2> = new_array(degree + 1)?;
    mc.curve_2d(1, &mut poles)?;
    if max_deg - degree <= 0 {
        return Ok(poles);
    }
    let mut raised: Array1<'static, Point2> = new_array(max_deg + 1)?;
    bsplclib::increase_degree_bezier(max_deg, &poles, None, &mut raised, None)?;
    Ok(raised)
}
