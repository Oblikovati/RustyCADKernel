// Rust translation of BlendFunc_ConstThroatInv.hxx/.cxx and
// BlendFunc_ConstThroatWithPenetrationInv.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The inverses of the constant-throat chamfers: the section that ends on a curve on one
//! of the surfaces.
use super::chamfer_plane::ChamferPlane;
use super::choice::choice_signs;
use super::const_throat::throat_residual;
use super::fillet_inverse::InverseCore;
use super::gen_chamf_inv::{ChamfInvEquations, GenChamfInvFunction};
use super::inverse_macro::impl_blend_inverse;
use super::linalg::{mset, vector, vget, vset};
use super::package::live_surface;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkgeombase::failure_capture::FailureCapture;
use rustycad_tkmath::gp::{Point3, Vector2, Vector3};
use rustycad_tkmath::math::{Matrix, Vector};

/// The fields of `BlendFunc_ConstThroatInv`, which the penetration inverse inherits.
pub(crate) struct ThroatInvState {
    core: InverseCore,
    penetration: bool,
    throat: f64,
    param: f64,
    sign1: f64,
    sign2: f64,
    plane: ChamferPlane,
    pts1: Point3,
    pts2: Point3,
    d1u1: Vector3,
    d1v1: Vector3,
    d1u2: Vector3,
    d1v2: Vector3,
}

/// The inherited `BlendFunc_GenChamfInv` fields and members (bounds, tolerances, curve).
impl std::ops::Deref for ThroatInvState {
    type Target = InverseCore;
    fn deref(&self) -> &InverseCore {
        &self.core
    }
}

impl std::ops::DerefMut for ThroatInvState {
    fn deref_mut(&mut self) -> &mut InverseCore {
        &mut self.core
    }
}

/// The derivatives of the points along the curve on surface (`temp`) and along the other
/// surface, with the stale differences `Derivatives` takes before evaluating.
struct ThroatInvJet {
    temp: Vector3,
    temp1: Vector3,
    temp2: Vector3,
    last: Vector3,
}

impl ThroatInvState {
    fn new(
        s1: &Handle<dyn Adaptor3dSurface>,
        s2: &Handle<dyn Adaptor3dSurface>,
        c: &Handle<dyn Adaptor3dCurve>,
        class: &'static str,
        penetration: bool,
    ) -> Self {
        Self {
            core: InverseCore::new(s1, s2, c, class),
            penetration,
            throat: 0.0,
            param: 0.0,
            sign1: 0.0,
            sign2: 0.0,
            plane: ChamferPlane::default(),
            pts1: Point3::default(),
            pts2: Point3::default(),
            d1u1: Vector3::default(),
            d1v1: Vector3::default(),
            d1u2: Vector3::default(),
            d1v2: Vector3::default(),
        }
    }

    /// `Set(theThroat, , Choix)`: the throat and the signs of the choice.
    fn set_throat(&mut self, throat: f64, choix: i32) {
        self.throat = throat;
        self.core.choix = choix;
        (self.sign1, self.sign2) = choice_signs(choix);
    }

    /// The curve point at `X(1)` with its derivative, the guide plane at `X(2)`, and the
    /// surface parameters `XX` of both points.
    fn position(&mut self, x: &Vector<'_>) -> Result<(Vector2, [f64; 4]), Failure> {
        let (p2d, v2d) = self.core.live_csurf()?.d1(vget(x, 1)?)?;
        self.param = vget(x, 2)?;
        self.plane.set(&self.core.curv, self.param)?;
        let other = [vget(x, 3)?, vget(x, 4)?];
        let xx = if self.core.first {
            [p2d.x(), p2d.y(), other[0], other[1]]
        } else {
            [other[0], other[1], p2d.x(), p2d.y()]
        };
        Ok((v2d, xx))
    }

    /// `Value(X, F)` of either class.
    fn value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure> {
        let (_, xx) = self.position(x)?;
        self.pts1 = live_surface(&self.core.surf1)?.d0(xx[0], xx[1])?;
        self.pts2 = live_surface(&self.core.surf2)?.d0(xx[2], xx[3])?;
        let frame = (self.plane.ptgui, self.plane.nplan, self.plane.the_d);
        let residual =
            throat_residual(frame, self.throat, self.penetration, (self.pts1, self.pts2));
        for (i, value) in (1..).zip(residual) {
            vset(f, i, value)?;
        }
        Ok(true)
    }

