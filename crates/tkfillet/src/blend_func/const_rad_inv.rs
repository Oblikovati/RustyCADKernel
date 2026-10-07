// Rust translation of BlendFunc_ConstRadInv.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The inverse of the constant-radius fillet function: the section that ends on a curve on
//! one of the surfaces.
use super::choice::signed_radii;
use super::fillet_inverse::{InverseCore, InverseRadius, PlaneDerivative, residual_is_solution};
use super::inverse_macro::impl_blend_inverse;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkgeombase::failure_capture::FailureCapture;
use rustycad_tkmath::math::{Matrix, Vector};

/// `BlendFunc_ConstRadInv`: the constant-radius fillet equations of variables
/// `(w, t, U, V)`: the parameter `w` on the curve on surface, the guide parameter `t` and
/// the point `(U, V)` on the other surface.
///
/// ```ignore
/// let mut inverse = ConstRadInv::new(&s1, &s2, &guide);
/// inverse.set_radius(1.0, 1);
/// inverse.set_curve(true, &curve_on_s1)?;
/// ```
pub struct ConstRadInv {
    core: InverseCore,
    ray1: f64,
    ray2: f64,
    capture: FailureCapture,
}

impl ConstRadInv {
    /// `BlendFunc_ConstRadInv(S1, S2, C)`; `ConstRadInv::new(&s1, &s2, &c)`.
    pub fn new(
        s1: &Handle<dyn Adaptor3dSurface>,
        s2: &Handle<dyn Adaptor3dSurface>,
        c: &Handle<dyn Adaptor3dCurve>,
    ) -> Self {
        Self {
            core: InverseCore::new(s1, s2, c, "BlendFunc_ConstRadInv"),
            ray1: 0.0,
            ray2: 0.0,
            capture: FailureCapture::default(),
        }
    }

    /// `Set(R, Choix)`: the radius and the choice, signed as `BlendFunc_ConstRad::Set`;
    /// `inverse.set_radius(1.0, 1)`.
    pub fn set_radius(&mut self, radius: f64, choix: i32) {
        self.core.choix = choix;
        (self.ray1, self.ray2) = signed_radii(radius, choix);
    }

    fn radius(&self) -> InverseRadius {
        InverseRadius::Constant {
            ray1: self.ray1,
            ray2: self.ray2,
        }
    }

    /// `IsSolution(Sol, Tol)`: `Value` at `sol`, then `|F(1)| <= tol` and
    /// `|F(2..4)|^2 <= tol^2`.
    fn is_solution_impl(&mut self, sol: &Vector<'_>, tol: f64) -> Result<bool, Failure> {
        residual_is_solution(|valsol| self.try_value(sol, valsol), tol)
    }

    fn try_value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure> {
        self.core.value(x, f, self.radius())
    }

    /// `Derivatives(X, D)`: the plane-normal derivative comes from `theD` here only.
    fn try_derivatives(&mut self, x: &Vector<'_>, d: &mut Matrix<'_>) -> Result<bool, Failure> {
        self.core
            .derivatives(x, d, self.radius(), PlaneDerivative::FromTheD)
    }

    fn try_values(
        &mut self,
        x: &Vector<'_>,
        f: &mut Vector<'_>,
        d: &mut Matrix<'_>,
    ) -> Result<bool, Failure> {
        self.core.values(x, f, d, self.radius())
    }
}

impl_blend_inverse!(ConstRadInv);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blend::BlendInverseBounds;
    use rustycad_tkmath::math::FunctionSet;

    #[test]
    fn variables_and_choice_are_native() {
        let null_surface: Handle<dyn Adaptor3dSurface> = Handle::default();
        let null_curve: Handle<dyn Adaptor3dCurve> = Handle::default();
        let mut inverse = ConstRadInv::new(&null_surface, &null_surface, &null_curve);
        inverse.set_radius(2., 7);
        assert_eq!((inverse.ray1, inverse.ray2), (-2., 2.));
        assert_eq!((inverse.variables(), inverse.equations()), (4, 4));
        let x = super::super::linalg::vector(4).unwrap();
        let mut f = super::super::linalg::vector(4).unwrap();
        assert!(!inverse.value(&x, &mut f));
        assert_eq!(
            inverse.take_failure().unwrap_err().exception_type(),
            "Standard_NullObject"
        );
    }
}
