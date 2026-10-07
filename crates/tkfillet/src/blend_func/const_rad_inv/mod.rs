// Rust translation of BlendFunc_ConstRadInv.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The inverse of the constant-radius fillet function: the section that ends on a curve on
//! one of the surfaces.
mod derivatives;

use super::const_rad::signed_radii;
use super::linalg::{vget, vset, widen};
use super::normal_terms::EPS;
use super::package::{compute_normal, live_curve, live_surface};
use crate::blend::{BlendFuncInv, BlendInverseBounds, func_inv_nb_variables};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkgeombase::failure_capture::FailureCapture;
use rustycad_tkmath::gp::{Point2, Point3, Vector3};
use rustycad_tkmath::math::{FunctionSet, FunctionSetWithDerivatives, Matrix, Vector};

/// `BlendFunc_ConstRadInv`: the constant-radius fillet equations of variables
/// `(w, t, U, V)`: the parameter `w` on the curve on surface, the guide parameter `t` and
/// the point `(U, V)` on the other surface.
///
/// ```ignore
/// let mut inverse = ConstRadInv::new(&s1, &s2, &guide);
/// inverse.set_radius(1.0, 1);
/// inverse.set_curve(true, &curve_on_s1)?;
/// ```
pub struct ConstRadInv {
    surf1: Handle<dyn Adaptor3dSurface>,
    surf2: Handle<dyn Adaptor3dSurface>,
    curv: Handle<dyn Adaptor3dCurve>,
    csurf: Handle<dyn Adaptor2dCurve2d>,
    ray1: f64,
    ray2: f64,
    choix: i32,
    first: bool,
    capture: FailureCapture,
}

impl ConstRadInv {
    /// `BlendFunc_ConstRadInv(S1, S2, C)`; `ConstRadInv::new(&s1, &s2, &c)`.
    pub fn new(
        s1: &Handle<dyn Adaptor3dSurface>,
        s2: &Handle<dyn Adaptor3dSurface>,
        c: &Handle<dyn Adaptor3dCurve>,
    ) -> Self {
        Self {
            surf1: s1.clone(),
            surf2: s2.clone(),
            curv: c.clone(),
            csurf: Handle::default(),
            ray1: 0.0,
            ray2: 0.0,
            choix: 0,
            first: false,
            capture: FailureCapture::default(),
        }
    }

    /// `Set(R, Choix)`: the radius and the choice, signed as `BlendFunc_ConstRad::Set`;
    /// `inverse.set_radius(1.0, 1)`.
    pub fn set_radius(&mut self, radius: f64, choix: i32) {
        self.choix = choix;
        (self.ray1, self.ray2) = signed_radii(radius, choix);
    }

    fn live_csurf(&self) -> Result<&dyn Adaptor2dCurve2d, Failure> {
        self.csurf.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "BlendFunc_ConstRadInv: no curve on surface; expected Set(OnFirst, COnSurf)",
            )
        })
    }

    /// The surface the curve lies on and the other one.
    fn surfaces(&self) -> (&Handle<dyn Adaptor3dSurface>, &Handle<dyn Adaptor3dSurface>) {
        if self.first {
            (&self.surf1, &self.surf2)
        } else {
            (&self.surf2, &self.surf1)
        }
    }

    /// `Value(X, F)`.
    fn try_value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure> {
        let (ptcur, d1cur) = live_curve(&self.curv)?.d1(vget(x, 2)?)?;
        let nplan = d1cur.normalized()?;
        let the_d = -nplan.xyz().dot(ptcur.xyz());
        let pt2d = self.live_csurf()?.value(vget(x, 1)?)?;
        let other = Point2::new(vget(x, 3)?, vget(x, 4)?);
        let (uv1, uv2) = if self.first {
            (pt2d, other)
        } else {
            (other, pt2d)
        };
        let d1 = live_surface(&self.surf1)?.d1(uv1.x(), uv1.y())?;
        let d2 = live_surface(&self.surf2)?.d1(uv2.x(), uv2.y())?;
        let (pts1, pts2) = (d1.point, d2.point);
        let f1 = (nplan.x() * (pts1.x() + pts2.x())
            + nplan.y() * (pts1.y() + pts2.y())
            + nplan.z() * (pts1.z() + pts2.z()))
            / 2.
            + the_d;
        let ns1 = repaired(&self.surf1, uv1, d1.d1u.crossed(d1.d1v))?;
        let ns2 = repaired(&self.surf2, uv2, d2.d1u.crossed(d2.d1v))?;
        let resul = self.section_residual(nplan, (ns1, ns2), (pts1, pts2));
        vset(f, 1, f1)?;
        vset(f, 2, resul.x())?;
        vset(f, 3, resul.y())?;
        vset(f, 4, resul.z())?;
        Ok(true)
    }

    /// `resul.SetLinearForm(ray1, ns1, -1., pts2, -ray2, ns2, pts1)` with the projected
    /// normals: the residual of the section of `Value`.
    fn section_residual(
        &self,
        nplan: Vector3,
        (ns1, ns2): (Vector3, Vector3),
        (pts1, pts2): (Point3, Point3),
    ) -> Vector3 {
        let ns1 = project(nplan, ns1);
        let ns2 = project(nplan, ns2);
        let mut resul = Vector3::default();
        resul.set_linear_form_three_offset(
            [self.ray1, -1., -self.ray2],
            [ns1, Vector3::from(pts2.xyz()), ns2],
            Vector3::from(pts1.xyz()),
        );
        resul
    }
}

