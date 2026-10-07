// Rust translation of BRepBlend_SurfCurvConstRadInv.hxx/.cxx and
// BRepBlend_SurfCurvEvolRadInv.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The inverses of the fillets between a surface and a restriction that end on a curve:
//! variables `(w, wcurv, wrst)`, the guide parameter, the parameter on the curve and the
//! parameter on the restriction of the surface.
use super::rad_inv_macro::impl_rad_inv;
use super::rad_inv_terms::{GuidePlane, NormalTerms, choice_ray, choice_sign, value_ref};
use super::{live_curve_of, live_curve2d_of};
use crate::blend::{BlendInverseBounds, BlendSurfCurvFuncInv, surf_curv_func_inv_nb_variables};
use crate::blend_func::choice::live_law;
use crate::blend_func::linalg::{mset, vector, vget, vset};
use crate::blend_func::package::live_surface;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkgeomalgo::law::LawFunction;
use rustycad_tkgeombase::failure_capture::FailureCapture;
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tkmath::math::{FunctionSet, FunctionSetWithDerivatives, Matrix, Vector};

/// The radius of a surface-curve inverse: a constant, or a law of the guide parameter.
enum CurvRadius {
    Constant,
    Evolutive {
        sg1: f64,
        law: Handle<dyn LawFunction>,
    },
}

/// The points of the jacobian: the curve point and derivative, the restriction derivative
/// and the surface derivatives at the restriction point.
struct CurvJet {
    ptcur: Point3,
    d1cur: Vector3,
    dwrstpts: Vector3,
    dwrstnsurf: Vector3,
    s: rustycad_tkg3d::geom::GeomSurfaceD2,
}

/// The fields and equations both surface-curve inverses share.
struct SurfCurvInv {
    surf: Handle<dyn Adaptor3dSurface>,
    curv: Handle<dyn Adaptor3dCurve>,
    guide: Handle<dyn Adaptor3dCurve>,
    rst: Handle<dyn Adaptor2dCurve2d>,
    ray: f64,
    choix: i32,
    radius: CurvRadius,
    class: &'static str,
    capture: FailureCapture,
}

impl SurfCurvInv {
    fn new(
        s: &Handle<dyn Adaptor3dSurface>,
        c: &Handle<dyn Adaptor3dCurve>,
        cg: &Handle<dyn Adaptor3dCurve>,
        radius: CurvRadius,
        class: &'static str,
    ) -> Self {
        Self {
            surf: s.clone(),
            curv: c.clone(),
            guide: cg.clone(),
            rst: Handle::default(),
            ray: 0.0,
            choix: 0,
            radius,
            class,
            capture: FailureCapture::default(),
        }
    }

    /// The radius and its derivative at `w`, right after the guide evaluation.
    fn radius_d1(&mut self, w: f64) -> Result<(f64, f64), Failure> {
        let CurvRadius::Evolutive { sg1, law } = &self.radius else {
            return Ok((self.ray, 0.));
        };
        let (ray, dray) = live_law(law, self.class)?.d1(w)?;
        self.ray = sg1 * ray;
        Ok((self.ray, sg1 * dray))
    }

    /// `Value(X, F)`.
    fn value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure> {
        let w = vget(x, 1)?;
        let (ptgui, d1gui) = live_curve_of(&self.guide, self.class)?.d1(w)?;
        if let CurvRadius::Evolutive { sg1, law } = &self.radius {
            self.ray = sg1 * live_law(law, self.class)?.value(w)?;
        }
        let nplan = d1gui.normalized()?;
        let the_d = -nplan.xyz().dot(ptgui.xyz());
        let ptcur = live_curve_of(&self.curv, self.class)?.value(vget(x, 2)?)?;
        vset(f, 1, nplan.xyz().dot(ptcur.xyz()) + the_d)?;
        let p2drst = live_curve2d_of(&self.rst, self.class)?.value(vget(x, 3)?)?;
        let d1 = live_surface(&self.surf)?.d1(p2drst.x(), p2drst.y())?;
        vset(f, 2, nplan.xyz().dot(d1.point.xyz()) + the_d)?;
        let reference = value_ref(
            self.ray,
            nplan,
            d1.d1u.crossed(d1.d1v),
            Vector3::between(ptcur, d1.point),
        );
        vset(f, 3, reference.square_magnitude() - self.ray * self.ray)?;
        Ok(true)
    }

