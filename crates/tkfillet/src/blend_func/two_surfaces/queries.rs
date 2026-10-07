//! The common queries of the fillet functions: bounds and tolerances, the end tangents,
//! the section circle, and the shape of the approximation.
use super::super::linalg::{vector, vget, vset, widen};
use super::super::package::{SectionCounts, get_minimal_weights, get_shape, live_surface};
use super::equations::center;
use super::solution::project_normal;
use super::{Radii, TwoSurfaceCore};
use crate::blend::SectionEndFrames;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkgeomalgo::geom_fill;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Axis2, Circle3, Direction3, Vector3};
use rustycad_tkmath::math::Vector;
use std::f64::consts::PI;

/// The circular section of `Section(Param, U1, V1, U2, V2, Pdeb, Pfin, C)`: the circle
/// and its parameter range.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CircleSection {
    pub pdeb: f64,
    pub pfin: f64,
    pub circle: Circle3,
}

/// `gp_Ax2(Center, np, ns1)` with the vectors converted to directions.
fn frame(origin: rustycad_tkmath::gp::Point3, np: Vector3, x: Vector3) -> Result<Axis2, Failure> {
    Axis2::with_x_direction(
        origin,
        Direction3::from_vector(np)?,
        Direction3::from_vector(x)?,
    )
}

impl TwoSurfaceCore {
    /// `GetTolerance(Tolerance, Tol)`: the U and V resolutions of both surfaces.
    pub(crate) fn get_tolerance_impl(
        &self,
        tolerance: &mut Vector<'_>,
        tol: f64,
    ) -> Result<(), Failure> {
        let (s1, s2) = (live_surface(&self.surf1)?, live_surface(&self.surf2)?);
        vset(tolerance, 1, s1.u_resolution(tol)?)?;
        vset(tolerance, 2, s1.v_resolution(tol)?)?;
        vset(tolerance, 3, s2.u_resolution(tol)?)?;
        vset(tolerance, 4, s2.v_resolution(tol)?)
    }

    /// `GetBounds(InfBound, SupBound)`: the surface bounds, widened by their range.
    pub(crate) fn get_bounds_impl(
        &self,
        inf: &mut Vector<'_>,
        sup: &mut Vector<'_>,
    ) -> Result<(), Failure> {
        let (s1, s2) = (live_surface(&self.surf1)?, live_surface(&self.surf2)?);
        // The native order: the four lower bounds, then the four upper bounds.
        vset(inf, 1, s1.first_u_parameter()?)?;
        vset(inf, 2, s1.first_v_parameter()?)?;
        vset(inf, 3, s2.first_u_parameter()?)?;
        vset(inf, 4, s2.first_v_parameter()?)?;
        vset(sup, 1, s1.last_u_parameter()?)?;
        vset(sup, 2, s1.last_v_parameter()?)?;
        vset(sup, 3, s2.last_u_parameter()?)?;
        vset(sup, 4, s2.last_v_parameter()?)?;
        for i in 1..=4 {
            widen(inf, sup, i)?;
        }
        Ok(())
    }

    /// `Tangent(U1, V1, U2, V2, TgF, TgL, NmF, NmL)`.
    pub(crate) fn tangent_impl(
        &self,
        uv: [f64; 4],
        radii: Radii,
    ) -> Result<SectionEndFrames, Failure> {
        let cached = uv
            == [
                vget(&self.xval, 1)?,
                vget(&self.xval, 2)?,
                vget(&self.xval, 3)?,
                vget(&self.xval, 4)?,
            ];
        let (nm_f, nm_l) = if cached {
            (self.s1.nsurf, self.s2.nsurf)
        } else {
            let d1 = live_surface(&self.surf1)?.d1(uv[0], uv[1])?;
            let d2 = live_surface(&self.surf2)?.d1(uv[2], uv[3])?;
            (d1.d1u.crossed(d1.d1v), d2.d1u.crossed(d2.d1v))
        };
        let nplan = self.plane.nplan;
        let ns1 = project_with_inverse(nplan, nm_f);
        let center = center(self.s1.pts, radii.ray1(), ns1);
        let mut tg_first = nplan.crossed(Vector3::between(center, self.s1.pts));
        let mut tg_last = nplan.crossed(Vector3::between(center, self.s2.pts));
        if self.choix % 2 == 1 {
            tg_first.reverse();
            tg_last.reverse();
        }
        Ok(SectionEndFrames {
            tg_first,
            tg_last,
            norm_first: nm_f,
            norm_last: nm_l,
        })
    }

    /// `TwistOnS1()` / `TwistOnS2()`: Standard_DomainError with `message` on a tangency
    /// point.
    pub(crate) fn twist(&self, second: bool, message: &str) -> Result<bool, Failure> {
        let tg = if second { self.tg2 } else { self.tg1 };
        self.when_not_tangent(tg.dot(self.plane.nplan) < 0., message)
    }

