// Rust translation of ModelingData/TKGeomBase/Approx/Approx_ComputeLine.gxx (static CheckMultiCurve)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The loop check of `Approx_ComputeLine`: a Bezier fit whose control polygon
//! turns back on itself while the points do not is rejected, and the point after
//! the longest chord is reported for `MakeMLOneMorePoint`.
use crate::app_par_curves::arrays::scratch;
use crate::app_par_curves::{LineTool, MultiCurve, MultiCurveEval};
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::{epsilon, real_last};
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3, resolution};

/// `MinScalProd`: the cosine under which two legs turn back.
const MIN_SCAL_PROD: f64 = -0.9;
/// `coeff` (2*2): the squared ratio of the longest to the shortest chord of a loop.
const CHORD_RATIO: f64 = 4.;

/// The point accessors the check needs in one dimension.
trait CheckPoint: Copy + Default {
    type Vector: Copy + Default;
    fn read<L: LineTool>(line: &L, index: i32, tab: &mut Array1<'_, Self>) -> Result<(), Failure>;
    fn square_distance(self, other: Self) -> f64;
    fn vector(from: Self, to: Self) -> Self::Vector;
    fn normalized(v: Self::Vector) -> Result<Self::Vector, Failure>;
    fn dot(a: Self::Vector, b: Self::Vector) -> f64;
}

impl CheckPoint for Point3 {
    type Vector = Vector3;
    fn read<L: LineTool>(line: &L, index: i32, tab: &mut Array1<'_, Self>) -> Result<(), Failure> {
        line.value_3d(index, tab)
    }
    fn square_distance(self, other: Self) -> f64 {
        Point3::square_distance(&self, &other)
    }
    fn vector(from: Self, to: Self) -> Vector3 {
        Vector3::between(from, to)
    }
    fn normalized(v: Vector3) -> Result<Vector3, Failure> {
        v.normalized()
    }
    fn dot(a: Vector3, b: Vector3) -> f64 {
        a.dot(b)
    }
}

impl CheckPoint for Point2 {
    type Vector = Vector2;
    fn read<L: LineTool>(line: &L, index: i32, tab: &mut Array1<'_, Self>) -> Result<(), Failure> {
        line.value_2d(index, tab)
    }
    fn square_distance(self, other: Self) -> f64 {
        Point2::square_distance(self, other)
    }
    fn vector(from: Self, to: Self) -> Vector2 {
        Vector2::between(from, to)
    }
    fn normalized(v: Vector2) -> Result<Vector2, Failure> {
        v.normalized()
    }
    fn dot(a: Vector2, b: Vector2) -> f64 {
        a.dot(b)
    }
}

// Point `slot` of multipoint `index` (read with the one-dimension Value overload).
fn line_point<P: CheckPoint, L: LineTool>(
    line: &L,
    index: i32,
    slot: i32,
    tab: &mut Array1<'static, P>,
) -> Result<P, Failure> {
    P::read(line, index, tab)?;
    tab.value(slot)
        .copied()
        .map_err(|e| array_failure(e, "Approx_ComputeLine::CheckMultiCurve"))
}

/// `CheckMultiCurve(theMultiCurve, theLine, theIndfirst, theIndlast, theIndbad)`:
/// false when the fit loops; `index_bad` is untouched for more than one 3D curve
/// (only simple cases are analysed) and otherwise receives the point to refine (0
/// when there is no loop).
pub(crate) fn check_multi_curve<L: LineTool>(
    curve: &MultiCurve,
    line: &L,
    [first, last]: [i32; 2],
    index_bad: &mut i32,
) -> Result<bool, Failure> {
    if line.nb_p3d()? > 1 {
        return Ok(true);
    }
    *index_bad = 0;
    // indbads[1..=3] natively; a larger curve index writes past the native array
    // (undefined behaviour), which only the first three entries are read after.
    let nb_cur: i32 = curve.nb_curves()?;
    let mut bads: Vec<i32> = vec![0; (nb_cur.max(3) + 1) as usize];
    let loop_found: bool = if curve.dimension(1)? == 3 {
        check_3d(curve, line, [first, last], &mut bads)?
    } else {
        match check_2d(curve, line, [first, last], &mut bads)? {
            Some(found) => found,
            None => {
                // A degenerate leg returns at once with its own index_bad.
                *index_bad = bads[0];
                return Ok(false);
            }
        }
    };
    if let Some(&bad) = bads[1..=3].iter().find(|&&bad| bad != 0) {
        *index_bad = bad;
    }
    if !loop_found {
        *index_bad = 0;
    }
    Ok(!loop_found)
}

// The 3D case: the first 3D curve, then the 2D curves for the other entries.
fn check_3d<L: LineTool>(
    curve: &MultiCurve,
    line: &L,
    range: [i32; 2],
    bads: &mut [i32],
) -> Result<bool, Failure> {
    let mut poles: Array1<'static, Point3> = scratch(curve.nb_poles())?;
    curve.curve_3d(1, &mut poles)?;
    let mut tab: Array1<'static, Point3> = scratch(line.nb_p3d()?)?;
    if !polygon_turns_back_3d(&poles)? || !points_loop::<Point3, L>(line, range, &mut tab)? {
        return Ok(false);
    }
    let (max_sq, min_sq) = chord_extremes::<Point3, L>(line, range, 1, &mut tab, &mut bads[1])?;
    if max_sq / min_sq < CHORD_RATIO {
        return Ok(false);
    }
    let mut tab2d: Array1<'static, Point2> = scratch(line.nb_p2d()?)?;
    for indcur in 2..=curve.nb_curves()? {
        let mut bad: i32 = bads[indcur as usize];
        chord_extremes::<Point2, L>(line, range, indcur - 1, &mut tab2d, &mut bad)?;
        bads[indcur as usize] = bad;
    }
    Ok(true)
}

// The polygon test of the 3D case: legs shorter than gp::Resolution() are skipped
// and the legs are normalized by their length.
fn polygon_turns_back_3d(poles: &Array1<'static, Point3>) -> Result<bool, Failure> {
    let pole = |i: i32| poles[i];
    let mut first_vec: Vector3 = Vector3::default();
    let mut indp: i32 = 2;
    while indp <= poles.upper() {
        first_vec = Vector3::between(pole(1), pole(indp));
        indp += 1;
        let length: f64 = first_vec.magnitude();
        if length > resolution() {
            first_vec /= length;
            break;
        }
    }
    let mut mid: Point3 = pole(indp - 1);
    while indp <= poles.upper() {
        let mut second_vec: Vector3 = Vector3::between(mid, pole(indp));
        let length: f64 = second_vec.magnitude();
        if length <= resolution() {
            indp += 1;
            continue;
        }
        second_vec /= length;
        if first_vec.dot(second_vec) < MIN_SCAL_PROD {
            return Ok(true);
        }
        (first_vec, mid) = (second_vec, pole(indp));
        indp += 1;
    }
    Ok(false)
}

// "Check: may be it is a real loop": true when, seen from every point, no two
// later points coincide with it or lie in opposite directions.
fn points_loop<P: CheckPoint, L: LineTool>(
    line: &L,
    [first, last]: [i32; 2],
    tab: &mut Array1<'static, P>,
) -> Result<bool, Failure> {
    let sq_tol: f64 = precision::square_confusion();
    for first_ind in first..=last - 2 {
        let first_pnt: P = line_point(line, first_ind, 1, tab)?;
        for k in first_ind + 1..last {
            let pnt1: P = line_point(line, k, 1, tab)?;
            let pnt2: P = line_point(line, k + 1, 1, tab)?;
            if first_pnt.square_distance(pnt1) <= sq_tol
                || first_pnt.square_distance(pnt2) <= sq_tol
            {
                return Ok(false);
            }
            let vec1 = P::normalized(P::vector(first_pnt, pnt1))?;
            let vec2 = P::normalized(P::vector(first_pnt, pnt2))?;
            if P::dot(vec1, vec2) < MIN_SCAL_PROD {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

// The largest squared chord of point `slot` (its end index goes to `bad`) and the
// smallest one above gp::Resolution().
fn chord_extremes<P: CheckPoint, L: LineTool>(
    line: &L,
    [first, last]: [i32; 2],
    slot: i32,
    tab: &mut Array1<'static, P>,
    bad: &mut i32,
) -> Result<(f64, f64), Failure> {
    let (mut max_sq, mut min_sq): (f64, f64) = (0., real_last());
    for k in first + 1..=last {
        let prev: P = line_point(line, k - 1, slot, tab)?;
        let cur: P = line_point(line, k, slot, tab)?;
        let sq: f64 = prev.square_distance(cur);
        if sq > max_sq {
            max_sq = sq;
            *bad = k;
        }
        if sq > resolution() && sq < min_sq {
            min_sq = sq;
        }
    }
    Ok((max_sq, min_sq))
}

// The 2D case (no 3D curve); None when a leg is degenerate (its index_bad is left
// in bads[0]).
fn check_2d<L: LineTool>(
    curve: &MultiCurve,
    line: &L,
    [first, last]: [i32; 2],
    bads: &mut [i32],
) -> Result<Option<bool>, Failure> {
    let mut poles: Array1<'static, Point2> = scratch(curve.nb_poles())?;
    curve.curve_2d(1, &mut poles)?;
    match polygon_turns_back_2d(&poles)? {
        Err(k) => {
            bads[0] = first + k - 1;
            return Ok(None);
        }
        Ok(false) => return Ok(Some(false)),
        Ok(true) => {}
    }
    let mut tab: Array1<'static, Point2> = scratch(line.nb_p2d()?)?;
    if !points_loop::<Point2, L>(line, [first, last], &mut tab)? {
        return Ok(Some(false));
    }
    let mut found: bool = true;
    for indcur in 1..=curve.nb_curves()? {
        let mut bad: i32 = bads[indcur as usize];
        let (max_sq, min_sq) =
            chord_extremes::<Point2, L>(line, [first, last], indcur, &mut tab, &mut bad)?;
        bads[indcur as usize] = bad;
        if max_sq / min_sq < CHORD_RATIO {
            found = false;
        }
    }
    Ok(Some(found))
}

// The polygon test of the 2D case: a leg under Epsilon(1)^2 in squared length
// stops the check (Err with its pole index k, the leg ending at pole k); the first
// leg is divided by Epsilon(1) instead of its length (native quirk), so any first
// turn wider than a right angle reads as a loop.
fn polygon_turns_back_2d(poles: &Array1<'static, Point2>) -> Result<Result<bool, i32>, Failure> {
    let pole = |i: i32| {
        poles
            .value(i)
            .copied()
            .map_err(|e| array_failure(e, "Approx_ComputeLine::CheckMultiCurve"))
    };
    let sq_norm_toler: f64 = epsilon(1.0) * epsilon(1.0);
    let mut first_vec: Vector2 = Vector2::between(pole(1)?, pole(2)?);
    if first_vec.square_magnitude() < sq_norm_toler {
        return Ok(Err(2));
    }
    first_vec /= sq_norm_toler.sqrt();
    let mut mid: Point2 = pole(2)?;
    for k in 3..=poles.upper() {
        let mut second_vec: Vector2 = Vector2::between(mid, pole(k)?);
        let sq: f64 = second_vec.square_magnitude();
        if sq < sq_norm_toler {
            return Ok(Err(k));
        }
        second_vec /= sq.sqrt();
        if first_vec.dot(second_vec) < MIN_SCAL_PROD {
            return Ok(Ok(true));
        }
        (first_vec, mid) = (second_vec, pole(k)?);
    }
    Ok(Ok(false))
}
