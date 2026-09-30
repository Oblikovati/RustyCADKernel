//! Complete Perform output compared with the unmodified TKBO shared library.
use super::{fclass_faces, flag, integer, real};
use rustycad_tkbo::int_tools::{Curve, FaceFace};
use rustycad_tkernel::standard::Failure;
pub fn case(values: &[f64]) {
    if let Err(error) = perform(values) {
        print!("{} ", error.exception_type());
    }
}
fn perform(v: &[f64]) -> Result<(), Failure> {
    let first = fclass_faces::face(v[0] as i32, 3.)?;
    let second = fclass_faces::face(v[1] as i32, 2.)?;
    let mut ff = FaceFace::default();
    ff.set_parameters(v[2] != 0., v[3] != 0., v[4] != 0., 1e-7);
    ff.perform(&first, &second, v[5] != 0.)?;
    flag(ff.is_done());
    flag(ff.tangent_faces());
    integer(ff.points().len() as i32);
    integer(ff.lines()?.len() as i32);
    for curve in ff.lines()? {
        curve_result(curve)?;
    }
    ff.prepare_lines_3d(v[6] != 0.)?;
    integer(ff.lines()?.len() as i32);
    Ok(())
}
fn curve_result(curve: &Curve) -> Result<(), Failure> {
    real(curve.tolerance());
    real(curve.tangential_tolerance());
    flag(curve.first_curve2d().is_null());
    flag(curve.second_curve2d().is_null());
    for parameter in [
        curve.curve().first_parameter(),
        curve.curve().last_parameter(),
    ] {
        real(parameter);
        let point = curve.curve().eval_d0(parameter)?;
        for coordinate in point.coordinates() {
            real(*coordinate);
        }
    }
    Ok(())
}
