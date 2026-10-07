// Rust translation of BRepBlend_SurfRstConstRad.hxx/.cxx and BRepBlend_SurfRstEvolRad.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The fillets between a surface and a restriction of another surface: variables
//! `(U, V, W)`, the point of the surface and the parameter on the restriction. The
//! constant-radius and evolutive-radius classes share this core; where their text
//! differs, the core branches on the law.
mod classes;
mod sections;
mod solution;

pub use classes::{BRepBlendSurfRstConstRad, BRepBlendSurfRstEvolRad};

use crate::blend_func::SectionShape;
use crate::blend_func::choice::live_law;
use crate::blend_func::linalg::{mset, vget, vset, widen};
use crate::blend_func::package::{live_curve, live_surface};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::GeomSurfaceD2;
use rustycad_tkgeomalgo::law::LawFunction;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};
use rustycad_tkmath::math::{Matrix, Vector};

/// The surface, the surface of the restriction and the restriction of a surface-restriction
/// fillet (`Surf`, `SurfRst`, `Rst`).
pub type SurfRstInputs<'a> = (
    &'a Handle<dyn Adaptor3dSurface>,
    &'a Handle<dyn Adaptor3dSurface>,
    &'a Handle<dyn Adaptor2dCurve2d>,
);

/// The radius law of the evolutive class: the sign of the choice, the trimmed and full
/// laws, and the derivative at the current parameter.
pub(crate) struct SurfRstLaw {
    sg1: f64,
    tevol: Handle<dyn LawFunction>,
    fevol: Handle<dyn LawFunction>,
    dray: f64,
}

/// The fields of `BRepBlend_SurfRstConstRad` and `BRepBlend_SurfRstEvolRad`.
pub(crate) struct SurfRstCore {
    surf: Handle<dyn Adaptor3dSurface>,
    surfrst: Handle<dyn Adaptor3dSurface>,
    rst: Handle<dyn Adaptor2dCurve2d>,
    cons: Adaptor3dCurveOnSurface,
    guide: Handle<dyn Adaptor3dCurve>,
    tguide: Handle<dyn Adaptor3dCurve>,
    surfref: Handle<dyn Adaptor3dSurface>,
    rstref: Handle<dyn Adaptor2dCurve2d>,
    pts: Point3,
    ptrst: Point3,
    pt2ds: Point2,
    pt2drst: Point2,
    prmrst: f64,
    istangent: bool,
    tgs: Vector3,
    tg2ds: Vector2,
    tgrst: Vector3,
    tg2drst: Vector2,
    ray: f64,
    choix: i32,
    ptgui: Point3,
    d1gui: Vector3,
    d2gui: Vector3,
    nplan: Vector3,
    normtg: f64,
    the_d: f64,
    maxang: f64,
    minang: f64,
    distmin: f64,
    shape: SectionShape,
    tconv: ParameterisationType,
    law: Option<SurfRstLaw>,
    class: &'static str,
}

/// `t3dto2d(a, b, A, B, C)`: the coordinates `(a, b)` of the projection of `A` on the
/// plane of `B` and `C` in the basis `(B, C)`.
pub(crate) fn t3dto2d(a: Vector3, b: Vector3, c: Vector3) -> (f64, f64) {
    let (ab, ac, bc, bb, cc) = (a.dot(b), a.dot(c), b.dot(c), b.dot(b), c.dot(c));
    let deno = bb * cc - bc * bc;
    ((ab * cc - ac * bc) / deno, (ac * bb - ab * bc) / deno)
}

impl SurfRstCore {
    pub(crate) fn new(
        (surf, surf_rst, rst): SurfRstInputs<'_>,
        c_guide: &Handle<dyn Adaptor3dCurve>,
        law: Option<&Handle<dyn LawFunction>>,
        class: &'static str,
    ) -> Result<Self, Failure> {
        Ok(Self {
            surf: surf.clone(),
            surfrst: surf_rst.clone(),
            rst: rst.clone(),
            cons: Adaptor3dCurveOnSurface::with_curve(rst, surf_rst)?,
            guide: c_guide.clone(),
            tguide: c_guide.clone(),
            surfref: Handle::default(),
            rstref: Handle::default(),
            pts: Point3::default(),
            ptrst: Point3::default(),
            pt2ds: Point2::default(),
            pt2drst: Point2::default(),
            prmrst: 0.0,
            istangent: true,
            tgs: Vector3::default(),
            tg2ds: Vector2::default(),
            tgrst: Vector3::default(),
            tg2drst: Vector2::default(),
            ray: 0.0,
            choix: 0,
            ptgui: Point3::default(),
            d1gui: Vector3::default(),
            d2gui: Vector3::default(),
            nplan: Vector3::default(),
            normtg: 0.0,
            the_d: 0.0,
            maxang: f64::MIN,
            minang: f64::MAX,
            distmin: f64::MAX,
            shape: SectionShape::Rational,
            tconv: ParameterisationType::TgtThetaOver2,
            law: law.map(|law| SurfRstLaw {
                sg1: 0.0,
                tevol: law.clone(),
                fevol: law.clone(),
                dray: 0.0,
            }),
            class,
        })
    }

