//! The equations shared by the inverses of the two fillet functions
//! (`BlendFunc_ConstRadInv`, `BlendFunc_EvolRadInv`): the section that ends on a curve on
//! one of the surfaces, of variables `(w, t, U, V)`.
//!
//! Both classes have the same text up to the radius: a constant pair of signed radii, or
//! the signs of the choice times a law value (with its derivative along the guide).
mod derivatives;

pub(crate) use derivatives::PlaneDerivative;

use super::linalg::{vget, vset, widen};
use super::normal_terms::EPS;
use super::package::{compute_normal, live_curve, live_surface};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkmath::gp::{Point2, Point3, Vector3};
use rustycad_tkmath::math::Vector;

/// The radius of an inverse fillet function at the current guide parameter.
#[derive(Clone, Copy, Debug)]
pub(crate) enum InverseRadius {
    /// `BlendFunc_ConstRadInv`: the signed radii.
    Constant { ray1: f64, ray2: f64 },
    /// `BlendFunc_EvolRadInv`: the signs of the choice, the law value and its derivative.
    Evolutive {
        sg1: f64,
        sg2: f64,
        ray: f64,
        dray: f64,
    },
}

impl InverseRadius {
    /// The signed radii: `ray1, ray2`, or `sg1 * ray, sg2 * ray` (the same bits as the
    /// native `sg1 * ray` products, the signs being exactly +-1).
    pub(crate) fn signed(self) -> (f64, f64) {
        match self {
            Self::Constant { ray1, ray2 } => (ray1, ray2),
            Self::Evolutive { sg1, sg2, ray, .. } => (sg1 * ray, sg2 * ray),
        }
    }
}

/// The fields of both inverse classes; `class` names the native class in failures.
pub(crate) struct InverseCore {
    pub(crate) surf1: Handle<dyn Adaptor3dSurface>,
    pub(crate) surf2: Handle<dyn Adaptor3dSurface>,
    pub(crate) curv: Handle<dyn Adaptor3dCurve>,
    pub(crate) csurf: Handle<dyn Adaptor2dCurve2d>,
    pub(crate) choix: i32,
    pub(crate) first: bool,
    pub(crate) class: &'static str,
}

impl InverseCore {
    /// The fields after the native constructor: no curve on surface, `first` false.
    pub(crate) fn new(
        s1: &Handle<dyn Adaptor3dSurface>,
        s2: &Handle<dyn Adaptor3dSurface>,
        c: &Handle<dyn Adaptor3dCurve>,
        class: &'static str,
    ) -> Self {
        Self {
            surf1: s1.clone(),
            surf2: s2.clone(),
            curv: c.clone(),
            csurf: Handle::default(),
            choix: 0,
            first: false,
            class,
        }
    }

    /// `Set(OnFirst, COnSurf)`.
    pub(crate) fn set_curve(&mut self, on_first: bool, curve: &Handle<dyn Adaptor2dCurve2d>) {
        self.first = on_first;
        self.csurf = curve.clone();
    }

