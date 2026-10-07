//! `BRepBlend_SurfRstConstRad` and `BRepBlend_SurfRstEvolRad` over [`SurfRstCore`].
use super::{SurfRstCore, SurfRstInputs};
use crate::blend::{
    BlendAppFunction, BlendPoint, BlendSurfRstFunction, SectionPoles, surf_rst_pnt1, surf_rst_pnt2,
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

impl SurfRstCore {
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

    /// `GetShape(NbPoles, NbKnots, Degree, NbPoles2d)`: two 2d poles.
    fn shape_counts(&mut self) -> (i32, i32, i32, i32) {
        let SectionCounts {
            nb_poles,
            nb_knots,
            degree,
        } = get_shape(self.shape, self.maxang, &mut self.tconv);
        (nb_poles, nb_knots, degree, 2)
    }

    /// `Resolution(IC2d, Tol, TolU, TolV)`: the surface for 1, the restriction surface else.
    fn resolution(&self, index: i32, tol: f64) -> Result<(f64, f64), Failure> {
        let surface = live_surface(if index == 1 {
            &self.surf
        } else {
            &self.surfrst
        })?;
        Ok((surface.u_resolution(tol)?, surface.v_resolution(tol)?))
    }

    /// `Values` of the class, for `IsSolution` and `Section`.
    fn class_values(
        core: &mut Self,
        x: &Vector<'_>,
        f: &mut Vector<'_>,
        d: &mut Matrix<'_>,
    ) -> Result<bool, Failure> {
        core.jacobian(x, d, Some(f))
    }
}

macro_rules! impl_surf_rst_function {
    ($class:ty) => {
        impl $class {
            /// `Set(SurfRef, RstRef)`: the reference surface and restriction of `Decroch`.
            pub fn set_refs(
                &mut self,
                surf_ref: &Handle<dyn Adaptor3dSurface>,
                rst_ref: &Handle<dyn Adaptor2dCurve2d>,
            ) {
                self.core.set_refs(surf_ref, rst_ref);
            }
            /// `Set(TypeSection)`: the shape of the sections.
            pub fn set_shape(&mut self, shape: SectionShape) {
                self.core.shape = shape;
            }
            /// `Section(Param, U, V, W, Pdeb, Pfin, C)`: the circular section at `param`;
            /// `let (pdeb, pfin, circle) = function.section_circle(t, (u, v, w))?;`.
            pub fn section_circle(
                &mut self,
                param: f64,
                uvw: (f64, f64, f64),
            ) -> Result<(f64, f64, Circle3), Failure> {
                self.core.section_circle(param, uvw)
            }
        }

        impl FunctionSet for $class {
            /// `NbVariables()`: 3.
            fn variables(&self) -> i32 {
                3
            }
            /// `NbEquations()`: 3.
            fn equations(&self) -> i32 {
                3
            }
            fn value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> bool {
                let result = self.core.value(x, f);
                self.capture.keep(result).unwrap_or(false)
            }
        }

        impl FunctionSetWithDerivatives for $class {
            fn derivatives(&mut self, x: &Vector<'_>, d: &mut Matrix<'_>) -> bool {
                let result = self.core.jacobian(x, d, None);
                self.capture.keep(result).unwrap_or(false)
            }
            fn values(&mut self, x: &Vector<'_>, f: &mut Vector<'_>, d: &mut Matrix<'_>) -> bool {
                let result = self.core.jacobian(x, d, Some(f));
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
                self.core.is_solution(sol, tol, SurfRstCore::class_values)
            }
            fn get_minimal_distance(&self) -> Result<f64, Failure> {
                Ok(self.core.distmin)
            }
            fn pnt1(&self) -> Point3 {
                surf_rst_pnt1(self)
            }
            fn pnt2(&self) -> Point3 {
                surf_rst_pnt2(self)
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
                self.core
                    .section_d1(point, values, SurfRstCore::class_values)
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

        impl BlendSurfRstFunction for $class {
            fn point_on_s(&self) -> Point3 {
                self.core.pts
            }
            fn point_on_rst(&self) -> Point3 {
                self.core.ptrst
            }
            fn pnt2d_on_s(&self) -> Point2 {
                self.core.pt2ds
            }
            fn pnt2d_on_rst(&self) -> Point2 {
                self.core.pt2drst
            }
            fn parameter_on_rst(&self) -> f64 {
                self.core.prmrst
            }
            fn is_tangency_point(&self) -> bool {
                self.core.istangent
            }
            fn tangent_on_s(&self) -> Result<Vector3, Failure> {
                self.core.when_not_tangent(self.core.tgs)
            }
            fn tangent2d_on_s(&self) -> Result<Vector2, Failure> {
                self.core.when_not_tangent(self.core.tg2ds)
            }
            fn tangent_on_rst(&self) -> Result<Vector3, Failure> {
                self.core.when_not_tangent(self.core.tgrst)
            }
            fn tangent2d_on_rst(&self) -> Result<Vector2, Failure> {
                self.core.when_not_tangent(self.core.tg2drst)
            }
            fn decroch(&self, sol: &Vector<'_>) -> Result<(bool, Vector3, Vector3), Failure> {
                self.core.decroch(sol)
            }
        }
    };
}

/// `BRepBlend_SurfRstConstRad`: the constant-radius fillet between a surface and a
/// restriction of another surface.
///
/// ```ignore
/// let mut function = BRepBlendSurfRstConstRad::new((&surf, &surf_rst, &rst), &guide)?;
/// function.set_radius(1.0, 1);
/// ```
pub struct BRepBlendSurfRstConstRad {
    core: SurfRstCore,
    capture: FailureCapture,
}

impl BRepBlendSurfRstConstRad {
    /// `BRepBlend_SurfRstConstRad(Surf, SurfRst, Rst, CGuide)`.
    pub fn new(
        surfaces: SurfRstInputs<'_>,
        c_guide: &Handle<dyn Adaptor3dCurve>,
    ) -> Result<Self, Failure> {
        let core = SurfRstCore::new(surfaces, c_guide, None, "BRepBlend_SurfRstConstRad")?;
        Ok(Self {
            core,
            capture: FailureCapture::default(),
        })
    }

    /// `Set(Radius, Choix)`: the radius, negative but for the choices 3 and 4.
    pub fn set_radius(&mut self, radius: f64, choix: i32) {
        self.core.choix = choix;
        self.core.ray = super::super::rad_inv_terms::choice_ray(radius, choix);
    }
}

impl_surf_rst_function!(BRepBlendSurfRstConstRad);

/// `BRepBlend_SurfRstEvolRad`: the evolutive-radius fillet between a surface and a
/// restriction of another surface.
///
/// ```ignore
/// let mut function = BRepBlendSurfRstEvolRad::new((&surf, &surf_rst, &rst), &guide, &law)?;
/// function.set_choix(1);
/// ```
pub struct BRepBlendSurfRstEvolRad {
    core: SurfRstCore,
    capture: FailureCapture,
}

impl BRepBlendSurfRstEvolRad {
    /// `BRepBlend_SurfRstEvolRad(Surf, SurfRst, Rst, CGuide, Evol)`.
    pub fn new(
        surfaces: SurfRstInputs<'_>,
        c_guide: &Handle<dyn Adaptor3dCurve>,
        evol: &Handle<dyn LawFunction>,
    ) -> Result<Self, Failure> {
        let core = SurfRstCore::new(surfaces, c_guide, Some(evol), "BRepBlend_SurfRstEvolRad")?;
        Ok(Self {
            core,
            capture: FailureCapture::default(),
        })
    }

    /// `Set(Choix)`: the sign of the radius, negative but for the choices 3 and 4.
    pub fn set_choix(&mut self, choix: i32) {
        self.core.choix = choix;
        if let Some(law) = &mut self.core.law {
            law.sg1 = super::super::rad_inv_terms::choice_sign(choix);
        }
    }
}

impl_surf_rst_function!(BRepBlendSurfRstEvolRad);
