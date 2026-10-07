// Rust translation of BlendFunc_GenChamfInv.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The inverses of the chamfer functions: the section that ends on a curve on one of the
//! surfaces. Their fields, bounds and tolerances are those of the fillet inverses
//! ([`super::fillet_inverse::InverseCore`]).
use crate::blend::BlendFuncInv;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{Matrix, Vector};

/// `BlendFunc_GenChamfInv`: an inverse chamfer function set by two distances (or a throat)
/// and a choice.
///
/// ```ignore
/// let mut inverse: Box<dyn GenChamfInvFunction> = Box::new(ChamfInv::new(&s1, &s2, &guide));
/// inverse.set_distances(1.0, 2.0, 1);
/// ```
pub trait GenChamfInvFunction: BlendFuncInv {
    /// `Set(Dist1, Dist2, Choix)`.
    fn set_distances(&mut self, dist1: f64, dist2: f64, choix: i32);
}

/// The equations of a chamfer inverse.
pub(crate) trait ChamfInvEquations {
    fn try_value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure>;
    fn try_derivatives(&mut self, x: &Vector<'_>, d: &mut Matrix<'_>) -> Result<bool, Failure>;

    /// `Values(X, F, D)` of `BlendFunc_GenChamfInv`: `Value`, then `Derivatives`, always
    /// true (both results are dropped natively).
    fn try_values(
        &mut self,
        x: &Vector<'_>,
        f: &mut Vector<'_>,
        d: &mut Matrix<'_>,
    ) -> Result<bool, Failure> {
        self.try_value(x, f)?;
        self.try_derivatives(x, d)?;
        Ok(true)
    }
}