    /// `Set(SurfRef, RstRef)`: the reference surface and restriction of `Decroch`.
    pub(crate) fn set_refs(
        &mut self,
        surf_ref: &Handle<dyn Adaptor3dSurface>,
        rst_ref: &Handle<dyn Adaptor2dCurve2d>,
    ) {
        self.surfref = surf_ref.clone();
        self.rstref = rst_ref.clone();
    }

    /// `Set(Param)`: the guide plane at `param` (the derivative and the normal are reset
    /// first, as natively) and, with a law, the radius and its derivative.
    pub(crate) fn set_param(&mut self, param: f64) -> Result<(), Failure> {
        self.d1gui = Vector3::default();
        self.nplan = Vector3::default();
        (self.ptgui, self.d1gui, self.d2gui) = live_curve(&self.tguide)?.d2(param)?;
        self.normtg = self.d1gui.magnitude();
        self.nplan = self.d1gui.normalized()?;
        self.the_d = -self.nplan.xyz().dot(self.ptgui.xyz());
        let Some(law) = &mut self.law else {
            return Ok(());
        };
        let (ray, dray) = live_law(&law.tevol, self.class)?.d1(param)?;
        self.ray = law.sg1 * ray;
        law.dray = law.sg1 * dray;
        Ok(())
    }

    /// `Set(First, Last)`: the guide (and the law) trimmed to the useful part.
    pub(crate) fn set_interval(&mut self, first: f64, last: f64) -> Result<(), Failure> {
        self.tguide = live_curve(&self.guide)?.trim(first, last, 1.0e-12)?;
        if let Some(law) = &mut self.law {
            law.tevol = live_law(&law.fevol, self.class)?.trim(first, last, 1.0e-12)?;
        }
        Ok(())
    }

    /// The restriction curve, or Standard_NullObject.
    fn live_rst(&self) -> Result<&dyn Adaptor2dCurve2d, Failure> {
        super::live_curve2d_of(&self.rst, self.class)
    }

    /// The radius of the law at `param` (`sg1 * tevol->Value(Param)`), kept in `ray`.
    fn law_radius(&mut self, param: f64) -> Result<(), Failure> {
        if let Some(law) = &self.law {
            self.ray = law.sg1 * live_law(&law.tevol, self.class)?.value(param)?;
        }
        Ok(())
    }

    /// `GetTolerance(Tolerance, Tol)`.
    pub(crate) fn get_tolerance(
        &self,
        tolerance: &mut Vector<'_>,
        tol: f64,
    ) -> Result<(), Failure> {
        let surface = live_surface(&self.surf)?;
        vset(tolerance, 1, surface.u_resolution(tol)?)?;
        vset(tolerance, 2, surface.v_resolution(tol)?)?;
        vset(tolerance, 3, self.cons.resolution(tol)?)
    }

    /// `GetBounds(InfBound, SupBound)`: the surface bounds widened by their range.
    pub(crate) fn get_bounds(
        &self,
        inf: &mut Vector<'_>,
        sup: &mut Vector<'_>,
    ) -> Result<(), Failure> {
        let surface = live_surface(&self.surf)?;
        vset(inf, 1, surface.first_u_parameter()?)?;
        vset(inf, 2, surface.first_v_parameter()?)?;
        vset(inf, 3, self.cons.first_parameter()?)?;
        vset(sup, 1, surface.last_u_parameter()?)?;
        vset(sup, 2, surface.last_v_parameter()?)?;
        vset(sup, 3, self.cons.last_parameter()?)?;
        widen(inf, sup, 1)?;
        widen(inf, sup, 2)
    }