    /// `Derivatives(X, D)` of either class: the differences to the points are those of the
    /// previous evaluation, as natively.
    fn derivatives(&mut self, x: &Vector<'_>, d: &mut Matrix<'_>) -> Result<bool, Failure> {
        let (v2d, xx) = self.position(x)?;
        let ptgui = self.plane.ptgui;
        let temp1 = Vector3::from(self.pts1.xyz() - ptgui.xyz());
        let temp2 = Vector3::from(self.pts2.xyz() - ptgui.xyz());
        let last = if self.penetration {
            Vector3::from(self.pts2.xyz() - self.pts1.xyz())
        } else {
            Vector3::from((self.pts1.xyz() + self.pts2.xyz()) / 2. - ptgui.xyz())
        };
        let d1 = live_surface(&self.core.surf1)?.d1(xx[0], xx[1])?;
        (self.pts1, self.d1u1, self.d1v1) = (d1.point, d1.d1u, d1.d1v);
        let d2 = live_surface(&self.core.surf2)?.d1(xx[2], xx[3])?;
        (self.pts2, self.d1u2, self.d1v2) = (d2.point, d2.d1u, d2.d1v);
        let (du, dv) = if self.core.first {
            (self.d1u1, self.d1v1)
        } else {
            (self.d1u2, self.d1v2)
        };
        let mut temp = Vector3::default();
        temp.set_linear_form(v2d.x(), du, v2d.y(), dv);
        let jet = ThroatInvJet {
            temp,
            temp1,
            temp2,
            last,
        };
        let columns = [
            self.curve_column(&jet),
            self.guide_column(&jet),
            self.surface_columns(&jet)[0],
            self.surface_columns(&jet)[1],
        ];
        for (j, column) in (1..).zip(columns) {
            for (i, value) in (1..).zip(column) {
                mset(d, i, j, value)?;
            }
        }
        Ok(true)
    }

    /// `D(i, 1)`: along the curve on surface.
    fn curve_column(&self, jet: &ThroatInvJet) -> [f64; 4] {
        let (nplan, ptgui, temp) = (self.plane.nplan, self.plane.ptgui, jet.temp);
        match (self.penetration, self.core.first) {
            (false, true) => {
                let vref1 = Vector3::between(ptgui, self.pts1);
                [nplan.dot(temp), 0., vref1.dot(temp), 2. * (vref1.dot(temp))]
            }
            (false, false) => {
                let vref2 = Vector3::between(ptgui, self.pts2);
                [
                    0.,
                    nplan.dot(temp),
                    vref2.dot(temp),
                    -2. * (vref2.dot(temp)),
                ]
            }
            (true, true) => [
                nplan.dot(temp),
                0.,
                2. * jet.temp1.dot(temp),
                temp.dot(jet.last) - temp.dot(jet.temp1),
            ],
            (true, false) => [0., nplan.dot(temp), 0., jet.temp1.dot(temp)],
        }
    }

    /// `D(i, 2)`: along the guide.
    fn guide_column(&self, jet: &ThroatInvJet) -> [f64; 4] {
        let (nplan, d1gui, dnplan) = (self.plane.nplan, self.plane.d1gui, self.plane.dnplan());
        let d12 = dnplan.dot(jet.temp1) - nplan.dot(d1gui);
        let d22 = dnplan.dot(jet.temp2) - nplan.dot(d1gui);
        if self.penetration {
            return [d12, d22, -2. * d1gui.dot(jet.temp1), -d1gui.dot(jet.last)];
        }
        [
            d12,
            d22,
            -2. * d1gui.dot(jet.last),
            2. * d1gui.dot(jet.temp1) - 2. * d1gui.dot(jet.temp2),
        ]
    }

