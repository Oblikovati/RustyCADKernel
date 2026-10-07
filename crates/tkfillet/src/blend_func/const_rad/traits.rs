//! `BlendFunc_ConstRad` as a math function set and a blend function.
use super::super::linalg::copy_matrix;
use super::super::linalg::copy_vector;
use super::super::package::live_curve;
use super::ConstRad;
use crate::blend::{
    BlendAppFunction, BlendFunction, BlendPoint, SectionEndFrames, SectionPoles,
    function_nb_variables, function_pnt1, function_pnt2,
};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::approx::{SweepD1, SweepD2};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point3, Vector2, Vector3};
use rustycad_tkmath::math::{FunctionSet, FunctionSetWithDerivatives, Matrix, Vector};

/// Standard_DomainError of a tangent query on a tangency point.
fn tangency(member: &str) -> Failure {
    Failure::with_message(
        "Standard_DomainError",
        &format!("BlendFunc_ConstRad::{member}"),
    )
}

impl ConstRad {
    fn try_value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure> {
        let ok = self.compute_values(x, 0, false, 0.)?;
        copy_vector(f, &self.eq.e)?;
        Ok(ok)
    }

    fn try_derivatives(&mut self, x: &Vector<'_>, d: &mut Matrix<'_>) -> Result<bool, Failure> {
        let ok = self.compute_values(x, 1, false, 0.)?;
        copy_matrix(d, &self.eq.dedx)?;
        Ok(ok)
    }

    fn try_values(
        &mut self,
        x: &Vector<'_>,
        f: &mut Vector<'_>,
        d: &mut Matrix<'_>,
    ) -> Result<bool, Failure> {
        let ok = self.compute_values(x, 1, false, 0.)?;
        copy_vector(f, &self.eq.e)?;
        copy_matrix(d, &self.eq.dedx)?;
        Ok(ok)
    }

    fn when_not_tangent<T>(&self, value: T, member: &str) -> Result<T, Failure> {
        if self.istangent {
            return Err(tangency(member));
        }
        Ok(value)
    }
}

impl FunctionSet for ConstRad {
    /// `NbVariables()`: 4 (`Blend_Function::NbVariables`).
    fn variables(&self) -> i32 {
        function_nb_variables()
    }
    /// `NbEquations()`: 4.
    fn equations(&self) -> i32 {
        4
    }
    fn value(&mut self, point: &Vector<'_>, residual: &mut Vector<'_>) -> bool {
        let result = self.try_value(point, residual);
        self.capture.keep(result).unwrap_or(false)
    }
}

impl FunctionSetWithDerivatives for ConstRad {
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

impl BlendAppFunction for ConstRad {
    fn take_failure(&mut self) -> Result<(), Failure> {
        self.capture.take()
    }
    fn set_param(&mut self, param: f64) -> Result<(), Failure> {
        self.param = param;
        Ok(())
    }
    /// The segmentation of the useful part of the curve ("precision is taken at random
    /// and small").
    fn set_interval(&mut self, first: f64, last: f64) -> Result<(), Failure> {
        self.tcurv = live_curve(&self.curv)?.trim(first, last, 1.0e-12)?;
        Ok(())
    }
    fn get_tolerance(&self, tolerance: &mut Vector<'_>, tol: f64) -> Result<(), Failure> {
        self.get_tolerance_impl(tolerance, tol)
    }
    fn get_bounds(&self, inf: &mut Vector<'_>, sup: &mut Vector<'_>) -> Result<(), Failure> {
        self.get_bounds_impl(inf, sup)
    }
    fn is_solution(&mut self, sol: &Vector<'_>, tol: f64) -> Result<bool, Failure> {
        self.is_solution_impl(sol, tol)
    }
    fn get_minimal_distance(&self) -> Result<f64, Failure> {
        Ok(self.distmin)
    }
    fn pnt1(&self) -> Point3 {
        function_pnt1(self)
    }
    fn pnt2(&self) -> Point3 {
        function_pnt2(self)
    }
    fn is_rational(&self) -> Result<bool, Failure> {
        Ok(matches!(
            self.shape,
            super::super::SectionShape::Rational | super::super::SectionShape::QuasiAngular
        ))
    }
    fn get_section_size(&self) -> Result<f64, Failure> {
        Ok(self.maxang * self.ray1.abs())
    }
    fn get_minimal_weight(&self, weights: &mut Array1<'_, f64>) -> Result<(), Failure> {
        self.get_minimal_weight_impl(weights)
    }
    fn nb_intervals(&self, shape: Shape) -> Result<i32, Failure> {
        self.nb_intervals_impl(shape)
    }
    fn intervals(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
        self.intervals_impl(bounds, shape)
    }
    fn get_shape(&mut self) -> Result<(i32, i32, i32, i32), Failure> {
        Ok(self.get_shape_impl())
    }
    fn get_tolerance_3d(
        &self,
        bound_tol: f64,
        surf_tol: f64,
        angle_tol: f64,
        tol3d: &mut Vector<'_>,
        tol1d: &mut Vector<'_>,
    ) -> Result<(), Failure> {
        self.get_tolerance_3d_impl([bound_tol, surf_tol, angle_tol], tol3d, tol1d)
    }
    fn knots(&mut self, knots: &mut Array1<'_, f64>) -> Result<(), Failure> {
        rustycad_tkgeomalgo::geom_fill::knots(self.tconv, knots)
    }
    fn mults(&mut self, mults: &mut Array1<'_, i32>) -> Result<(), Failure> {
        rustycad_tkgeomalgo::geom_fill::mults(self.tconv, mults)
    }
    fn section_d1(&mut self, point: &BlendPoint, values: SweepD1<'_, '_>) -> Result<bool, Failure> {
        self.section_d1_impl(point, values)
    }
    fn section(&mut self, point: &BlendPoint, values: SectionPoles<'_, '_>) -> Result<(), Failure> {
        self.section_impl(point, values)
    }
    fn section_d2(&mut self, point: &BlendPoint, values: SweepD2<'_, '_>) -> Result<bool, Failure> {
        self.section_d2_impl(point, values)
    }
    fn resolution(&self, index: i32, tol: f64) -> Result<(f64, f64), Failure> {
        self.resolution_impl(index, tol)
    }
}

impl BlendFunction for ConstRad {
    fn point_on_s1(&self) -> Point3 {
        self.s1.pts
    }
    fn point_on_s2(&self) -> Point3 {
        self.s2.pts
    }
    fn is_tangency_point(&self) -> bool {
        self.istangent
    }
    fn tangent_on_s1(&self) -> Result<Vector3, Failure> {
        self.when_not_tangent(self.tg1, "TangentOnS1")
    }
    fn tangent2d_on_s1(&self) -> Result<Vector2, Failure> {
        self.when_not_tangent(self.tg12d, "Tangent2dOnS1")
    }
    fn tangent_on_s2(&self) -> Result<Vector3, Failure> {
        self.when_not_tangent(self.tg2, "TangentOnS2")
    }
    fn tangent2d_on_s2(&self) -> Result<Vector2, Failure> {
        self.when_not_tangent(self.tg22d, "Tangent2dOnS2")
    }
    fn tangent(&self, u1: f64, v1: f64, u2: f64, v2: f64) -> Result<SectionEndFrames, Failure> {
        self.tangent_impl(u1, v1, u2, v2)
    }
    fn twist_on_s1(&self) -> Result<bool, Failure> {
        self.twist(false)
    }
    fn twist_on_s2(&self) -> Result<bool, Failure> {
        self.twist(true)
    }
}
