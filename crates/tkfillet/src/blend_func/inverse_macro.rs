//! The math function set and `Blend_FuncInv` members of the inverse functions on a curve
//! on surface (the fillet and chamfer inverses), implemented once.

/// Implements `FunctionSet`, `FunctionSetWithDerivatives`, `BlendInverseBounds` and
/// `BlendFuncInv` for an inverse function on a curve on surface: a class with an
/// `InverseCore` `core`, `capture`, and the inherent `try_value`, `try_derivatives`,
/// `try_values` and `is_solution_impl`.
macro_rules! impl_blend_inverse {
    ($class:ty) => {
        impl rustycad_tkmath::math::FunctionSet for $class {
            /// 4 (`Blend_FuncInv::NbVariables`).
            fn variables(&self) -> i32 {
                $crate::blend::func_inv_nb_variables()
            }
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

        impl $crate::blend::BlendInverseBounds for $class {
            fn take_failure(&mut self) -> Result<(), rustycad_tkernel::standard::Failure> {
                self.capture.take()
            }
            fn get_tolerance(
                &self,
                tolerance: &mut rustycad_tkmath::math::Vector<'_>,
                tol: f64,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                self.core.get_tolerance(tolerance, tol)
            }
            fn get_bounds(
                &self,
                inf: &mut rustycad_tkmath::math::Vector<'_>,
                sup: &mut rustycad_tkmath::math::Vector<'_>,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                self.core.get_bounds(inf, sup)
            }
            fn is_solution(
                &mut self,
                sol: &rustycad_tkmath::math::Vector<'_>,
                tol: f64,
            ) -> Result<bool, rustycad_tkernel::standard::Failure> {
                self.is_solution_impl(sol, tol)
            }
        }

        impl $crate::blend::BlendFuncInv for $class {
            /// `Set(OnFirst, COnSurf)`.
            fn set_curve(
                &mut self,
                on_first: bool,
                curve: &rustycad_tkernel::standard::Handle<
                    dyn rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d,
                >,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                self.core.set_curve(on_first, curve);
                Ok(())
            }
        }
    };
}

pub(crate) use impl_blend_inverse;
