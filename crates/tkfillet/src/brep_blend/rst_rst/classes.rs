//! `BRepBlend_RstRstConstRad` and `BRepBlend_RstRstEvolRad` over [`RstRstCore`].
use super::RstRstCore;
use crate::blend::{
    BlendAppFunction, BlendPoint, BlendRstRstFunction, DecrochStatus, SectionPoles, rst_rst_pnt1,
    rst_rst_pnt2,
};
use crate::blend_func::SectionShape;
use crate::blend_func::law_intervals::{law_intervals, law_nb_intervals};
use crate::blend_func::package::{
    SectionCounts, get_minimal_weights, get_shape, live_curve, live_surface, next_shape,
};
use crate::blend_func::two_surfaces::arc_tolerances;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkgeomalgo::geom_fill;
use rustycad_tkgeomalgo::law::LawFunction;
use rustycad_tkgeombase::approx::{SweepD1, SweepD2};
use rustycad_tkgeombase::failure_capture::FailureCapture;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Circle3, Point2, Point3, Vector2, Vector3};
use rustycad_tkmath::math::{FunctionSet, FunctionSetWithDerivatives, Matrix, Vector};

/// The first surface and restriction, then the second ones (`Surf1`, `Rst1`, `Surf2`, `Rst2`).
pub type RstRstInputs<'a> = (
    &'a Handle<dyn Adaptor3dSurface>,
    &'a Handle<dyn Adaptor2dCurve2d>,
    &'a Handle<dyn Adaptor3dSurface>,
    &'a Handle<dyn Adaptor2dCurve2d>,
);

impl RstRstCore {
    fn when_not_tangent<T>(&self, value: T) -> Result<T, Failure> {
        if self.istangent {
            return Err(Failure::with_message("Standard_DomainError", ""));
        }
        Ok(value)
    }

    fn nb_intervals(&self, shape: Shape) -> Result<i32, Failure> {
        match &self.law {
            None => live_curve(&self.guide)?.nb_intervals(next_shape(shape)),
            Some(law) => law_nb_intervals(&self.guide, &law.fevol, self.class, shape),
        }
    }

    fn intervals(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
        match &self.law {
            None => live_curve(&self.guide)?.intervals(bounds, next_shape(shape)),
            Some(law) => law_intervals(&self.guide, &law.fevol, self.class, bounds, shape),
        }
    }

    fn shape_counts(&mut self) -> (i32, i32, i32, i32) {
        let SectionCounts {
            nb_poles,
            nb_knots,
            degree,
        } = get_shape(self.shape, self.maxang, &mut self.tconv);
        (nb_poles, nb_knots, degree, 2)
    }

    fn resolution(&self, index: i32, tol: f64) -> Result<(f64, f64), Failure> {
        let surface = live_surface(if index == 1 {
            &self.r1.surf
        } else {
            &self.r2.surf
        })?;
        Ok((surface.u_resolution(tol)?, surface.v_resolution(tol)?))
    }
}

