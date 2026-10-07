// Rust translation of BlendFunc_EvolRadInv.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The inverse of the evolutive-radius fillet function: the section that ends on a curve
//! on one of the surfaces, the radius being a law of the guide parameter.
use super::choice::{choice_signs, live_law};
use super::fillet_inverse::{InverseCore, InverseRadius, impl_fillet_inverse};
use super::linalg::{vector, vget};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkgeomalgo::law::LawFunction;
use rustycad_tkgeombase::failure_capture::FailureCapture;
use rustycad_tkmath::math::{Matrix, Vector};

/// The class name of the failures.
const CLASS: &str = "BlendFunc_EvolRadInv";

/// `BlendFunc_EvolRadInv`: the evolutive-radius fillet equations of variables
/// `(w, t, U, V)`. As natively, the signs and the side are unset (0 and false) until
/// `Set(Choix)` and `Set(OnFirst, COnSurf)`.
///
/// ```ignore
/// let mut inverse = EvolRadInv::new(&s1, &s2, &guide, &law);
/// inverse.set_choix(1);
/// inverse.set_curve(true, &curve_on_s1)?;
/// ```
pub struct EvolRadInv {
    core: InverseCore,
    fevol: Handle<dyn LawFunction>,
    sg1: f64,
    sg2: f64,
    capture: FailureCapture,
}

impl EvolRadInv {
    /// `BlendFunc_EvolRadInv(S1, S2, C, Law)`; `EvolRadInv::new(&s1, &s2, &c, &law)`.
    pub fn new(
        s1: &Handle<dyn Adaptor3dSurface>,
        s2: &Handle<dyn Adaptor3dSurface>,
        c: &Handle<dyn Adaptor3dCurve>,
        law: &Handle<dyn LawFunction>,
    ) -> Self {
        Self {
            core: InverseCore::new(s1, s2, c, CLASS),
            fevol: law.clone(),
            sg1: 0.0,
            sg2: 0.0,
            capture: FailureCapture::default(),
        }
    }

    /// `Set(Choix)`: the choice, which signs the offsets of both surfaces as
    /// `BlendFunc_EvolRad::Set(Choix)`; `inverse.set_choix(3)`.
    pub fn set_choix(&mut self, choix: i32) {
        self.core.choix = choix;
        (self.sg1, self.sg2) = choice_signs(choix);
    }

    fn radius(&self, ray: f64, dray: f64) -> InverseRadius {
        InverseRadius::Evolutive {
            sg1: self.sg1,
            sg2: self.sg2,
            ray,
            dray,
        }
    }

    /// `Value(X, F)`: the law is evaluated at `X(2)` first.
    fn try_value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure> {
        let ray = live_law(&self.fevol, CLASS)?.value(vget(x, 2)?)?;
        self.core.value(x, f, self.radius(ray, 0.))
    }

    /// `Derivatives(X, D)`: natively `Values` into a scratch residual.
    fn try_derivatives(&mut self, x: &Vector<'_>, d: &mut Matrix<'_>) -> Result<bool, Failure> {
        let mut f = vector(4)?;
        self.try_values(x, &mut f, d)
    }

    /// `Values(X, F, D)`: the law value and derivative at `X(2)` first.
    fn try_values(
        &mut self,
        x: &Vector<'_>,
        f: &mut Vector<'_>,
        d: &mut Matrix<'_>,
    ) -> Result<bool, Failure> {
        let (ray, dray) = live_law(&self.fevol, CLASS)?.d1(vget(x, 2)?)?;
        self.core.values(x, f, d, self.radius(ray, dray))
    }
}

impl_fillet_inverse!(EvolRadInv);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blend::BlendInverseBounds;
    use rustycad_tkmath::math::FunctionSet;

    #[test]
    fn a_null_law_fails_before_the_curves() {
        let null_surface: Handle<dyn Adaptor3dSurface> = Handle::default();
        let null_curve: Handle<dyn Adaptor3dCurve> = Handle::default();
        let null_law: Handle<dyn LawFunction> = Handle::default();
        let mut inverse = EvolRadInv::new(&null_surface, &null_surface, &null_curve, &null_law);
        inverse.set_choix(4);
        assert_eq!((inverse.sg1, inverse.sg2), (1., -1.));
        let x = vector(4).unwrap();
        let mut f = vector(4).unwrap();
        assert!(!inverse.value(&x, &mut f));
        let failure = inverse.take_failure().unwrap_err();
        assert!(failure.message().contains("null radius law"), "{failure:?}");
    }
}
