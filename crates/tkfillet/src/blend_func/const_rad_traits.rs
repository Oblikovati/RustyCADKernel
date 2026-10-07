//! The math function set and blend function traits of the two fillet functions,
//! implemented once for `BlendFunc_ConstRad` and `BlendFunc_EvolRad`.
//!
//! A class provides `core`, `capture`, `compute_values`, `radii`, `is_solution_impl`,
//! `set_interval_impl`, `tolerance_arc`, `section_size`, `nb_intervals_impl`,
//! `intervals_impl` and `tangent_message`.

/// Implements `FunctionSet`, `FunctionSetWithDerivatives`, `BlendAppFunction` and
/// `BlendFunction` for a fillet function built on `TwoSurfaceCore`.
macro_rules! impl_two_surface_function {
    ($class:ty) => {
        impl $class {
            fn try_value(
                &mut self,
                x: &rustycad_tkmath::math::Vector<'_>,
                f: &mut rustycad_tkmath::math::Vector<'_>,
            ) -> Result<bool, rustycad_tkernel::standard::Failure> {
                let ok = self.compute_values(x, 0, false, 0.)?;
                $crate::blend_func::linalg::copy_vector(f, &self.core.eq.e)?;
                Ok(ok)
            }
            fn try_derivatives(
                &mut self,
                x: &rustycad_tkmath::math::Vector<'_>,
                d: &mut rustycad_tkmath::math::Matrix<'_>,
            ) -> Result<bool, rustycad_tkernel::standard::Failure> {
                let ok = self.compute_values(x, 1, false, 0.)?;
                $crate::blend_func::linalg::copy_matrix(d, &self.core.eq.dedx)?;
                Ok(ok)
            }
            fn try_values(
                &mut self,
                x: &rustycad_tkmath::math::Vector<'_>,
                f: &mut rustycad_tkmath::math::Vector<'_>,
                d: &mut rustycad_tkmath::math::Matrix<'_>,
            ) -> Result<bool, rustycad_tkernel::standard::Failure> {
                let ok = self.compute_values(x, 1, false, 0.)?;
                $crate::blend_func::linalg::copy_vector(f, &self.core.eq.e)?;
                $crate::blend_func::linalg::copy_matrix(d, &self.core.eq.dedx)?;
                Ok(ok)
            }
        }

        impl rustycad_tkmath::math::FunctionSet for $class {
            /// `NbVariables()`: 4 (`Blend_Function::NbVariables`).
            fn variables(&self) -> i32 {
                $crate::blend::function_nb_variables()
            }
            /// `NbEquations()`: 4.
            fn equations(&self) -> i32 {
                4
            }
            fn value(
                &mut self,
                point: &rustycad_tkmath::math::Vector<'_>,
                residual: &mut rustycad_tkmath::math::Vector<'_>,
            ) -> bool {
                let result = self.try_value(point, residual);
                self.capture.keep(result).unwrap_or(false)
            }
        }

        impl rustycad_tkmath::math::FunctionSetWithDerivatives for $class {
            fn derivatives(
                &mut self,
                point: &rustycad_tkmath::math::Vector<'_>,
                jacobian: &mut rustycad_tkmath::math::Matrix<'_>,
            ) -> bool {
                let result = self.try_derivatives(point, jacobian);
                self.capture.keep(result).unwrap_or(false)
            }
            fn values(
                &mut self,
                point: &rustycad_tkmath::math::Vector<'_>,
                residual: &mut rustycad_tkmath::math::Vector<'_>,
                jacobian: &mut rustycad_tkmath::math::Matrix<'_>,
            ) -> bool {
                let result = self.try_values(point, residual, jacobian);
                self.capture.keep(result).unwrap_or(false)
            }
        }

        $crate::blend_func::const_rad::traits::impl_two_surface_app_function!($class);
        $crate::blend_func::const_rad::traits::impl_two_surface_blend_function!($class);
    };
}