    pub(crate) fn live_csurf(&self) -> Result<&dyn Adaptor2dCurve2d, Failure> {
        self.csurf.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                &format!(
                    "{}: no curve on surface; expected Set(OnFirst, COnSurf)",
                    self.class
                ),
            )
        })
    }

    /// The surface the curve lies on is `surf1` when `first`; this is the other one.
    fn other_surface(&self) -> &Handle<dyn Adaptor3dSurface> {
        if self.first { &self.surf2 } else { &self.surf1 }
    }

    /// The `(U, V)` of both surfaces: the curve point `on_curve` and `(X(3), X(4))`.
    fn surface_parameters(&self, on_curve: Point2, other: Point2) -> (Point2, Point2) {
        if self.first {
            (on_curve, other)
        } else {
            (other, on_curve)
        }
    }

    /// `Value(X, F)` once the radius at `X(2)` is known.
    pub(crate) fn value(
        &self,
        x: &Vector<'_>,
        f: &mut Vector<'_>,
        radius: InverseRadius,
    ) -> Result<bool, Failure> {
        let (ptcur, d1cur) = live_curve(&self.curv)?.d1(vget(x, 2)?)?;
        let nplan = d1cur.normalized()?;
        let the_d = -nplan.xyz().dot(ptcur.xyz());
        let pt2d = self.live_csurf()?.value(vget(x, 1)?)?;
        let (uv1, uv2) = self.surface_parameters(pt2d, Point2::new(vget(x, 3)?, vget(x, 4)?));
        let d1 = live_surface(&self.surf1)?.d1(uv1.x(), uv1.y())?;
        let d2 = live_surface(&self.surf2)?.d1(uv2.x(), uv2.y())?;
        let (pts1, pts2) = (d1.point, d2.point);
        vset(f, 1, plane_residual(nplan, the_d, pts1, pts2))?;
        let ns1 = repaired(&self.surf1, uv1, d1.d1u.crossed(d1.d1v))?;
        let ns2 = repaired(&self.surf2, uv2, d2.d1u.crossed(d2.d1v))?;
        let resul = section_residual(radius.signed(), nplan, (ns1, ns2), (pts1, pts2));
        vset(f, 2, resul.x())?;
        vset(f, 3, resul.y())?;
        vset(f, 4, resul.z())?;
        Ok(true)
    }

    /// `GetTolerance(Tolerance, Tol)`: the resolutions of the curve on surface, of the
    /// guide and of the other surface.
    pub(crate) fn get_tolerance(
        &self,
        tolerance: &mut Vector<'_>,
        tol: f64,
    ) -> Result<(), Failure> {
        vset(tolerance, 1, self.live_csurf()?.resolution(tol)?)?;
        vset(tolerance, 2, live_curve(&self.curv)?.resolution(tol)?)?;
        let other = live_surface(self.other_surface())?;
        vset(tolerance, 3, other.u_resolution(tol)?)?;
        vset(tolerance, 4, other.v_resolution(tol)?)
    }

    /// `GetBounds(InfBound, SupBound)`: the bounds of the curve on surface, of the guide,
    /// and of the other surface widened by its range.
    pub(crate) fn get_bounds(
        &self,
        inf: &mut Vector<'_>,
        sup: &mut Vector<'_>,
    ) -> Result<(), Failure> {
        let (csurf, curv) = (self.live_csurf()?, live_curve(&self.curv)?);
        vset(inf, 1, csurf.first_parameter()?)?;
        vset(inf, 2, curv.first_parameter()?)?;
        vset(sup, 1, csurf.last_parameter()?)?;
        vset(sup, 2, curv.last_parameter()?)?;
        let other = live_surface(self.other_surface())?;
        vset(inf, 3, other.first_u_parameter()?)?;
        vset(inf, 4, other.first_v_parameter()?)?;
        vset(sup, 3, other.last_u_parameter()?)?;
        vset(sup, 4, other.last_v_parameter()?)?;
        widen(inf, sup, 3)?;
        widen(inf, sup, 4)
    }
}

/// `IsSolution` of the fillet inverses: `Value` (the closure) at the solution, then
/// whether `|F(1)| <= tol` and `|F(2..4)|^2 <= tol^2`.
pub(crate) fn residual_is_solution(
    value: impl FnOnce(&mut Vector<'static>) -> Result<bool, Failure>,
    tol: f64,
) -> Result<bool, Failure> {
    let mut valsol = super::linalg::vector(4)?;
    value(&mut valsol)?;
    let valsol = &valsol;
    let (a, b, c, d) = (
        vget(valsol, 1)?,
        vget(valsol, 2)?,
        vget(valsol, 3)?,
        vget(valsol, 4)?,
    );
    Ok(a.abs() <= tol && b * b + c * c + d * d <= tol * tol)
}

/// `F(1)`: the middle of the two points in the plane normal to the guide.
fn plane_residual(nplan: Vector3, the_d: f64, pts1: Point3, pts2: Point3) -> f64 {
    (nplan.x() * (pts1.x() + pts2.x())
        + nplan.y() * (pts1.y() + pts2.y())
        + nplan.z() * (pts1.z() + pts2.z()))
        / 2.
        + the_d
}

/// `resul.SetLinearForm(ray1, ns1, -1., pts2, -ray2, ns2, pts1)` with the projected
/// normals: `F(2..4)` of `Value`.
fn section_residual(
    (ray1, ray2): (f64, f64),
    nplan: Vector3,
    (ns1, ns2): (Vector3, Vector3),
    (pts1, pts2): (Point3, Point3),
) -> Vector3 {
    let (ns1, ns2) = (project(nplan, ns1), project(nplan, ns2));
    let mut resul = Vector3::default();
    resul.set_linear_form_three_offset(
        [ray1, -1., -ray2],
        [ns1, Vector3::from(pts2.xyz()), ns2],
        Vector3::from(pts1.xyz()),
    );
    resul
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
