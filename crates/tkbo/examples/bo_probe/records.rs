use super::{flag, integer, real};
use rustycad_tkbo::bop_ds::{IndexRange, Pair, Pave, Point};
use rustycad_tkmath::gp::{Point2, Point3};
fn index_state(value: &IndexRange, query: i32) {
    integer(value.first());
    integer(value.last());
    let (first, last) = value.indices();
    integer(first);
    integer(last);
    flag(value.contains(query));
    let mut text = String::new();
    value.dump(&mut text).unwrap();
    print!("{text} ");
}
pub fn index_case(v: &[f64]) {
    index_state(&IndexRange::default(), v[4] as i32);
    let mut value = IndexRange::new(v[0] as i32, v[1] as i32);
    index_state(&value, v[4] as i32);
    value.set_first(v[2] as i32);
    value.set_last(v[3] as i32);
    index_state(&value, v[4] as i32);
    value.set_indices(v[1] as i32, v[0] as i32);
    index_state(&value, v[4] as i32);
}
fn pave_state(value: &Pave) {
    integer(value.index());
    real(value.parameter());
    let (index, parameter) = value.contents();
    integer(index);
    real(parameter);
    print!("{} ", value.hash_code());
    let mut text = String::new();
    value.dump(&mut text).unwrap();
    print!("{text} ");
}
pub fn pave_case(v: &[f64]) {
    pave_state(&Pave::default());
    let mut a = Pave::new(v[0] as i32, v[1]);
    let b = Pave::new(v[2] as i32, v[3]);
    pave_state(&a);
    pave_state(&b);
    flag(a.is_equal(&b));
    flag(a == b);
    flag(a.is_less(&b));
    flag(b.is_less(&a));
    a.set_index(v[2] as i32);
    a.set_parameter(v[3]);
    pave_state(&a);
    flag(a.is_equal(&b));
}
fn pair_state(value: &Pair) {
    let (a, b) = value.indices();
    integer(a);
    integer(b);
    print!("{} ", value.hash_code());
}
pub fn pair_case(v: &[f64]) {
    pair_state(&Pair::default());
    let mut a = Pair::new(v[0] as i32, v[1] as i32);
    let b = Pair::new(v[2] as i32, v[3] as i32);
    pair_state(&a);
    pair_state(&b);
    flag(a.is_equal(&b));
    flag(a == b);
    flag(a.is_less(&b));
    flag(b.is_less(&a));
    a.set_indices(v[3] as i32, v[2] as i32);
    pair_state(&a);
    flag(a == b);
}
fn point_state(value: &Point) {
    for coordinate in [
        value.pnt().x(),
        value.pnt().y(),
        value.pnt().z(),
        value.pnt2d1().x(),
        value.pnt2d1().y(),
        value.pnt2d2().x(),
        value.pnt2d2().y(),
    ] {
        real(coordinate);
    }
    integer(value.index());
}
pub fn point_case(v: &[f64]) {
    let mut value = Point::default();
    point_state(&value);
    value.set_pnt(Point3::new(v[0], v[1], v[2]));
    value.set_pnt2d1(Point2::new(v[3], v[4]));
    value.set_pnt2d2(Point2::new(v[5], v[6]));
    value.set_index(v[7] as i32);
    point_state(&value);
}
