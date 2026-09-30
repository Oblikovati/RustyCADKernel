// Rust translation of BRepLib_MakeFace::IsDegenerated (BRepLib_MakeFace.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib_MakeFace::IsDegenerated`: whether a curve is a point within a tolerance.
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::Point3;

/// `IsDegenerated(theCurve, theMaxTol, theActTol)`: a circle of radius at most
/// `max_tol` (with its radius, at least Precision::Confusion(), as the actual
/// tolerance), or a Bezier or B-spline curve whose poles are all within `max_tol` of
/// the first one (with 1.000001 times the largest distance); None otherwise (natively
/// false with the actual tolerance Precision::Confusion()).
pub(crate) fn degenerated_tolerance(
    curve: &Handle<dyn GeomCurve>,
    max_tol: f64,
) -> Result<Option<f64>, Failure> {
    let adaptor = GeomAdaptorCurve::with_curve(curve)?;
    let confusion: f64 = precision::confusion();
    match adaptor.get_type()? {
        CurveType::Circle => {
            let radius: f64 = adaptor.circle()?.radius();
            if radius > max_tol {
                return Ok(None);
            }
            Ok(Some(std_max(radius, confusion)))
        }
        CurveType::BSplineCurve => {
            let bspline = adaptor.bspline()?;
            let poles = match bspline.get() {
                Some(b) => (1..=b.nb_poles())
                    .map(|i| b.pole(i))
                    .collect::<Result<Vec<Point3>, Failure>>()?,
                None => Vec::new(),
            };
            Ok(pole_spread(&poles, max_tol))
        }
        CurveType::BezierCurve => {
            let bezier = adaptor.bezier()?;
            let poles = match bezier.get() {
                Some(b) => (1..=b.nb_poles())
                    .map(|i| b.pole(i))
                    .collect::<Result<Vec<Point3>, Failure>>()?,
                None => Vec::new(),
            };
            Ok(pole_spread(&poles, max_tol))
        }
        _ => Ok(None),
    }
}

/// The actual tolerance of poles within `max_tol` of the first one, None beyond.
fn pole_spread(poles: &[Point3], max_tol: f64) -> Option<f64> {
    let max_tol2: f64 = max_tol * max_tol;
    let mut max_pole_dist2: f64 = 0.0;
    let p1: Point3 = *poles.first()?;
    for p2 in &poles[1..] {
        let pole_dist2: f64 = p1.square_distance(p2);
        if pole_dist2 > max_tol2 {
            return None;
        }
        if pole_dist2 > max_pole_dist2 {
            max_pole_dist2 = pole_dist2;
        }
    }
    Some(std_max(
        1.000001 * max_pole_dist2.sqrt(),
        precision::confusion(),
    ))
}
