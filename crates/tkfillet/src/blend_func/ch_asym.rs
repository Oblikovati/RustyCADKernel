// Rust translation of BlendFunc_ChAsym.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The asymmetric chamfer: a distance on the first surface and an angle with it.
use super::chamfer_macro::impl_chamfer_function;
use super::gen_chamfer::{ChamferEquations, GenChamferCore, chamfer_tangent};
use super::linalg::{copy_matrix, copy_vector, matrix, mset, vector, vget, vset};
use super::package::{live_curve, live_surface};
use crate::blend::{BlendFunction, BlendPoint, SectionEndFrames};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkgeombase::approx::SweepD1;
use rustycad_tkgeombase::failure_capture::FailureCapture;
use rustycad_tkmath::gp::{Line3, Point3, Vector2, Vector3};
use rustycad_tkmath::math::{Matrix, Vector};

#[path = "ch_asym_solution.rs"]
mod solution;

/// `BlendFunc_ChAsym`: the chamfer whose first point is at the distance `Dist1` from the
/// guide point and whose section makes the angle `Angle` with the first surface. The
/// equations and Jacobian of the last `ComputeValues` are kept (`FX`, `DX`).
///
/// ```ignore
/// let mut chamfer = ChAsym::new(&s1, &s2, &guide);
/// chamfer.set_dist_angle(1.0, 0.5, 1);
/// ```
pub struct ChAsym {
    core: GenChamferCore,
    tcurv: Handle<dyn Adaptor3dCurve>,
    param: f64,
    dist1: f64,
    angle: f64,
    tgang: f64,
    nplan: Vector3,
    pt1: Point3,
    tsurf1: Vector3,
    pt2: Point3,
    fx: Vector<'static>,
    dx: Matrix<'static>,
    istangent: bool,
    tg1: Vector3,
    tg12d: Vector2,
    tg2: Vector3,
    tg22d: Vector2,
    capture: FailureCapture,
}

impl ChAsym {
    const CLASS: &'static str = "BlendFunc_ChAsym";

    /// `BlendFunc_ChAsym(S1, S2, C)`: no distance nor angle yet (`RealLast`), a tangency
    /// point; `ChAsym::new(&s1, &s2, &guide)?`.
    pub fn new(
        s1: &Handle<dyn Adaptor3dSurface>,
        s2: &Handle<dyn Adaptor3dSurface>,
        c: &Handle<dyn Adaptor3dCurve>,
    ) -> Result<Self, Failure> {
        Ok(Self {
            core: GenChamferCore::new(s1, s2, c),
            tcurv: c.clone(),
            param: 0.0,
            dist1: f64::MAX,
            angle: f64::MAX,
            tgang: f64::MAX,
            nplan: Vector3::default(),
            pt1: Point3::default(),
            tsurf1: Vector3::default(),
            pt2: Point3::default(),
            fx: vector(4)?,
            dx: matrix(4, 4)?,
            istangent: true,
            tg1: Vector3::default(),
            tg12d: Vector2::default(),
            tg2: Vector3::default(),
            tg22d: Vector2::default(),
            capture: FailureCapture::default(),
        })
    }

    /// `Set(Dist1, Angle, Choix)`: the distance (its absolute value), the angle and the
    /// choice; `chamfer.set_dist_angle(1.0, 0.5, 1)`.
    pub fn set_dist_angle(&mut self, dist1: f64, angle: f64, choix: i32) {
        self.dist1 = dist1.abs();
        self.angle = angle;
        self.tgang = angle.tan();
        self.core.choix = choix;
    }

    /// `Section(Param, U1, V1, U2, V2, Pdeb, Pfin, C)`: the line between the two points;
    /// `let (pdeb, pfin, line) = chamfer.section_line([u1, v1, u2, v2])?;`.
    pub fn section_line(&self, uv: [f64; 4]) -> Result<(f64, f64, Line3), Failure> {
        self.core.section_line(uv)
    }

    fn set_param_impl(&mut self, param: f64) -> Result<(), Failure> {
        self.param = param;
        Ok(())
    }

