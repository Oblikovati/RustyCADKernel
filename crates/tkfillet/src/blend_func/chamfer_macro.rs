//! The math function set and `Blend_AppFunction` members of the chamfer functions,
//! implemented once for `BlendFunc_Chamfer`, `BlendFunc_ConstThroat`,
//! `BlendFunc_ConstThroatWithPenetration` and `BlendFunc_ChAsym`.

/// Implements `FunctionSet`, `FunctionSetWithDerivatives` and `BlendAppFunction` for a
/// chamfer function: a class with `capture`, `ChamferEquations` and the inherent
/// `set_param_impl`, `set_interval_impl`, `is_solution_impl`, `section_d1_impl` and
/// `CLASS` (its native name).
macro_rules! impl_chamfer_function {
    ($class:ty) => {
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
                let result = $crate::blend_func::gen_chamfer::ChamferEquations::try_value(
                    self, point, residual,
                );
                self.capture.keep(result).unwrap_or(false)
            }
        }

        impl rustycad_tkmath::math::FunctionSetWithDerivatives for $class {
            fn derivatives(
                &mut self,
                point: &rustycad_tkmath::math::Vector<'_>,
                jacobian: &mut rustycad_tkmath::math::Matrix<'_>,
            ) -> bool {
                let result = $crate::blend_func::gen_chamfer::ChamferEquations::try_derivatives(
                    self, point, jacobian,
                );
                self.capture.keep(result).unwrap_or(false)
            }
            fn values(
                &mut self,
                point: &rustycad_tkmath::math::Vector<'_>,
                residual: &mut rustycad_tkmath::math::Vector<'_>,
                jacobian: &mut rustycad_tkmath::math::Matrix<'_>,
            ) -> bool {
                let result = $crate::blend_func::gen_chamfer::ChamferEquations::try_values(
                    self, point, residual, jacobian,
                );
                self.capture.keep(result).unwrap_or(false)
            }
        }

        impl $crate::blend::BlendAppFunction for $class {
            fn take_failure(&mut self) -> Result<(), rustycad_tkernel::standard::Failure> {
                self.capture.take()
            }
            fn set_param(&mut self, param: f64) -> Result<(), rustycad_tkernel::standard::Failure> {
                self.set_param_impl(param)
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
                $crate::blend_func::gen_chamfer::ChamferEquations::chamfer_core(self)
                    .get_tolerance(tolerance, tol)
            }
            fn get_bounds(
                &self,
                inf: &mut rustycad_tkmath::math::Vector<'_>,
                sup: &mut rustycad_tkmath::math::Vector<'_>,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                $crate::blend_func::gen_chamfer::ChamferEquations::chamfer_core(self)
                    .get_bounds(inf, sup)
            }
            fn is_solution(
                &mut self,
                sol: &rustycad_tkmath::math::Vector<'_>,
                tol: f64,
            ) -> Result<bool, rustycad_tkernel::standard::Failure> {
                self.is_solution_impl(sol, tol)
            }
            fn get_minimal_distance(&self) -> Result<f64, rustycad_tkernel::standard::Failure> {
                Ok($crate::blend_func::gen_chamfer::ChamferEquations::chamfer_core(self).distmin)
            }
            fn pnt1(&self) -> rustycad_tkmath::gp::Point3 {
                $crate::blend::function_pnt1(self)
            }
            fn pnt2(&self) -> rustycad_tkmath::gp::Point3 {
                $crate::blend::function_pnt2(self)
            }
            /// `IsRational()`: false, the sections are lines.
            fn is_rational(&self) -> Result<bool, rustycad_tkernel::standard::Failure> {
                Ok(false)
            }
            /// `GetSectionSize()`: Standard_NotImplemented ("not necessary as not rational").
            fn get_section_size(&self) -> Result<f64, rustycad_tkernel::standard::Failure> {
                Err($crate::blend_func::gen_chamfer::section_size_not_implemented(Self::CLASS))
            }
            /// `GetMinimalWeight(Weigths)`: all 1.
            fn get_minimal_weight(
                &self,
                weights: &mut rustycad_tkernel::ncollection::Array1<'_, f64>,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                weights.init(1.0);
                Ok(())
            }
            fn nb_intervals(
                &self,
                shape: rustycad_tkmath::geom_abs::Shape,
            ) -> Result<i32, rustycad_tkernel::standard::Failure> {
                $crate::blend_func::gen_chamfer::ChamferEquations::chamfer_core(self)
                    .nb_intervals(shape)
            }
            fn intervals(
                &self,
                bounds: &mut rustycad_tkernel::ncollection::Array1<'_, f64>,
                shape: rustycad_tkmath::geom_abs::Shape,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                $crate::blend_func::gen_chamfer::ChamferEquations::chamfer_core(self)
                    .intervals(bounds, shape)
            }
            fn get_shape(
                &mut self,
            ) -> Result<(i32, i32, i32, i32), rustycad_tkernel::standard::Failure> {
                Ok($crate::blend_func::gen_chamfer::linear_shape())
            }
            fn get_tolerance_3d(
                &self,
                bound_tol: f64,
                _surf_tol: f64,
                _angle_tol: f64,
                tol3d: &mut rustycad_tkmath::math::Vector<'_>,
                _tol1d: &mut rustycad_tkmath::math::Vector<'_>,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                tol3d.init(bound_tol);
                Ok(())
            }
            fn knots(
                &mut self,
                knots: &mut rustycad_tkernel::ncollection::Array1<'_, f64>,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                $crate::blend_func::gen_chamfer::linear_knots(knots)
            }
            fn mults(
                &mut self,
                mults: &mut rustycad_tkernel::ncollection::Array1<'_, i32>,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                $crate::blend_func::gen_chamfer::linear_mults(mults)
            }
            fn section_d1(
                &mut self,
                point: &$crate::blend::BlendPoint,
                values: rustycad_tkgeombase::approx::SweepD1<'_, '_>,
            ) -> Result<bool, rustycad_tkernel::standard::Failure> {
                self.section_d1_impl(point, values)
            }
            fn section(
                &mut self,
                point: &$crate::blend::BlendPoint,
                values: $crate::blend::SectionPoles<'_, '_>,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                $crate::blend_func::gen_chamfer::section_d0(self, point, values)
            }
            /// The second-order section: false, not computed.
            fn section_d2(
                &mut self,
                _point: &$crate::blend::BlendPoint,
                _values: rustycad_tkgeombase::approx::SweepD2<'_, '_>,
            ) -> Result<bool, rustycad_tkernel::standard::Failure> {
                Ok(false)
            }
            fn resolution(
                &self,
                index: i32,
                tol: f64,
            ) -> Result<(f64, f64), rustycad_tkernel::standard::Failure> {
                $crate::blend_func::gen_chamfer::ChamferEquations::chamfer_core(self)
                    .resolution(index, tol)
            }
        }
    };
}

pub(crate) use impl_chamfer_function;
