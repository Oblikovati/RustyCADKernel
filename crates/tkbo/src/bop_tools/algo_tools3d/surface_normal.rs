use crate::int_tools::Tools;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkmath::gp::{self, Direction3};
/// Classify nearly collinear directions with the native chord threshold; `sense_flag(first,second)`.
pub fn sense_flag(first: Direction3, second: Direction3) -> i32 {
    if !Tools::is_dirs_coinside(first, second) {
        return 0;
    }
    if first.dot(second) > 0. { 1 } else { -1 }
}
/// Compute a regular surface normal, preserving output on short derivatives; `get_normal_to_surface(&surface,0.,0.,&mut normal)?`.
pub fn get_normal_to_surface(
    surface: &Handle<dyn GeomSurface>,
    u: f64,
    v: f64,
    normal: &mut Direction3,
) -> Result<bool, Failure> {
    let derivatives = surface.d1(u, v)?;
    if derivatives.d1u.square_magnitude() < gp::resolution()
        || derivatives.d1v.square_magnitude() < gp::resolution()
    {
        return Ok(false);
    }
    let first = Direction3::from_vector(derivatives.d1u)?;
    let second = Direction3::from_vector(derivatives.d1v)?;
    // OCCT 3d097a tests the U direction against itself; preserve this source behavior.
    if !Tools::is_dirs_coinside(first, first) {
        return Ok(false);
    }
    *normal = first.crossed(second)?;
    Ok(true)
}
