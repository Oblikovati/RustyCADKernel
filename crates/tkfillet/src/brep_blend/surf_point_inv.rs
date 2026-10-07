// Rust translation of BRepBlend_SurfPointConstRadInv.hxx/.cxx and
// BRepBlend_SurfPointEvolRadInv.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The inverses of the fillets between a surface and a restriction that end on a point:
//! variables `(w, U, V)`, the guide parameter and the point of the surface.
use super::live_curve_of;
use super::rad_inv_macro::impl_rad_inv;
use super::rad_inv_terms::{GuidePlane, NormalTerms, choice_ray, choice_sign, value_ref};
use crate::blend::{BlendInverseBounds, BlendSurfPointFuncInv, surf_point_func_inv_nb_variables};
use crate::blend_func::choice::live_law;
use crate::blend_func::linalg::{mset, vector, vget, vset};
use crate::blend_func::package::live_surface;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::GeomSurfaceD2;
use rustycad_tkgeomalgo::law::LawFunction;
use rustycad_tkgeombase::failure_capture::FailureCapture;
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tkmath::math::{FunctionSet, FunctionSetWithDerivatives, Matrix, Vector};

/// The radius of a surface-point inverse: a constant, or a law of the guide parameter.
enum PointRadius {
    Constant(f64),
    Evolutive {
        sg1: f64,
        law: Handle<dyn LawFunction>,
    },
}

/// `BRepBlend_SurfPointConstRadInv` and `BRepBlend_SurfPointEvolRadInv`: one class per
/// radius kind sharing these fields and equations.
struct SurfPointInv {
    surf: Handle<dyn Adaptor3dSurface>,
    curv: Handle<dyn Adaptor3dCurve>,
    point: Point3,
    ray: f64,
    choix: i32,
    radius: PointRadius,
    class: &'static str,
    capture: FailureCapture,
}

impl SurfPointInv {
    fn new(
        s: &Handle<dyn Adaptor3dSurface>,
        c: &Handle<dyn Adaptor3dCurve>,
        radius: PointRadius,
        class: &'static str,
    ) -> Self {
        Self {
            surf: s.clone(),
            curv: c.clone(),
            point: Point3::default(),
            ray: 0.0,
            choix: 0,
            radius,
            class,
            capture: FailureCapture::default(),
        }
    }

    /// The radius and its derivative at the guide parameter `w`: the law (signed by the
    /// choice) or the constant with a null derivative.
    fn radius_d1(&mut self, w: f64) -> Result<(f64, f64), Failure> {
        match &self.radius {
            PointRadius::Constant(ray) => Ok((*ray, 0.)),
            PointRadius::Evolutive { sg1, law } => {
                let (ray, dray) = live_law(law, self.class)?.d1(w)?;
                self.ray = sg1 * ray;
                Ok((self.ray, sg1 * dray))
            }
        }
    }

    /// `Value(X, F)`.
    fn value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure> {
        let (ptcur, d1cur) = live_curve_of(&self.curv, self.class)?.d1(vget(x, 1)?)?;
        let nplan = d1cur.normalized()?;
        if let PointRadius::Evolutive { sg1, law } = &self.radius {
            self.ray = sg1 * live_law(law, self.class)?.value(vget(x, 1)?)?;
        }
        let the_d = -nplan.xyz().dot(ptcur.xyz());
        let d1 = live_surface(&self.surf)?.d1(vget(x, 2)?, vget(x, 3)?)?;
        vset(f, 1, nplan.xyz().dot(self.point.xyz()) + the_d)?;
        vset(f, 2, nplan.xyz().dot(d1.point.xyz()) + the_d)?;
        let corde = Vector3::from(d1.point.xyz() - self.point.xyz());
        let reference = value_ref(self.ray, nplan, d1.d1u.crossed(d1.d1v), corde);
        vset(f, 3, reference.square_magnitude() - self.ray * self.ray)?;
        Ok(true)
    }

