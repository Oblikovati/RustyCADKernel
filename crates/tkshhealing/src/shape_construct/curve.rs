// Rust translation of ModelingAlgorithms/TKShHealing/ShapeConstruct/ShapeConstruct_Curve.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.
//
// sln 29.12.2001 OCC90: FixKnots.

//! `ShapeConstruct_Curve`: moves the ends of lines and B-splines onto points, converts
//! curves to B-splines and spaces B-spline knots.
mod adjust;
mod convert_2d;
mod convert_3d;
mod knots;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;

/// `ShapeConstruct_Curve`: adjusts curves to have their ends at given points (lines and
/// B-splines only) and converts curves to B-splines.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
/// use rustycad_tkmath::gp::{Axis1, Direction3, Point3};
/// use rustycad_tkshhealing::shape_construct::ShapeConstructCurve;
/// let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(Axis1::new(
///     Point3::new(0.0, 0.0, 0.0),
///     Direction3::new(1.0, 0.0, 0.0).unwrap(),
/// )))
/// .cast();
/// let p2 = Point3::new(1.0, 1.0, 0.0);
/// let adjusted = ShapeConstructCurve.adjust_curve(&line, Point3::new(0.0, 1.0, 0.0), p2, true, true);
/// assert!(adjusted.unwrap());
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct ShapeConstructCurve;

fn null_curve(member: &str) -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        &format!("ShapeConstruct_Curve::{member}: null curve; expected a curve handle"),
    )
}

/// The segment bounds of `ConvertToBSpline`: `(first, last)` narrowed where the curve
/// range goes beyond them by more than `PConfusion`; `None` when nothing is narrowed.
fn segment_bounds(first: f64, last: f64, fbsp: f64, lbsp: f64) -> Option<(f64, f64)> {
    let trim_first = first > fbsp + precision::p_confusion();
    let trim_last = last < lbsp - precision::p_confusion();
    if !trim_first && !trim_last {
        return None;
    }
    Some((
        if trim_first { first } else { fbsp },
        if trim_last { last } else { lbsp },
    ))
}