macro_rules! impl_rst_rst_function {
    ($class:ty) => {
        impl $class {
            /// `Set(SurfRef1, RstRef1, SurfRef2, RstRef2)`: the references of `Decroch`.
            pub fn set_refs(&mut self, refs: RstRstInputs<'_>) {
                self.core.set_refs(refs);
            }
            /// `Set(TypeSection)`: the shape of the sections.
            pub fn set_shape(&mut self, shape: SectionShape) {
                self.core.shape = shape;
            }
            /// `Section(Param, U, V, Pdeb, Pfin, C)`: the circular section at `param`;
            /// `let (pdeb, pfin, circle) = function.section_circle(t, (u, v))?;`.
            pub fn section_circle(
                &mut self,
                param: f64,
                uv: (f64, f64),
            ) -> Result<(f64, f64, Circle3), Failure> {
                self.core.section_circle(param, uv)
            }
            /// `CenterCircleRst1Rst2(PtRst1, PtRst2, np, Center, VdMed)`.
            pub fn center_circle_rst1_rst2(
                &self,
                pt_rst1: Point3,
                pt_rst2: Point3,
                np: Vector3,
            ) -> Result<(bool, Point3, Vector3), Failure> {
                self.core.center_circle_rst1_rst2(pt_rst1, pt_rst2, np)
            }
        }

        impl FunctionSet for $class {
            /// `NbVariables()`: 2.
            fn variables(&self) -> i32 {
                2
            }
            /// `NbEquations()`: 2.
            fn equations(&self) -> i32 {
                2
            }
            fn value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> bool {
                let result = self.core.value(x, f);
                self.capture.keep(result).unwrap_or(false)
            }
        }

        impl FunctionSetWithDerivatives for $class {
            fn derivatives(&mut self, x: &Vector<'_>, d: &mut Matrix<'_>) -> bool {
                let result = self.core.derivatives(x, d);
                self.capture.keep(result).unwrap_or(false)
            }
            fn values(&mut self, x: &Vector<'_>, f: &mut Vector<'_>, d: &mut Matrix<'_>) -> bool {
                let result = self.core.values(x, f, d);
                self.capture.keep(result).unwrap_or(false)
            }
        }

        impl BlendAppFunction for $class {
            fn take_failure(&mut self) -> Result<(), Failure> {
                self.capture.take()
            }
            fn set_param(&mut self, param: f64) -> Result<(), Failure> {
                self.core.set_param(param)
            }
            fn set_interval(&mut self, first: f64, last: f64) -> Result<(), Failure> {
                self.core.set_interval(first, last)
            }
            fn get_tolerance(&self, tolerance: &mut Vector<'_>, tol: f64) -> Result<(), Failure> {
                self.core.get_tolerance(tolerance, tol)
            }
            fn get_bounds(
                &self,
                inf: &mut Vector<'_>,
                sup: &mut Vector<'_>,
            ) -> Result<(), Failure> {
                self.core.get_bounds(inf, sup)
            }
            fn is_solution(&mut self, sol: &Vector<'_>, tol: f64) -> Result<bool, Failure> {
                self.core.is_solution(sol, tol)
            }
            fn get_minimal_distance(&self) -> Result<f64, Failure> {
                Ok(self.core.distmin)
            }
            fn pnt1(&self) -> Point3 {
                rst_rst_pnt1(self)
            }
            fn pnt2(&self) -> Point3 {
                rst_rst_pnt2(self)
            }
            fn is_rational(&self) -> Result<bool, Failure> {
                Ok(matches!(
                    self.core.shape,
                    SectionShape::Rational | SectionShape::QuasiAngular
                ))
            }
            fn get_section_size(&self) -> Result<f64, Failure> {
                Ok(self.core.maxang * self.core.ray.abs())
            }
            /// `GetMinimalWeight(Weigths)` ("it is supposed that it does not depend on the radius").
            fn get_minimal_weight(&self, weights: &mut Array1<'_, f64>) -> Result<(), Failure> {
                get_minimal_weights(
                    self.core.shape,
                    self.core.tconv,
                    self.core.minang,
                    self.core.maxang,
                    weights,
                )
            }
            fn nb_intervals(&self, shape: Shape) -> Result<i32, Failure> {
                self.core.nb_intervals(shape)
            }
            fn intervals(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
                self.core.intervals(bounds, shape)
            }
            fn get_shape(&mut self) -> Result<(i32, i32, i32, i32), Failure> {
                Ok(self.core.shape_counts())
            }
            fn get_tolerance_3d(
                &self,
                bound_tol: f64,
                surf_tol: f64,
                angle_tol: f64,
                tol3d: &mut Vector<'_>,
                tol1d: &mut Vector<'_>,
            ) -> Result<(), Failure> {
                let angle_radius = (self.core.minang, self.core.ray.abs());
                arc_tolerances(
                    self.core.tconv,
                    angle_radius,
                    [bound_tol, surf_tol, angle_tol],
                    (tol3d, tol1d),
                )
            }
            fn knots(&mut self, knots: &mut Array1<'_, f64>) -> Result<(), Failure> {
                geom_fill::knots(self.core.tconv, knots)
            }
            fn mults(&mut self, mults: &mut Array1<'_, i32>) -> Result<(), Failure> {
                geom_fill::mults(self.core.tconv, mults)
            }
            fn section_d1(
                &mut self,
                point: &BlendPoint,
                values: SweepD1<'_, '_>,
            ) -> Result<bool, Failure> {
                self.core.section_d1(point, values)
            }
            fn section(
                &mut self,
                point: &BlendPoint,
                values: SectionPoles<'_, '_, '_, '_>,
            ) -> Result<(), Failure> {
                self.core.section_d0(point, values)
            }
            /// The second-order section: false, not computed.
            fn section_d2(
                &mut self,
                _point: &BlendPoint,
                _values: SweepD2<'_, '_>,
            ) -> Result<bool, Failure> {
                Ok(false)
            }
            fn resolution(&self, index: i32, tol: f64) -> Result<(f64, f64), Failure> {
                self.core.resolution(index, tol)
            }
        }

        impl BlendRstRstFunction for $class {
            fn point_on_rst1(&self) -> Point3 {
                self.core.ptrst1
            }
            fn point_on_rst2(&self) -> Point3 {
                self.core.ptrst2
            }
            fn pnt2d_on_rst1(&self) -> Point2 {
                self.core.pt2drst1
            }
            fn pnt2d_on_rst2(&self) -> Point2 {
                self.core.pt2drst2
            }
            fn parameter_on_rst1(&self) -> f64 {
                self.core.prmrst1
            }
            fn parameter_on_rst2(&self) -> f64 {
                self.core.prmrst2
            }
            fn is_tangency_point(&self) -> bool {
                self.core.istangent
            }
            fn tangent_on_rst1(&self) -> Result<Vector3, Failure> {
                self.core.when_not_tangent(self.core.tgrst1)
            }
            fn tangent2d_on_rst1(&self) -> Result<Vector2, Failure> {
                self.core.when_not_tangent(self.core.tg2drst1)
            }
            fn tangent_on_rst2(&self) -> Result<Vector3, Failure> {
                self.core.when_not_tangent(self.core.tgrst2)
            }
            fn tangent2d_on_rst2(&self) -> Result<Vector2, Failure> {
                self.core.when_not_tangent(self.core.tg2drst2)
            }
            fn decroch(&self, sol: &Vector<'_>) -> Result<(DecrochStatus, [Vector3; 4]), Failure> {
                self.core.decroch(sol)
            }
        }
    };
}

