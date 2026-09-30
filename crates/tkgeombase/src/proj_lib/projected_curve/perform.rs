// Rust translation of ProjLib_ProjectedCurve::Perform (elementary, Bezier and
// B-spline surfaces, and the analytic fallback) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Perform`: the dispatch on the surface kind, the pole trims of spheres and
//! spline surfaces, and the analytic approximation when the exact projector fails.
use super::ProjectedState;
use super::singular::{Box2d, Trims, extend_ends, iso_is_deg, trim_c3d};
use crate::geom2d_convert::curve_to_bspline_curve;
use crate::native_failure::failure;
use crate::proj_lib::{
    ComputeApprox, ComputeApproxOnPolarSurface, Cone, Cylinder, Plane, Project, Projector, Sphere,
    Torus,
};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::geom_abs::{CurveType, IsoType, SurfaceType};
use rustycad_tkmath::gp::Point3;
use std::f64::consts::PI;

/// The data of one Perform shared by its branches.
pub(super) struct PerformData {
    pub(super) first_par: f64,
    pub(super) last_par: f64,
    pub(super) s_type: SurfaceType,
    pub(super) c_type: CurveType,
    pub(super) dt: f64,
    pub(super) bounds: Box2d,
    pub(super) trims: Trims,
}

/// The static `Project(P, C)`: the exact projection of a line or conic; the other
/// kinds are left to the approximation.
fn project_curve(projector: &mut dyn Project, curve: &dyn Adaptor3dCurve) -> Result<(), Failure> {
    match curve.get_type()? {
        CurveType::Line => projector.project_line(&curve.line()?),
        CurveType::Circle => projector.project_circle(&curve.circle()?),
        CurveType::Ellipse => projector.project_ellipse(&curve.ellipse()?),
        CurveType::Hyperbola => projector.project_hyperbola(&curve.hyperbola()?),
        CurveType::Parabola => projector.project_parabola(&curve.parabola()?),
        // BSpline, Bezier, offset and other curves: try the approximation.
        _ => Ok(()),
    }
}

pub(super) fn live_surface(state: &ProjectedState) -> Result<&dyn Adaptor3dSurface, Failure> {
    state.surface.get().ok_or_else(|| {
        failure(
            "Standard_NullObject",
            "ProjLib_ProjectedCurve: no surface is loaded",
        )
    })
}

pub(super) fn live_curve(
    curve: &Handle<dyn Adaptor3dCurve>,
) -> Result<&dyn Adaptor3dCurve, Failure> {
    curve
        .get()
        .ok_or_else(|| failure("Standard_NullObject", "ProjLib_ProjectedCurve: null curve"))
}

impl ProjectedState {
    /// `Perform(C)` on the plain state.
    pub(super) fn perform(&mut self, curve: &Handle<dyn Adaptor3dCurve>) -> Result<(), Failure> {
        self.tolerance = self.tolerance.max(precision::confusion());
        self.curve = curve.clone();
        let c: &dyn Adaptor3dCurve = live_curve(curve)?;
        let (first_par, last_par) = (c.first_parameter()?, c.last_parameter()?);
        let s: &dyn Adaptor3dSurface = live_surface(self)?;
        let bounds: Box2d = [
            s.first_u_parameter()?,
            s.last_u_parameter()?,
            s.first_v_parameter()?,
            s.last_v_parameter()?,
        ];
        let mut data = PerformData {
            first_par,
            last_par,
            s_type: s.get_type()?,
            c_type: c.get_type()?,
            dt: (last_par - first_par) * 0.01,
            bounds,
            trims: Trims::default(),
        };
        let analytical: bool = match data.s_type {
            SurfaceType::Plane => self.elementary(Plane::with_plane(&s.plane()?))?,
            SurfaceType::Cylinder => self.elementary(Cylinder::with_cylinder(&s.cylinder()?))?,
            SurfaceType::Cone => self.elementary(Cone::with_cone(&s.cone()?))?,
            SurfaceType::Sphere => self.sphere(&mut data)?,
            SurfaceType::Torus => self.elementary(Torus::with_torus(&s.torus()?))?,
            SurfaceType::BezierSurface | SurfaceType::BSplineSurface => {
                self.spline_surface(&mut data)?;
                false
            }
            _ => {
                if !self.general(&mut data)? {
                    return Ok(());
                }
                false
            }
        };
        if !self.result.is_done() && analytical && !self.analytic_fallback(&data)? {
            return Ok(());
        }
        self.place_in_periods()
    }

    // An exact projector on an elementary surface; the result is its base object.
    fn elementary<P: Project + Into<Projector>>(
        &mut self,
        mut projector: P,
    ) -> Result<bool, Failure> {
        project_curve(&mut projector, live_curve(&self.curve)?)?;
        self.result = projector.into();
        Ok(true)
    }

    // A sphere: an exact result is placed in the pseudo-period (the sphere is not
    // periodic in V); otherwise the curve is trimmed near the poles.
    fn sphere(&mut self, data: &mut PerformData) -> Result<bool, Failure> {
        let s: &dyn Adaptor3dSurface = live_surface(self)?;
        let mut projector: Sphere = Sphere::with_sphere(&s.sphere()?);
        let c: &dyn Adaptor3dCurve = live_curve(&self.curve)?;
        project_curve(&mut projector, c)?;
        if projector.is_done() {
            projector.set_in_bounds(c.first_parameter()?)?;
        } else {
            self.trim_at_sphere_poles(data)?;
        }
        self.result = projector.into();
        Ok(true)
    }

