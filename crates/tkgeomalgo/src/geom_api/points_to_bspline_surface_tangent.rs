// Rust translation of the static helpers of GeomAPI_PointsToBSplineSurface.cxx
// (BuildParameters, BuildPeriodicTangent) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The parameters and the closing tangent of the periodic U pass of
//! GeomAPI_PointsToBSplineSurface.
use crate::geom2d_api::points_to_bspline::has_meaningful_span;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::app_def::{BSpParLeastSquareOfMyBSplGradientOfBSplineCompute, MultiLine};
use rustycad_tkgeombase::app_par_curves::{Constraint, MultiCurve, MultiCurveEval};
use rustycad_tkgeombase::approx::ParametrizationType;
use rustycad_tkmath::math::Vector;

const MEMBER: &str = "GeomAPI_PointsToBSplineSurface";

fn at(values: &Array1<'_, f64>, index: i32) -> Result<f64, Failure> {
    values
        .value(index)
        .copied()
        .map_err(|e| e.into_failure(MEMBER))
}

fn put(values: &mut Array1<'_, f64>, index: i32, value: f64) -> Result<(), Failure> {
    values
        .set_value(index, value)
        .map_err(|e| e.into_failure(MEMBER))
}

/// `BuildParameters(theLine, theParT, thePars)`: 0 and 1 for two multipoints; the
/// cumulated distances between consecutive multipoints (their square roots for the
/// centripetal type) scaled to `[0, 1]` (uniform when the total is not meaningful);
/// uniform for the other types.
pub(super) fn build_parameters(
    the_line: &MultiLine,
    the_par_t: ParametrizationType,
    the_pars: &mut Array1<'_, f64>,
) -> Result<(), Failure> {
    let (first_p, last_p) = (1, the_line.nb_multi_points()?);
    if last_p - first_p + 1 == 2 {
        put(the_pars, first_p, 0.0)?;
        return put(the_pars, last_p, 1.0);
    }
    let chordal = matches!(
        the_par_t,
        ParametrizationType::ChordLength | ParametrizationType::Centripetal
    );
    if !chordal {
        return uniform_parameters(the_pars, first_p, last_p);
    }
    put(the_pars, first_p, 0.0)?;
    for i in first_p + 1..=last_p {
        let dist: f64 = multi_point_distance(the_line, i)?;
        let step: f64 = if the_par_t == ParametrizationType::ChordLength {
            dist
        } else {
            dist.sqrt()
        };
        let value: f64 = at(the_pars, i - 1)? + step;
        put(the_pars, i, value)?;
    }
    if !has_meaningful_span(at(the_pars, last_p)?) {
        return uniform_parameters(the_pars, first_p, last_p);
    }
    // `thePars(i) /= thePars(lastP)`: every division, the last one included, reads the
    // total before it is replaced.
    let total: f64 = at(the_pars, last_p)?;
    for i in first_p..=last_p {
        let value: f64 = at(the_pars, i)? / total;
        put(the_pars, i, value)?;
    }
    Ok(())
}

// The distance between the multipoints `i - 1` and `i`: the square root of the sum of
// the square distances of their points.
fn multi_point_distance(the_line: &MultiLine, i: i32) -> Result<f64, Failure> {
    let (a_mpc, a_mpc1) = (the_line.value(i - 1)?, the_line.value(i)?);
    let mut dist: f64 = 0.0;
    for j in 1..=the_line.nb_points()? {
        let (a_p1, a_p2) = (a_mpc.point(j)?, a_mpc1.point(j)?);
        dist += a_p2.square_distance(&a_p1);
    }
    Ok(dist.sqrt())
}

// `(i - firstP) / (lastP - firstP)`; the native non-chordal branch divides by
// `double(lastP - double(firstP))`, the degenerate chordal one by
// `double(lastP - firstP)`, which give the same value.
fn uniform_parameters(
    the_pars: &mut Array1<'_, f64>,
    first_p: i32,
    last_p: i32,
) -> Result<(), Failure> {
    for i in first_p..=last_p {
        put(
            the_pars,
            i,
            (f64::from(i) - f64::from(first_p)) / f64::from(last_p - first_p),
        )?;
    }
    Ok(())
}

// The first derivatives at `u` of the curves of `curve`, as `(x, y, z)` triples.
fn derivatives(curve: &MultiCurve, nb_p3d: i32, u: f64) -> Result<Vec<f64>, Failure> {
    let mut values: Vec<f64> = Vec::with_capacity(3 * nb_p3d as usize);
    for i in 1..=nb_p3d {
        let (_, a_v) = curve.d1_3d(i, u)?;
        values.extend([a_v.x(), a_v.y(), a_v.z()]);
    }
    Ok(values)
}

// The Bezier least-squares fit of the multipoints `first..=last` on their parameters,
// with `nnpol` poles and the constraint `cons` at both ends.
fn fit_bezier(
    the_line: &MultiLine,
    (first, last): (i32, i32),
    the_pars: &Array1<'_, f64>,
    (cons, nnpol): (Constraint, i32),
) -> Result<MultiCurve, Failure> {
    let mut sq = BSpParLeastSquareOfMyBSplGradientOfBSplineCompute::without_parameters(
        the_line, first, last, cons, cons, nnpol,
    )?;
    let mut p: Vector<'static> = Vector::with_bounds(first, last)?;
    for i in first..=last {
        p[i] = at(the_pars, i)?;
    }
    sq.perform(&p)?;
    sq.bezier_value()
}

/// `BuildPeriodicTangent(theLine, thePars, theTang)`: nothing for two multipoints or
/// fewer; otherwise the half sum of the start derivative of a Bezier fit of the first
/// (at most 9) multipoints and the end derivative of a fit of the last ones (the same
/// fit when it covers them all), with tangency constraints above 4 poles.
pub(super) fn build_periodic_tangent(
    the_line: &MultiLine,
    the_pars: &Array1<'_, f64>,
    the_tang: &mut Vector<'_>,
) -> Result<(), Failure> {
    let (firstpt, lastpt) = (1, the_line.nb_multi_points()?);
    let nbpoints: i32 = lastpt - firstpt + 1;
    if nbpoints <= 2 {
        return Ok(());
    }
    let nnp: i32 = nbpoints.min(9);
    let lastp: i32 = lastpt.min(firstpt + nnp - 1);
    let cons = if nnp <= 4 {
        Constraint::PassPoint
    } else {
        Constraint::TangencyPoint
    };
    let nb_p3d: i32 = the_line.nb_points()?;
    let c1 = fit_bezier(the_line, (firstpt, lastp), the_pars, (cons, nnp))?;
    let v1: Vec<f64> = derivatives(&c1, nb_p3d, 0.0)?;
    let firstp: i32 = firstpt.max(lastpt - nnp + 1);
    let v2: Vec<f64> = if firstp == firstpt && lastp == lastpt {
        derivatives(&c1, nb_p3d, 1.0)?
    } else {
        let c2 = fit_bezier(the_line, (firstp, lastpt), the_pars, (cons, nnp))?;
        derivatives(&c2, nb_p3d, 1.0)?
    };
    // `theTang = 0.5 * (V1 + V2)`.
    let lower: i32 = the_tang.lower();
    for (k, (a, b)) in v1.iter().zip(&v2).enumerate() {
        the_tang[lower + k as i32] = 0.5 * (a + b);
    }
    Ok(())
}
