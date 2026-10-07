// Rust translation of BlendFunc_ChAsymInv.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The inverse of the asymmetric chamfer: the section that ends on a curve on one of the
//! surfaces.
use super::fillet_inverse::InverseCore;
use super::inverse_macro::impl_blend_inverse;
use super::linalg::{copy_matrix, copy_vector, matrix, mset, vector, vget, vset};
use super::package::{live_curve, live_surface};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkgeombase::failure_capture::FailureCapture;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};
use rustycad_tkmath::math::{Matrix, Vector};

/// `BlendFunc_ChAsymInv`: the equations of [`super::ChAsym`] of variables `(w, t, U, V)`:
/// the parameter `w` on the curve on surface, the guide parameter `t` and the point
/// `(U, V)` on the other surface. The equations and Jacobian of the last
/// `ComputeValues` are kept (`FX`, `DX`).
///
/// ```ignore
/// let mut inverse = ChAsymInv::new(&s1, &s2, &guide)?;
/// inverse.set_dist_angle(1.0, 0.5, 1);
/// ```
pub struct ChAsymInv {
    core: InverseCore,
    dist1: f64,
    angle: f64,
    tgang: f64,
    fx: Vector<'static>,
    dx: Matrix<'static>,
    capture: FailureCapture,
}

/// The guide frame of `ComputeValues`: point, derivative, plane normal and its
/// derivative, and the signed speed (both 0 without derivatives).
#[derive(Default)]
struct InverseGuide {
    ptgui: Point3,
    d1gui: Vector3,
    nplan: Vector3,
    dnplan: Vector3,
    normg: f64,
}

/// The surface values of `ComputeValues`: the curve point derivative `v2d`, both points,
/// the first (and second, for a Jacobian) derivatives of surface 1, the first of surface 2.
#[derive(Default)]
struct InverseJet {
    v2d: Vector2,
    pts1: Point3,
    pts2: Point3,
    d1u1: Vector3,
    d1v1: Vector3,
    d2u1: Vector3,
    d2v1: Vector3,
    d2uv1: Vector3,
    d1u2: Vector3,
    d1v2: Vector3,
}

/// The terms of the angle equation shared by the residual and the Jacobian.
struct AngleTerms {
    nps1: Vector3,
    s1s2: Vector3,
    nsurf1: Vector3,
    tsurf1: Vector3,
    psca_inv: f64,
    f4: f64,
}

impl ChAsymInv {
    /// `BlendFunc_ChAsymInv(S1, S2, C)`: no distance nor angle yet (`RealLast`);
    /// `ChAsymInv::new(&s1, &s2, &guide)?`.
    pub fn new(
        s1: &Handle<dyn Adaptor3dSurface>,
        s2: &Handle<dyn Adaptor3dSurface>,
        c: &Handle<dyn Adaptor3dCurve>,
    ) -> Result<Self, Failure> {
        Ok(Self {
            core: InverseCore::new(s1, s2, c, "BlendFunc_ChAsymInv"),
            dist1: f64::MAX,
            angle: f64::MAX,
            tgang: f64::MAX,
            fx: vector(4)?,
            dx: matrix(4, 4)?,
            capture: FailureCapture::default(),
        })
    }

    /// `Set(Dist1, Angle, Choix)`: the distance (its absolute value), the angle and the
    /// choice; `inverse.set_dist_angle(1.0, 0.5, 1)`.
    pub fn set_dist_angle(&mut self, dist1: f64, angle: f64, choix: i32) {
        self.dist1 = dist1.abs();
        self.angle = angle;
        self.tgang = angle.tan();
        self.core.choix = choix;
    }

    /// The guide at `w`: with `derivatives`, `D2` and the normal derivative.
    fn guide(&self, w: f64, derivatives: bool) -> Result<InverseGuide, Failure> {
        let curve = live_curve(&self.core.curv)?;
        let mut guide = InverseGuide::default();
        if derivatives {
            let d2gui;
            (guide.ptgui, guide.d1gui, d2gui) = curve.d2(w)?;
            guide.nplan = guide.d1gui.normalized()?;
            guide.normg = guide.d1gui.magnitude();
            guide.dnplan = (d2gui - guide.nplan.dot(d2gui) * guide.nplan) / guide.normg;
        } else {
            (guide.ptgui, guide.d1gui) = curve.d1(w)?;
            guide.nplan = guide.d1gui.normalized()?;
        }
        if self.core.choix % 2 != 0 {
            guide.nplan.reverse();
            guide.dnplan.reverse();
            guide.normg = -guide.normg;
        }
        Ok(guide)
    }

