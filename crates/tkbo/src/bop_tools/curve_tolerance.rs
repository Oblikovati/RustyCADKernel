use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::standard::Failure;
use rustycad_tktopalgo::brep_lib::BRepLibCheckCurveOnSurface;
/// Measure edge/surface discrepancy, retaining output values on an unsuccessful check; `compute_curve_tolerance(&face,&edge,&mut distance,&mut parameter)?`.
pub fn compute_curve_tolerance(
    face: &Face,
    edge: &Edge,
    distance: &mut f64,
    parameter: &mut f64,
) -> Result<bool, Failure> {
    let mut check = BRepLibCheckCurveOnSurface::new();
    check.init(edge, face)?;
    check.perform();
    if !check.is_done() {
        return Ok(false);
    }
    *distance = check.max_distance();
    *parameter = check.max_parameter();
    Ok(true)
}
