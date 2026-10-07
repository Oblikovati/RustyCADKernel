// Rust translation of BlendFunc_ConstThroat.hxx/.cxx and
// BlendFunc_ConstThroatWithPenetration.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The chamfers of constant throat: the symmetric one, whose middle is at the throat
//! distance from the guide, and the one with penetration, whose first point is.
use super::chamfer_macro::impl_chamfer_function;
use super::chamfer_plane::ChamferPlane;
use super::gen_chamfer::{GenChamferCore, chamfer_tangent, gen_section_d1, value_then_derivatives};
use super::linalg::{matrix, mset, point_from, vector, vget, vset};
use super::package::live_surface;
use crate::blend::{BlendFunction, BlendPoint, SectionEndFrames};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkgeombase::approx::SweepD1;
use rustycad_tkgeombase::failure_capture::FailureCapture;
use rustycad_tkmath::gp::{Line3, Point3, Vector2, Vector3};
use rustycad_tkmath::math::{Gauss, Matrix, Vector};

/// The fields of `BlendFunc_ConstThroat`, which `BlendFunc_ConstThroatWithPenetration`
/// inherits; `penetration` selects the overridden equations.
pub(crate) struct ThroatState {
    pub(crate) core: GenChamferCore,
    penetration: bool,
    pts1: Point3,
    pts2: Point3,
    d1u1: Vector3,
    d1v1: Vector3,
    d1u2: Vector3,
    d1v2: Vector3,
    istangent: bool,
    tg1: Vector3,
    tg12d: Vector2,
    tg2: Vector3,
    tg22d: Vector2,
    param: f64,
    throat: f64,
    plane: ChamferPlane,
}

impl ThroatState {
    fn new(
        s1: &Handle<dyn Adaptor3dSurface>,
        s2: &Handle<dyn Adaptor3dSurface>,
        c: &Handle<dyn Adaptor3dCurve>,
        penetration: bool,
    ) -> Self {
        Self {
            core: GenChamferCore::new(s1, s2, c),
            penetration,
            pts1: Point3::default(),
            pts2: Point3::default(),
            d1u1: Vector3::default(),
            d1v1: Vector3::default(),
            d1u2: Vector3::default(),
            d1v2: Vector3::default(),
            istangent: false,
            tg1: Vector3::default(),
            tg12d: Vector2::default(),
            tg2: Vector3::default(),
            tg22d: Vector2::default(),
            param: 0.0,
            throat: 0.0,
            plane: ChamferPlane::default(),
        }
    }

    /// `Set(aThroat, , Choix)`.
    fn set_throat(&mut self, throat: f64, choix: i32) {
        self.throat = throat;
        self.core.choix = choix;
    }

    /// `Set(Param)`: the guide frame.
    fn set_param(&mut self, param: f64) -> Result<(), Failure> {
        self.param = param;
        self.plane.set(&self.core.curv, param)
    }

    /// The middle of the two points.
    fn middle(&self) -> rustycad_tkmath::gp::Xyz {
        (self.pts1.xyz() + self.pts2.xyz()) / 2.
    }

    /// `Value(X, F)` of either class.
    fn value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure> {
        self.pts1 = live_surface(&self.core.surf1)?.d0(vget(x, 1)?, vget(x, 2)?)?;
        self.pts2 = live_surface(&self.core.surf2)?.d0(vget(x, 3)?, vget(x, 4)?)?;
        let frame = (self.plane.ptgui, self.plane.nplan, self.plane.the_d);
        let residual =
            throat_residual(frame, self.throat, self.penetration, (self.pts1, self.pts2));
        for (i, value) in (1..).zip(residual) {
            vset(f, i, value)?;
        }
        Ok(true)
    }

    /// `Derivatives(X, D)` of either class.
    fn derivatives(&mut self, x: &Vector<'_>, d: &mut Matrix<'_>) -> Result<bool, Failure> {
        let d1 = live_surface(&self.core.surf1)?.d1(vget(x, 1)?, vget(x, 2)?)?;
        let d2 = live_surface(&self.core.surf2)?.d1(vget(x, 3)?, vget(x, 4)?)?;
        (self.pts1, self.d1u1, self.d1v1) = (d1.point, d1.d1u, d1.d1v);
        (self.pts2, self.d1u2, self.d1v2) = (d2.point, d2.d1u, d2.d1v);
        let rows = [
            [
                self.plane.nplan.dot(self.d1u1),
                self.plane.nplan.dot(self.d1v1),
                0.,
                0.,
            ],
            [
                0.,
                0.,
                self.plane.nplan.dot(self.d1u2),
                self.plane.nplan.dot(self.d1v2),
            ],
        ];
        let rows = [rows[0], rows[1], self.third_row(), self.fourth_row()];
        for (i, row) in (1..).zip(rows) {
            for (j, value) in (1..).zip(row) {
                mset(d, i, j, value)?;
            }
        }
        Ok(true)
    }