    /// `Set(First, Last)`: the guide trimmed to its useful part ("the precision is taken
    /// arbitrarily small").
    fn set_interval_impl(&mut self, first: f64, last: f64) -> Result<(), Failure> {
        self.tcurv = live_curve(&self.core.curv)?.trim(first, last, 1.0e-12)?;
        Ok(())
    }

    /// The guide plane normal, reversed for an odd choice.
    fn oriented(&self, np: Vector3) -> Vector3 {
        if self.core.choix % 2 != 0 {
            np.reversed()
        } else {
            np
        }
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
        let (ptgui, d1gui) = live_curve(&self.tcurv)?.d1(self.param)?;
        self.nplan = d1gui.normalized()?;
        let np = self.oriented(self.nplan);
        let jet = self.surface_jet(x, deg_f == 0 && deg_l == 0)?;
        let nsurf1 = jet.d1u1.crossed(jet.d1v1);
        self.tsurf1 = nsurf1.crossed(np);
        let (nps1, s1s2) = (
            Vector3::between(ptgui, self.pt1),
            Vector3::between(self.pt1, self.pt2),
        );
        let psca_inv = 1. / self.tsurf1.dot(s1s2);
        let f4 = np.dot(self.tsurf1.crossed(s1s2)) * psca_inv;
        if deg_f == 0 {
            let dist = ptgui.xyz().dot(*np.xyz());
            vset(&mut self.fx, 1, self.pt1.xyz().dot(*np.xyz()) - dist)?;
            vset(&mut self.fx, 2, self.pt2.xyz().dot(*np.xyz()) - dist)?;
            vset(
                &mut self.fx,
                3,
                self.dist1 * self.dist1 - nps1.square_magnitude(),
            )?;
            vset(&mut self.fx, 4, self.tgang - f4)?;
        }
        if deg_l == 1 {
            self.jacobian(&jet, np, (nps1, s1s2), (psca_inv, f4))?;
        }
        Ok(true)
    }

    /// The points (and derivatives) of both surfaces: with `values_only`, the first
    /// derivatives of surface 1 and the point of surface 2; otherwise the second
    /// derivatives of surface 1 and the first of surface 2.
    fn surface_jet(&mut self, x: &Vector<'_>, values_only: bool) -> Result<AsymJet, Failure> {
        let (s1, s2) = (
            live_surface(&self.core.surf1)?,
            live_surface(&self.core.surf2)?,
        );
        let (u1, v1, u2, v2) = (vget(x, 1)?, vget(x, 2)?, vget(x, 3)?, vget(x, 4)?);
        let mut jet = AsymJet::default();
        if values_only {
            let d1 = s1.d1(u1, v1)?;
            (self.pt1, jet.d1u1, jet.d1v1) = (d1.point, d1.d1u, d1.d1v);
            self.pt2 = s2.value(u2, v2)?;
            return Ok(jet);
        }
        let first = s1.d2(u1, v1)?;
        (self.pt1, jet.d1u1, jet.d1v1) = (first.point, first.d1u, first.d1v);
        (jet.d2u1, jet.d2v1, jet.d2uv1) = (first.d2u, first.d2v, first.d2uv);
        let second = s2.d1(u2, v2)?;
        (self.pt2, jet.d1u2, jet.d1v2) = (second.point, second.d1u, second.d1v);
        Ok(jet)
    }

