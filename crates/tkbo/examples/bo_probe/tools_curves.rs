use super::{flag, integer, real, tools_geometry};
use rustycad_tkbo::int_tools::{Curve, Tools};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkmath::{bnd::BoundingBox3, gp::Axis2d};
fn descriptor(curve: &Curve) {
    flag(curve.has_bounds());
    real(curve.curve().first_parameter());
    real(curve.curve().last_parameter());
    real(curve.tolerance());
    real(curve.tangential_tolerance());
    for pc in [curve.first_curve2d(), curve.second_curve2d()] {
        flag(pc.is_null());
        if !pc.is_null() {
            real(pc.first_parameter());
            real(pc.last_parameter());
        }
    }
}
fn run(v: &[f64]) -> Result<(), Failure> {
    let geom = tools_geometry::geometry(v[0] as i32, v[1], v[2])?;
    flag(Tools::is_closed(&geom)?);
    real(Tools::curve_tolerance(&geom, v[3])?);
    let pc: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::default())).into_cast();
    let null = Handle::default();
    let curve = Curve::new(
        &geom,
        if v[4] == 0. { &null } else { &pc },
        &pc,
        v[3],
        v[3] * 2.,
    );
    let mut out = split_and_bounds(&curve)?;
    reject(v, curve, &mut out)?;
    Ok(())
}
pub fn case(v: &[f64]) {
    if let Err(error) = run(v) {
        print!("{} ", error.exception_type());
    }
}

fn split_and_bounds(curve: &Curve) -> Result<Vec<Curve>, Failure> {
    let mut out = vec![curve.clone()];
    integer(Tools::split_curve(curve, &mut out)?);
    integer(out.len() as i32);
    for item in &out {
        descriptor(item);
    }
    let mut bounds = BoundingBox3::default();
    flag(Tools::check_curve(curve, &mut bounds)?);
    for x in bounds.get()? {
        real(x);
    }
    Ok(out)
}

fn reject(v: &[f64], curve: Curve, out: &mut Vec<Curve>) -> Result<(), Failure> {
    let null = Handle::default();
    let other = tools_geometry::geometry(v[5] as i32, v[1], v[2])?;
    let mut input = vec![
        curve.clone(),
        Curve::new(&other, &null, &null, 0., 0.),
        curve,
    ];
    Tools::reject_lines(&mut input, out)?;
    integer(out.len() as i32);
    for item in input.iter().chain(out.iter()) {
        descriptor(item);
    }
    Ok(())
}