    /// `D(3, j)`: the derivative of the throat equation.
    fn third_row(&self) -> [f64; 4] {
        if self.penetration {
            let vref1 = Vector3::between(self.plane.ptgui, self.pts1);
            return [2. * vref1.dot(self.d1u1), 2. * vref1.dot(self.d1v1), 0., 0.];
        }
        let mid = || Vector3::from(self.middle() - self.plane.ptgui.xyz());
        [
            mid().dot(self.d1u1),
            mid().dot(self.d1v1),
            mid().dot(self.d1u2),
            mid().dot(self.d1v2),
        ]
    }

    /// `D(4, j)`: the derivative of the symmetry (or penetration) equation.
    fn fourth_row(&self) -> [f64; 4] {
        let vref1 = Vector3::between(self.plane.ptgui, self.pts1);
        if self.penetration {
            let vec12 = Vector3::between(self.pts1, self.pts2);
            return [
                self.d1u1.dot(vec12) - vref1.dot(self.d1u1),
                self.d1v1.dot(vec12) - vref1.dot(self.d1v1),
                vref1.dot(self.d1u2),
                vref1.dot(self.d1v2),
            ];
        }
        let vref2 = Vector3::between(self.plane.ptgui, self.pts2);
        [
            2. * vref1.dot(self.d1u1),
            2. * vref1.dot(self.d1v1),
            -2. * vref2.dot(self.d1u2),
            -2. * vref2.dot(self.d1v2),
        ]
    }

    /// The residual test of `IsSolution`: `F(4)` is a squared distance only without
    /// penetration.
    fn within(&self, valsol: &Vector<'_>, tol: f64) -> Result<bool, Failure> {
        let tol4 = if self.penetration { tol } else { tol * tol };
        Ok(vget(valsol, 1)?.abs() <= tol
            && vget(valsol, 2)?.abs() <= tol
            && vget(valsol, 3)?.abs() <= tol * tol
            && vget(valsol, 4)?.abs() <= tol4)
    }

    /// `IsSolution(Sol, Tol)` of either class: the residual test, then the tangents along
    /// the guide by Gauss (a failed decomposition marks a tangency point).
    fn is_solution(&mut self, sol: &Vector<'_>, tol: f64) -> Result<bool, Failure> {
        let (mut valsol, mut gradsol) = (vector(4)?, matrix(4, 4)?);
        self.value(sol, &mut valsol)?;
        self.derivatives(sol, &mut gradsol)?;
        self.core.tol = tol;
        if !self.within(&valsol, tol)? {
            return Ok(false);
        }
        let mut secmember = self.guide_derivative(sol)?;
        let resol = Gauss::new(&gradsol)?;
        self.istangent = !resol.is_done();
        if resol.is_done() {
            resol.solve(&mut secmember)?;
            self.apply_tangents([
                vget(&secmember, 1)?,
                vget(&secmember, 2)?,
                vget(&secmember, 3)?,
                vget(&secmember, 4)?,
            ]);
        }
        self.core.distmin = std_min(self.core.distmin, self.pts1.distance(&self.pts2));
        Ok(true)
    }

