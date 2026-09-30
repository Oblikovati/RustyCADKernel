use super::{flag, integer, real};
use rustycad_tkbo::int_tools::{CommonPrt, Range, Tools};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Direction3, Line3, Plane3, Point3};
pub fn case(v: &[f64]) {
    let r = Range::new(v[0], v[1]);
    let s = Range::new(v[2], v[3]);
    flag(Tools::is_on_pave(v[4], &r, v[5]));
    flag(Tools::is_on_pave1(v[4], &r, v[5]));
    flag(Tools::is_in_range(&r, &s, v[5]));
    real(Tools::compute_int_range(v[0], v[1], v[6]));
    let a = Direction3::new(1., 0., 0.).unwrap();
    let b = Direction3::new(v[7], v[8], v[9]).unwrap();
    flag(Tools::is_dirs_coinside(a, b));
    flag(Tools::is_dirs_coinside_tolerance(a, b, v[5]));
    parameters(r, s, v[4], v[6]);
    if let Err(error) = segment(v) {
        print!("{} ", error.exception_type());
    }
}
fn segment(v: &[f64]) -> Result<(), Failure> {
    let direction = Direction3::new(v[7], v[8], v[9])?;
    let line = Line3::new(Point3::new(0., 0., v[4]), direction);
    let plane = Plane3::default();
    let mut point = Point3::new(31., 32., 33.);
    let (mut t, mut tol, mut lo, mut hi) = (34., 35., 36., 37.);
    integer(Tools::seg_pln(
        &line, v[0], v[1], v[5], &plane, v[6], &mut point, &mut t, &mut tol, &mut lo, &mut hi,
    )?);
    for x in [point.x(), point.y(), point.z(), t, tol, lo, hi] {
        real(x);
    }
    Ok(())
}

fn parameters(r: Range, s: Range, first: f64, last: f64) {
    let mut part = CommonPrt::default();
    part.set_range1(r);
    part.append_range2(s);
    part.set_vertex_parameter1(first);
    part.set_vertex_parameter2(last);
    let (mut a, mut b) = (31., 32.);
    Tools::vertex_parameter(&part, &mut a);
    real(a);
    Tools::vertex_parameters(&part, &mut a, &mut b).unwrap();
    real(a);
    real(b);
    part.change_ranges2().clear();
    a = 31.;
    b = 32.;
    let error = Tools::vertex_parameters(&part, &mut a, &mut b).unwrap_err();
    print!("{} ", error.exception_type());
    real(a);
    real(b);
}
