// Rust translation of Blend_FuncInv.hxx/.cxx, Blend_CurvPointFuncInv.hxx/.cxx,
// Blend_SurfPointFuncInv.hxx/.cxx and Blend_SurfCurvFuncInv.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The inverse blend functions: they find the section that ends on a given curve on
//! surface or point, used to stop a walking on a restriction.
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::math::{FunctionSetWithDerivatives, Vector};

/// The members shared by every inverse blend function: the solver bounds and tolerances,
/// the solution test, and the exception kept by the solver callbacks (port only).
pub trait BlendInverseBounds: FunctionSetWithDerivatives + Send {
    /// The exception kept by the last solver callbacks, if any; `function.take_failure()?`.
    fn take_failure(&mut self) -> Result<(), Failure>;
    /// `GetTolerance(Tolerance, Tol)`: the parametric tolerance of each variable.
    fn get_tolerance(&self, tolerance: &mut Vector<'_>, tol: f64) -> Result<(), Failure>;
    /// `GetBounds(InfBound, SupBound)`: the bounds of each variable.
    fn get_bounds(&self, inf: &mut Vector<'_>, sup: &mut Vector<'_>) -> Result<(), Failure>;
    /// `IsSolution(Sol, Tol)`: whether `sol` is a zero.
    fn is_solution(&mut self, sol: &Vector<'_>, tol: f64) -> Result<bool, Failure>;
}

/// `Blend_FuncInv`: the section ending on the curve on surface `COnSurf`, of variables
/// `(t, w, U, V)` (the guide parameter, the curve parameter and the point on the other
/// surface).
pub trait BlendFuncInv: BlendInverseBounds {
    /// `Set(OnFirst, COnSurf)`: the curve, on the first surface when `on_first`.
    fn set_curve(
        &mut self,
        on_first: bool,
        curve: &Handle<dyn Adaptor2dCurve2d>,
    ) -> Result<(), Failure>;
}

/// `Blend_FuncInv::NbVariables()`: 4; `func_inv_nb_variables() == 4`.
pub fn func_inv_nb_variables() -> i32 {
    4
}

/// `Blend_CurvPointFuncInv`: the section of a surface-curve blend passing through a
/// point, of variables `(t, w)`.
pub trait BlendCurvPointFuncInv: BlendInverseBounds {
    /// `Set(P)`: the point the section must reach.
    fn set_point(&mut self, point: Point3);
}

/// `Blend_CurvPointFuncInv::NbVariables()`: 2; `curv_point_func_inv_nb_variables() == 2`.
pub fn curv_point_func_inv_nb_variables() -> i32 {
    2
}

/// `Blend_SurfPointFuncInv`: the section of a surface-curve blend passing through a
/// point of the surface, of variables `(w, U, V)`.
pub trait BlendSurfPointFuncInv: BlendInverseBounds {
    /// `Set(P)`: the point the section must reach.
    fn set_point(&mut self, point: Point3);
}

/// `Blend_SurfPointFuncInv::NbVariables()`: 3; `surf_point_func_inv_nb_variables() == 3`.
pub fn surf_point_func_inv_nb_variables() -> i32 {
    3
}

/// `Blend_SurfCurvFuncInv`: the section of a surface-curve blend ending on a restriction
/// of the surface, of variables `(w, U, V)`.
pub trait BlendSurfCurvFuncInv: BlendInverseBounds {
    /// `Set(Rst)`: the restriction.
    fn set_rst(&mut self, rst: &Handle<dyn Adaptor2dCurve2d>);
}

/// `Blend_SurfCurvFuncInv::NbVariables()`: 3; `surf_curv_func_inv_nb_variables() == 3`.
pub fn surf_curv_func_inv_nb_variables() -> i32 {
    3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variable_counts_are_native() {
        assert_eq!(func_inv_nb_variables(), 4);
        assert_eq!(curv_point_func_inv_nb_variables(), 2);
        assert_eq!(surf_point_func_inv_nb_variables(), 3);
        assert_eq!(surf_curv_func_inv_nb_variables(), 3);
    }
}