/// `BRepBlend_RstRstConstRad`: the constant-radius fillet between two restrictions.
///
/// ```ignore
/// let mut function = BRepBlendRstRstConstRad::new((&s1, &rst1, &s2, &rst2), &guide)?;
/// function.set_radius(1.0, 1);
/// ```
pub struct BRepBlendRstRstConstRad {
    core: RstRstCore,
    capture: FailureCapture,
}

impl BRepBlendRstRstConstRad {
    /// `BRepBlend_RstRstConstRad(Surf1, Rst1, Surf2, Rst2, CGuide)`.
    pub fn new(
        restrictions: RstRstInputs<'_>,
        c_guide: &Handle<dyn Adaptor3dCurve>,
    ) -> Result<Self, Failure> {
        let core = RstRstCore::new(restrictions, c_guide, None, "BRepBlend_RstRstConstRad")?;
        Ok(Self {
            core,
            capture: FailureCapture::default(),
        })
    }

    /// `Set(Radius, Choix)`: the radius (its absolute value) and the choice.
    pub fn set_radius(&mut self, radius: f64, choix: i32) {
        self.core.choix = choix;
        self.core.ray = radius.abs();
    }
}

impl_rst_rst_function!(BRepBlendRstRstConstRad);

/// `BRepBlend_RstRstEvolRad`: the evolutive-radius fillet between two restrictions; the
/// radius is the law value, unsigned.
///
/// ```ignore
/// let mut function = BRepBlendRstRstEvolRad::new((&s1, &rst1, &s2, &rst2), &guide, &law)?;
/// function.set_choix(1);
/// ```
pub struct BRepBlendRstRstEvolRad {
    core: RstRstCore,
    capture: FailureCapture,
}

impl BRepBlendRstRstEvolRad {
    /// `BRepBlend_RstRstEvolRad(Surf1, Rst1, Surf2, Rst2, CGuide, Evol)`.
    pub fn new(
        restrictions: RstRstInputs<'_>,
        c_guide: &Handle<dyn Adaptor3dCurve>,
        evol: &Handle<dyn LawFunction>,
    ) -> Result<Self, Failure> {
        let core = RstRstCore::new(restrictions, c_guide, Some(evol), "BRepBlend_RstRstEvolRad")?;
        Ok(Self {
            core,
            capture: FailureCapture::default(),
        })
    }

    /// `Set(Choix)`: the choice.
    pub fn set_choix(&mut self, choix: i32) {
        self.core.choix = choix;
    }
}

impl_rst_rst_function!(BRepBlendRstRstEvolRad);
