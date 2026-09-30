use super::{flag, real};
use rustycad_tkbo::int_tools::Tools;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{Axis1, Axis2d, Axis3, Direction3, Point3};
fn run(v: &[f64]) -> Result<(), Failure> {
    let curve = spatial(v[0], v[1])?;
    let pcurve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::new(Axis2d::default())).into_cast();
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomPlane::new(Axis3::default())).into_cast();
    let (mut distance, mut parameter) = (31., 32.);
    flag(Tools::compute_tolerance(
        &curve,
        &pcurve,
        &surface,
        v[2],
        v[3],
        &mut distance,
        &mut parameter,
        v[4],
        v[5] != 0.,
    )?);
    [distance, parameter].into_iter().for_each(real);
    Ok(())
}
pub fn case(v: &[f64]) {
    if let Err(error) = run(v) {
        print!("{} ", error.exception_type());
    }
}

fn spatial(y: f64, z: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(Axis1::new(
        Point3::new(0., y, z),
        Direction3::new(1., 0., 0.)?,
    )))
    .into_cast();
    Ok(curve)
}