/// The normal `ns`, or the higher-order normal at `uv` when `ns` degenerates.
pub(crate) fn repaired(
    surface: &Handle<dyn Adaptor3dSurface>,
    uv: Point2,
    ns: Vector3,
) -> Result<Vector3, Failure> {
    if ns.magnitude() >= EPS {
        return Ok(ns);
    }
    Ok(compute_normal(surface, uv)?.unwrap_or(ns))
}

/// `ns.SetLinearForm(nplan.Dot(ns) / norm, nplan, -1. / norm, ns)` with `norm = |nplan ^
/// ns|` (1 below `Eps`).
fn project(nplan: Vector3, ns: Vector3) -> Vector3 {
    let mut norm = nplan.crossed(ns).magnitude();
    if norm < EPS {
        norm = 1.; // Unsatisfactory, but it is not necessary to stop
    }
    let mut out = Vector3::default();
    out.set_linear_form(nplan.dot(ns) / norm, nplan, -1. / norm, ns);
    out
}

impl FunctionSet for ConstRadInv {
    /// 4 (`Blend_FuncInv::NbVariables`).
    fn variables(&self) -> i32 {
        func_inv_nb_variables()
    }
    fn equations(&self) -> i32 {
        4
    }
    fn value(&mut self, point: &Vector<'_>, residual: &mut Vector<'_>) -> bool {
        let result = self.try_value(point, residual);
        self.capture.keep(result).unwrap_or(false)
    }
}

impl FunctionSetWithDerivatives for ConstRadInv {
    fn derivatives(&mut self, point: &Vector<'_>, jacobian: &mut Matrix<'_>) -> bool {
        let result = self.try_derivatives(point, jacobian);
        self.capture.keep(result).unwrap_or(false)
    }
    fn values(
        &mut self,
        point: &Vector<'_>,
        residual: &mut Vector<'_>,
        jacobian: &mut Matrix<'_>,
    ) -> bool {
        let result = self.try_values(point, residual, jacobian);
        self.capture.keep(result).unwrap_or(false)
    }
}

impl BlendInverseBounds for ConstRadInv {
    fn take_failure(&mut self) -> Result<(), Failure> {
        self.capture.take()
    }
    /// The resolutions of the curve on surface, of the guide and of the other surface.
    fn get_tolerance(&self, tolerance: &mut Vector<'_>, tol: f64) -> Result<(), Failure> {
        vset(tolerance, 1, self.live_csurf()?.resolution(tol)?)?;
        vset(tolerance, 2, live_curve(&self.curv)?.resolution(tol)?)?;
        let other = live_surface(self.surfaces().1)?;
        vset(tolerance, 3, other.u_resolution(tol)?)?;
        vset(tolerance, 4, other.v_resolution(tol)?)
    }
    /// The bounds of the curve on surface, of the guide, and of the other surface widened
    /// by its range.
    fn get_bounds(&self, inf: &mut Vector<'_>, sup: &mut Vector<'_>) -> Result<(), Failure> {
        let (csurf, curv) = (self.live_csurf()?, live_curve(&self.curv)?);
        vset(inf, 1, csurf.first_parameter()?)?;
        vset(inf, 2, curv.first_parameter()?)?;
        vset(sup, 1, csurf.last_parameter()?)?;
        vset(sup, 2, curv.last_parameter()?)?;
        let other = live_surface(self.surfaces().1)?;
        vset(inf, 3, other.first_u_parameter()?)?;
        vset(inf, 4, other.first_v_parameter()?)?;
        vset(sup, 3, other.last_u_parameter()?)?;
        vset(sup, 4, other.last_v_parameter()?)?;
        widen(inf, sup, 3)?;
        widen(inf, sup, 4)
    }
    /// Whether `|F(1)| <= tol` and `|F(2..4)|^2 <= tol^2` at `sol`.
    fn is_solution(&mut self, sol: &Vector<'_>, tol: f64) -> Result<bool, Failure> {
        let mut valsol = super::linalg::vector(4)?;
        self.try_value(sol, &mut valsol)?;
        let (a, b, c, d) = (
            vget(&valsol, 1)?,
            vget(&valsol, 2)?,
            vget(&valsol, 3)?,
            vget(&valsol, 4)?,
        );
        Ok(a.abs() <= tol && b * b + c * c + d * d <= tol * tol)
    }
}

impl BlendFuncInv for ConstRadInv {
    /// `Set(OnFirst, COnSurf)`.
    fn set_curve(
        &mut self,
        on_first: bool,
        curve: &Handle<dyn Adaptor2dCurve2d>,
    ) -> Result<(), Failure> {
        self.first = on_first;
        self.csurf = curve.clone();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variables_and_choice_are_native() {
        let null_surface: Handle<dyn Adaptor3dSurface> = Handle::default();
        let null_curve: Handle<dyn Adaptor3dCurve> = Handle::default();
        let mut inverse = ConstRadInv::new(&null_surface, &null_surface, &null_curve);
        inverse.set_radius(2., 7);
        assert_eq!((inverse.ray1, inverse.ray2), (-2., 2.));
        assert_eq!((inverse.variables(), inverse.equations()), (4, 4));
        let x = super::super::linalg::vector(4).unwrap();
        let mut f = super::super::linalg::vector(4).unwrap();
        assert!(!inverse.value(&x, &mut f));
        assert_eq!(
            inverse.take_failure().unwrap_err().exception_type(),
            "Standard_NullObject"
        );
    }
}