    /// The circle of `Section(Param, U1, V1, U2, V2, ...)` after `ComputeValues(X, 0,
    /// true, Param)`.
    pub(crate) fn section_circle_impl(&self, radii: Radii) -> Result<CircleSection, Failure> {
        let nplan = self.plane.nplan;
        let mut ns1 = project_normal(nplan, self.s1.nsurf);
        let mut np = nplan;
        let origin = center(self.s1.pts, radii.ray1(), ns1);
        // ns1 is oriented from the center to pts1.
        if radii.positive(false) {
            ns1.reverse();
        }
        if self.choix % 2 != 0 {
            np.reverse();
        }
        let radius = radii.circle_radius();
        let mut circle = Circle3::new(frame(origin, np, ns1)?, radius)?;
        let mut pfin = elclib::circle_parameter(circle.position(), self.s2.pts);
        // Test negative and almost null angles: singular case.
        if pfin > 1.5 * PI {
            np.reverse();
            circle = Circle3::new(frame(origin, np, ns1)?, radius)?;
            pfin = elclib::circle_parameter(circle.position(), self.s2.pts);
        }
        if pfin < precision::p_confusion() {
            pfin += precision::p_confusion();
        }
        Ok(CircleSection {
            pdeb: 0.,
            pfin,
            circle,
        })
    }

    /// The point `X` of `Section(Param, U1, V1, U2, V2, ...)`.
    pub(crate) fn uv_vector(uv: [f64; 4]) -> Result<Vector<'static>, Failure> {
        let mut x = vector(4)?;
        for (i, value) in (1..).zip(uv) {
            vset(&mut x, i, value)?;
        }
        Ok(x)
    }

    pub(crate) fn get_minimal_weight_impl(
        &self,
        weights: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        // It is supposed that it does not depend on the radius.
        get_minimal_weights(self.shape, self.tconv, self.minang, self.maxang, weights)
    }

    pub(crate) fn get_shape_impl(&mut self) -> (i32, i32, i32, i32) {
        let SectionCounts {
            nb_poles,
            nb_knots,
            degree,
        } = get_shape(self.shape, self.maxang, &mut self.tconv);
        (nb_poles, nb_knots, degree, 2)
    }

    /// The body of `GetTolerance(BoundTol, SurfTol, AngleTol, Tol3d, Tol1d)` with the
    /// angle and radius of the class (`GeomFill::GetTolerance(TConv, angle, radius, ...)`).
    pub(crate) fn get_tolerance_3d_impl(
        &self,
        angle_radius: (f64, f64),
        tols: [f64; 3],
        tol3d: &mut Vector<'_>,
        tol1d: &mut Vector<'_>,
    ) -> Result<(), Failure> {
        arc_tolerances(self.tconv, angle_radius, tols, (tol3d, tol1d))
    }

    /// `Resolution(IC2d, Tol, TolU, TolV)`: the resolutions of the first surface for
    /// `index == 1`, else of the second.
    pub(crate) fn resolution_impl(&self, index: i32, tol: f64) -> Result<(f64, f64), Failure> {
        let surface = live_surface(if index == 1 { &self.surf1 } else { &self.surf2 })?;
        Ok((surface.u_resolution(tol)?, surface.v_resolution(tol)?))
    }
}

/// The projected normal of `Tangent`: `invnorm1 = 1 / |nplan ^ ns1|` (1 below `Eps`),
/// multiplied instead of divided as `project_normal` does.
fn project_with_inverse(nplan: Vector3, ns1: Vector3) -> Vector3 {
    let mut invnorm1 = nplan.crossed(ns1).magnitude();
    invnorm1 = if invnorm1 < super::super::normal_terms::EPS {
        1.
    } else {
        1. / invnorm1
    };
    let mut out = Vector3::default();
    out.set_linear_form(nplan.dot(ns1) * invnorm1, nplan, -invnorm1, ns1);
    out
}

/// The approximation tolerances of a circular section: `Tol1d` and `Tol3d` at `SurfTol`,
/// the poles next to the ends at `min(Tol, SurfTol)` and the ends at `min(Tol, BoundTol)`,
/// `Tol` from `GeomFill::GetTolerance(TConv, angle, radius, AngleTol, SurfTol)`;
/// `arc_tolerances(tconv, (minang, ray), [bound, surf, angle], (&mut t3, &mut t1))?`.
pub(crate) fn arc_tolerances(
    tconv: rustycad_tkmath::convert::ParameterisationType,
    (angle, radius): (f64, f64),
    [bound_tol, surf_tol, angle_tol]: [f64; 3],
    (tol3d, tol1d): (&mut Vector<'_>, &mut Vector<'_>),
) -> Result<(), Failure> {
    let (low, up) = (tol3d.lower(), tol3d.upper());
    let tol = geom_fill::get_tolerance(tconv, angle, radius, angle_tol, surf_tol)?;
    tol1d.init(surf_tol);
    tol3d.init(surf_tol);
    vset(tol3d, low + 1, std_min(tol, surf_tol))?;
    vset(tol3d, up - 1, std_min(tol, surf_tol))?;
    vset(tol3d, low, std_min(tol, bound_tol))?;
    vset(tol3d, up, std_min(tol, bound_tol))
}
