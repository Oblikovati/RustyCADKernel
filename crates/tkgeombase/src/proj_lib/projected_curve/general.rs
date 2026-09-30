// Rust translation of the default case of ProjLib_ProjectedCurve::Perform
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Any other surface: the approximation of the first part of `CompProjectedCurve`.
//! On a surface of revolution, a curve end on the axis is trimmed first and the
//! projection extended to the V of the axis point on the basis curve.
use super::ProjectedState;
use super::perform::{PerformData, live_curve, live_surface};
use super::singular::extend_ends;
use crate::app_par_curves::Constraint;
use crate::approx::{CurveOnSurface, CurveOnSurfaceOptions};
use crate::extrema::ExtPC;
use crate::proj_lib::CompProjectedCurve;
use crate::proj_lib::compute_approx::compute_tol_uv;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dBSplineCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::{Shape, SurfaceType};
use rustycad_tkmath::gp::{Axis1, Line3, Point3};

impl ProjectedState {
    /// The default case; false when the projection has no part (Perform stops).
    pub(super) fn general(&mut self, data: &mut PerformData) -> Result<bool, Failure> {
        let mut vsingular: [f64; 2] = [0.0, 0.0];
        let (mut f, mut l) = (0.0, 0.0);
        data.dt = 0.0;
        if live_surface(self)?.get_type()? == SurfaceType::SurfaceOfRevolution {
            (f, l) = self.trim_on_axis(data, &mut vsingular)?;
        }
        let s: &dyn Adaptor3dSurface = live_surface(self)?;
        let (tol_u, tol_v) = compute_tol_uv(s, self.tolerance)?;
        let (tol_u, tol_v) = (
            tol_u.max(precision::confusion()),
            tol_v.max(precision::confusion()),
        );
        let tol_2d: f64 = (tol_u * tol_u + tol_v * tol_v).sqrt();
        let max_dist: f64 = if self.max_dist > 0. {
            self.max_dist
        } else {
            100. * self.tolerance
        };
        let projector =
            CompProjectedCurve::with_max_dist(&self.surface, &self.curve, tol_u, tol_v, max_dist)?;
        // Within ProjLib the result must be one and only one curve, not a point.
        if projector.nb_curves() <= 0 {
            return Ok(false);
        }
        let (udeb, ufin) = projector.bounds(1)?;
        let continuity: Shape = if self.bnd_pnt == Constraint::PassPoint {
            Shape::C0
        } else {
            Shape::C1
        };
        let options = CurveOnSurfaceOptions {
            max_segments: if self.max_segments > 0 {
                self.max_segments
            } else {
                16
            },
            max_degree: if self.deg_max > 0 { self.deg_max } else { 14 },
            continuity,
            only_3d: false,
            only_2d: true,
        };
        let hprojector: Handle<dyn Adaptor2dCurve2d> = Handle::new(projector).cast();
        let appr = CurveOnSurface::new(
            &hprojector,
            &self.surface,
            udeb,
            ufin,
            self.tolerance,
            options,
        )?;
        let mut res: Handle<Geom2dBSplineCurve> = appr.curve_2d();
        if res.is_null() {
            return Ok(true);
        }
        let (err_u, err_v) = (appr.max_error_2d_u(), appr.max_error_2d_v());
        let new_tol_2d: f64 = (err_u * err_u + err_v * err_v).sqrt();
        self.tolerance *= new_tol_2d / tol_2d;
        if data.trims.is_trimmed[0] || data.trims.is_trimmed[1] {
            self.extend_on_axis(&mut res, data, vsingular, (f, l, continuity, new_tol_2d))?;
        }
        self.set_bspline_result(res);
        Ok(true)
    }

    // The extensions of a surface of revolution (V singular cases 3 and 4), then a
    // C1 smoothing of a C0 result, the knot count read at each step.
    fn extend_on_axis(
        &self,
        res: &mut Handle<Geom2dBSplineCurve>,
        data: &PerformData,
        vsingular: [f64; 2],
        (_f, _l, continuity, new_tol_2d): (f64, f64, Shape, f64),
    ) -> Result<(), Failure> {
        let s: &dyn Adaptor3dSurface = live_surface(self)?;
        let (u1, u2) = (s.first_u_parameter()?, s.last_u_parameter()?);
        let (v1, v2) = (s.first_v_parameter()?, s.last_v_parameter()?);
        let boxes = [[u1, u2, vsingular[0], v2], [u1, u2, v1, vsingular[1]]];
        extend_ends(
            res,
            &data.trims,
            boxes,
            [3, 4],
            (data.first_par, data.last_par),
        )?;
        if continuity != Shape::C0 {
            return Ok(());
        }
        // Try to smooth the curve to C1.
        let Some(spline) = res.get() else {
            return Ok(());
        };
        let degree: i32 = spline.degree();
        let smooth_tol: f64 = precision::confusion().max(new_tol_2d);
        let mut ij: i32 = 2;
        while ij < spline.nb_knots() {
            if !spline.remove_knot(ij, degree - 1, smooth_tol)? {
                break;
            }
            ij += 1;
        }
        Ok(())
    }

    // A curve end on the axis of revolution: trimmed by 1% of the range, and the V
    // of the end point on the basis curve (nearest minimum), or its parameter on
    // the axis when the extrema fails. Returns the trimmed bounds.
    fn trim_on_axis(
        &mut self,
        data: &mut PerformData,
        vsingular: &mut [f64; 2],
    ) -> Result<(f64, f64), Failure> {
        let s: &dyn Adaptor3dSurface = live_surface(self)?;
        let axis: Axis1 = s.axe_of_revolution()?;
        let lin: Line3 = Line3::new(axis.location(), axis.direction());
        let c: &dyn Adaptor3dCurve = live_curve(&self.curve)?;
        let (mut f, mut l) = (c.first_parameter()?, c.last_parameter()?);
        data.dt = (l - f) * 0.01;
        let p: Point3 = c.value(f)?;
        if lin.distance(p) < precision::confusion() {
            data.trims.is_trimmed[0] = true;
            f += data.dt;
            self.curve = c.trim(f, l, precision::confusion())?;
            vsingular[0] = self.basis_parameter(p, &lin)?;
        }
        let p: Point3 = live_curve(&self.curve)?.value(l)?;
        if lin.distance(p) < precision::confusion() {
            data.trims.is_trimmed[1] = true;
            l -= data.dt;
            self.curve = live_curve(&self.curve)?.trim(f, l, precision::confusion())?;
            vsingular[1] = self.basis_parameter(p, &lin)?;
        }
        Ok((f, l))
    }

    // The parameter of the nearest minimum of `p` on the basis curve (RealLast
    // start), or of `p` on the axis when the extrema fails.
    fn basis_parameter(&self, p: Point3, lin: &Line3) -> Result<f64, Failure> {
        let basis: Handle<dyn Adaptor3dCurve> = live_surface(self)?.basis_curve()?;
        let extrema = ExtPC::with_curve(p, live_curve(&basis)?, self.tolerance)?;
        if !extrema.is_done() {
            return elclib::parameter(lin, p);
        }
        let (mut min_dist, mut value): (f64, f64) = (f64::MAX, 0.0);
        for idx in 1..=extrema.nb_ext()? {
            if extrema.is_min(idx)? && extrema.square_distance(idx)? < min_dist {
                min_dist = extrema.square_distance(idx)?;
                value = extrema.point(idx)?.parameter();
            }
        }
        Ok(value)
    }
}
