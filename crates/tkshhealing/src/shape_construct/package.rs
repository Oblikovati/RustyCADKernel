// Rust translation of ShapeConstruct/ShapeConstruct.cxx (ConvertCurveToBSpline,
// ConvertSurfaceToBSpline) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The package functions of `ShapeConstruct`: B-spline conversion of curves and surfaces
//! and the p-curve join of `JoinPCurves` (in `join_pcurves.rs`).
mod join_pcurves;
mod surface_bspline;
pub use surface_bspline::{ShapeConstructApproxSettings, ShapeConstructSurfaceBounds};

use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dConic, Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomConic, GeomCurve, GeomTrimmedCurve};
use rustycad_tkgeombase::{geom_convert, geom2d_convert};
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::geom_abs::Shape as Continuity;

/// The package `ShapeConstruct` (static functions only).
///
/// ```
/// use rustycad_tkshhealing::shape_construct::ShapeConstruct;
/// let _ = ShapeConstruct;
/// ```
pub struct ShapeConstruct;

impl ShapeConstruct {
    /// `ConvertCurveToBSpline(C3D, First, Last, Tol3d, Continuity, MaxSegments, MaxDegree)`:
    /// the curve itself when it is a B-spline, else the approximation of the curve trimmed
    /// to `[first, last]` (degree at most 6 for a conic); when the approximation has no
    /// result or raises, the exact conversion of the untrimmed curve.
    /// Example: `ShapeConstruct::convert_curve_to_bspline(&c, 0.0, 1.0, 1e-4, Continuity::C1, 9, 9)?`.
    pub fn convert_curve_to_bspline(
        c3d: &Handle<dyn GeomCurve>,
        first: f64,
        last: f64,
        tol3d: f64,
        continuity: Continuity,
        max_segments: i32,
        max_degree: i32,
    ) -> Result<Handle<GeomBSplineCurve>, Failure> {
        let spline = c3d.cast::<GeomBSplineCurve>();
        if !spline.is_null() {
            return Ok(spline);
        }
        let mut max_deg = max_degree;
        if !c3d.cast::<dyn GeomConic>().is_null() {
            max_deg = max_deg.min(6);
        }
        // Protection against parabolas: the trimmed curve is built outside the native try.
        let trimmed: Handle<dyn GeomCurve> =
            Handle::new(GeomTrimmedCurve::new(c3d, first, last, true, true)?).cast();
        let approx = geom_convert::ApproxCurve::from_curve(
            &trimmed,
            tol3d,
            continuity,
            max_segments,
            max_deg,
        );
        match approx {
            Ok(approx) if approx.has_result() => Ok(approx.curve()),
            // The native catch retries the same exact conversion, so its failure propagates.
            _ => geom_convert::curve_to_bspline_curve(c3d, ParameterisationType::QuasiAngular),
        }
    }

    /// `ConvertCurveToBSpline(C2D, ...)`: a conic is trimmed to `[first, last]` and
    /// approximated (the exact conversion of the trimmed conic when the approximation has
    /// no result); another non-B-spline curve is converted exactly; a B-spline is returned
    /// as is. Native catches nothing here, so every failure propagates.
    /// Example: `ShapeConstruct::convert_curve2d_to_bspline(&c, 0.0, 1.0, 1e-6, Continuity::C1, 9, 9)?`.
    pub fn convert_curve2d_to_bspline(
        c2d: &Handle<dyn Geom2dCurve>,
        first: f64,
        last: f64,
        tol2d: f64,
        continuity: Continuity,
        max_segments: i32,
        max_degree: i32,
    ) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
        if !c2d.cast::<dyn Geom2dConic>().is_null() {
            let trimmed: Handle<dyn Geom2dCurve> =
                Handle::new(Geom2dTrimmedCurve::new(c2d, first, last, true, true)?).cast();
            let approx = geom2d_convert::ApproxCurve::from_curve(
                &trimmed,
                tol2d,
                continuity,
                max_segments,
                max_degree,
            )?;
            if approx.has_result() {
                return Ok(approx.curve());
            }
            return geom2d_convert::curve_to_bspline_curve(
                &trimmed,
                ParameterisationType::QuasiAngular,
            );
        }
        let spline = c2d.cast::<Geom2dBSplineCurve>();
        if !spline.is_null() {
            return Ok(spline);
        }
        geom2d_convert::curve_to_bspline_curve(c2d, ParameterisationType::QuasiAngular)
    }
}

#[cfg(test)]
#[path = "package/tests.rs"]
mod tests;