/// The `BlendAppFunction` part of [`impl_two_surface_function`].
macro_rules! impl_two_surface_app_function {
    ($class:ty) => {
        impl $crate::blend::BlendAppFunction for $class {
            fn take_failure(&mut self) -> Result<(), rustycad_tkernel::standard::Failure> {
                self.capture.take()
            }
            fn set_param(&mut self, param: f64) -> Result<(), rustycad_tkernel::standard::Failure> {
                self.core.param = param;
                Ok(())
            }
            fn set_interval(
                &mut self,
                first: f64,
                last: f64,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                self.set_interval_impl(first, last)
            }
            fn get_tolerance(
                &self,
                tolerance: &mut rustycad_tkmath::math::Vector<'_>,
                tol: f64,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                self.core.get_tolerance_impl(tolerance, tol)
            }
            fn get_bounds(
                &self,
                inf: &mut rustycad_tkmath::math::Vector<'_>,
                sup: &mut rustycad_tkmath::math::Vector<'_>,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                self.core.get_bounds_impl(inf, sup)
            }
            fn is_solution(
                &mut self,
                sol: &rustycad_tkmath::math::Vector<'_>,
                tol: f64,
            ) -> Result<bool, rustycad_tkernel::standard::Failure> {
                self.is_solution_impl(sol, tol)
            }
            fn get_minimal_distance(&self) -> Result<f64, rustycad_tkernel::standard::Failure> {
                Ok(self.core.distmin)
            }
            fn pnt1(&self) -> rustycad_tkmath::gp::Point3 {
                $crate::blend::function_pnt1(self)
            }
            fn pnt2(&self) -> rustycad_tkmath::gp::Point3 {
                $crate::blend::function_pnt2(self)
            }
            fn is_rational(&self) -> Result<bool, rustycad_tkernel::standard::Failure> {
                Ok(matches!(
                    self.core.shape,
                    $crate::blend_func::SectionShape::Rational
                        | $crate::blend_func::SectionShape::QuasiAngular
                ))
            }
            fn get_section_size(&self) -> Result<f64, rustycad_tkernel::standard::Failure> {
                Ok(self.section_size())
            }
            fn get_minimal_weight(
                &self,
                weights: &mut rustycad_tkernel::ncollection::Array1<'_, f64>,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                self.core.get_minimal_weight_impl(weights)
            }
            fn nb_intervals(
                &self,
                shape: rustycad_tkmath::geom_abs::Shape,
            ) -> Result<i32, rustycad_tkernel::standard::Failure> {
                self.nb_intervals_impl(shape)
            }
            fn intervals(
                &self,
                bounds: &mut rustycad_tkernel::ncollection::Array1<'_, f64>,
                shape: rustycad_tkmath::geom_abs::Shape,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                self.intervals_impl(bounds, shape)
            }
            fn get_shape(
                &mut self,
            ) -> Result<(i32, i32, i32, i32), rustycad_tkernel::standard::Failure> {
                Ok(self.core.get_shape_impl())
            }
            fn get_tolerance_3d(
                &self,
                bound_tol: f64,
                surf_tol: f64,
                angle_tol: f64,
                tol3d: &mut rustycad_tkmath::math::Vector<'_>,
                tol1d: &mut rustycad_tkmath::math::Vector<'_>,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                self.core.get_tolerance_3d_impl(
                    self.tolerance_arc(),
                    [bound_tol, surf_tol, angle_tol],
                    tol3d,
                    tol1d,
                )
            }
            fn knots(
                &mut self,
                knots: &mut rustycad_tkernel::ncollection::Array1<'_, f64>,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                rustycad_tkgeomalgo::geom_fill::knots(self.core.tconv, knots)
            }
            fn mults(
                &mut self,
                mults: &mut rustycad_tkernel::ncollection::Array1<'_, i32>,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                rustycad_tkgeomalgo::geom_fill::mults(self.core.tconv, mults)
            }
            fn section_d1(
                &mut self,
                point: &$crate::blend::BlendPoint,
                values: rustycad_tkgeombase::approx::SweepD1<'_, '_>,
            ) -> Result<bool, rustycad_tkernel::standard::Failure> {
                let sol = $crate::blend_func::two_surfaces::point_parameters(point)?;
                self.compute_values(&sol, 1, true, point.parameter())?;
                let radii = self.radii();
                self.core.section_d1(&sol, values, radii)
            }
            fn section(
                &mut self,
                point: &$crate::blend::BlendPoint,
                values: $crate::blend::SectionPoles<'_, '_>,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                let x = $crate::blend_func::two_surfaces::point_parameters(point)?;
                self.compute_values(&x, 0, true, point.parameter())?;
                let radii = self.radii();
                self.core.section_d0(&x, values, radii)
            }
            fn section_d2(
                &mut self,
                point: &$crate::blend::BlendPoint,
                values: rustycad_tkgeombase::approx::SweepD2<'_, '_>,
            ) -> Result<bool, rustycad_tkernel::standard::Failure> {
                let x = $crate::blend_func::two_surfaces::point_parameters(point)?;
                self.compute_values(&x, 2, true, point.parameter())?;
                let radii = self.radii();
                self.core.section_d2(&x, values, radii)
            }
            fn resolution(
                &self,
                index: i32,
                tol: f64,
            ) -> Result<(f64, f64), rustycad_tkernel::standard::Failure> {
                self.core.resolution_impl(index, tol)
            }
        }
    };
}

