//! The math function set and `Blend_*FuncInv` bounds of the radius inverses of BRepBlend,
//! implemented once: a class wrapping a core with `capture`, `value`, `jacobian(x, d, f)`,
//! `get_tolerance`, `get_bounds` and `is_solution`.

/// Implements `FunctionSet` (with the variable and equation counts), `FunctionSetWithDerivatives`
/// and `BlendInverseBounds` for a radius inverse.
macro_rules! impl_rad_inv {
    ($class:ty, $variables:expr, $equations:expr) => {
        impl FunctionSet for $class {
            fn variables(&self) -> i32 {
                $variables
            }
            fn equations(&self) -> i32 {
                $equations
            }
            fn value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> bool {
                let result = self.0.value(x, f);
                self.0.capture.keep(result).unwrap_or(false)
            }
        }

        impl FunctionSetWithDerivatives for $class {
            fn derivatives(&mut self, x: &Vector<'_>, d: &mut Matrix<'_>) -> bool {
                let result = self.0.jacobian(x, d, None);
                self.0.capture.keep(result).unwrap_or(false)
            }
            fn values(&mut self, x: &Vector<'_>, f: &mut Vector<'_>, d: &mut Matrix<'_>) -> bool {
                let result = self.0.jacobian(x, d, Some(f));
                self.0.capture.keep(result).unwrap_or(false)
            }
        }

        impl BlendInverseBounds for $class {
            fn take_failure(&mut self) -> Result<(), Failure> {
                self.0.capture.take()
            }
            fn get_tolerance(&self, tolerance: &mut Vector<'_>, tol: f64) -> Result<(), Failure> {
                self.0.get_tolerance(tolerance, tol)
            }
            fn get_bounds(
                &self,
                inf: &mut Vector<'_>,
                sup: &mut Vector<'_>,
            ) -> Result<(), Failure> {
                self.0.get_bounds(inf, sup)
            }
            fn is_solution(&mut self, sol: &Vector<'_>, tol: f64) -> Result<bool, Failure> {
                self.0.is_solution(sol, tol)
            }
        }
    };
}

pub(crate) use impl_rad_inv;
