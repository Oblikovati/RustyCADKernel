//! OCCT's type-dependent 3D distribution, including sampling wrapper bases.
use super::ShapeAnalysisCurve;
use rustycad_tkernel::{
    ncollection::Sequence,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::{
    adaptor3d::Adaptor3dCurve,
    geom::{
        GeomBSplineCurve, GeomBezierCurve, GeomCircle, GeomCurve, GeomLine, GeomOffsetCurve,
        GeomTrimmedCurve,
    },
    geom_adaptor::GeomAdaptorCurve,
};
use rustycad_tkmath::gp::Point3;

fn sample_multiplier(range: (f64, f64), delta: f64) -> i32 {
    let factor = ((range.1 - range.0) / delta).ceil();
    // Match the pinned x86-64 GCC conversion, including integer-indefinite output.
    if !(-2147483648.0..2147483648.0).contains(&factor) {
        return i32::MIN;
    }
    factor as i32
}
fn curve_sample_count(curve: &Handle<dyn GeomCurve>, factor: i32) -> i32 {
    if !curve.cast::<GeomLine>().is_null() {
        return 2;
    }
    if !curve.cast::<GeomCircle>().is_null() {
        return 360_i32.wrapping_mul(factor);
    }
    if let Some(spline) = curve.cast::<GeomBSplineCurve>().get() {
        return spline
            .nb_knots()
            .wrapping_mul(spline.degree())
            .wrapping_mul(factor)
            .max(2);
    }
    if let Some(bezier) = curve.cast::<GeomBezierCurve>().get() {
        return 3 + bezier.nb_poles();
    }
    100_i32.wrapping_mul(factor)
}
fn wrapper_basis(curve: &Handle<dyn GeomCurve>) -> Option<Handle<dyn GeomCurve>> {
    if let Some(offset) = curve.cast::<GeomOffsetCurve>().get() {
        return Some(offset.basis_curve());
    }
    curve
        .cast::<GeomTrimmedCurve>()
        .get()
        .map(GeomTrimmedCurve::basis_curve)
}
fn append_samples(
    curve: &Handle<dyn GeomCurve>,
    range: (f64, f64),
    count: i32,
    points: &mut Sequence<Point3>,
) -> Result<(), Failure> {
    let adaptor = GeomAdaptorCurve::with_curve(curve)?;
    let step = (range.1 - range.0) / f64::from(count.wrapping_sub(1));
    for i in 0..count.wrapping_sub(1) {
        points.append(adaptor.value(range.0 + step * f64::from(i))?);
    }
    points.append(adaptor.value(range.1)?);
    Ok(())
}
impl ShapeAnalysisCurve {
    /// Append the original 3D distribution; `ShapeAnalysisCurve::get_sample_points(&curve,0.0,1.0,&mut points)?`.
    pub fn get_sample_points(
        curve: &Handle<dyn GeomCurve>,
        first: f64,
        last: f64,
        points: &mut Sequence<Point3>,
    ) -> Result<bool, Failure> {
        let geometry = curve.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "ShapeAnalysis_Curve: null curve; expected live Geom_Curve",
            )
        })?;
        let delta = geometry.last_parameter() - geometry.first_parameter();
        if delta == 0.0 {
            return Ok(false);
        }
        let factor = sample_multiplier((first, last), delta);
        if let Some(basis) = wrapper_basis(curve) {
            return Self::get_sample_points(&basis, first, last, points);
        }
        append_samples(
            curve,
            (first, last),
            curve_sample_count(curve, factor),
            points,
        )?;
        Ok(true)
    }
}
#[cfg(test)]
mod tests {
    use super::sample_multiplier;
    #[test]
    fn sample_multiplier_matches_ceiling_and_native_integer_indefinite() {
        assert_eq!(sample_multiplier((0.0, 1.1), 1.0), 2);
        assert_eq!(sample_multiplier((1.0, 0.0), 1.0), -1);
        assert_eq!(sample_multiplier((0.0, f64::INFINITY), 1.0), i32::MIN);
        assert_eq!(sample_multiplier((0.0, f64::NAN), 1.0), i32::MIN);
    }
}
