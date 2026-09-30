use super::{ShapeConstructCurve, null_curve, segment_bounds};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCurve, GeomGeometry, GeomLine, GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeombase::{approx::Curve3d as ApproxCurve, geom_convert};
use rustycad_tkmath::{convert::ParameterisationType, geom_abs::Shape};

impl ShapeConstructCurve {
    /// `ConvertToBSpline(C, first, last, prec)`: a B-spline (the curve itself when it is
    /// one, else its B-spline form for a Bezier curve or a line, trimmed to
    /// `[first, last]`), segmented to `[first, last]` on a copy when its range is wider;
    /// conics, other curves and failed conversions are approximated
    /// (`Approx_Curve3d(prec, C1)`, natively with at most 9 segments of degree 1000). A null
    /// handle when everything fails; `ShapeConstructCurve.convert_to_bspline(&c, 0.0, 1.0, 1e-7)?`.
    pub fn convert_to_bspline(
        &self,
        c: &Handle<dyn GeomCurve>,
        first: f64,
        last: f64,
        prec: f64,
    ) -> Result<Handle<GeomBSplineCurve>, Failure> {
        c.get().ok_or_else(|| null_curve("ConvertToBSpline"))?;
        let mut spline = initial_bspline(c, first, last)?;
        if !spline.is_null() && segment_copy(&mut spline, first, last).is_ok() {
            return Ok(spline);
        }
        let fallback = if spline.is_null() {
            c.clone()
        } else {
            spline.cast()
        };
        Ok(approximate(&fallback, first, last, prec).unwrap_or_default())
    }
}

fn initial_bspline(
    curve: &Handle<dyn GeomCurve>,
    first: f64,
    last: f64,
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let spline = curve.cast::<GeomBSplineCurve>();
    if !spline.is_null() {
        return Ok(spline);
    }
    if curve.cast::<GeomBezierCurve>().is_null() && curve.cast::<GeomLine>().is_null() {
        return Ok(spline);
    }
    // The trimmed curve is built outside the native `try`: its failure propagates.
    let trimmed: Handle<dyn GeomCurve> =
        Handle::new(GeomTrimmedCurve::new(curve, first, last, true, true)?).cast();
    Ok(
        geom_convert::curve_to_bspline_curve(&trimmed, ParameterisationType::TgtThetaOver2)
            .unwrap_or_default(),
    )
}

/// `bspl->Copy()` then `Segment(f, l)` on the copy.
fn segment_copy(
    spline: &mut Handle<GeomBSplineCurve>,
    first: f64,
    last: f64,
) -> Result<(), Failure> {
    let bounds = segment_bounds(
        first,
        last,
        spline.first_parameter(),
        spline.last_parameter(),
    );
    let Some((first, last)) = bounds else {
        return Ok(());
    };
    // OCCT 3d097a0 assigns the copy before Segment: fallback must retain it even on failure.
    *spline = spline.copy()?.cast();
    spline.segment(first, last, precision::p_confusion())
}

/// `Approx_Curve3d(new GeomAdaptor_Curve(newc, first, last), prec, GeomAbs_C1, 9, 1000)`:
/// its curve when it is done or has a result.
fn approximate(
    newc: &Handle<dyn GeomCurve>,
    first: f64,
    last: f64,
    prec: f64,
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let adaptor: Handle<dyn Adaptor3dCurve> =
        Handle::new(GeomAdaptorCurve::with_range(newc, first, last)?).cast();
    let conv = ApproxCurve::new(&adaptor, prec, Shape::C1, 9, 1000)?;
    if conv.is_done() || conv.has_result() {
        return Ok(conv.curve());
    }
    Ok(Handle::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn segmentation_failure_retains_the_assigned_copy_for_fallback() {
        let axis = rustycad_tkmath::gp::Axis1::default();
        let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(axis)).cast();
        let mut spline = ShapeConstructCurve
            .convert_to_bspline(&line, 0., 1., 1e-7)
            .unwrap();
        let original = spline.clone();
        assert!(segment_copy(&mut spline, 0.75, 0.25).is_err());
        assert_ne!(spline.object_address(), original.object_address());
        assert_eq!(
            (original.first_parameter(), original.last_parameter()),
            (0., 1.)
        );
    }
}