    /// `F(3)` from `vref`: `(|vref / ray|^2 - 1) ray^2` with a constant radius ("more
    /// stable numerically"), `|vref|^2 - ray^2` with a law.
    fn radius_residual(&self, vref: Vector3) -> f64 {
        if self.law.is_some() {
            return vref.square_magnitude() - self.ray * self.ray;
        }
        let scaled = vref / self.ray;
        (scaled.square_magnitude() - 1.) * self.ray * self.ray
    }

    /// `Value(X, F)`.
    pub(crate) fn value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure> {
        let d1 = live_surface(&self.surf)?.d1(vget(x, 1)?, vget(x, 2)?)?;
        self.pts = d1.point;
        self.ptrst = self.cons.value(vget(x, 3)?)?;
        vset(f, 1, self.nplan.xyz().dot(self.pts.xyz()) + self.the_d)?;
        vset(f, 2, self.nplan.xyz().dot(self.ptrst.xyz()) + self.the_d)?;
        let ns = d1.d1u.crossed(d1.d1v);
        let norm = self.nplan.crossed(ns).magnitude();
        let mut projected = Vector3::default();
        projected.set_linear_form(self.nplan.dot(ns) / norm, self.nplan, -1. / norm, ns);
        let mut vref = Vector3::default();
        vref.set_scaled_sum(self.ray, projected, Vector3::between(self.ptrst, self.pts));
        vset(f, 3, self.radius_residual(vref))?;
        Ok(true)
    }

    /// `Derivatives(X, D)` and, with `f`, `Values(X, F, D)`.
    pub(crate) fn jacobian(
        &mut self,
        x: &Vector<'_>,
        d: &mut Matrix<'_>,
        f: Option<&mut Vector<'_>>,
    ) -> Result<bool, Failure> {
        let s = live_surface(&self.surf)?.d2(vget(x, 1)?, vget(x, 2)?)?;
        self.pts = s.point;
        let (ptrst, d1) = self.cons.d1(vget(x, 3)?)?;
        self.ptrst = ptrst;
        let nplan = self.nplan;
        let rows = [
            [nplan.dot(s.d1u), nplan.dot(s.d1v), 0.],
            [0., 0., nplan.dot(d1)],
        ];
        for (i, row) in (1..).zip(rows) {
            for (j, value) in (1..).zip(row) {
                mset(d, i, j, value)?;
            }
        }
        let vref = self.radius_row(&s, d1, d)?;
        if let Some(f) = f {
            vset(f, 1, nplan.xyz().dot(self.pts.xyz()) + self.the_d)?;
            vset(f, 2, nplan.xyz().dot(self.ptrst.xyz()) + self.the_d)?;
            vset(f, 3, self.radius_residual(vref))?;
        }
        Ok(true)
    }

    /// `D(3, j)`; returns `vref`, the vector from the restriction point to the center.
    fn radius_row(
        &self,
        s: &GeomSurfaceD2,
        d1: Vector3,
        d: &mut Matrix<'_>,
    ) -> Result<Vector3, Failure> {
        let (nplan, ray) = (self.nplan, self.ray);
        let ns = s.d1u.crossed(s.d1v);
        let ncrossns = nplan.crossed(ns);
        let norm = ncrossns.magnitude();
        let ndotns = nplan.dot(ns);
        let mut vref = Vector3::default();
        vref.set_linear_form(ndotns, nplan, -1., ns);
        vref /= norm;
        let projected = vref;
        vref.set_scaled_sum(ray, projected, Vector3::between(self.ptrst, self.pts));
        let column = |temp: Vector3, d1: Vector3| {
            let grosterme = ncrossns.dot(nplan.crossed(temp)) / norm / norm;
            let mut resul = Vector3::default();
            resul.set_linear_form_three_offset(
                [
                    -ray / norm * (grosterme * ndotns - nplan.dot(temp)),
                    ray * grosterme / norm,
                    -ray / norm,
                ],
                [nplan, ns, temp],
                d1,
            );
            resul.dot(vref) * 2.
        };
        mset(
            d,
            3,
            1,
            column(s.d2u.crossed(s.d1v) + s.d1u.crossed(s.d2uv), s.d1u),
        )?;
        mset(
            d,
            3,
            2,
            column(s.d2uv.crossed(s.d1v) + s.d1u.crossed(s.d2v), s.d1v),
        )?;
        mset(d, 3, 3, d1.dot(vref) * (-2.))?;
        Ok(vref)
    }
}
