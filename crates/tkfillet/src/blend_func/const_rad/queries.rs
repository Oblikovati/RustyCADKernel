//! The queries of `BlendFunc_ConstRad`: bounds and tolerances, the section circle, the
//! shape of the approximation and the rotation axis of the guide.
use super::super::linalg::{vector, vget, vset};
use super::super::package::{
    SectionCounts, get_minimal_weights, get_shape, live_surface, next_shape,
};
use super::ConstRad;
use super::solution::project_normal;
use crate::blend::SectionEndFrames;
use crate::blend_func::linalg::point_from;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeomalgo::geom_fill;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{self, Axis1, Axis2, Circle3, Direction3, Point3, Vector3};
use rustycad_tkmath::math::Vector;
use std::f64::consts::PI;

/// The guide curve behind a handle.
pub(crate) fn live_curve(
    curve: &Handle<dyn Adaptor3dCurve>,
) -> Result<&dyn Adaptor3dCurve, Failure> {
    curve
        .get()
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "BlendFunc: null guide curve"))
}

/// `InfBound(i) -= range; SupBound(i) += range` for finite bounds.
pub(crate) fn widen(inf: &mut Vector<'_>, sup: &mut Vector<'_>, i: i32) -> Result<(), Failure> {
    let (low, up) = (vget(inf, i)?, vget(sup, i)?);
    if !precision::is_infinite(low) && !precision::is_infinite(up) {
        let range = up - low;
        vset(inf, i, low - range)?;
        vset(sup, i, up + range)?;
    }
    Ok(())
}

