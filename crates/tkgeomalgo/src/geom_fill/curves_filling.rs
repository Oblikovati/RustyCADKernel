// Rust translation of the steps that GeomFill_BezierCurves.cxx and GeomFill_BSplineCurves.cxx
// repeat, at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The steps shared by `GeomFill_BezierCurves` and `GeomFill_BSplineCurves`: the
//! choice of the filling scheme, the closing segment of three curves, the joining of
//! two curves in the curved style and the grids of a scheme.
use super::filling::Rows;
use super::{
    GeomFillCoons, GeomFillCurved, GeomFillFilling, GeomFillFillingStyle, GeomFillStretch,
};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg2d::geom2d::arrays::{at, one_based, put};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkg3d::geom::GeomBoundedCurve;
use rustycad_tkmath::gp::Point3;

/// The object behind a curve handle; Standard_NullObject for a null one (a null
/// dereference natively).
pub(super) fn live<T: ?Sized + RuntimeTyped>(handle: &Handle<T>) -> Result<&T, Failure> {
    handle
        .get()
        .ok_or_else(|| errors::null_object("GeomFill: null boundary curve"))
}

/// The pole rows of the four arranged curves and, when one is rational, their weights.
pub(super) struct FourBoundaries<'r, 'a> {
    pub(super) poles: Rows<'r, 'a, Point3>,
    pub(super) weights: Option<Rows<'r, 'a, f64>>,
}

/// The `switch (Type)` of the four-curve `Init`: the stretched and curved schemes take
/// the rows in order, the Coons scheme as `(P1, P4, P3, P2)`; the result is the
/// `GeomFill_Filling` part of the scheme, as the native slicing assignment keeps.
pub(super) fn fill_scheme(
    b: &FourBoundaries<'_, '_>,
    style: GeomFillFillingStyle,
) -> Result<GeomFillFilling, Failure> {
    let [p1, p2, p3, p4] = b.poles;
    let coons_poles = [p1, p4, p3, p2];
    match (style, b.weights) {
        (GeomFillFillingStyle::StretchStyle, None) => {
            Ok((*GeomFillStretch::with_poles(b.poles)?).clone())
        }
        (GeomFillFillingStyle::CoonsStyle, None) => {
            Ok((*GeomFillCoons::with_poles(coons_poles)?).clone())
        }
        (GeomFillFillingStyle::CurvedStyle, None) => {
            Ok((*GeomFillCurved::with_poles(b.poles)?).clone())
        }
        (GeomFillFillingStyle::StretchStyle, Some(w)) => {
            Ok((*GeomFillStretch::with_weights(b.poles, w)?).clone())
        }
        (GeomFillFillingStyle::CoonsStyle, Some([w1, w2, w3, w4])) => {
            Ok((*GeomFillCoons::with_weights(coons_poles, [w1, w4, w3, w2])?).clone())
        }
        (GeomFillFillingStyle::CurvedStyle, Some(w)) => {
            Ok((*GeomFillCurved::with_weights(b.poles, w)?).clone())
        }
    }
}

/// The two poles of the segment that closes three curves: the end of `c1` that does
/// not touch `c2` (within the square of Precision::Confusion, on squared
/// distances), then the same end of `c3`.
pub(super) fn closing_poles(
    curves: [&dyn GeomBoundedCurve; 3],
) -> Result<Array1<'static, Point3>, Failure> {
    let tol: f64 = precision::confusion() * precision::confusion();
    let (c2_start, c2_end) = (curves[1].start_point()?, curves[1].end_point()?);
    let mut poles: Array1<'static, Point3> = one_based(2)?;
    for (index, curve) in [(1, curves[0]), (2, curves[2])] {
        let start: Point3 = curve.start_point()?;
        let free: Point3 =
            if start.square_distance(&c2_start) > tol && start.square_distance(&c2_end) > tol {
                start
            } else {
                curve.end_point()?
            };
        put(&mut poles, index, free)?;
    }
    Ok(poles)
}

/// The two copies of the curved `Init(C1, C2)` and the caller's first curve.
pub(super) struct CurvedPair<'c> {
    pub(super) copies: [&'c dyn GeomBoundedCurve; 2],
    pub(super) caller_first: &'c dyn GeomBoundedCurve,
    pub(super) message: &'static str,
}

