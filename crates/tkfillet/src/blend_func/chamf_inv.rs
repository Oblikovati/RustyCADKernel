// Rust translation of BlendFunc_ChamfInv.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The inverse of the chamfer of two distances: one chord on each surface, one of them
//! ending on a curve on surface.
use super::Corde;
use super::choice::choice_signs;
use super::fillet_inverse::InverseCore;
use super::gen_chamf_inv::{ChamfInvEquations, GenChamfInvFunction};
use super::inverse_macro::impl_blend_inverse;
use super::linalg::{matrix, mset, vector, vget, vset};
use super::package::live_surface;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkgeombase::failure_capture::FailureCapture;
use rustycad_tkmath::gp::{Point2, Vector2, Vector3};
use rustycad_tkmath::math::{Matrix, Vector};

/// `BlendFunc_ChamfInv`: the equations of [`super::Chamfer`] of variables `(w, t, U, V)`:
/// the parameter `w` on the curve on surface, the guide parameter `t` and the point
/// `(U, V)` on the other surface. Unlike the chamfer, `Set(Dist1, Dist2, Choix)` signs the
/// distances by the choice.
///
/// ```ignore
/// let mut inverse = ChamfInv::new(&s1, &s2, &guide);
/// inverse.set_distances(1.0, 2.0, 1);
/// inverse.set_curve(true, &curve_on_s1)?;
/// ```
pub struct ChamfInv {
    core: InverseCore,
    corde1: Corde,
    corde2: Corde,
    capture: FailureCapture,
}

impl ChamfInv {
    /// `BlendFunc_ChamfInv(S1, S2, C)`; `ChamfInv::new(&s1, &s2, &guide)`.
    pub fn new(
        s1: &Handle<dyn Adaptor3dSurface>,
        s2: &Handle<dyn Adaptor3dSurface>,
        c: &Handle<dyn Adaptor3dCurve>,
    ) -> Self {
        Self {
            core: InverseCore::new(s1, s2, c, "BlendFunc_ChamfInv"),
            corde1: Corde::new(s1, c),
            corde2: Corde::new(s2, c),
            capture: FailureCapture::default(),
        }
    }

    /// The point of the curve on surface at `w` with its derivative, and both chords at
    /// the guide parameter `t`.
    fn position(&mut self, w: f64, t: f64) -> Result<(Point2, Vector2), Failure> {
        let point = self.core.live_csurf()?.d1(w)?;
        self.corde1.set_param(t)?;
        self.corde2.set_param(t)?;
        Ok(point)
    }

    /// `IsSolution(Sol, Tol)`: both chords solved (the second only when the first is),
    /// the curve point on its surface; the chords keep their guide parameter.
    fn is_solution_impl(&mut self, sol: &Vector<'_>, tol: f64) -> Result<bool, Failure> {
        let (p2d, _) = self.core.live_csurf()?.d1(vget(sol, 1)?)?;
        let (sol1, sol2) = (pair(p2d.x(), p2d.y())?, pair(vget(sol, 3)?, vget(sol, 4)?)?);
        let (on1, on2) = if self.core.first {
            (&sol1, &sol2)
        } else {
            (&sol2, &sol1)
        };
        Ok(self.corde1.is_solution(on1, tol)? && self.corde2.is_solution(on2, tol)?)
    }
}

/// A vector of bounds `[1, 2]`.
fn pair(a: f64, b: f64) -> Result<Vector<'static>, Failure> {
    let mut x = vector(2)?;
    vset(&mut x, 1, a)?;
    vset(&mut x, 2, b)?;
    Ok(x)
}

impl ChamfInvEquations for ChamfInv {
    /// `Value(X, F)`: the chord through the curve point and the chord through `(U, V)`.
    fn try_value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure> {
        let (p2d, _) = self.position(vget(x, 1)?, vget(x, 2)?)?;
        let (x1, x2) = (pair(p2d.x(), p2d.y())?, pair(vget(x, 3)?, vget(x, 4)?)?);
        let (mut f1, mut f2) = (vector(2)?, vector(2)?);
        if self.core.first {
            self.corde1.value(&x1, &mut f1)?;
            self.corde2.value(&x2, &mut f2)?;
        } else {
            self.corde1.value(&x2, &mut f1)?;
            self.corde2.value(&x1, &mut f2)?;
        }
        for (i, value) in (1..).zip([vget(&f1, 1)?, vget(&f1, 2)?, vget(&f2, 1)?, vget(&f2, 2)?]) {
            vset(f, i, value)?;
        }
        Ok(true)
    }