    /// The `(U, V)` of both surfaces: the curve point and `(X(3), X(4))`.
    fn parameters(&self, on_curve: Point2, x: &Vector<'_>) -> Result<[f64; 4], Failure> {
        let other = [vget(x, 3)?, vget(x, 4)?];
        Ok(if self.core.first {
            [on_curve.x(), on_curve.y(), other[0], other[1]]
        } else {
            [other[0], other[1], on_curve.x(), on_curve.y()]
        })
    }

    /// The surfaces at `x`: with `derivatives`, the curve derivative, `D2` of surface 1 and
    /// `D1` of surface 2; otherwise `D1` of surface 1 and the point of surface 2.
    fn surfaces(&self, x: &Vector<'_>, derivatives: bool) -> Result<InverseJet, Failure> {
        let csurf = self.core.live_csurf()?;
        let (s1, s2) = (
            live_surface(&self.core.surf1)?,
            live_surface(&self.core.surf2)?,
        );
        let mut jet = InverseJet::default();
        if !derivatives {
            let uv = self.parameters(csurf.value(vget(x, 1)?)?, x)?;
            let d1 = s1.d1(uv[0], uv[1])?;
            (jet.pts1, jet.d1u1, jet.d1v1) = (d1.point, d1.d1u, d1.d1v);
            jet.pts2 = s2.value(uv[2], uv[3])?;
            return Ok(jet);
        }
        let (pt2d, v2d) = csurf.d1(vget(x, 1)?)?;
        let uv = self.parameters(pt2d, x)?;
        let d2 = s1.d2(uv[0], uv[1])?;
        (jet.v2d, jet.pts1, jet.d1u1, jet.d1v1) = (v2d, d2.point, d2.d1u, d2.d1v);
        (jet.d2u1, jet.d2v1, jet.d2uv1) = (d2.d2u, d2.d2v, d2.d2uv);
        let d1 = s2.d1(uv[2], uv[3])?;
        (jet.pts2, jet.d1u2, jet.d1v2) = (d1.point, d1.d1u, d1.d1v);
        Ok(jet)
    }

    /// `ComputeValues(X, DegF, DegL)`: the equations (`DegF == 0`) and the Jacobian
    /// (`DegL == 1`); false when `DegF > DegL`.
    pub fn compute_values(
        &mut self,
        x: &Vector<'_>,
        deg_f: i32,
        deg_l: i32,
    ) -> Result<bool, Failure> {
        if deg_f > deg_l {
            return Ok(false);
        }
        let derivatives = !(deg_f == 0 && deg_l == 0);
        let guide = self.guide(vget(x, 2)?, derivatives)?;
        let jet = self.surfaces(x, derivatives)?;
        let terms = angle_terms(&guide, &jet);
        if deg_f == 0 {
            let dist = guide.ptgui.xyz().dot(*guide.nplan.xyz());
            vset(
                &mut self.fx,
                1,
                jet.pts1.xyz().dot(*guide.nplan.xyz()) - dist,
            )?;
            vset(
                &mut self.fx,
                2,
                jet.pts2.xyz().dot(*guide.nplan.xyz()) - dist,
            )?;
            vset(
                &mut self.fx,
                3,
                self.dist1 * self.dist1 - terms.nps1.square_magnitude(),
            )?;
            vset(&mut self.fx, 4, self.tgang - terms.f4)?;
        }
        if deg_l == 1 {
            let rows = if self.core.first {
                first_rows(&guide, &jet, &terms)
            } else {
                second_rows(&guide, &jet, &terms)
            };
            for (i, row) in (1..).zip(rows) {
                for (j, value) in (1..).zip(row) {
                    mset(&mut self.dx, i, j, value)?;
                }
            }
        }
        Ok(true)
    }

