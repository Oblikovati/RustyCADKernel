use super::{flag, integer, real};
use rustycad_tkbo::int_tools::Curve;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::{
    geom::{GeomPlane, GeomSurface},
    geom_adaptor::GeomAdaptorSurface,
};
use rustycad_tkmath::gp::{Direction3, Point3};
fn surface(
    z: f64,
    dx: f64,
    dy: f64,
    dz: f64,
    size: f64,
) -> Result<Handle<GeomAdaptorSurface>, Failure> {
    let plane = GeomPlane::from_point(Point3::new(0., 0., z), Direction3::new(dx, dy, dz)?)?;
    let geometry: Handle<dyn GeomSurface> = Handle::new(plane).cast();
    Ok(Handle::new(GeomAdaptorSurface::with_range(
        &geometry,
        ((-size, size, -size, size), (0., 0.)),
    )?))
}
fn run(v: &[f64]) -> Result<(), Failure> {
    let surfaces = [
        surface(0., 0., 0., 1., v[4])?,
        surface(v[0], v[1], v[2], v[3], v[4])?,
    ];
    let pair = super::face_planes::PlaneIntersection {
        surfaces: &surfaces,
        face_tolerances: [v[5], v[6]],
        angular: 1e-8,
        tangential: v[5] + v[6],
        approximation: [v[7] != 0., v[8] != 0.],
    };
    let mut curves = vec![Curve::default()];
    let mut tangent = true;
    pair.perform(&mut curves, &mut tangent)?;
    flag(tangent);
    integer(curves.len() as i32);
    for curve in curves.iter().skip(1) {
        descriptor(curve)?;
    }
    Ok(())
}
fn descriptor(curve: &Curve) -> Result<(), Failure> {
    real(curve.tolerance());
    real(curve.tangential_tolerance());
    for t in [
        curve.curve().first_parameter(),
        curve.curve().last_parameter(),
    ] {
        real(t);
        let point = curve.curve().eval_d0(t)?;
        for c in [point.x(), point.y(), point.z()] {
            real(c);
        }
    }
    for pc in [curve.first_curve2d(), curve.second_curve2d()] {
        flag(pc.is_null());
        if !pc.is_null() {
            pcurve_descriptor(pc)?;
        }
    }
    Ok(())
}
pub fn case(v: &[f64]) {
    if let Err(error) = run(v) {
        print!("{} ", error.exception_type());
    }
}

fn pcurve_descriptor(
    curve: &Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>,
) -> Result<(), Failure> {
    for parameter in [curve.first_parameter(), curve.last_parameter()] {
        real(parameter);
        let point = curve.eval_d0(parameter)?;
        real(point.x());
        real(point.y());
    }
    Ok(())
}
