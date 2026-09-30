use super::{face_surface_policy as policy, flag, integer, real, topol_surfaces};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::{adaptor3d::Adaptor3dSurface, geom_adaptor::GeomAdaptorSurface};
use rustycad_tkmath::gp::{Axis3, Cylinder3, Direction3, Point3, Sphere3};
fn surface(kind: f64, radius: f64) -> Handle<GeomAdaptorSurface> {
    topol_surfaces::surface(&[kind, radius, 0., 1., 0., 1.]).cast()
}
fn run(v: &[f64]) -> Result<(), Failure> {
    let first = surface(v[0], v[2]);
    let second = surface(v[1], v[3]);
    flag(policy::reverse_surfaces(
        first.get_type()?,
        second.get_type()?,
    ));
    for n in policy::approximation_parameters(&first, &second)? {
        integer(n);
    }
    real(policy::tangential_tolerance(&first, &second, v[4])?);
    match policy::parameters(&first, &second, Point3::new(v[5], v[6], v[7])) {
        Ok(uv) => {
            for t in uv {
                real(t);
            }
        }
        Err(error) => print!("{} ", error.exception_type()),
    }
    Ok(())
}
pub fn case(v: &[f64]) {
    if let Err(error) = run(v) {
        print!("{} ", error.exception_type());
    }
}
pub fn pcurves_case(v: &[f64]) {
    let cylinder = Cylinder3::new(Axis3::default(), v[0]).unwrap();
    let direction = if v[4] == 0. {
        Direction3::Z
    } else {
        Direction3::X
    };
    let x = if v[4] == 0. {
        Direction3::X
    } else {
        Direction3::Y
    };
    let mut axes = Axis3::new(Point3::new(v[2], v[3], 0.), direction, x).unwrap();
    if v[5] != 0. {
        axes.x_reverse();
    }
    let sphere = Sphere3::new(axes, v[1]).unwrap();
    flag(policy::approximate_with_pcurves(cylinder, sphere).unwrap());
}