    /// `IsSolution(Sol, Tol)`: the residual test, the angle equation scaled by the surface
    /// speeds at `Sol` (guide plane not reversed by the choice, as natively).
    fn is_solution_impl(&mut self, sol: &Vector<'_>, tol: f64) -> Result<bool, Failure> {
        let (_, d1gui) = live_curve(&self.core.curv)?.d1(vget(sol, 2)?)?;
        let nplan = d1gui.normalized()?;
        let pt2d = self.core.live_csurf()?.value(vget(sol, 1)?)?;
        let uv = self.parameters(pt2d, sol)?;
        let d1 = live_surface(&self.core.surf1)?.d1(uv[0], uv[1])?;
        let pts2 = live_surface(&self.core.surf2)?.value(uv[2], uv[3])?;
        let tsurf1 = d1.d1u.crossed(d1.d1v).crossed(nplan);
        let s1s2 = Vector3::between(d1.point, pts2);
        let psca_inv = 1. / tsurf1.dot(s1s2);
        let (nordu1, nordv1) = (d1.d1u.magnitude(), d1.d1v.magnitude());
        let temp = 2. * (nordu1 + nordv1) * s1s2.magnitude() + 2. * nordu1 * nordv1;
        let mut valsol = vector(4)?;
        self.try_value(sol, &mut valsol)?;
        Ok(vget(&valsol, 1)?.abs() < tol
            && vget(&valsol, 2)?.abs() < tol
            && vget(&valsol, 3)?.abs() < 2. * self.dist1 * tol
            && vget(&valsol, 4)?.abs() < tol * (1. + self.tgang) * psca_inv.abs() * temp)
    }

    /// `Value(X, F)`: `ComputeValues(X, 0, 0)`, then `F = FX`.
    fn try_value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure> {
        let error = self.compute_values(x, 0, 0)?;
        copy_vector(f, &self.fx)?;
        Ok(error)
    }

    /// `Derivatives(X, D)`: `ComputeValues(X, 1, 1)`, then `D = DX`.
    fn try_derivatives(&mut self, x: &Vector<'_>, d: &mut Matrix<'_>) -> Result<bool, Failure> {
        let error = self.compute_values(x, 1, 1)?;
        copy_matrix(d, &self.dx)?;
        Ok(error)
    }

    /// `Values(X, F, D)`: `ComputeValues(X, 0, 1)`.
    fn try_values(
        &mut self,
        x: &Vector<'_>,
        f: &mut Vector<'_>,
        d: &mut Matrix<'_>,
    ) -> Result<bool, Failure> {
        let error = self.compute_values(x, 0, 1)?;
        copy_vector(f, &self.fx)?;
        copy_matrix(d, &self.dx)?;
        Ok(error)
    }
}

fn angle_terms(guide: &InverseGuide, jet: &InverseJet) -> AngleTerms {
    let nps1 = Vector3::between(guide.ptgui, jet.pts1);
    let s1s2 = Vector3::between(jet.pts1, jet.pts2);
    let nsurf1 = jet.d1u1.crossed(jet.d1v1);
    let tsurf1 = nsurf1.crossed(guide.nplan);
    let psca_inv = 1. / s1s2.dot(tsurf1);
    let f4 = guide.nplan.dot(tsurf1.crossed(s1s2)) * psca_inv;
    AngleTerms {
        nps1,
        s1s2,
        nsurf1,
        tsurf1,
        psca_inv,
        f4,
    }
}

/// The derivative of the angle equation along the guide, for the vector `temp_vec`.
fn angle_guide_term(guide: &InverseGuide, t: &AngleTerms, temp_vec: Vector3) -> f64 {
    let dwtsurf1 = t.nsurf1.crossed(guide.dnplan);
    let mut temp = t.f4 * dwtsurf1.dot(t.s1s2);
    temp -= guide.dnplan.dot(temp_vec) + guide.nplan.dot(dwtsurf1.crossed(t.s1s2));
    t.psca_inv * temp
}

