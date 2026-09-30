use super::{ShapeConstructCurve, null_curve, segment_bounds};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCurve, Geom2dGeometry, Geom2dLine,
    Geom2dTrimmedCurve,
};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeombase::{approx::Curve2d as ApproxCurve, geom2d_convert};
use rustycad_tkmath::{convert::ParameterisationType, geom_abs::Shape};

impl ShapeConstructCurve {
    /// `ConvertToBSpline(C2d, first, last, prec)`: [`Self::convert_to_bspline`] for a
    /// p-curve (`Approx_Curve2d(first, last, prec, prec, C1)`, degree 9, 1000 segments);
    /// `ShapeConstructCurve.convert_to_bspline_2d(&c, 0.0, 1.0, 1e-7)?`.
    pub fn convert_to_bspline_2d(
        &self,
        c: &Handle<dyn Geom2dCurve>,
        first: f64,
        last: f64,
        prec: f64,
    ) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
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
    curve: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    let spline = curve.cast::<Geom2dBSplineCurve>();
    if !spline.is_null() {
        return Ok(spline);
    }
    if curve.cast::<Geom2dBezierCurve>().is_null() && curve.cast::<Geom2dLine>().is_null() {
        return Ok(spline);
    }
    // The trimmed curve is built outside the native `try`: its failure propagates.
    let trimmed: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dTrimmedCurve::new(curve, first, last, true, true)?).cast();
    Ok(
        geom2d_convert::curve_to_bspline_curve(&trimmed, ParameterisationType::TgtThetaOver2)
            .unwrap_or_default(),
    )
}

/// `bspl->Copy()` then `Segment(f, l)` on the copy.
fn segment_copy(
    spline: &mut Handle<Geom2dBSplineCurve>,
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

/// `Approx_Curve2d(new Geom2dAdaptor_Curve(newc, first, last), first, last, prec, prec,
/// GeomAbs_C1, 9, 1000)`: its curve when it is done or has a result.
fn approximate(
    newc: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
    prec: f64,
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    let adaptor: Handle<dyn Adaptor2dCurve2d> =
        Handle::new(Geom2dAdaptorCurve::with_range(newc, first, last)?).cast();
    let conv = ApproxCurve::new(&adaptor, first, last, prec, prec, Shape::C1, 9, 1000)?;
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
        let axis = rustycad_tkmath::gp::Axis2d::default();
        let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(axis)).cast();
        let mut spline = ShapeConstructCurve
            .convert_to_bspline_2d(&line, 0., 1., 1e-7)
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
