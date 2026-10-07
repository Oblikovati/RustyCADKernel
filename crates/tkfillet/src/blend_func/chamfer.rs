// Rust translation of BlendFunc_Chamfer.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The chamfer of two distances: one chord on each surface.
use super::Corde;
use super::chamfer_macro::impl_chamfer_function;
use super::gen_chamfer::{
    ChamferEquations, GenChamferCore, GenChamferFunction, chamfer_tangent, gen_section_d1,
    value_then_derivatives,
};
use super::linalg::{matrix, mset, vector, vget, vset};
use crate::blend::{BlendFunction, BlendPoint, SectionEndFrames};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkgeombase::approx::SweepD1;
use rustycad_tkgeombase::failure_capture::FailureCapture;
use rustycad_tkmath::gp::{Line3, Point3, Vector2, Vector3};
use rustycad_tkmath::math::{Matrix, Vector};

/// `BlendFunc_Chamfer`: the chamfer whose ends are at the distances `Dist1` and `Dist2`
/// from the guide point, each in the plane normal to the guide; the equations of the two
/// [`Corde`]s side by side. As natively, `Set(Dist1, Dist2, Choix)` keeps the distances
/// unsigned.
///
/// ```ignore
/// let mut chamfer = Chamfer::new(&s1, &s2, &guide);
/// chamfer.set_distances(1.0, 2.0, 1);
/// ```
pub struct Chamfer {
    core: GenChamferCore,
    corde1: Corde,
    corde2: Corde,
    capture: FailureCapture,
}

impl Chamfer {
    const CLASS: &'static str = "BlendFunc_Chamfer";

    /// `BlendFunc_Chamfer(S1, S2, CG)`; `Chamfer::new(&s1, &s2, &guide)`.
    pub fn new(
        s1: &Handle<dyn Adaptor3dSurface>,
        s2: &Handle<dyn Adaptor3dSurface>,
        cg: &Handle<dyn Adaptor3dCurve>,
    ) -> Self {
        Self {
            core: GenChamferCore::new(s1, s2, cg),
            corde1: Corde::new(s1, cg),
            corde2: Corde::new(s2, cg),
            capture: FailureCapture::default(),
        }
    }

    /// `Set(Param)`: both chords at the guide parameter.
    fn set_param_impl(&mut self, param: f64) -> Result<(), Failure> {
        self.corde1.set_param(param)?;
        self.corde2.set_param(param)
    }

    /// `Set(First, Last)` of `BlendFunc_GenChamfer`: nothing.
    fn set_interval_impl(&mut self, _first: f64, _last: f64) -> Result<(), Failure> {
        Ok(())
    }

    /// `IsSolution(Sol, Tol)`: both chords solved (the second only when the first is);
    /// keeps `Tol` and the minimal distance between the two points.
    fn is_solution_impl(&mut self, sol: &Vector<'_>, tol: f64) -> Result<bool, Failure> {
        let (sol1, sol2) = split(sol)?;
        let issol = self.corde1.is_solution(&sol1, tol)? && self.corde2.is_solution(&sol2, tol)?;
        self.core.tol = tol;
        if issol {
            let distance = self.corde1.point_on_s().distance(&self.corde2.point_on_s());
            self.core.distmin = std_min(self.core.distmin, distance);
        }
        Ok(issol)
    }

    fn section_d1_impl(
        &mut self,
        point: &BlendPoint,
        values: SweepD1<'_, '_>,
    ) -> Result<bool, Failure> {
        gen_section_d1(self, point, values)
    }
}

/// `(X(1), X(2))` and `(X(3), X(4))` as two vectors of bounds `[1, 2]`.
fn split(x: &Vector<'_>) -> Result<(Vector<'static>, Vector<'static>), Failure> {
    let (mut x1, mut x2) = (vector(2)?, vector(2)?);
    vset(&mut x1, 1, vget(x, 1)?)?;
    vset(&mut x1, 2, vget(x, 2)?)?;
    vset(&mut x2, 1, vget(x, 3)?)?;
    vset(&mut x2, 2, vget(x, 4)?)?;
    Ok((x1, x2))
}

