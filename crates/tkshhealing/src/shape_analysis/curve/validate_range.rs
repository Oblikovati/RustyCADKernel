//! Native range repair, including in-place reversal (OCC966, PRO7747, PRO7656).
use super::ShapeAnalysisCurve;
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBoundedCurve, GeomCurve};
use rustycad_tkmath::elclib;

fn clamp_parameter(parameter: &mut f64, bounds: (f64, f64)) {
    if *parameter < bounds.0 {
        *parameter = bounds.0;
    } else if *parameter > bounds.1 {
        *parameter = bounds.1;
    }
}
fn repair_origin(first: &mut f64, last: &mut f64, bounds: (f64, f64)) -> bool {
    if (*last - bounds.0).abs() < precision::p_confusion() {
        *last = bounds.1;
        return true;
    }
    if (*first - bounds.1).abs() < precision::p_confusion() {
        *first = bounds.0;
        return true;
    }
    false
}
fn reverse_range(curve: &dyn GeomCurve, first: &mut f64, last: &mut f64) -> Result<(), Failure> {
    if *first > *last {
        *first = curve.reversed_parameter(*first)?;
        *last = curve.reversed_parameter(*last)?;
        curve.reverse()?;
    }
    Ok(())
}
fn repair_closed(
    curve: &dyn GeomCurve,
    first: &mut f64,
    last: &mut f64,
    bounds: (f64, f64),
    tolerance: f64,
) -> Result<(), Failure> {
    if repair_origin(first, last, bounds) {
        return Ok(());
    }
    // S4136: the 3D endpoint check deliberately supplements the parameter check.
    if curve.value(*first)?.distance(&curve.value(bounds.0)?) < tolerance {
        *first = bounds.0;
    }
    if curve.value(*last)?.distance(&curve.value(bounds.1)?) < tolerance {
        *last = bounds.1;
    }
    if *first > *last {
        std::mem::swap(first, last);
    }
    Ok(())
}
fn repair_spline(
    spline: &GeomBSplineCurve,
    first: &mut f64,
    last: &mut f64,
    bounds: (f64, f64),
    tolerance: f64,
) -> Result<bool, Failure> {
    if spline.start_point()?.distance(&spline.end_point()?) <= tolerance {
        if !repair_origin(first, last, bounds) {
            std::mem::swap(first, last);
        }
    } else {
        reverse_range(spline, first, last)?;
    }
    if *first == *last {
        *first = bounds.0;
        *last = bounds.1;
        return Ok(false);
    }
    Ok(true)
}
fn repair_nonperiodic(
    curve: &Handle<dyn GeomCurve>,
    first: &mut f64,
    last: &mut f64,
    bounds: (f64, f64),
    tolerance: f64,
) -> Result<bool, Failure> {
    if *first < *last {
        return Ok(true);
    }
    if curve.is_closed()? {
        repair_closed(&**curve, first, last, bounds, tolerance)?;
        return Ok(true);
    }
    if let Some(spline) = curve.cast::<GeomBSplineCurve>().get() {
        return repair_spline(spline, first, last, bounds, tolerance);
    }
    reverse_range(&**curve, first, last)?;
    if *first == *last {
        *first -= precision::p_confusion();
        *last += precision::p_confusion();
    }
    Ok(false)
}
impl ShapeAnalysisCurve {
    /// Repair endpoints and sometimes reverse the shared curve; `ShapeAnalysisCurve.validate_range(&curve,&mut first,&mut last,1e-7)?`.
    pub fn validate_range(
        &self,
        curve: &Handle<dyn GeomCurve>,
        first: &mut f64,
        last: &mut f64,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        let geometry = curve.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "ShapeAnalysis_Curve: null curve; expected live Geom_Curve",
            )
        })?;
        let bounds = (geometry.first_parameter(), geometry.last_parameter());
        if !curve.cast::<dyn GeomBoundedCurve>().is_null() && !geometry.is_closed()? {
            clamp_parameter(first, bounds);
            clamp_parameter(last, bounds);
        }
        if Self::is_periodic(curve)? {
            (*first, *last) = elclib::adjust_periodic(
                bounds.0,
                bounds.1,
                precision::p_confusion(),
                *first,
                *last,
            );
            return Ok(true);
        }
        repair_nonperiodic(curve, first, last, bounds, tolerance)
    }
}
