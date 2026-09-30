// Rust translation of the file-static helpers TRI_SOLUTION, MIN_SOLUTION and
// isOnBoundary of ModelingAlgorithms/TKTopAlgo/BRepExtrema/BRepExtrema_DistanceSS.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The solution filters of `BRepExtrema_DistanceSS`.
use crate::brep_class::BRepClassFaceClassifier;
use crate::brep_extrema::BRepExtremaSolutionElem;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Edge, Face, ShapeIterator};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::gp::{Point2, Point3};

/// `TRI_SOLUTION`: true when no solution of `seq_sol` is within Precision::Confusion()
/// of `pt` (the point is a new solution).
pub(super) fn tri_solution(seq_sol: &[BRepExtremaSolutionElem], pt: &Point3) -> bool {
    !seq_sol
        .iter()
        .any(|solution| solution.point().distance(pt) <= precision::confusion())
}

/// `MIN_SOLUTION`: appends to `seq_sol1`/`seq_sol2` the pairs of `solutions` whose
/// distance is within `eps` of `dst_ref`, in order.
pub(super) fn min_solution(
    solutions: (&[BRepExtremaSolutionElem], &[BRepExtremaSolutionElem]),
    dst_ref: f64,
    eps: f64,
    seq_sol1: &mut Vec<BRepExtremaSolutionElem>,
    seq_sol2: &mut Vec<BRepExtremaSolutionElem>,
) {
    for (solution1, solution2) in solutions.0.iter().zip(solutions.1) {
        if (solution1.dist() - dst_ref).abs() < eps {
            seq_sol1.push(solution1.clone());
            seq_sol2.push(solution2.clone());
        }
    }
}

/// `isOnBoundary`: true when a vertex of `edge` (visited with the iterator) has its
/// parameter within `p_tol` of `param` and its point within its tolerance of `sol`.
pub(super) fn is_on_boundary(
    edge: &Edge,
    sol: &Point3,
    param: f64,
    p_tol: f64,
) -> Result<bool, Failure> {
    for child in ShapeIterator::new(edge, true, true) {
        let vertex = topods::vertex(&child)?;
        let v_param: f64 = BRepTool::parameter(&vertex, edge)?;
        if (v_param - param).abs() < p_tol
            && BRepTool::pnt(&vertex)?.distance(sol) < BRepTool::vertex_tolerance(&vertex)?
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// `fabs(a - b) < eps`: false for a NaN difference, as natively.
pub(super) fn near(a: f64, b: f64, eps: f64) -> bool {
    (a - b).abs() < eps
}

/// The smallest of the square distances `1..=nb` that `square_distance` reads (the
/// Dstmin loop of every overload), before its square root.
pub(super) fn min_of(
    nb: i32,
    square_distance: &impl Fn(i32) -> Result<f64, Failure>,
) -> Result<f64, Failure> {
    let mut dst_min: f64 = square_distance(1)?;
    for i in 2..=nb {
        let s_dst: f64 = square_distance(i)?;
        if s_dst < dst_min {
            dst_min = s_dst;
        }
    }
    Ok(dst_min)
}

/// The acceptance of a candidate distance `dst` against the reference `dst_ref`:
/// smaller by more than `eps`, or equal within `eps`.
pub(super) fn accepts(dst: f64, dst_ref: f64, eps: f64) -> bool {
    dst < dst_ref - eps || near(dst, dst_ref, eps)
}

/// The nearest extrema of an overload: `None` when there is no extremum or when the
/// smallest distance is not accepted against `dst_ref`; else that distance and the
/// indices `1..=nb` within `eps` of it, in order (the `Dstmin` test and loop).
pub(super) fn nearest_extrema(
    nb: i32,
    square_distance: impl Fn(i32) -> Result<f64, Failure>,
    dst_ref: f64,
    eps: f64,
) -> Result<Option<(f64, Vec<i32>)>, Failure> {
    if nb <= 0 {
        return Ok(None);
    }
    let dst_min: f64 = min_of(nb, &square_distance)?.sqrt();
    if !accepts(dst_min, dst_ref, eps) {
        return Ok(None);
    }
    let mut indices: Vec<i32> = Vec::new();
    for i in 1..=nb {
        if near(dst_min, square_distance(i)?.sqrt(), eps) {
            indices.push(i);
        }
    }
    Ok(Some((dst_min, indices)))
}

/// Whether the face classifier puts `(u, v)` strictly inside `face` (the check that the
/// parameter does not correspond to a vertex or an edge of the face).
pub(super) fn is_in(
    classifier: &mut BRepClassFaceClassifier,
    face: &Face,
    (u, v): (f64, f64),
    tol: f64,
) -> Result<bool, Failure> {
    classifier.perform(face, &Point2::new(u, v), tol, false, 0.1)?;
    Ok(classifier.state() == State::In)
}
