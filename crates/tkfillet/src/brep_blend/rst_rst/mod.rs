// Rust translation of BRepBlend_RstRstConstRad.hxx/.cxx and BRepBlend_RstRstEvolRad.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The fillets between two restrictions: variables `(U, V)`, the parameters on the
//! restrictions. The constant-radius and evolutive-radius classes share this core; where
//! their text differs, the core branches on the law.
mod classes;
mod sections;
mod solution;

pub use classes::{BRepBlendRstRstConstRad, BRepBlendRstRstEvolRad, RstRstInputs};

use crate::blend_func::SectionShape;
use crate::blend_func::choice::live_law;
use crate::blend_func::linalg::{mset, vget, vset};
use crate::blend_func::package::live_curve;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkgeomalgo::law::LawFunction;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};
use rustycad_tkmath::math::{Matrix, Vector};

/// The radius law of the evolutive class (its value is the radius, unsigned).
pub(crate) struct RstRstLaw {
    tevol: Handle<dyn LawFunction>,
    fevol: Handle<dyn LawFunction>,
    dray: f64,
}

/// One restriction: its surface, its curve and the curve on the surface.
pub(crate) struct Restriction {
    surf: Handle<dyn Adaptor3dSurface>,
    rst: Handle<dyn Adaptor2dCurve2d>,
    cons: Adaptor3dCurveOnSurface,
}

/// The fields of `BRepBlend_RstRstConstRad` and `BRepBlend_RstRstEvolRad`.
pub(crate) struct RstRstCore {
    r1: Restriction,
    r2: Restriction,
    guide: Handle<dyn Adaptor3dCurve>,
    tguide: Handle<dyn Adaptor3dCurve>,
    refs: [(Handle<dyn Adaptor3dSurface>, Handle<dyn Adaptor2dCurve2d>); 2],
    ptrst1: Point3,
    ptrst2: Point3,
    pt2drst1: Point2,
    pt2drst2: Point2,
    prmrst1: f64,
    prmrst2: f64,
    istangent: bool,
    tgrst1: Vector3,
    tg2drst1: Vector2,
    tgrst2: Vector3,
    tg2drst2: Vector2,
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
    law: Option<RstRstLaw>,
    class: &'static str,
}

impl Restriction {
    fn new(
        surf: &Handle<dyn Adaptor3dSurface>,
        rst: &Handle<dyn Adaptor2dCurve2d>,
    ) -> Result<Self, Failure> {
        Ok(Self {
            surf: surf.clone(),
            rst: rst.clone(),
            cons: Adaptor3dCurveOnSurface::with_curve(rst, surf)?,
        })
    }
}

impl RstRstCore {
    pub(crate) fn new(
        (surf1, rst1, surf2, rst2): RstRstInputs<'_>,
        c_guide: &Handle<dyn Adaptor3dCurve>,
        law: Option<&Handle<dyn LawFunction>>,
        class: &'static str,
    ) -> Result<Self, Failure> {
        Ok(Self {
            r1: Restriction::new(surf1, rst1)?,
            r2: Restriction::new(surf2, rst2)?,
            guide: c_guide.clone(),
            tguide: c_guide.clone(),
            refs: Default::default(),
            ptrst1: Point3::default(),
            ptrst2: Point3::default(),
            pt2drst1: Point2::default(),
            pt2drst2: Point2::default(),
            prmrst1: 0.0,
            prmrst2: 0.0,
            istangent: true,
            tgrst1: Vector3::default(),
            tg2drst1: Vector2::default(),
            tgrst2: Vector3::default(),
            tg2drst2: Vector2::default(),
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
            law: law.map(|law| RstRstLaw {
                tevol: law.clone(),
                fevol: law.clone(),
                dray: 0.0,
            }),
            class,
        })
    }

    /// `Set(SurfRef1, RstRef1, SurfRef2, RstRef2)`: the references of `Decroch`.
    pub(crate) fn set_refs(&mut self, refs: RstRstInputs<'_>) {
        let (s1, r1, s2, r2) = refs;
        self.refs = [(s1.clone(), r1.clone()), (s2.clone(), r2.clone())];
    }