    /// `Derivatives(X, D)` and, with `f`, `Values(X, F, D)` (which evaluates the surface
    /// before the plane, as natively).
    fn jacobian(
        &mut self,
        x: &Vector<'_>,
        d: &mut Matrix<'_>,
        f: Option<&mut Vector<'_>>,
    ) -> Result<bool, Failure> {
        let w = vget(x, 1)?;
        let (ptcur, d1cur, d2cur) = live_curve_of(&self.curv, self.class)?.d2(w)?;
        let radius = self.radius_d1(w)?;
        let surface = live_surface(&self.surf)?;
        let (u, v) = (vget(x, 2)?, vget(x, 3)?);
        let early = if f.is_some() {
            Some(surface.d2(u, v)?)
        } else {
            None
        };
        let plane = GuidePlane::new(ptcur, d1cur, d2cur);
        mset(d, 1, 1, plane.derivative_offset(self.point))?;
        mset(d, 1, 2, 0.)?;
        mset(d, 1, 3, 0.)?;
        let s = match early {
            Some(s) => s,
            None => surface.d2(u, v)?,
        };
        self.radius_rows(&plane, &s, radius, (d, f))?;
        Ok(true)
    }

    /// `D(2, j)`, `D(3, j)` and, for `Values`, `F`.
    fn radius_rows(
        &self,
        plane: &GuidePlane,
        s: &GeomSurfaceD2,
        (ray, dray): (f64, f64),
        (d, f): (&mut Matrix<'_>, Option<&mut Vector<'_>>),
    ) -> Result<(), Failure> {
        let nplan = plane.nplan;
        mset(d, 2, 1, plane.derivative_offset(s.point))?;
        mset(d, 2, 2, nplan.dot(s.d1u))?;
        mset(d, 2, 3, nplan.dot(s.d1v))?;
        let nsurf = s.d1u.crossed(s.d1v);
        let terms = NormalTerms::new(ray, nplan, nsurf);
        let reference = terms.reference(Vector3::between(self.point, s.point));
        if let Some(f) = f {
            vset(f, 1, plane.offset(self.point))?;
            vset(f, 2, plane.offset(s.point))?;
            vset(f, 3, reference.square_magnitude() - ray * ray)?;
        }
        let dwref = terms.guide_ref(nplan, plane.dnplan, nsurf);
        let duref = terms.surface_ref(nplan, s.d2u.crossed(s.d1v) + s.d1u.crossed(s.d2uv), s.d1u);
        let dvref = terms.surface_ref(nplan, s.d1u.crossed(s.d2v) + s.d2uv.crossed(s.d1v), s.d1v);
        let reference = reference + reference;
        mset(
            d,
            3,
            1,
            self.guide_term(&terms, reference, dwref, (ray, dray)),
        )?;
        mset(d, 3, 2, reference.dot(duref))?;
        mset(d, 3, 3, reference.dot(dvref))
    }

    /// `D(3, 1)`: with a law, `dwref` gains `dray unsurnorm temp` and the law term
    /// `-2 dray ray`.
    fn guide_term(
        &self,
        terms: &NormalTerms,
        reference: Vector3,
        dwref: Vector3,
        (ray, dray): (f64, f64),
    ) -> f64 {
        if let PointRadius::Constant(_) = self.radius {
            return reference.dot(dwref);
        }
        reference.dot(terms.with_law(dwref, dray)) - 2. * dray * ray
    }

    fn get_tolerance(&self, tolerance: &mut Vector<'_>, tol: f64) -> Result<(), Failure> {
        let surface = live_surface(&self.surf)?;
        vset(
            tolerance,
            1,
            live_curve_of(&self.curv, self.class)?.resolution(tol)?,
        )?;
        vset(tolerance, 2, surface.u_resolution(tol)?)?;
        vset(tolerance, 3, surface.v_resolution(tol)?)
    }

    fn get_bounds(&self, inf: &mut Vector<'_>, sup: &mut Vector<'_>) -> Result<(), Failure> {
        let (curve, surface) = (
            live_curve_of(&self.curv, self.class)?,
            live_surface(&self.surf)?,
        );
        vset(inf, 1, curve.first_parameter()?)?;
        vset(sup, 1, curve.last_parameter()?)?;
        vset(inf, 2, surface.first_u_parameter()?)?;
        vset(sup, 2, surface.last_u_parameter()?)?;
        vset(inf, 3, surface.first_v_parameter()?)?;
        vset(sup, 3, surface.last_v_parameter()?)
    }

    /// `IsSolution(Sol, Tol)`: `|F(1)|, |F(2)| <= tol` and `|F(3)| <= 2 tol |ray|`.
    fn is_solution(&mut self, sol: &Vector<'_>, tol: f64) -> Result<bool, Failure> {
        let mut valsol = vector(3)?;
        self.value(sol, &mut valsol)?;
        Ok(vget(&valsol, 1)?.abs() <= tol
            && vget(&valsol, 2)?.abs() <= tol
            && vget(&valsol, 3)?.abs() <= 2. * tol * self.ray.abs())
    }
}