/// The derivative of the angle equation along a parameter of the second point of
/// derivative `dpts2`.
fn angle_second_term(nplan: Vector3, t: &AngleTerms, dpts2: Vector3) -> f64 {
    t.psca_inv * (t.f4 * t.tsurf1.dot(dpts2) - nplan.dot(t.tsurf1.crossed(dpts2)))
}

/// The derivative of the angle equation along a parameter of the first point of
/// derivative `dpts1`, `dtsurf1` being the derivative of `tsurf1`.
fn angle_first_term(nplan: Vector3, t: &AngleTerms, dpts1: Vector3, dtsurf1: Vector3) -> f64 {
    let mut temp = t.f4 * (dtsurf1.dot(t.s1s2) - t.tsurf1.dot(dpts1));
    temp += nplan.dot(t.tsurf1.crossed(dpts1) - dtsurf1.crossed(t.s1s2));
    t.psca_inv * temp
}

/// `DX` when the curve lies on surface 1.
fn first_rows(guide: &InverseGuide, jet: &InverseJet, t: &AngleTerms) -> [[f64; 4]; 4] {
    let (nplan, v2d) = (guide.nplan, jet.v2d);
    let dw1pts1 = v2d.x() * jet.d1u1 + v2d.y() * jet.d1v1;
    let dw1du1 = v2d.x() * jet.d2u1 + v2d.y() * jet.d2uv1;
    let dw1dv1 = v2d.x() * jet.d2uv1 + v2d.y() * jet.d2v1;
    let dw1csurf = (dw1du1.crossed(jet.d1v1) + jet.d1u1.crossed(dw1dv1)).crossed(nplan);
    let nps2 = Vector3::between(guide.ptgui, jet.pts2);
    let temp_vec = 2. * t.nps1;
    [
        [
            nplan.dot(dw1pts1),
            guide.dnplan.dot(t.nps1) - guide.normg,
            0.,
            0.,
        ],
        [
            0.,
            guide.dnplan.dot(nps2) - guide.normg,
            nplan.dot(jet.d1u2),
            nplan.dot(jet.d1v2),
        ],
        [-dw1pts1.dot(temp_vec), guide.d1gui.dot(temp_vec), 0., 0.],
        [
            angle_first_term(nplan, t, dw1pts1, dw1csurf),
            angle_guide_term(guide, t, temp_vec),
            angle_second_term(nplan, t, jet.d1u2),
            angle_second_term(nplan, t, jet.d1v2),
        ],
    ]
}

/// `DX` when the curve lies on surface 2: the guide term uses the reversed `2 nps1`, as
/// natively.
fn second_rows(guide: &InverseGuide, jet: &InverseJet, t: &AngleTerms) -> [[f64; 4]; 4] {
    let (nplan, v2d) = (guide.nplan, jet.v2d);
    let d1utsurf1 = (jet.d2u1.crossed(jet.d1v1) + jet.d1u1.crossed(jet.d2uv1)).crossed(nplan);
    let d1vtsurf1 = (jet.d2uv1.crossed(jet.d1v1) + jet.d1u1.crossed(jet.d2v1)).crossed(nplan);
    let dw2pts2 = v2d.x() * jet.d1u2 + v2d.y() * jet.d1v2;
    let nps2 = Vector3::between(guide.ptgui, jet.pts2);
    let temp_vec = 2. * t.nps1;
    let reversed = temp_vec.reversed();
    [
        [
            0.,
            guide.dnplan.dot(t.nps1) - guide.normg,
            nplan.dot(jet.d1u1),
            nplan.dot(jet.d1v1),
        ],
        [
            nplan.dot(dw2pts2),
            guide.dnplan.dot(nps2) - guide.normg,
            0.,
            0.,
        ],
        [
            0.,
            guide.d1gui.dot(temp_vec),
            jet.d1u1.dot(reversed),
            jet.d1v1.dot(reversed),
        ],
        [
            angle_second_term(nplan, t, dw2pts2),
            angle_guide_term(guide, t, reversed),
            angle_first_term(nplan, t, jet.d1u1, d1utsurf1),
            angle_first_term(nplan, t, jet.d1v1, d1vtsurf1),
        ],
    ]
}

impl_blend_inverse!(ChAsymInv);

#[cfg(test)]
#[path = "ch_asym_inv_tests.rs"]
mod tests;