    /// `Set(Param)`: the guide plane at `param` and, with a law, the radius and its
    /// derivative (the evolutive class resets the derivative and the normal first).
    pub(crate) fn set_param(&mut self, param: f64) -> Result<(), Failure> {
        if self.law.is_some() {
            self.d1gui = Vector3::default();
            self.nplan = Vector3::default();
        }
        (self.ptgui, self.d1gui, self.d2gui) = live_curve(&self.tguide)?.d2(param)?;
        self.normtg = self.d1gui.magnitude();
        self.nplan = self.d1gui.normalized()?;
        self.the_d = -self.nplan.xyz().dot(self.ptgui.xyz());
        let Some(law) = &mut self.law else {
            return Ok(());
        };
        (self.ray, law.dray) = live_law(&law.tevol, self.class)?.d1(param)?;
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

    /// The radius of the law at `param` (`tevol->Value(Param)`), kept in `ray`.
    fn law_radius(&mut self, param: f64) -> Result<(), Failure> {
        if let Some(law) = &self.law {
            self.ray = live_law(&law.tevol, self.class)?.value(param)?;
        }
        Ok(())
    }

    /// `GetTolerance(Tolerance, Tol)`: the resolutions of both curves on surface.
    pub(crate) fn get_tolerance(
        &self,
        tolerance: &mut Vector<'_>,
        tol: f64,
    ) -> Result<(), Failure> {
        vset(tolerance, 1, self.r1.cons.resolution(tol)?)?;
        vset(tolerance, 2, self.r2.cons.resolution(tol)?)
    }

    /// `GetBounds(InfBound, SupBound)`: the bounds of both curves on surface.
    pub(crate) fn get_bounds(
        &self,
        inf: &mut Vector<'_>,
        sup: &mut Vector<'_>,
    ) -> Result<(), Failure> {
        vset(inf, 1, self.r1.cons.first_parameter()?)?;
        vset(inf, 2, self.r2.cons.first_parameter()?)?;
        vset(sup, 1, self.r1.cons.last_parameter()?)?;
        vset(sup, 2, self.r2.cons.last_parameter()?)
    }

    /// `Value(X, F)`: both restriction points in the guide plane.
    pub(crate) fn value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure> {
        self.ptrst1 = self.r1.cons.value(vget(x, 1)?)?;
        self.ptrst2 = self.r2.cons.value(vget(x, 2)?)?;
        vset(f, 1, self.nplan.xyz().dot(self.ptrst1.xyz()) + self.the_d)?;
        vset(f, 2, self.nplan.xyz().dot(self.ptrst2.xyz()) + self.the_d)?;
        Ok(true)
    }

    /// `Derivatives(X, D)`.
    pub(crate) fn derivatives(
        &mut self,
        x: &Vector<'_>,
        d: &mut Matrix<'_>,
    ) -> Result<bool, Failure> {
        let (p1, d11) = self.r1.cons.d1(vget(x, 1)?)?;
        let (p2, d21) = self.r2.cons.d1(vget(x, 2)?)?;
        (self.ptrst1, self.ptrst2) = (p1, p2);
        mset(d, 1, 1, self.nplan.dot(d11))?;
        mset(d, 1, 2, 0.)?;
        mset(d, 2, 1, 0.)?;
        mset(d, 2, 2, self.nplan.dot(d21))?;
        Ok(true)
    }

    /// `Values(X, F, D)`: `Value` then `Derivatives`, always true.
    pub(crate) fn values(
        &mut self,
        x: &Vector<'_>,
        f: &mut Vector<'_>,
        d: &mut Matrix<'_>,
    ) -> Result<bool, Failure> {
        self.value(x, f)?;
        self.derivatives(x, d)?;
        Ok(true)
    }

    /// `CenterCircleRst1Rst2(PtRst1, PtRst2, np, Center, VdMed)`: the center of the circle
    /// of the radius through both points, on the side the choice gives, with the direction
    /// of the perpendicular bisector; false (the origin as center) when the points are
    /// too far apart. `let (found, center, vdmed) = core.center_circle_rst1_rst2(p1, p2, np)?;`.
    pub(crate) fn center_circle_rst1_rst2(
        &self,
        pt_rst1: Point3,
        pt_rst2: Point3,
        np: Vector3,
    ) -> Result<(bool, Point3, Vector3), Failure> {
        let rst1rst2 = Vector3::between(pt_rst1, pt_rst2);
        let mut vd_med = rst1rst2.crossed(np);
        let norm2 = rst1rst2.square_magnitude();
        let dist = self.ray * self.ray - 0.25 * norm2;
        if self.choix > 2 {
            vd_med.reverse();
        }
        if dist < -1.0e-7 {
            return Ok((false, Point3::default(), vd_med));
        }
        let middle = *rst1rst2.xyz() * 0.5 + pt_rst1.xyz();
        if dist > 1.0e-7 {
            let vdmed_nor = vd_med.normalized()?;
            let center =
                crate::blend_func::linalg::point_from(middle + *vdmed_nor.xyz() * dist.sqrt());
            return Ok((true, center, vd_med));
        }
        Ok((true, crate::blend_func::linalg::point_from(middle), vd_med))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blend_func::test_fixtures::right_angle;

    #[test]
    fn a_null_restriction_is_a_null_object() {
        let (s1, s2, guide) = right_angle();
        let rst: Handle<dyn Adaptor2dCurve2d> = Handle::default();
        let core = RstRstCore::new((&s1, &rst, &s2, &rst), &guide, None, "test");
        assert_eq!(core.err().unwrap().exception_type(), "Standard_NullObject");
    }

    #[test]
    fn t3dto2d_decomposes_in_the_basis() {
        let (b, c) = (Vector3::new(1., 0., 0.), Vector3::new(1., 1., 0.));
        let (x, y) = crate::brep_blend::surf_rst::t3dto2d(2. * b + 3. * c, b, c);
        assert!((x - 2.).abs() < 1e-12 && (y - 3.).abs() < 1e-12);
    }
}
