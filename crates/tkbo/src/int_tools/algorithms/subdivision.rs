//! Stage subdivision values before resizing the caller's array, matching native failures.
use super::IntTools;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkernel::{ncollection::Array1, standard::Failure};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeombase::gcpnts::QuasiUniformDeflection;
use rustycad_tkmath::geom_abs::{CurveType, Shape};
impl IntTools {
    /// Samples with relative circular deflection; native bounds are maximum then minimum; `IntTools::prepare_args(&curve,1.,0.,10,0.01,&mut args)?`.
    pub fn prepare_args(
        curve: &BRepAdaptorCurve,
        maximum: f64,
        minimum: f64,
        count: i32,
        deflection: f64,
        arguments: &mut Array1<'_, f64>,
    ) -> Result<i32, Failure> {
        let kind = curve.get_type()?;
        let step = (maximum - minimum) / f64::from(count);
        let mut parameters = Vec::new();
        for index in 1..=count {
            let current = minimum + f64::from(index - 1) * step;
            parameters.push(current);
            let next = if index == count {
                maximum
            } else {
                current + step
            };
            if step > 1e-5 && supports_radius(kind) {
                let status =
                    append_deflection(curve, (current, next), deflection, &mut parameters)?;
                if status != 0 {
                    return Ok(status);
                }
            }
        }
        parameters.push(maximum);
        arguments
            .resize_bounds(0, parameters.len() as i32 - 1, false)
            .map_err(|error| error.into_failure("IntTools::PrepareArgs"))?;
        arguments.as_mut_slice().copy_from_slice(&parameters);
        Ok(0)
    }
}
fn supports_radius(kind: CurveType) -> bool {
    !matches!(
        kind,
        CurveType::BSplineCurve
            | CurveType::BezierCurve
            | CurveType::OffsetCurve
            | CurveType::Ellipse
            | CurveType::OtherCurve
    )
}
fn append_deflection(
    curve: &BRepAdaptorCurve,
    range: (f64, f64),
    deflection: f64,
    parameters: &mut Vec<f64>,
) -> Result<i32, Failure> {
    let mut radius = 0.;
    let status = IntTools::get_radius(curve, range.0, range.1, &mut radius)?;
    if status < 0 {
        return Ok(1);
    }
    if status != 0 {
        return Ok(0);
    }
    let mut distribution = QuasiUniformDeflection::new();
    distribution.initialize_range(
        curve as &dyn Adaptor3dCurve,
        deflection * radius,
        range.0,
        range.1,
        Shape::C1,
    )?;
    if !distribution.is_done() {
        return Ok(2);
    }
    for index in 2..distribution.nb_points()? {
        parameters.push(distribution.parameter(index)?);
    }
    Ok(0)
}