    /// `DX` (`DegL == 1`).
    fn jacobian(
        &mut self,
        jet: &AsymJet,
        np: Vector3,
        (nps1, s1s2): (Vector3, Vector3),
        (psca_inv, f4): (f64, f64),
    ) -> Result<(), Failure> {
        let (d1u1, d1v1) = (jet.d1u1, jet.d1v1);
        let d1utsurf1 = (jet.d2u1.crossed(d1v1) + d1u1.crossed(jet.d2uv1)).crossed(np);
        let d1vtsurf1 = (jet.d2uv1.crossed(d1v1) + d1u1.crossed(jet.d2v1)).crossed(np);
        let temp_vec = -2. * nps1;
        let tsurf1 = self.tsurf1;
        let along1 = |d1: Vector3, dtsurf1: Vector3| {
            let mut temp = f4 * (dtsurf1.dot(s1s2) - tsurf1.dot(d1));
            temp += np.dot(tsurf1.crossed(d1) - dtsurf1.crossed(s1s2));
            temp * psca_inv
        };
        let along2 = |d1: Vector3| (f4 * tsurf1.dot(d1) - np.dot(tsurf1.crossed(d1))) * psca_inv;
        let rows = [
            [np.dot(d1u1), np.dot(d1v1), 0., 0.],
            [0., 0., np.dot(jet.d1u2), np.dot(jet.d1v2)],
            [d1u1.dot(temp_vec), d1v1.dot(temp_vec), 0., 0.],
            [
                along1(d1u1, d1utsurf1),
                along1(d1v1, d1vtsurf1),
                along2(jet.d1u2),
                along2(jet.d1v2),
            ],
        ];
        for (i, row) in (1..).zip(rows) {
            for (j, value) in (1..).zip(row) {
                mset(&mut self.dx, i, j, value)?;
            }
        }
        Ok(())
    }

    fn section_d1_impl(
        &mut self,
        point: &BlendPoint,
        values: SweepD1<'_, '_>,
    ) -> Result<bool, Failure> {
        self.section_with_tangents(point, values)
    }

    fn when_not_tangent<T>(&self, value: T, member: &str) -> Result<T, Failure> {
        if self.istangent {
            return Err(Failure::with_message(
                "Standard_DomainError",
                &format!("BlendFunc_ChAsym::{member}"),
            ));
        }
        Ok(value)
    }
}

/// The surface derivatives of `ComputeValues` (the second ones only for a Jacobian).
#[derive(Default)]
struct AsymJet {
    d1u1: Vector3,
    d1v1: Vector3,
    d2u1: Vector3,
    d2v1: Vector3,
    d2uv1: Vector3,
    d1u2: Vector3,
    d1v2: Vector3,
}

impl ChamferEquations for ChAsym {
    fn chamfer_core(&self) -> &GenChamferCore {
        &self.core
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

impl BlendFunction for ChAsym {
    fn point_on_s1(&self) -> Point3 {
        self.pt1
    }
    fn point_on_s2(&self) -> Point3 {
        self.pt2
    }
    fn is_tangency_point(&self) -> bool {
        self.istangent
    }
    fn tangent_on_s1(&self) -> Result<Vector3, Failure> {
        self.when_not_tangent(self.tg1, "TangentOnS1")
    }
    fn tangent2d_on_s1(&self) -> Result<Vector2, Failure> {
        self.when_not_tangent(self.tg12d, "Tangent2dOnS1")
    }
    fn tangent_on_s2(&self) -> Result<Vector3, Failure> {
        self.when_not_tangent(self.tg2, "TangentOnS2")
    }
    fn tangent2d_on_s2(&self) -> Result<Vector2, Failure> {
        self.when_not_tangent(self.tg22d, "Tangent2dOnS2")
    }
    /// `Tangent(U1, V1, U2, V2, ...)` in the guide plane at the current parameter.
    fn tangent(&self, u1: f64, v1: f64, u2: f64, v2: f64) -> Result<SectionEndFrames, Failure> {
        let (_, d1gui) = live_curve(&self.tcurv)?.d1(self.param)?;
        chamfer_tangent(&self.core, d1gui.normalized()?, [u1, v1, u2, v2])
    }
    /// `TwistOnS1()`: whether the tangent on S1 opposes the guide plane normal.
    fn twist_on_s1(&self) -> Result<bool, Failure> {
        Ok(self
            .when_not_tangent(self.tg1, "TwistOnS1")?
            .dot(self.nplan)
            < 0.)
    }
    fn twist_on_s2(&self) -> Result<bool, Failure> {
        Ok(self
            .when_not_tangent(self.tg2, "TwistOnS2")?
            .dot(self.nplan)
            < 0.)
    }
}

impl_chamfer_function!(ChAsym);

#[cfg(test)]
#[path = "ch_asym_tests.rs"]
mod tests;
