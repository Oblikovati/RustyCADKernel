// Rust translation of AddPointInCurrentLine, MakeWalkingPoint and OpenLine of
// ModelingAlgorithms/TKGeomAlgo/IntWalk/IntWalk_IWalking.gxx ("IntWalk_IWalking_6.gxx") at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The line points of the walking: a path point added to a line, a walking point made
//! from the function, and the opening of a line to restart in the other direction.
use super::{IWalkingTypes, IntWalkIWalking, TheIWFunction, TheIWLine, sequence_value, wd_index};
use crate::int_surf::{IntSurfPathPoint, IntSurfPathPointTool, IntSurfPntOn2S};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkmath::math::{Matrix, Vector};

/// `sp.Values(UV, F, D)` with scratch outputs, to leave the function evaluated at `uv`
/// (the tests call it before reading the point and the tangents).
pub(super) fn evaluate_at<F: TheIWFunction>(sp: &mut F, uv: &Vector<'_>) -> Result<(), Failure> {
    let mut f: Vector<'static> = Vector::with_bounds(1, 1)?;
    let mut d: Matrix<'static> = Matrix::with_bounds(1, 1, 1, 2).map_err(errors::array_bounds)?;
    sp.checked_values(uv, &mut f, &mut d)?;
    Ok(())
}

/// `MakeWalkingPoint(Case, U, V, sp, Psol)`: `psol` becomes the function point with
/// the parameters `(u, v)` on the walked surface (on S1 when `reversed`). Cases 1 and
/// 2 read the last evaluation; 11 and 12 evaluate the function at `(u, v)` first;
/// any other case raises Standard_ConstructionError.
pub(super) fn make_walking_point<F: TheIWFunction>(
    case: i32,
    (u, v): (f64, f64),
    sp: &mut F,
    reversed: bool,
    psol: &mut IntSurfPntOn2S,
) -> Result<(), Failure> {
    if case == 1 || case == 2 {
        psol.set_value_on(sp.point(), reversed, u, v);
        return Ok(());
    }
    if case == 11 || case == 12 {
        let mut uv: Vector<'static> = Vector::with_bounds(1, 2)?;
        (uv[1], uv[2]) = (u, v);
        evaluate_at(sp, &uv)?;
        return make_walking_point(case - 10, (u, v), sp, reversed, psol);
    }
    Err(errors::construction(&format!(
        "IntWalk_IWalking::MakeWalkingPoint: case {case}, expected 1, 2, 11 or 12"
    )))
}

impl<T: IWalkingTypes> IntWalkIWalking<T> {
    /// `AddPointInCurrentLine(N, PathPnt, CurrentLine)`: appends the path point with
    /// the start parameters of path point `n`.
    pub(super) fn add_point_in_current_line(
        &self,
        n: i32,
        path_pnt: &IntSurfPathPoint,
        current_line: &T::Line,
    ) -> Result<(), Failure> {
        let data = self.wd1[wd_index(&self.wd1, n)?];
        let mut psol = IntSurfPntOn2S::default();
        psol.set_value_on(
            IntSurfPathPointTool::value3d(path_pnt),
            self.reversed,
            data.ustart,
            data.vstart,
        );
        current_line.add_point(&psol);
        Ok(())
    }
    /// `OpenLine(N, Psol, Pnts1, sp, Line)`: the line becomes open at its first point
    /// (on path point `n` when positive, with `psol` added when negative), is reversed
    /// and restarts from its first point in the other direction.
    pub(super) fn open_line(
        &mut self,
        n: i32,
        psol: &IntSurfPntOn2S,
        pnts1: &[IntSurfPathPoint],
        sp: &mut T::Function,
        line: &T::Line,
    ) -> Result<(), Failure> {
        self.previous_point = line.value(1)?;
        let mut uv: Vector<'static> = Vector::with_bounds(1, 2)?;
        (uv[1], uv[2]) = self.surface_parameters(&self.previous_point);
        evaluate_at(sp, &uv)?;
        self.previousd3d = sp.direction3d()?;
        self.previousd2d = sp.direction2d()?;
        if n > 0 {
            let path_pnt: &IntSurfPathPoint = sequence_value(pnts1, n, "path point")?;
            line.add_status_first_point(false, true, n, path_pnt);
            self.add_point_in_current_line(n, path_pnt, line)?;
        } else {
            if n < 0 {
                line.add_point(psol);
            }
            line.add_status_first(false, false);
        }
        line.reverse();
        line.set_tangent_vector(self.previousd3d.reversed(), line.nb_points());
        Ok(())
    }
}
