//! Port of MathSys_NewtonTypes/2D/3D/4D at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{NewtonBoundsN, NewtonOptions, NewtonResultN, solve_3d, solve_4d};
use crate::gp::{Point3, Vector3};
/// Curve second derivative evaluation; `CurveD2::default()`.
#[derive(Clone, Copy, Default)]
pub struct CurveD2 {
    /// Curve point; `evaluation.point`.
    pub point: Point3,
    /// First derivative; `evaluation.d1`.
    pub d1: Vector3,
    /// Second derivative; `evaluation.d2`.
    pub d2: Vector3,
}
/// Surface second derivative evaluation; `SurfaceD2::default()`.
#[derive(Clone, Copy, Default)]
pub struct SurfaceD2 {
    /// Surface point; `evaluation.point`.
    pub point: Point3,
    /// First U derivative; `evaluation.du`.
    pub du: Vector3,
    /// First V derivative; `evaluation.dv`.
    pub dv: Vector3,
    /// Second U derivative; `evaluation.duu`.
    pub duu: Vector3,
    /// Second V derivative; `evaluation.dvv`.
    pub dvv: Vector3,
    /// Mixed derivative; `evaluation.duv`.
    pub duv: Vector3,
}
/// Native curve evaluator contract; implement for `solve_curve_surface_extrema_3d`.
pub trait CurveEvaluatorD2 {
    /// Supplies a point and both derivatives; `curve.d2(t)`.
    fn d2(&mut self, t: f64) -> CurveD2;
}
/// Native surface evaluator contract; implement for `solve_surface_surface_extrema_4d`.
pub trait SurfaceEvaluatorD2 {
    /// Supplies the point and five derivatives; `surface.d2(u,v)`.
    fn d2(&mut self, u: f64, v: f64) -> SurfaceD2;
}
/// Finds native curve/surface extrema; `solve_curve_surface_extrema_3d(&mut curve, &mut surface, [0.0;3], &bounds, &options)`.
pub fn solve_curve_surface_extrema_3d<C: CurveEvaluatorD2, S: SurfaceEvaluatorD2>(
    curve: &mut C,
    surface: &mut S,
    start: [f64; 3],
    bounds: &NewtonBoundsN<3>,
    options: &NewtonOptions,
) -> NewtonResultN<3> {
    let mut function = |x: &[f64; 3], f: &mut [f64; 3], j: &mut [[f64; 3]; 3]| {
        let c: CurveD2 = curve.d2(x[0]);
        let s: SurfaceD2 = surface.d2(x[1], x[2]);
        curve_system(c, s, f, j);
        true
    };
    solve_3d(&mut function, start, bounds, options)
}
fn curve_system(c: CurveD2, s: SurfaceD2, f: &mut [f64; 3], j: &mut [[f64; 3]; 3]) {
    let d: Vector3 = Vector3::between(s.point, c.point);
    *f = [d.dot(c.d1), -d.dot(s.du), -d.dot(s.dv)];
    j[0][0] = c.d1.dot(c.d1) + d.dot(c.d2);
    j[0][1] = -s.du.dot(c.d1);
    j[0][2] = -s.dv.dot(c.d1);
    j[1][0] = j[0][1];
    j[2][0] = j[0][2];
    j[1][1] = s.du.dot(s.du) - d.dot(s.duu);
    j[1][2] = s.dv.dot(s.du) - d.dot(s.duv);
    j[2][1] = j[1][2];
    j[2][2] = s.dv.dot(s.dv) - d.dot(s.dvv);
}
/// Finds native surface/surface extrema; `solve_surface_surface_extrema_4d(&mut first, &mut second, [0.0;4], &bounds, &options)`.
pub fn solve_surface_surface_extrema_4d<S: SurfaceEvaluatorD2, T: SurfaceEvaluatorD2>(
    first: &mut S,
    second: &mut T,
    start: [f64; 4],
    bounds: &NewtonBoundsN<4>,
    options: &NewtonOptions,
) -> NewtonResultN<4> {
    let mut function = |x: &[f64; 4], f: &mut [f64; 4], j: &mut [[f64; 4]; 4]| {
        let a: SurfaceD2 = first.d2(x[0], x[1]);
        let b: SurfaceD2 = second.d2(x[2], x[3]);
        surface_system(a, b, f, j);
        true
    };
    solve_4d(&mut function, start, bounds, options)
}
fn surface_system(a: SurfaceD2, b: SurfaceD2, f: &mut [f64; 4], j: &mut [[f64; 4]; 4]) {
    let d: Vector3 = Vector3::between(b.point, a.point);
    *f = [d.dot(a.du), d.dot(a.dv), -d.dot(b.du), -d.dot(b.dv)];
    j[0][0] = a.du.dot(a.du) + d.dot(a.duu);
    j[0][1] = a.dv.dot(a.du) + d.dot(a.duv);
    j[1][0] = j[0][1];
    j[1][1] = a.dv.dot(a.dv) + d.dot(a.dvv);
    j[2][2] = b.du.dot(b.du) - d.dot(b.duu);
    j[2][3] = b.dv.dot(b.du) - d.dot(b.duv);
    j[3][2] = j[2][3];
    j[3][3] = b.dv.dot(b.dv) - d.dot(b.dvv);
    cross_blocks(a, b, j);
}
fn cross_blocks(a: SurfaceD2, b: SurfaceD2, j: &mut [[f64; 4]; 4]) {
    j[0][2] = -b.du.dot(a.du);
    j[0][3] = -b.dv.dot(a.du);
    j[1][2] = -b.du.dot(a.dv);
    j[1][3] = -b.dv.dot(a.dv);
    j[2][0] = j[0][2];
    j[2][1] = j[1][2];
    j[3][0] = j[0][3];
    j[3][1] = j[1][3];
}