    /// `Derivatives(X, D)` and, with `f`, `Values(X, F, D)`.
    fn jacobian(
        &mut self,
        x: &Vector<'_>,
        d: &mut Matrix<'_>,
        mut f: Option<&mut Vector<'_>>,
    ) -> Result<bool, Failure> {
        let w = vget(x, 1)?;
        let (ptgui, d1gui, d2gui) = live_curve_of(&self.guide, self.class)?.d2(w)?;
        let radius = self.radius_d1(w)?;
        let plane = GuidePlane::new(ptgui, d1gui, d2gui);
        let jet = self.curve_rows(x, &plane, d, f.as_deref_mut())?;
        self.radius_row(&plane, &jet, radius, (d, f))?;
        Ok(true)
    }

    /// `D(1, j)`, `D(2, j)` (and `F(1)`, `F(2)`): the curve and the restriction points in
    /// the guide plane.
    fn curve_rows(
        &self,
        x: &Vector<'_>,
        plane: &GuidePlane,
        d: &mut Matrix<'_>,
        mut f: Option<&mut Vector<'_>>,
    ) -> Result<CurvJet, Failure> {
        let (ptcur, d1cur) = live_curve_of(&self.curv, self.class)?.d1(vget(x, 2)?)?;
        if let Some(f) = f.as_deref_mut() {
            vset(f, 1, plane.offset(ptcur))?;
        }
        mset(d, 1, 1, plane.derivative_offset(ptcur))?;
        mset(d, 1, 2, plane.nplan.xyz().dot(*d1cur.xyz()))?;
        mset(d, 1, 3, 0.)?;
        let (p2drst, d1rst) = live_curve2d_of(&self.rst, self.class)?.d1(vget(x, 3)?)?;
        let s = live_surface(&self.surf)?.d2(p2drst.x(), p2drst.y())?;
        if let Some(f) = f {
            vset(f, 2, plane.offset(s.point))?;
        }
        mset(d, 2, 1, plane.derivative_offset(s.point))?;
        mset(d, 2, 2, 0.)?;
        let mut dwrstpts = Vector3::default();
        dwrstpts.set_linear_form(d1rst.x(), s.d1u, d1rst.y(), s.d1v);
        mset(d, 2, 3, plane.nplan.xyz().dot(*dwrstpts.xyz()))?;
        let dunsurf = s.d2u.crossed(s.d1v) + s.d1u.crossed(s.d2uv);
        let dvnsurf = s.d1u.crossed(s.d2v) + s.d2uv.crossed(s.d1v);
        let mut dwrstnsurf = Vector3::default();
        dwrstnsurf.set_linear_form(d1rst.x(), dunsurf, d1rst.y(), dvnsurf);
        Ok(CurvJet {
            ptcur,
            d1cur,
            dwrstpts,
            dwrstnsurf,
            s,
        })
    }

    /// `D(3, j)` (and `F(3)`): the radius equation.
    fn radius_row(
        &self,
        plane: &GuidePlane,
        jet: &CurvJet,
        (ray, dray): (f64, f64),
        (d, f): (&mut Matrix<'_>, Option<&mut Vector<'_>>),
    ) -> Result<(), Failure> {
        let nsurf = jet.s.d1u.crossed(jet.s.d1v);
        let terms = NormalTerms::new(ray, plane.nplan, nsurf);
        let reference = terms.reference(Vector3::between(jet.ptcur, jet.s.point));
        if let Some(f) = f {
            vset(f, 3, reference.square_magnitude() - ray * ray)?;
        }
        let dwguiref = terms.guide_ref(plane.nplan, plane.dnplan, nsurf);
        let dwrstref = terms.surface_ref(plane.nplan, jet.dwrstnsurf, jet.dwrstpts);
        let reference = reference + reference;
        let d31 = match self.radius {
            CurvRadius::Constant => reference.dot(dwguiref),
            CurvRadius::Evolutive { .. } => {
                reference.dot(terms.with_law(dwguiref, dray)) - 2. * dray * ray
            }
        };
        mset(d, 3, 1, d31)?;
        mset(d, 3, 2, -reference.dot(jet.d1cur))?;
        mset(d, 3, 3, reference.dot(dwrstref))
    }

    fn get_tolerance(&self, tolerance: &mut Vector<'_>, tol: f64) -> Result<(), Failure> {
        vset(
            tolerance,
            1,
            live_curve_of(&self.guide, self.class)?.resolution(tol)?,
        )?;
        vset(
            tolerance,
            2,
            live_curve_of(&self.curv, self.class)?.resolution(tol)?,
        )?;
        let surface = live_surface(&self.surf)?;
        let (ru, rv) = (surface.u_resolution(tol)?, surface.v_resolution(tol)?);
        vset(
            tolerance,
            3,
            live_curve2d_of(&self.rst, self.class)?.resolution(std_min(ru, rv))?,
        )
    }