    /// `Derivatives(X, D)`: along the curve, along the guide (`DerFguide`) and along the
    /// other surface.
    fn try_derivatives(&mut self, x: &Vector<'_>, d: &mut Matrix<'_>) -> Result<bool, Failure> {
        let (p2d, v2d) = self.position(vget(x, 1)?, vget(x, 2)?)?;
        let (x1, x2) = (pair(p2d.x(), p2d.y())?, pair(vget(x, 3)?, vget(x, 4)?)?);
        let mut block = matrix(2, 2)?;
        // The chord of the curve on surface gives the guide frame (both chords share it).
        let (on_curve, df1, df2, surface) = if self.core.first {
            self.corde2.derivatives(&x2, &mut block)?;
            let df1 = self.corde1.der_f_guide(&x1)?;
            let df2 = self.corde2.der_f_guide(&x2)?;
            (&self.corde1, df1, df2, &self.core.surf1)
        } else {
            self.corde1.derivatives(&x2, &mut block)?;
            let df1 = self.corde1.der_f_guide(&x2)?;
            let df2 = self.corde2.der_f_guide(&x1)?;
            (&self.corde2, df1, df2, &self.core.surf2)
        };
        let (ptgui, nplan) = (on_curve.point_on_guide(), on_curve.n_plan());
        let d1 = live_surface(surface)?.d1(p2d.x(), p2d.y())?;
        // Derivatives along T, the parameter on the curve on surface.
        let mut temp = Vector3::default();
        temp.set_linear_form(v2d.x(), d1.d1u, v2d.y(), d1.d1v);
        let along = [
            nplan.dot(temp),
            2. * (Vector3::between(ptgui, d1.point).dot(temp)),
        ];
        let column1 = if self.core.first {
            [along[0], along[1], 0., 0.]
        } else {
            [0., 0., along[0], along[1]]
        };
        // Derivatives along W, the guide parameter.
        let column2 = [df1.x(), df1.y(), df2.x(), df2.y()];
        for (i, (a, b)) in (1..).zip(column1.into_iter().zip(column2)) {
            mset(d, i, 1, a)?;
            mset(d, i, 2, b)?;
        }
        surface_block(d, &block, self.core.first)?;
        Ok(true)
    }
}

/// The derivatives along `(U, V)`: the Jacobian of the chord on the other surface in rows
/// 3-4 (curve on surface 1) or 1-2 (curve on surface 2), zeros elsewhere.
fn surface_block(d: &mut Matrix<'_>, block: &Matrix<'_>, first: bool) -> Result<(), Failure> {
    for i in 1..3 {
        for j in 3..5 {
            let value = *block.value(i, j - 2)?;
            let (top, bottom) = if first { (0., value) } else { (value, 0.) };
            mset(d, i, j, top)?;
            mset(d, i + 2, j, bottom)?;
        }
    }
    Ok(())
}

impl GenChamfInvFunction for ChamfInv {
    /// `Set(Dist1, Dist2, Choix)`: the distances signed by the choice.
    fn set_distances(&mut self, dist1: f64, dist2: f64, choix: i32) {
        self.core.choix = choix;
        let (sg1, sg2) = choice_signs(choix);
        self.corde1.set_dist(sg1 * dist1);
        self.corde2.set_dist(sg2 * dist2);
    }
}

impl_blend_inverse!(ChamfInv);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blend::{BlendFuncInv, BlendInverseBounds};
    use crate::blend_func::test_fixtures::right_angle;
    use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
    use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
    use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
    use rustycad_tkmath::gp::Direction2;
    use rustycad_tkmath::math::FunctionSetWithDerivatives;

    /// The chamfer inverse of distances 1 and 2 ending on the line `v = 1` of the plane
    /// z = 0, parameterized by `u`.
    fn inverse() -> ChamfInv {
        let (s1, s2, guide) = right_angle();
        let mut inverse = ChamfInv::new(&s1, &s2, &guide);
        inverse.set_distances(1.0, 2.0, 1);
        let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(
            Point2::new(0., 1.),
            Direction2::new(1., 0.).unwrap(),
        ))
        .cast();
        let curve: Handle<dyn Adaptor2dCurve2d> =
            Handle::new(Geom2dAdaptorCurve::with_curve(&line).unwrap()).cast();
        inverse.set_curve(true, &curve).unwrap();
        inverse
    }

    #[test]
    fn the_exact_section_is_a_solution() {
        let mut inverse = inverse();
        let mut x = vector(4).unwrap();
        for (i, value) in (1..).zip([0.75, 0.75, 0.75, -2.0]) {
            vset(&mut x, i, value).unwrap();
        }
        let (mut f, mut d) = (vector(4).unwrap(), matrix(4, 4).unwrap());
        assert!(inverse.values(&x, &mut f, &mut d));
        assert_eq!([f[1], f[2], f[3], f[4]], [0.0; 4]);
        // Moving on the curve moves the first point off the guide plane.
        assert_eq!(*d.value(1, 1).unwrap(), 1.0);
        assert_eq!(*d.value(1, 2).unwrap(), -1.0);
        assert!(inverse.is_solution(&x, 1e-9).unwrap());
    }

    #[test]
    fn bounds_cover_the_curve_the_guide_and_the_other_surface() {
        let mut inverse = inverse();
        inverse.set_distances(1.0, 2.0, 4);
        assert_eq!(inverse.core.choix, 4);
        let mut bounds = (vector(4).unwrap(), vector(4).unwrap());
        inverse.get_bounds(&mut bounds.0, &mut bounds.1).unwrap();
        // Infinite lines and planes are not widened.
        assert_eq!(
            [bounds.0[1], bounds.0[2], bounds.1[4]],
            [-2e100, -2e100, 2e100]
        );
    }
}
