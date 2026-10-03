//! Planar gp oracle counterpart; prints the same token stream as
//! tools/oracle/tkmath_gp_planar_probe.cxx.
use crate::elementary_probe::{Fields, attempt, end, flag, real};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Axis2d, Direction2, Matrix2, Xy};
mod geometry;
mod transforms;
pub use geometry::geom_case;
pub use transforms::{gtrsf_case, trsf_case};

pub fn pair(value: Xy) {
    real(value.x());
    real(value.y());
}
pub fn mat(value: Matrix2) {
    value
        .coefficients()
        .iter()
        .flatten()
        .copied()
        .for_each(real);
}
pub fn dir(value: Direction2) {
    pair(value.xy());
}
pub fn ax(value: Axis2d) {
    pair(value.location().xy());
    dir(value.direction());
}
pub fn read_mat(fields: &mut Fields) -> Matrix2 {
    let (a, b, c, d) = (fields.real(), fields.real(), fields.real(), fields.real());
    Matrix2::new([[a, b], [c, d]])
}
/// Prints a fallible value or the raised exception type.
pub fn show<T>(value: Result<T, Failure>, print: impl FnOnce(T)) {
    attempt(|| {
        print(value?);
        Ok(())
    });
}

pub fn xy_case(values: &[f64]) {
    let mut fields = Fields::new(values);
    let (a, b) = (fields.xy(), fields.xy());
    let (s, t) = (fields.real(), fields.real());
    let m = read_mat(&mut fields);
    let index = fields.int();
    xy_metrics(a, b, s);
    xy_products(a, b, s, m);
    xy_forms(a, b, s, t);
    let mut c = a;
    attempt(|| c.set_coord(index, s));
    pair(c);
    show(a.coord(index), real);
    end();
}
fn xy_metrics(a: Xy, b: Xy, s: f64) {
    real(a.modulus());
    real(a.square_modulus());
    flag(a.is_equal(&b, s));
    real(a.crossed(b));
    real(a ^ b);
    real(a.cross_magnitude(b));
    real(a.cross_square_magnitude(b));
    real(a.dot(b));
    real(a * b);
}
fn xy_products(a: Xy, b: Xy, s: f64, m: Matrix2) {
    let values = [
        a + b,
        a - b,
        a * s,
        s * a,
        a.multiplied_components(b),
        a.multiplied_matrix(m),
        m * a,
        a / s,
        a.reversed(),
        -a,
    ];
    values.into_iter().for_each(pair);
    let mut c = a;
    c.multiply_matrix(m);
    pair(c);
    c = a;
    c += b;
    pair(c);
    c -= b;
    pair(c);
    c = a;
    c *= b;
    pair(c);
    c = a;
    c /= s;
    pair(c);
}
fn xy_forms(a: Xy, b: Xy, s: f64, t: f64) {
    show(a.normalized(), pair);
    let mut c = a;
    attempt(|| c.normalize());
    pair(c);
    c.set_linear_form(s, a, t, b);
    pair(c);
    c.set_linear_form_offset(s, a, t, b, a);
    pair(c);
    c.set_scaled_sum(s, a, b);
    pair(c);
    c.set_sum(a, b);
    pair(c);
}

pub fn mat_case(values: &[f64]) {
    let mut fields = Fields::new(values);
    let (m, o) = (read_mat(&mut fields), read_mat(&mut fields));
    let s = fields.real();
    let (n, index) = (fields.int(), fields.int());
    real(m.determinant());
    flag(m.is_singular());
    pair(m.diagonal());
    pair(m.column(index).unwrap());
    pair(m.row(index).unwrap());
    mat_products(m, o, s, n);
    mat_setters(m, s, index);
    mat_in_place(m, o, s, n);
    show(m.value(index, 1), real);
    let mut c = m;
    attempt(|| c.set_value(1, index, s));
    mat(c);
    end();
}
fn mat_products(m: Matrix2, o: Matrix2, s: f64, n: i32) {
    mat(m.transposed());
    mat(m.inverted().unwrap());
    mat(m.powered(n).unwrap());
    mat(m.multiplied(o));
    mat(m * o);
    let mut c = m;
    c.pre_multiply(o);
    mat(c);
    [m.added(o), m - o, m.multiplied_scalar(s), s * m, m / s]
        .into_iter()
        .for_each(mat);
}
fn mat_setters(m: Matrix2, s: f64, index: i32) {
    let mut c = m;
    c.set_column(index, Xy::new(s, -s)).unwrap();
    mat(c);
    c = m;
    c.set_row(index, Xy::new(s, -s)).unwrap();
    mat(c);
    c = m;
    c.set_rotation(s);
    mat(c);
    c = m;
    c.set_scale(s);
    mat(c);
    c = m;
    c.set_diagonal(s, -s);
    mat(c);
    let (first, second) = (m.row(1).unwrap(), m.row(2).unwrap());
    mat(Matrix2::from_columns(first, second));
    c.set_columns(first, second);
    mat(c);
    c = m;
    c.transpose();
    mat(c);
}
fn mat_in_place(m: Matrix2, o: Matrix2, s: f64, n: i32) {
    let mut c = m;
    c.power(n).unwrap();
    mat(c);
    c = m;
    c *= s;
    mat(c);
    c = m;
    c += o;
    mat(c);
    c = m;
    c -= o;
    mat(c);
    c = m;
    c.multiply(o);
    mat(c);
    c = m;
    c /= s;
    mat(c);
}