    fn get_bounds(&self, inf: &mut Vector<'_>, sup: &mut Vector<'_>) -> Result<(), Failure> {
        let (guide, curv) = (
            live_curve_of(&self.guide, self.class)?,
            live_curve_of(&self.curv, self.class)?,
        );
        let rst = live_curve2d_of(&self.rst, self.class)?;
        vset(inf, 1, guide.first_parameter()?)?;
        vset(sup, 1, guide.last_parameter()?)?;
        vset(inf, 2, curv.first_parameter()?)?;
        vset(sup, 2, curv.last_parameter()?)?;
        vset(inf, 3, rst.first_parameter()?)?;
        vset(sup, 3, rst.last_parameter()?)
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

/// `BRepBlend_SurfCurvConstRadInv`: the constant-radius section ending on a curve.
///
/// ```ignore
/// let mut inverse = BRepBlendSurfCurvConstRadInv::new(&surface, &curve, &guide);
/// inverse.set_radius(1.0, 1);
/// inverse.set_rst(&restriction);
/// ```
pub struct BRepBlendSurfCurvConstRadInv(SurfCurvInv);

impl BRepBlendSurfCurvConstRadInv {
    /// `BRepBlend_SurfCurvConstRadInv(S, C, Cg)`.
    pub fn new(
        s: &Handle<dyn Adaptor3dSurface>,
        c: &Handle<dyn Adaptor3dCurve>,
        cg: &Handle<dyn Adaptor3dCurve>,
    ) -> Self {
        Self(SurfCurvInv::new(
            s,
            c,
            cg,
            CurvRadius::Constant,
            "BRepBlend_SurfCurvConstRadInv",
        ))
    }

    /// `Set(R, Choix)`: the radius, negative but for the choices 3 and 4.
    pub fn set_radius(&mut self, radius: f64, choix: i32) {
        self.0.choix = choix;
        self.0.ray = choice_ray(radius, choix);
    }
}

/// `BRepBlend_SurfCurvEvolRadInv`: the evolutive-radius section ending on a curve.
///
/// ```ignore
/// let mut inverse = BRepBlendSurfCurvEvolRadInv::new(&surface, &curve, &guide, &law);
/// inverse.set_choix(1);
/// ```
pub struct BRepBlendSurfCurvEvolRadInv(SurfCurvInv);

impl BRepBlendSurfCurvEvolRadInv {
    /// `BRepBlend_SurfCurvEvolRadInv(S, C, Cg, Evol)`: the sign is 0 until `Set(Choix)`.
    pub fn new(
        s: &Handle<dyn Adaptor3dSurface>,
        c: &Handle<dyn Adaptor3dCurve>,
        cg: &Handle<dyn Adaptor3dCurve>,
        evol: &Handle<dyn LawFunction>,
    ) -> Self {
        let radius = CurvRadius::Evolutive {
            sg1: 0.0,
            law: evol.clone(),
        };
        Self(SurfCurvInv::new(
            s,
            c,
            cg,
            radius,
            "BRepBlend_SurfCurvEvolRadInv",
        ))
    }

    /// `Set(Choix)`: the sign of the radius, negative but for the choices 3 and 4.
    pub fn set_choix(&mut self, choix: i32) {
        self.0.choix = choix;
        if let CurvRadius::Evolutive { sg1, .. } = &mut self.0.radius {
            *sg1 = choice_sign(choix);
        }
    }
}

macro_rules! impl_surf_curv_rst {
    ($class:ty) => {
        impl_rad_inv!($class, surf_curv_func_inv_nb_variables(), 3);

        impl BlendSurfCurvFuncInv for $class {
            /// `Set(Rst)`: the restriction the section ends on.
            fn set_rst(&mut self, rst: &Handle<dyn Adaptor2dCurve2d>) {
                self.0.rst = rst.clone();
            }
        }
    };
}

impl_surf_curv_rst!(BRepBlendSurfCurvConstRadInv);
impl_surf_curv_rst!(BRepBlendSurfCurvEvolRadInv);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_restriction_is_a_null_object() {
        let surface: Handle<dyn Adaptor3dSurface> = Handle::default();
        let curve: Handle<dyn Adaptor3dCurve> = Handle::default();
        let mut inverse = BRepBlendSurfCurvConstRadInv::new(&surface, &curve, &curve);
        inverse.set_radius(1.5, 3);
        assert_eq!(
            (inverse.0.ray, inverse.variables(), inverse.equations()),
            (1.5, 3, 3)
        );
        let (mut inf, mut sup) = (vector(3).unwrap(), vector(3).unwrap());
        let error = inverse.get_bounds(&mut inf, &mut sup).unwrap_err();
        assert_eq!(error.exception_type(), "Standard_NullObject");
    }
}