impl ChamferEquations for Chamfer {
    fn chamfer_core(&self) -> &GenChamferCore {
        &self.core
    }

    /// `Value(X, F)`: the equations of the first chord, then of the second.
    fn try_value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure> {
        let (x1, x2) = split(x)?;
        let mut part = vector(2)?;
        self.corde1.value(&x1, &mut part)?;
        vset(f, 1, vget(&part, 1)?)?;
        vset(f, 2, vget(&part, 2)?)?;
        self.corde2.value(&x2, &mut part)?;
        vset(f, 3, vget(&part, 1)?)?;
        vset(f, 4, vget(&part, 2)?)?;
        Ok(true)
    }

    /// `Derivatives(X, D)`: the block-diagonal Jacobian of the two chords.
    fn try_derivatives(&mut self, x: &Vector<'_>, d: &mut Matrix<'_>) -> Result<bool, Failure> {
        let (x1, x2) = split(x)?;
        let mut block = matrix(2, 2)?;
        self.corde1.derivatives(&x1, &mut block)?;
        put_block(d, &block, (0, 2))?;
        self.corde2.derivatives(&x2, &mut block)?;
        put_block(d, &block, (2, 0))?;
        Ok(true)
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

/// `D(i + offset, j + offset) = block(i, j)` and `D(i + offset, j + zero) = 0`, row by row
/// as natively.
fn put_block(
    d: &mut Matrix<'_>,
    block: &Matrix<'_>,
    (offset, zero): (i32, i32),
) -> Result<(), Failure> {
    for i in 1..3 {
        for j in 1..3 {
            let value = *block.value(i, j)?;
            mset(d, i + offset, j + offset, value)?;
            mset(d, i + offset, j + zero, 0.)?;
        }
    }
    Ok(())
}

impl GenChamferFunction for Chamfer {
    /// `Set(Dist1, Dist2, Choix)`; `chamfer.set_distances(1.0, 2.0, 1)`.
    fn set_distances(&mut self, dist1: f64, dist2: f64, choix: i32) {
        self.corde1.set_dist(dist1);
        self.corde2.set_dist(dist2);
        self.core.choix = choix;
    }

    fn section_line(&self, uv: [f64; 4]) -> Result<(f64, f64, Line3), Failure> {
        self.core.section_line(uv)
    }
}

impl BlendFunction for Chamfer {
    fn point_on_s1(&self) -> Point3 {
        self.corde1.point_on_s()
    }
    fn point_on_s2(&self) -> Point3 {
        self.corde2.point_on_s()
    }
    /// `IsTangencyPoint()`: both chords at a tangency point.
    fn is_tangency_point(&self) -> bool {
        self.corde1.is_tangency_point() && self.corde2.is_tangency_point()
    }
    fn tangent_on_s1(&self) -> Result<Vector3, Failure> {
        self.corde1.tangent_on_s()
    }
    fn tangent2d_on_s1(&self) -> Result<Vector2, Failure> {
        self.corde1.tangent2d_on_s()
    }
    fn tangent_on_s2(&self) -> Result<Vector3, Failure> {
        self.corde2.tangent_on_s()
    }
    fn tangent2d_on_s2(&self) -> Result<Vector2, Failure> {
        self.corde2.tangent2d_on_s()
    }
    /// `Tangent(U1, V1, U2, V2, ...)` in the guide plane of the first chord.
    fn tangent(&self, u1: f64, v1: f64, u2: f64, v2: f64) -> Result<SectionEndFrames, Failure> {
        chamfer_tangent(&self.core, self.corde1.n_plan(), [u1, v1, u2, v2])
    }
}

impl_chamfer_function!(Chamfer);

#[cfg(test)]
#[path = "chamfer_tests.rs"]
mod tests;
