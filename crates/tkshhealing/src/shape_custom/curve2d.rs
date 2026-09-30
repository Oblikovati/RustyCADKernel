// Rust translation of ModelingAlgorithms/TKShHealing/ShapeCustom/ShapeCustom_Curve2d.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeCustom_Curve2d`: linear B-spline and Bezier p-curves as lines, and knot removal
//! (docs/shhealing-design.md section 7).
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCurve, Geom2dLine};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Direction2, Line2, Point2, Vector2};

/// The line of `ConvertToLine2d` and its output parameters.
#[derive(Clone)]
pub struct Line2dConversion {
    /// The new line.
    pub line: Handle<Geom2dLine>,
    /// `theNewFirst`: the parameter of the first point on the line.
    pub first: f64,
    /// `theNewLast`: the parameter of the last point on the line.
    pub last: f64,
    /// `theDeviation`: the largest distance of a pole from the line.
    pub deviation: f64,
}

/// `ShapeCustom_Curve2d`: the static tools that turn linear p-curves into lines and
/// remove the needless knots of a 2D B-spline.
///
/// ```
/// use rustycad_tkernel::ncollection::Array1;
/// use rustycad_tkmath::gp::Point2;
/// use rustycad_tkshhealing::shape_custom::ShapeCustomCurve2d;
/// let points = [Point2::new(0.0, 0.0), Point2::new(1.0, 0.0), Point2::new(2.0, 0.0)];
/// let poles = Array1::from_shared(&points, 1);
/// assert_eq!(ShapeCustomCurve2d::is_linear(&poles, 1e-7).unwrap(), Some(0.0));
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct ShapeCustomCurve2d;

/// `thePoles(i)`: the pole at index `i`, Standard_OutOfRange outside the array bounds
/// (the native loops read the indices 1..Length whatever the lower bound is).
fn pole(poles: &Array1<'_, Point2>, index: i32) -> Result<Point2, Failure> {
    poles.value(index).copied().map_err(|_| {
        Failure::with_type(
            "Standard_OutOfRange",
            &format!(
                "ShapeCustom_Curve2d::IsLinear: pole index {index} outside [{}, {}]",
                poles.lower(),
                poles.upper()
            ),
            "",
        )
    })
}

/// The first pair of poles at the largest square distance (strictly larger than every
/// earlier pair), with that square distance.
fn farthest_pair(poles: &Array1<'_, Point2>) -> Result<(i32, i32, f64), Failure> {
    let count = poles.length();
    let (mut i_max1, mut i_max2, mut d_max) = (0, 0, 0.0);
    for i in 1..count {
        for j in i + 1..=count {
            let dist = pole(poles, i)?.square_distance(pole(poles, j)?);
            if dist > d_max {
                (d_max, i_max1, i_max2) = (dist, i, j);
            }
        }
    }
    Ok((i_max1, i_max2, d_max))
}

/// The static `GetLine(P1, P2, c1, cf, cl)`: the line through `p1` towards `p2`, moved
/// to its own point at `c1`, with the parameters of both points on it.
fn get_line(p1: Point2, p2: Point2, c1: f64) -> Result<(Line2, f64, f64), Failure> {
    let direction = Direction2::from_vector(Vector2::between(p1, p2))?;
    let mut line = Line2::new(p1, direction);
    line.set_location(elclib::value(c1, &line));
    let first = elclib::parameter(&line, p1)?;
    let last = elclib::parameter(&line, p2)?;
    Ok((line, first, last))
}

impl ShapeCustomCurve2d {
    /// `IsLinear(thePoles, theTolerance, theDeviation)`: the deviation (the largest
    /// distance of a pole from the line through the two farthest poles) when every pole
    /// lies within `tolerance` of that line; `None` (native false, the deviation left
    /// unchanged) otherwise, and for fewer than two poles or poles closer than
    /// `Precision::PConfusion()`; `ShapeCustomCurve2d::is_linear(&poles, 1e-7)?`.
    pub fn is_linear(poles: &Array1<'_, Point2>, tolerance: f64) -> Result<Option<f64>, Failure> {
        if poles.length() < 2 {
            return Ok(None);
        }
        let (i_max1, i_max2, d_max) = farthest_pair(poles)?;
        let d_preci = precision::p_confusion() * precision::p_confusion();
        if d_max < d_preci {
            return Ok(None);
        }
        let tol2 = tolerance * tolerance;
        let (a, b) = (pole(poles, i_max1)?, pole(poles, i_max2)?);
        let line = Line2::new(a, Direction2::from_vector(Vector2::between(a, b))?);
        let mut a_max = 0.0;
        for i in 1..=poles.length() {
            let dist = line.square_distance(pole(poles, i)?);
            if dist > tol2 {
                return Ok(None);
            }
            if dist > a_max {
                a_max = dist;
            }
        }
        Ok(Some(a_max.sqrt()))
    }