/// The `BlendFunction` part of [`impl_two_surface_function`].
macro_rules! impl_two_surface_blend_function {
    ($class:ty) => {
        impl $crate::blend::BlendFunction for $class {
            fn point_on_s1(&self) -> rustycad_tkmath::gp::Point3 {
                self.core.s1.pts
            }
            fn point_on_s2(&self) -> rustycad_tkmath::gp::Point3 {
                self.core.s2.pts
            }
            fn is_tangency_point(&self) -> bool {
                self.core.istangent
            }
            fn tangent_on_s1(
                &self,
            ) -> Result<rustycad_tkmath::gp::Vector3, rustycad_tkernel::standard::Failure> {
                self.core
                    .when_not_tangent(self.core.tg1, &self.tangent_message("TangentOnS1"))
            }
            fn tangent2d_on_s1(
                &self,
            ) -> Result<rustycad_tkmath::gp::Vector2, rustycad_tkernel::standard::Failure> {
                self.core
                    .when_not_tangent(self.core.tg12d, &self.tangent_message("Tangent2dOnS1"))
            }
            fn tangent_on_s2(
                &self,
            ) -> Result<rustycad_tkmath::gp::Vector3, rustycad_tkernel::standard::Failure> {
                self.core
                    .when_not_tangent(self.core.tg2, &self.tangent_message("TangentOnS2"))
            }
            fn tangent2d_on_s2(
                &self,
            ) -> Result<rustycad_tkmath::gp::Vector2, rustycad_tkernel::standard::Failure> {
                self.core
                    .when_not_tangent(self.core.tg22d, &self.tangent_message("Tangent2dOnS2"))
            }
            fn tangent(
                &self,
                u1: f64,
                v1: f64,
                u2: f64,
                v2: f64,
            ) -> Result<$crate::blend::SectionEndFrames, rustycad_tkernel::standard::Failure> {
                self.core.tangent_impl([u1, v1, u2, v2], self.radii())
            }
            fn twist_on_s1(&self) -> Result<bool, rustycad_tkernel::standard::Failure> {
                self.core.twist(false, &self.tangent_message("TwistOnS1"))
            }
            fn twist_on_s2(&self) -> Result<bool, rustycad_tkernel::standard::Failure> {
                self.core.twist(true, &self.tangent_message("TwistOnS2"))
            }
        }
    };
}

pub(crate) use {
    impl_two_surface_app_function, impl_two_surface_blend_function, impl_two_surface_function,
};

use super::ConstRad;
use crate::blend_func::package::live_curve;
use crate::blend_func::package::next_shape;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::geom_abs::Shape;

impl ConstRad {
    /// The native message of a tangent query on a tangency point.
    fn tangent_message(&self, member: &str) -> String {
        format!("BlendFunc_ConstRad::{member}")
    }

    fn nb_intervals_impl(&self, shape: Shape) -> Result<i32, Failure> {
        live_curve(self.guide())?.nb_intervals(next_shape(shape))
    }

    fn intervals_impl(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
        live_curve(self.guide())?.intervals(bounds, next_shape(shape))
    }
}

impl_two_surface_function!(ConstRad);