impl ConstRad {
    /// `GetTolerance(Tolerance, Tol)`: the U and V resolutions of both surfaces.
    pub(super) fn get_tolerance_impl(
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
    pub(super) fn get_bounds_impl(
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
    pub(super) fn tangent_impl(
        &self,
        u1: f64,
        v1: f64,
        u2: f64,
        v2: f64,
    ) -> Result<SectionEndFrames, Failure> {
        let cached = [u1, v1, u2, v2]
            == [
                vget(&self.xval, 1)?,
                vget(&self.xval, 2)?,
                vget(&self.xval, 3)?,
                vget(&self.xval, 4)?,
            ];
        let (nm_f, nm_l) = if cached {
            (self.s1.nsurf, self.s2.nsurf)
        } else {
            let d1 = live_surface(&self.surf1)?.d1(u1, v1)?;
            let d2 = live_surface(&self.surf2)?.d1(u2, v2)?;
            (d1.d1u.crossed(d1.d1v), d2.d1u.crossed(d2.d1v))
        };
        let nplan = self.plane.nplan;
        let ns1 = project_normal_inverse(nplan, nm_f);
        let center = point_from(self.s1.pts.xyz() + self.ray1 * *ns1.xyz());
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

    /// `TwistOnS1()`: Standard_DomainError on a tangency point.
    pub(super) fn twist(&self, second: bool) -> Result<bool, Failure> {
        if self.istangent {
            let member = if second { "TwistOnS2" } else { "TwistOnS1" };
            return Err(Failure::with_message(
                "Standard_DomainError",
                &format!("BlendFunc_ConstRad::{member}"),
            ));
        }
        let tg = if second { self.tg2 } else { self.tg1 };
        Ok(tg.dot(self.plane.nplan) < 0.)
    }

    /// `Section(Param, U1, V1, U2, V2, Pdeb, Pfin, C)`: the circular section at `param`
    /// through the two points, with its parameter range `(Pdeb, Pfin)`;
    /// `let (pdeb, pfin, circle) = function.section_circle(t, [u1, v1, u2, v2])?;`.
    pub fn section_circle(
        &mut self,
        param: f64,
        uv: [f64; 4],
    ) -> Result<(f64, f64, Circle3), Failure> {
        let mut x = vector(4)?;
        for (i, value) in (1..).zip(uv) {
            vset(&mut x, i, value)?;
        }
        self.compute_values(&x, 0, true, param)?;
        let nplan = self.plane.nplan;
        let mut ns1 = project_normal(nplan, self.s1.nsurf);
        let mut np = nplan;
        let center = point_from(self.s1.pts.xyz() + self.ray1 * *ns1.xyz());
        // ns1 is oriented from the center to pts1.
        if self.ray1 > 0. {
            ns1.reverse();
        }
        if self.choix % 2 != 0 {
            np.reverse();
        }
        let radius = self.ray1.abs();
        let mut circle = Circle3::new(frame(center, np, ns1)?, radius)?;
        let mut pfin = elclib::circle_parameter(circle.position(), self.s2.pts);
        // Test negative and almost null angles: singular case.
        if pfin > 1.5 * PI {
            np.reverse();
            circle = Circle3::new(frame(center, np, ns1)?, radius)?;
            pfin = elclib::circle_parameter(circle.position(), self.s2.pts);
        }
        if pfin < precision::p_confusion() {
            pfin += precision::p_confusion();
        }
        Ok((0., pfin, circle))
    }

    pub(super) fn get_minimal_weight_impl(
        &self,
        weights: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        // It is supposed that it does not depend on the radius.
        get_minimal_weights(self.shape, self.tconv, self.minang, self.maxang, weights)
    }

    pub(super) fn nb_intervals_impl(&self, shape: Shape) -> Result<i32, Failure> {
        live_curve(&self.curv)?.nb_intervals(next_shape(shape))
    }

    pub(super) fn intervals_impl(
        &self,
        bounds: &mut Array1<'_, f64>,
        shape: Shape,
    ) -> Result<(), Failure> {
        live_curve(&self.curv)?.intervals(bounds, next_shape(shape))
    }

    pub(super) fn get_shape_impl(&mut self) -> (i32, i32, i32, i32) {
        let SectionCounts {
            nb_poles,
            nb_knots,
            degree,
        } = get_shape(self.shape, self.maxang, &mut self.tconv);
        (nb_poles, nb_knots, degree, 2)
    }

    /// `GetTolerance(BoundTol, SurfTol, AngleTol, Tol3d, Tol1d)`.
    pub(super) fn get_tolerance_3d_impl(
        &self,
        tols: [f64; 3],
        tol3d: &mut Vector<'_>,
        tol1d: &mut Vector<'_>,
    ) -> Result<(), Failure> {
        let [bound_tol, surf_tol, angle_tol] = tols;
        let (low, up) = (tol3d.lower(), tol3d.upper());
        let tol = geom_fill::get_tolerance(
            self.tconv,
            self.minang,
            self.ray1.abs(),
            angle_tol,
            surf_tol,
        )?;
        tol1d.init(surf_tol);
        tol3d.init(surf_tol);
        vset(tol3d, low + 1, std_min(tol, surf_tol))?;
        vset(tol3d, up - 1, std_min(tol, surf_tol))?;
        vset(tol3d, low, std_min(tol, bound_tol))?;
        vset(tol3d, up, std_min(tol, bound_tol))
    }

    /// `AxeRot(Prm)`: the axis of the osculating circle of the guide at `prm` (along the
    /// plane normal when the guide is straight); `function.axe_rot(0.5)?`.
    pub fn axe_rot(&self, prm: f64) -> Result<Axis1, Failure> {
        let (ptgui, d1gui, d2gui) = live_curve(&self.curv)?.d2(prm)?;
        let normtg = d1gui.magnitude();
        let np = d1gui.normalized()?;
        let mut dnp = Vector3::default();
        dnp.set_linear_form(1. / normtg, d2gui, -1. / normtg * np.dot(d2gui), np);
        let dirax = np.crossed(dnp);
        let mut axrot = Axis1::default();
        if dirax.magnitude() >= gp::resolution() {
            axrot.set_direction(Direction3::from_vector(dirax)?);
        } else {
            axrot.set_direction(Direction3::from_vector(np)?); // To avoid stop
        }
        let oriax = if dnp.magnitude() >= gp::resolution() {
            point_from(ptgui.xyz() + (normtg / dnp.magnitude()) * *dnp.normalized()?.xyz())
        } else {
            point_from(ptgui.xyz())
        };
        axrot.set_location(oriax);
        Ok(axrot)
    }

    /// `Resolution(IC2d, Tol, TolU, TolV)`: the resolutions of the first surface for
    /// `index == 1`, else of the second.
    pub(super) fn resolution_impl(&self, index: i32, tol: f64) -> Result<(f64, f64), Failure> {
        let surface = live_surface(if index == 1 { &self.surf1 } else { &self.surf2 })?;
        Ok((surface.u_resolution(tol)?, surface.v_resolution(tol)?))
    }
}

/// `gp_Ax2(Center, np, ns1)` with the vectors converted to directions.
fn frame(center: Point3, np: Vector3, x: Vector3) -> Result<Axis2, Failure> {
    Axis2::with_x_direction(
        center,
        Direction3::from_vector(np)?,
        Direction3::from_vector(x)?,
    )
}

/// The projected normal of `Tangent`: `invnorm1 = 1 / |nplan ^ ns1|` (1 below `Eps`),
/// multiplied instead of divided as `project_normal` does.
fn project_normal_inverse(nplan: Vector3, ns1: Vector3) -> Vector3 {
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