    /// The right-hand side of the tangent system; the differences are taken before the
    /// points are evaluated again at `sol`, as natively.
    fn guide_derivative(&mut self, sol: &Vector<'_>) -> Result<Vector<'static>, Failure> {
        let dnplan = self.plane.dnplan();
        let (nplan, d1gui) = (self.plane.nplan, self.plane.d1gui);
        let temp1 = Vector3::from(self.pts1.xyz() - self.plane.ptgui.xyz());
        let temp2 = Vector3::from(self.pts2.xyz() - self.plane.ptgui.xyz());
        let last = if self.penetration {
            Vector3::from(self.pts2.xyz() - self.pts1.xyz())
        } else {
            Vector3::from(self.middle() - self.plane.ptgui.xyz())
        };
        self.evaluate_d1(sol)?;
        let mut secmember = vector(4)?;
        vset(&mut secmember, 1, nplan.dot(d1gui) - dnplan.dot(temp1))?;
        vset(&mut secmember, 2, nplan.dot(d1gui) - dnplan.dot(temp2))?;
        if self.penetration {
            vset(&mut secmember, 3, 2. * d1gui.dot(temp1))?;
            vset(&mut secmember, 4, d1gui.dot(last))?;
        } else {
            vset(&mut secmember, 3, 2. * d1gui.dot(last))?;
            vset(
                &mut secmember,
                4,
                2. * d1gui.dot(temp2) - 2. * d1gui.dot(temp1),
            )?;
        }
        Ok(secmember)
    }

    fn evaluate_d1(&mut self, sol: &Vector<'_>) -> Result<(), Failure> {
        let d1 = live_surface(&self.core.surf1)?.d1(vget(sol, 1)?, vget(sol, 2)?)?;
        let d2 = live_surface(&self.core.surf2)?.d1(vget(sol, 3)?, vget(sol, 4)?)?;
        (self.pts1, self.d1u1, self.d1v1) = (d1.point, d1.d1u, d1.d1v);
        (self.pts2, self.d1u2, self.d1v2) = (d2.point, d2.d1u, d2.d1v);
        Ok(())
    }

    fn apply_tangents(&mut self, s: [f64; 4]) {
        self.tg1.set_linear_form(s[0], self.d1u1, s[1], self.d1v1);
        self.tg2.set_linear_form(s[2], self.d1u2, s[3], self.d1v2);
        self.tg12d = Vector2::new(s[0], s[1]);
        self.tg22d = Vector2::new(s[2], s[3]);
    }

    /// A tangent, or Standard_DomainError naming `class::member` on a tangency point.
    fn when_not_tangent<T>(&self, value: T, class: &str, member: &str) -> Result<T, Failure> {
        if self.istangent {
            return Err(Failure::with_message(
                "Standard_DomainError",
                &format!("{class}::{member}"),
            ));
        }
        Ok(value)
    }
}

/// The four equations of the constant-throat chamfers (and of their inverses) for the
/// points `pts1`, `pts2` and the guide frame `(ptgui, nplan, theD)`: both points in the
/// guide plane, then the throat and symmetry (or penetration) equations.
pub(crate) fn throat_residual(
    (ptgui, nplan, the_d): (Point3, Vector3, f64),
    throat: f64,
    penetration: bool,
    (pts1, pts2): (Point3, Point3),
) -> [f64; 4] {
    let f1 = nplan.xyz().dot(pts1.xyz()) + the_d;
    let f2 = nplan.xyz().dot(pts2.xyz()) + the_d;
    let throat2 = throat * throat;
    let vref1 = Vector3::between(ptgui, pts1);
    if penetration {
        return [
            f1,
            f2,
            vref1.square_magnitude() - throat2,
            vref1.dot(Vector3::between(pts1, pts2)),
        ];
    }
    let ptmid = point_from((pts1.xyz() + pts2.xyz()) / 2.);
    let vmid = Vector3::between(ptgui, ptmid);
    let vref2 = Vector3::between(ptgui, pts2);
    [
        f1,
        f2,
        vmid.square_magnitude() - throat2,
        vref1.square_magnitude() - vref2.square_magnitude(),
    ]
}

/// Implements the chamfer traits of a constant-throat class over its `state`.
macro_rules! impl_throat_function {
    ($class:ty) => {
        impl $class {
            /// `BlendFunc_ConstThroat(S1, S2, C)` (or the penetration class).
            pub fn new(
                s1: &Handle<dyn Adaptor3dSurface>,
                s2: &Handle<dyn Adaptor3dSurface>,
                c: &Handle<dyn Adaptor3dCurve>,
            ) -> Self {
                Self {
                    state: ThroatState::new(s1, s2, c, Self::PENETRATION),
                    capture: FailureCapture::default(),
                }
            }
            fn set_param_impl(&mut self, param: f64) -> Result<(), Failure> {
                self.state.set_param(param)
            }
            /// `Set(First, Last)` of `BlendFunc_GenChamfer`: nothing.
            fn set_interval_impl(&mut self, _first: f64, _last: f64) -> Result<(), Failure> {
                Ok(())
            }
            fn is_solution_impl(&mut self, sol: &Vector<'_>, tol: f64) -> Result<bool, Failure> {
                self.state.is_solution(sol, tol)
            }
            fn section_d1_impl(
                &mut self,
                point: &BlendPoint,
                values: SweepD1<'_, '_>,
            ) -> Result<bool, Failure> {
                gen_section_d1(self, point, values)
            }
        }

        impl $crate::blend_func::gen_chamfer::ChamferEquations for $class {
            fn chamfer_core(&self) -> &GenChamferCore {
                &self.state.core
            }
            fn try_value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure> {
                self.state.value(x, f)
            }
            fn try_derivatives(
                &mut self,
                x: &Vector<'_>,
                d: &mut Matrix<'_>,
            ) -> Result<bool, Failure> {
                self.state.derivatives(x, d)
            }
            fn try_values(
                &mut self,
                x: &Vector<'_>,
                f: &mut Vector<'_>,
                d: &mut Matrix<'_>,
            ) -> Result<bool, Failure> {
                value_then_derivatives(self, x, f, d)
            }
        }

        impl $crate::blend_func::GenChamferFunction for $class {
            /// `Set(aThroat, , Choix)`: the throat; the second distance is unused.
            fn set_distances(&mut self, dist1: f64, _dist2: f64, choix: i32) {
                self.state.set_throat(dist1, choix);
            }
            fn section_line(&self, uv: [f64; 4]) -> Result<(f64, f64, Line3), Failure> {
                self.state.core.section_line(uv)
            }
        }

        impl BlendFunction for $class {
            fn point_on_s1(&self) -> Point3 {
                self.state.pts1
            }
            fn point_on_s2(&self) -> Point3 {
                self.state.pts2
            }
            fn is_tangency_point(&self) -> bool {
                self.state.istangent
            }
            fn tangent_on_s1(&self) -> Result<Vector3, Failure> {
                self.state
                    .when_not_tangent(self.state.tg1, Self::CLASS, "TangentOnS1")
            }
            fn tangent2d_on_s1(&self) -> Result<Vector2, Failure> {
                self.state
                    .when_not_tangent(self.state.tg12d, Self::CLASS, "Tangent2dOnS1")
            }
            fn tangent_on_s2(&self) -> Result<Vector3, Failure> {
                self.state
                    .when_not_tangent(self.state.tg2, Self::CLASS, "TangentOnS2")
            }
            fn tangent2d_on_s2(&self) -> Result<Vector2, Failure> {
                self.state
                    .when_not_tangent(self.state.tg22d, Self::CLASS, "Tangent2dOnS2")
            }
            /// `Tangent(U1, V1, U2, V2, ...)` of `BlendFunc_ConstThroat` in the guide plane.
            fn tangent(
                &self,
                u1: f64,
                v1: f64,
                u2: f64,
                v2: f64,
            ) -> Result<SectionEndFrames, Failure> {
                chamfer_tangent(&self.state.core, self.state.plane.nplan, [u1, v1, u2, v2])
            }
        }

        impl_chamfer_function!($class);
    };
}

/// `BlendFunc_ConstThroat`: the chamfer whose two points are in the plane normal to the
/// guide, at the same distance from the guide point, with their middle at the throat
/// distance.
///
/// ```ignore
/// let mut chamfer = ConstThroat::new(&s1, &s2, &guide);
/// chamfer.set_distances(1.0, 0.0, 1);
/// ```
pub struct ConstThroat {
    state: ThroatState,
    capture: FailureCapture,
}

impl ConstThroat {
    const CLASS: &'static str = "BlendFunc_ConstThroat";
    const PENETRATION: bool = false;
}

impl_throat_function!(ConstThroat);

/// `BlendFunc_ConstThroatWithPenetration`: the chamfer whose first point is at the throat
/// distance from the guide point and whose second point is on the normal to the chord
/// from the guide point.
///
/// ```ignore
/// let mut chamfer = ConstThroatWithPenetration::new(&s1, &s2, &guide);
/// chamfer.set_distances(1.0, 0.0, 1);
/// ```
pub struct ConstThroatWithPenetration {
    state: ThroatState,
    capture: FailureCapture,
}

impl ConstThroatWithPenetration {
    const CLASS: &'static str = "BlendFunc_ConstThroatWithPenetration";
    const PENETRATION: bool = true;
}

impl_throat_function!(ConstThroatWithPenetration);

#[cfg(test)]
#[path = "const_throat_tests.rs"]
mod tests;