/// `BRepBlend_SurfPointConstRadInv`: the constant-radius section through a point.
///
/// ```ignore
/// let mut inverse = BRepBlendSurfPointConstRadInv::new(&surface, &guide);
/// inverse.set_radius(1.0, 1);
/// inverse.set_point(point);
/// ```
pub struct BRepBlendSurfPointConstRadInv(SurfPointInv);

impl BRepBlendSurfPointConstRadInv {
    /// `BRepBlend_SurfPointConstRadInv(S, C)`; `BRepBlendSurfPointConstRadInv::new(&s, &c)`.
    pub fn new(s: &Handle<dyn Adaptor3dSurface>, c: &Handle<dyn Adaptor3dCurve>) -> Self {
        Self(SurfPointInv::new(
            s,
            c,
            PointRadius::Constant(0.0),
            "BRepBlend_SurfPointConstRadInv",
        ))
    }

    /// `Set(R, Choix)`: the radius, negative but for the choices 3 and 4;
    /// `inverse.set_radius(1.0, 3)`.
    pub fn set_radius(&mut self, radius: f64, choix: i32) {
        self.0.choix = choix;
        self.0.ray = choice_ray(radius, choix);
        self.0.radius = PointRadius::Constant(self.0.ray);
    }
}

impl_rad_inv!(
    BRepBlendSurfPointConstRadInv,
    surf_point_func_inv_nb_variables(),
    3
);

impl BlendSurfPointFuncInv for BRepBlendSurfPointConstRadInv {
    /// `Set(P)`: the point the section ends on.
    fn set_point(&mut self, point: Point3) {
        self.0.point = point;
    }
}

/// `BRepBlend_SurfPointEvolRadInv`: the evolutive-radius section through a point.
///
/// ```ignore
/// let mut inverse = BRepBlendSurfPointEvolRadInv::new(&surface, &guide, &law);
/// inverse.set_choix(1);
/// ```
pub struct BRepBlendSurfPointEvolRadInv(SurfPointInv);

impl BRepBlendSurfPointEvolRadInv {
    /// `BRepBlend_SurfPointEvolRadInv(S, C, Evol)`: the sign is unset (0) until `Set(Choix)`.
    pub fn new(
        s: &Handle<dyn Adaptor3dSurface>,
        c: &Handle<dyn Adaptor3dCurve>,
        evol: &Handle<dyn LawFunction>,
    ) -> Self {
        let radius = PointRadius::Evolutive {
            sg1: 0.0,
            law: evol.clone(),
        };
        Self(SurfPointInv::new(
            s,
            c,
            radius,
            "BRepBlend_SurfPointEvolRadInv",
        ))
    }

    /// `Set(Choix)`: the sign of the radius, negative but for the choices 3 and 4;
    /// `inverse.set_choix(3)`.
    pub fn set_choix(&mut self, choix: i32) {
        self.0.choix = choix;
        if let PointRadius::Evolutive { sg1, .. } = &mut self.0.radius {
            *sg1 = choice_sign(choix);
        }
    }
}

impl_rad_inv!(
    BRepBlendSurfPointEvolRadInv,
    surf_point_func_inv_nb_variables(),
    3
);

impl BlendSurfPointFuncInv for BRepBlendSurfPointEvolRadInv {
    /// `Set(P)`: the point the section ends on.
    fn set_point(&mut self, point: Point3) {
        self.0.point = point;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_choice_signs_the_radius() {
        assert_eq!(
            (choice_ray(-2.0, 3), choice_ray(2.0, 1), choice_ray(2.0, 9)),
            (2.0, -2.0, -2.0)
        );
        assert_eq!((choice_sign(4), choice_sign(5)), (1.0, -1.0));
        let null_surface: Handle<dyn Adaptor3dSurface> = Handle::default();
        let null_curve: Handle<dyn Adaptor3dCurve> = Handle::default();
        let mut inverse = BRepBlendSurfPointConstRadInv::new(&null_surface, &null_curve);
        inverse.set_radius(2.0, 4);
        assert_eq!(
            (inverse.0.ray, inverse.variables(), inverse.equations()),
            (2.0, 3, 3)
        );
        let mut f = vector(3).unwrap();
        assert!(!inverse.value(&vector(3).unwrap(), &mut f));
        assert_eq!(
            inverse.take_failure().unwrap_err().exception_type(),
            "Standard_NullObject"
        );
    }
}