    /// `D(i, 3)` and `D(i, 4)`: along the other surface.
    fn surface_columns(&self, jet: &ThroatInvJet) -> [[f64; 4]; 2] {
        let nplan = self.plane.nplan;
        let ptgui = self.plane.ptgui;
        let along = |du: Vector3| -> [f64; 4] {
            match (self.penetration, self.core.first) {
                (false, true) => {
                    let mid = Vector3::from((self.pts1.xyz() + self.pts2.xyz()) / 2. - ptgui.xyz());
                    [
                        0.,
                        nplan.dot(du),
                        mid.dot(du),
                        -2. * Vector3::between(ptgui, self.pts2).dot(du),
                    ]
                }
                (false, false) => {
                    let mid = Vector3::from((self.pts1.xyz() + self.pts2.xyz()) / 2. - ptgui.xyz());
                    [
                        nplan.dot(du),
                        0.,
                        mid.dot(du),
                        2. * Vector3::between(ptgui, self.pts1).dot(du),
                    ]
                }
                (true, true) => [0., nplan.dot(du), 0., jet.temp1.dot(du)],
                (true, false) => [
                    nplan.dot(du),
                    0.,
                    2. * jet.temp1.dot(du),
                    du.dot(jet.last) - du.dot(jet.temp1),
                ],
            }
        };
        if self.core.first {
            [along(self.d1u2), along(self.d1v2)]
        } else {
            [along(self.d1u1), along(self.d1v1)]
        }
    }

    /// `IsSolution(Sol, Tol)` of either class: `Value`, then the residual test (`F(4)`
    /// is a squared distance only without penetration).
    fn is_solution(&mut self, sol: &Vector<'_>, tol: f64) -> Result<bool, Failure> {
        let mut valsol = vector(4)?;
        self.value(sol, &mut valsol)?;
        let tol4 = if self.penetration { tol } else { tol * tol };
        Ok(vget(&valsol, 1)?.abs() <= tol
            && vget(&valsol, 2)?.abs() <= tol
            && vget(&valsol, 3)?.abs() <= tol * tol
            && vget(&valsol, 4)?.abs() <= tol4)
    }
}

/// Implements the inverse traits of a constant-throat inverse over its `state`.
macro_rules! impl_throat_inverse {
    ($class:ty, $name:literal, $penetration:literal) => {
        impl $class {
            /// The native constructor (`S1`, `S2`, `C`).
            pub fn new(
                s1: &Handle<dyn Adaptor3dSurface>,
                s2: &Handle<dyn Adaptor3dSurface>,
                c: &Handle<dyn Adaptor3dCurve>,
            ) -> Self {
                Self {
                    core: ThroatInvState::new(s1, s2, c, $name, $penetration),
                    capture: FailureCapture::default(),
                }
            }
            fn is_solution_impl(&mut self, sol: &Vector<'_>, tol: f64) -> Result<bool, Failure> {
                self.core.is_solution(sol, tol)
            }
        }

        impl ChamfInvEquations for $class {
            fn try_value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure> {
                self.core.value(x, f)
            }
            fn try_derivatives(
                &mut self,
                x: &Vector<'_>,
                d: &mut Matrix<'_>,
            ) -> Result<bool, Failure> {
                self.core.derivatives(x, d)
            }
        }

        impl GenChamfInvFunction for $class {
            /// `Set(theThroat, , Choix)`: the second distance is unused.
            fn set_distances(&mut self, dist1: f64, _dist2: f64, choix: i32) {
                self.core.set_throat(dist1, choix);
            }
        }

        impl_blend_inverse!($class);
    };
}

/// `BlendFunc_ConstThroatInv`: the equations of [`super::ConstThroat`] of variables
/// `(w, t, U, V)`.
///
/// ```ignore
/// let mut inverse = ConstThroatInv::new(&s1, &s2, &guide);
/// inverse.set_distances(1.0, 0.0, 1);
/// ```
pub struct ConstThroatInv {
    core: ThroatInvState,
    capture: FailureCapture,
}

impl_throat_inverse!(ConstThroatInv, "BlendFunc_ConstThroatInv", false);

/// `BlendFunc_ConstThroatWithPenetrationInv`: the equations of
/// [`super::ConstThroatWithPenetration`] of variables `(w, t, U, V)`.
///
/// ```ignore
/// let mut inverse = ConstThroatWithPenetrationInv::new(&s1, &s2, &guide);
/// inverse.set_distances(1.0, 0.0, 1);
/// ```
pub struct ConstThroatWithPenetrationInv {
    core: ThroatInvState,
    capture: FailureCapture,
}

impl_throat_inverse!(
    ConstThroatWithPenetrationInv,
    "BlendFunc_ConstThroatWithPenetrationInv",
    true
);

#[cfg(test)]
#[path = "const_throat_inv_tests.rs"]
mod tests;
