use super::{integer, real};
use crate::bean_line_plane;
use rustycad_tkbo::int_tools::Range;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Direction3, Line3, Plane3, Point3};
pub(super) fn run(values: &[f64]) -> Result<(), Failure> {
    let line = Line3::new(
        Point3::new(values[0], values[1], values[2]),
        Direction3::new(values[3], values[4], values[5])?,
    );
    let mut plane = Plane3::default();
    plane.set_location(Point3::new(0., 0., values[6]));
    if values[7] != 0. {
        plane.u_reverse();
    }
    let range = Range::new(values[8], values[9]);
    let bounds = [values[10], values[11], values[12], values[13]];
    let result = bean_line_plane::intersect_line_plane(
        line,
        plane,
        range,
        bounds,
        [values[14], values[15]],
    )?;
    integer(1);
    real(f64::MAX);
    integer(i32::from(result.is_some()));
    if let Some(range) = result {
        real(range.first());
        real(range.last());
    }
    Ok(())
}