/// Joins the copies at a common start (within Precision::Confusion), reversing the
/// second, both, or (native quirk) the caller's first curve when the end of the first
/// copy meets the start of the second; Standard_OutOfRange when no ends touch.
pub(super) fn curved_pair(pair: &CurvedPair<'_>) -> Result<(), Failure> {
    let eps: f64 = precision::confusion();
    let [cc1, cc2] = pair.copies;
    let (s1, e1) = (cc1.start_point()?, cc1.end_point()?);
    let (s2, e2) = (cc2.start_point()?, cc2.end_point()?);
    if s1.is_equal(&s2, eps) {
        return Ok(());
    }
    if s1.is_equal(&e2, eps) {
        return cc2.reverse();
    }
    if e1.is_equal(&s2, eps) {
        return pair.caller_first.reverse();
    }
    if e1.is_equal(&e2, eps) {
        cc1.reverse()?;
        return cc2.reverse();
    }
    Err(errors::out_of_range(pair.message))
}

/// `GeomFill_Curved(P1, P2)` or `GeomFill_Curved(P1, P2, W1, W2)` of the joined pair.
pub(super) fn curved_two(
    rows: &[Array1<'static, Point3>; 2],
    weights: Option<&[Array1<'static, f64>; 2]>,
) -> Result<GeomFillFilling, Failure> {
    let curved = match weights {
        Some([w1, w2]) => GeomFillCurved::with_two_weighted_rows(&rows[0], &rows[1], w1, w2)?,
        None => GeomFillCurved::with_two_rows(&rows[0], &rows[1])?,
    };
    Ok((*curved).clone())
}

/// The pole grid of a scheme, and its weight grid when the scheme is rational
/// (`Caro.Poles(Poles)`, `Caro.Weights(Weights)` on grids `(1..NbU, 1..NbV)`).
pub(super) type SchemeGrids = (Array2<'static, Point3>, Option<Array2<'static, f64>>);

/// The grids a surface is built from, read from the scheme as natively.
pub(super) fn scheme_grids(caro: &GeomFillFilling) -> Result<SchemeGrids, Failure> {
    let (nu, nv) = (caro.nb_u_poles()?, caro.nb_v_poles()?);
    let table = |e| errors::array_bounds(e);
    let mut poles: Array2<'static, Point3> = Array2::with_bounds(1, nu, 1, nv).map_err(table)?;
    caro.poles(&mut poles)?;
    if !caro.is_rational() {
        return Ok((poles, None));
    }
    let mut weights: Array2<'static, f64> = Array2::with_bounds(1, nu, 1, nv).map_err(table)?;
    caro.weights(&mut weights)?;
    Ok((poles, Some(weights)))
}

/// The two-column grids of a ruled surface: the first `n` poles (and weights, when
/// given) of each curve as columns 1 and 2.
pub(super) fn ruled_grids(
    poles: [&Array1<'_, Point3>; 2],
    weights: Option<[&Array1<'_, f64>; 2]>,
    n: i32,
) -> Result<SchemeGrids, Failure> {
    Ok((
        two_columns(poles, n)?,
        weights.map(|w| two_columns(w, n)).transpose()?,
    ))
}

/// `Grid(i, 1) = A(i); Grid(i, 2) = B(i)` for `i` in `1..=n`.
fn two_columns<T: Copy + Default>(
    columns: [&Array1<'_, T>; 2],
    n: i32,
) -> Result<Array2<'static, T>, Failure> {
    let mut grid: Array2<'static, T> =
        Array2::with_bounds(1, n, 1, 2).map_err(errors::array_bounds)?;
    for i in 1..=n {
        grid[(i, 1)] = at(columns[0], i)?;
        grid[(i, 2)] = at(columns[1], i)?;
    }
    Ok(grid)
}

/// The second search of `Arrange` at chain slot `i`: a curve that starts at `end`, or
/// ends there (then replaced by its reversed copy), moves to slot `i`.
pub(super) fn take_open<C>(
    gc: &mut [Handle<C>],
    i: usize,
    end: Point3,
    tol: f64,
) -> Result<bool, Failure>
where
    C: GeomBoundedCurve + RuntimeTyped,
{
    for j in i..gc.len() {
        let candidate: &C = live(&gc[j])?;
        if candidate.start_point()?.distance(&end) < tol {
            gc.swap(i, j);
            return Ok(true);
        }
        if candidate.end_point()?.distance(&end) < tol {
            gc[j] = candidate.reversed()?.cast();
            gc.swap(i, j);
            return Ok(true);
        }
    }
    Ok(false)
}

/// The result of `Arrange`: the first two chained curves, and reversed copies of the
/// last two.
pub(super) fn chained<C>(gc: &[Handle<C>]) -> Result<[Handle<C>; 4], Failure>
where
    C: GeomBoundedCurve + RuntimeTyped,
{
    let reversed =
        |k: usize| -> Result<Handle<C>, Failure> { Ok(live(&gc[k])?.reversed()?.cast()) };
    Ok([gc[0].clone(), gc[1].clone(), reversed(2)?, reversed(3)?])
}