    fn trim_at_sphere_poles(&mut self, data: &mut PerformData) -> Result<(), Failure> {
        let s: &dyn Adaptor3dSurface = live_surface(self)?;
        let (vmax, minang) = (PI / 2., 1.0e-5 * PI);
        let sphere = s.sphere()?;
        let r: f64 = sphere.radius();
        let c: &dyn Adaptor3dCurve = live_curve(&self.curve)?;
        let (pf, pl) = (
            c.value(c.first_parameter()?)?,
            c.value(c.last_parameter()?)?,
        );
        let location: Point3 = sphere.position().location();
        let maxdist: f64 = pf.distance(&location).max(pl.distance(&location));
        let tol_conf: f64 = (r * minang).max((r - maxdist).abs());
        // The surface has poles at V = Vmin and Vmax.
        let (pole_min, pole_max) = (
            s.value(data.bounds[0], -vmax)?,
            s.value(data.bounds[0], vmax)?,
        );
        trim_c3d(
            &mut self.curve,
            &mut data.trims,
            data.dt,
            pole_min,
            3,
            tol_conf,
        )?;
        trim_c3d(
            &mut self.curve,
            &mut data.trims,
            data.dt,
            pole_max,
            4,
            tol_conf,
        )?;
        Ok(())
    }

    // A Bezier or B-spline surface: trims at degenerated bound isos, then the polar
    // approximation, extended back to the poles.
    fn spline_surface(&mut self, data: &mut PerformData) -> Result<(), Failure> {
        let c: &dyn Adaptor3dCurve = live_curve(&self.curve)?;
        let (f, l) = (c.first_parameter()?, c.last_parameter()?);
        data.dt = (l - f) * 0.01;
        let surface: Handle<dyn Adaptor3dSurface> = self.surface.clone();
        let s: &dyn Adaptor3dSurface = surface.get().ok_or_else(|| {
            failure(
                "Standard_NullObject",
                "ProjLib_ProjectedCurve: no surface is loaded",
            )
        })?;
        let [u1, u2, v1, v2] = data.bounds;
        let tol: f64 = self.tolerance;
        let conf: f64 = precision::confusion();
        let checks: [(f64, IsoType, (f64, f64), i32); 4] = [
            (u1, IsoType::IsoU, (u1, v1), 1),
            (u2, IsoType::IsoU, (u2, v1), 2),
            (v1, IsoType::IsoV, (u1, v1), 3),
            (v2, IsoType::IsoV, (u1, v2), 4),
        ];
        for (param, iso, pole, case) in checks {
            if iso_is_deg(s, param, iso, 0., tol)? {
                let pole_point: Point3 = s.value(pole.0, pole.1)?;
                trim_c3d(
                    &mut self.curve,
                    &mut data.trims,
                    data.dt,
                    pole_point,
                    case,
                    conf,
                )?;
            }
        }
        let mut polar: ComputeApproxOnPolarSurface = ComputeApproxOnPolarSurface::new();
        polar.set_tolerance(self.tolerance);
        polar.set_degree(self.deg_min, self.deg_max);
        polar.set_max_segments(self.max_segments);
        polar.set_bnd_pnt(self.bnd_pnt);
        polar.set_max_dist(self.max_dist);
        polar.perform(&self.curve, &self.surface)?;
        let mut res: Handle<Geom2dBSplineCurve> = polar.bspline();
        if res.is_null() {
            return Ok(());
        }
        self.tolerance = polar.tolerance();
        if data.trims.is_trimmed[0] || data.trims.is_trimmed[1] {
            let cases = data.trims.singular_case;
            extend_ends(
                &mut res,
                &data.trims,
                [data.bounds; 2],
                cases,
                (data.first_par, data.last_par),
            )?;
        }
        self.set_bspline_result(res);
        Ok(())
    }

    pub(super) fn set_bspline_result(&mut self, res: Handle<Geom2dBSplineCurve>) {
        self.result.set_bspline(&res);
        self.result.done();
        self.result.set_type(CurveType::BSplineCurve);
    }

    // The analytic approximation when the exact projector failed; false when it
    // fails too (the Perform then stops before the periodic placement).
    fn analytic_fallback(&mut self, data: &PerformData) -> Result<bool, Failure> {
        let mut comp: ComputeApprox = ComputeApprox::new();
        comp.set_tolerance(self.tolerance);
        comp.set_degree(self.deg_min, self.deg_max);
        comp.set_max_segments(self.max_segments);
        comp.set_bnd_pnt(self.bnd_pnt);
        comp.perform(&self.curve, &self.surface)?;
        if comp.bezier().is_null() && comp.bspline().is_null() {
            return Ok(false);
        }
        self.result.done();
        let mut res: Handle<Geom2dBSplineCurve> = if comp.bspline().is_null() {
            let bezier: Handle<dyn Geom2dCurve> = comp.bezier().cast();
            curve_to_bspline_curve(&bezier, ParameterisationType::TgtThetaOver2)?
        } else {
            comp.bspline()
        };
        if data.trims.is_trimmed[0] || data.trims.is_trimmed[1] {
            let cases = data.trims.singular_case;
            extend_ends(
                &mut res,
                &data.trims,
                [data.bounds; 2],
                cases,
                (data.first_par, data.last_par),
            )?;
            self.result.set_bspline(&res);
            self.result.set_type(CurveType::BSplineCurve);
        } else if data.s_type == SurfaceType::Plane && data.c_type == CurveType::BezierCurve {
            self.result.set_type(CurveType::BezierCurve);
            self.result.set_bezier(&comp.bezier());
        } else {
            self.result.set_type(CurveType::BSplineCurve);
            self.result.set_bspline(&comp.bspline());
        }
        // The periodicity flag.
        if data.s_type == SurfaceType::Plane
            && data.c_type == CurveType::BSplineCurve
            && live_curve(&self.curve)?.is_periodic()?
        {
            self.result.set_periodic();
        }
        self.tolerance = comp.tolerance();
        Ok(true)
    }
}
