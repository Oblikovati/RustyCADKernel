//! Geometric closure and unwrapped periodicity (OCCT OCC966).
use super::ShapeAnalysisCurve;
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dOffsetCurve, Geom2dTrimmedCurve, native::std_max};
use rustycad_tkg3d::geom::{GeomCurve, GeomOffsetCurve, GeomTrimmedCurve};

fn null_analysis_curve() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "ShapeAnalysis_Curve: null curve; expected live curve",
    )
}
impl ShapeAnalysisCurve {
    /// Compare endpoints with the native tolerance floor; `ShapeAnalysisCurve::is_closed(&curve,1e-7)?`.
    pub fn is_closed(curve: &Handle<dyn GeomCurve>, tolerance: f64) -> Result<bool, Failure> {
        let geometry = curve.get().ok_or_else(null_analysis_curve)?;
        if geometry.is_closed()? {
            return Ok(true);
        }
        let tolerance = std_max(tolerance, precision::confusion());
        let first = geometry.first_parameter();
        let last = geometry.last_parameter();
        if precision::is_infinite(first) || precision::is_infinite(last) {
            return Ok(false);
        }
        Ok(geometry
            .value(first)?
            .square_distance(&geometry.value(last)?)
            <= tolerance * tolerance)
    }
    /// Inspect the underlying curve's periodicity (OCC966); `ShapeAnalysisCurve::is_periodic(&curve)?`.
    pub fn is_periodic(curve: &Handle<dyn GeomCurve>) -> Result<bool, Failure> {
        let mut basis = curve.clone();
        loop {
            if let Some(offset) = basis.cast::<GeomOffsetCurve>().get() {
                basis = offset.basis_curve();
                continue;
            }
            if let Some(trimmed) = basis.cast::<GeomTrimmedCurve>().get() {
                basis = trimmed.basis_curve();
                continue;
            }
            return Ok(basis.get().ok_or_else(null_analysis_curve)?.is_periodic());
        }
    }
    /// Inspect the underlying p-curve's periodicity (OCC966); `ShapeAnalysisCurve::is_periodic_2d(&curve)?`.
    pub fn is_periodic_2d(curve: &Handle<dyn Geom2dCurve>) -> Result<bool, Failure> {
        let mut basis = curve.clone();
        loop {
            if let Some(offset) = basis.cast::<Geom2dOffsetCurve>().get() {
                basis = offset.basis_curve();
                continue;
            }
            if let Some(trimmed) = basis.cast::<Geom2dTrimmedCurve>().get() {
                basis = trimmed.basis_curve();
                continue;
            }
            return Ok(basis.get().ok_or_else(null_analysis_curve)?.is_periodic());
        }
    }
}