    /// `ConvertToLine2d(theCurve, c1, c2, theTolerance, cf, cl, theDeviation)`: a line
    /// through the points at `c1` and `c2` when the curve is a B-spline or a Bezier curve
    /// whose poles are linear within `tolerance`, located so that `c1` is the parameter of
    /// its own point, with the parameters of both points and the deviation; `None` (the
    /// native null line, the outputs left unchanged) for points closer than the
    /// tolerance, another curve kind or non-linear poles. Standard_NullObject for a null
    /// curve; `ShapeCustomCurve2d::convert_to_line2d(&curve, 0.0, 1.0, 1e-7)?`.
    pub fn convert_to_line2d(
        curve: &Handle<dyn Geom2dCurve>,
        c1: f64,
        c2: f64,
        tolerance: f64,
    ) -> Result<Option<Line2dConversion>, Failure> {
        let stored = curve.get().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "ShapeCustom_Curve2d::ConvertToLine2d: null curve",
                "",
            )
        })?;
        let (p1, p2) = (stored.value(c1)?, stored.value(c2)?);
        if p1.square_distance(p2) < tolerance * tolerance {
            return Ok(None);
        }
        let Some(poles) = linear_candidate_poles(curve) else {
            return Ok(None);
        };
        let Some(deviation) = Self::is_linear(&poles, tolerance)? else {
            return Ok(None);
        };
        let (line, first, last) = get_line(p1, p2, c1)?;
        Ok(Some(Line2dConversion {
            line: Handle::new(Geom2dLine::from_lin2d(line)),
            first,
            last,
            deviation,
        }))
    }

    /// `SimplifyBSpline2d(theBSpline2d, theTolerance)`: walks the inner knots from the
    /// last one down and removes one multiplicity of each knot where the curve is
    /// `C(degree - mult)` and the two local derivatives of that order are parallel
    /// (a failed `RemoveKnot` is ignored, as the native `try/catch`); true when a knot
    /// was removed. A null derivative raises, as natively;
    /// `ShapeCustomCurve2d::simplify_bspline2d(&bspline, 1e-7)?`.
    pub fn simplify_bspline2d(
        bspline: &Geom2dBSplineCurve,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        let init_nb_knots = bspline.nb_knots();
        let mut nb_knots = init_nb_knots;
        let mut knot_index = nb_knots - 1;
        let mut to_remove = true;
        while to_remove && nb_knots > 2 {
            remove_parallel_knot(bspline, knot_index, tolerance)?;
            knot_index -= 1;
            nb_knots = bspline.nb_knots();
            if knot_index == 1 || knot_index == nb_knots {
                to_remove = false;
            }
        }
        Ok(init_nb_knots > nb_knots)
    }
}

/// The poles of a B-spline or a Bezier curve (`DownCast` to either), none for another
/// curve.
fn linear_candidate_poles(curve: &Handle<dyn Geom2dCurve>) -> Option<Array1<'static, Point2>> {
    if let Some(bspline) = curve.cast::<Geom2dBSplineCurve>().get() {
        return Some(bspline.poles());
    }
    curve
        .cast::<Geom2dBezierCurve>()
        .get()
        .map(Geom2dBezierCurve::poles)
}

/// One step of the `SimplifyBSpline2d` loop at `knot_index`: `RemoveKnot(index, mult -
/// 1, tolerance)` when the derivatives of order `degree - mult` (above 1) on both sides
/// of the knot are parallel within `Precision::Angular()`.
fn remove_parallel_knot(
    bspline: &Geom2dBSplineCurve,
    knot_index: i32,
    tolerance: f64,
) -> Result<(), Failure> {
    let mult = bspline.multiplicity(knot_index)?;
    let deg_mult = bspline.degree() - mult;
    if deg_mult <= 1 || !bspline.is_cn(deg_mult)? {
        return Ok(());
    }
    let u = bspline.knot(knot_index)?;
    let before = bspline.local_dn(u, knot_index - 1, knot_index, deg_mult)?;
    let after = bspline.local_dn(u, knot_index, knot_index + 1, deg_mult)?;
    if before.is_parallel(after, precision::angular())? {
        // The native `catch (Standard_Failure const&) {}` drops the failure.
        let _ = bspline.remove_knot(knot_index, mult - 1, tolerance);
    }
    Ok(())
}
