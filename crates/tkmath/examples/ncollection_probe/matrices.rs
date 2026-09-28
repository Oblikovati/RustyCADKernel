//! Matrix requests for float and double elements.
use super::{Printed, flag, json, vec3, vec4};
use rustycad_tkernel::ncollection::{Mat3, Mat4, Vec4};

/// `ncv_matrix_common`: products, scalar and element-wise operations, inversion, predicates,
/// row/column access and the JSON dump, in native print order.
macro_rules! matrix_common {
    ($a:expr, $b:expr, $v:expr, $s:expr, $matrix:ident, $t:ty, $out:ident, $vout:ident) => {{
        let (a, b, v, s) = ($a, $b, $v, $s);
        [a * b, $matrix::product(&a, &b), a.multiplied(&b)]
            .into_iter()
            .for_each($out);
        let mut c = a;
        c.multiply(&b);
        $out(c);
        c = a;
        c *= b;
        $out(c);
        $vout(a * v);
        [a * s, a.multiplied_scalar(s)].into_iter().for_each($out);
        c = a;
        c.multiply_scalar(s);
        $out(c);
        c = a;
        c *= s;
        $out(c);
        [a / s, a.divided(s)].into_iter().for_each($out);
        c = a;
        c.divide(s);
        $out(c);
        c = a;
        c /= s;
        $out(c);
        [a + b, a.added(&b), a - b, a.subtracted(&b)]
            .into_iter()
            .for_each($out);
        c = a;
        c += b;
        $out(c);
        c = a;
        c -= b;
        $out(c);
        [-a, a.negated(), a.transposed()].into_iter().for_each($out);
        c = a;
        c.transpose();
        $out(c);
        $out(a.adjoint());
        let mut inverse = b;
        let mut determinant = 91.0;
        flag(a.inverted_into(&mut inverse, &mut determinant));
        Printed::print(determinant);
        $out(inverse);
        inverse = b;
        flag(a.inverted_to(&mut inverse));
        $out(inverse);
        match a.inverted() {
            Ok(value) => $out(value),
            Err(error) => print!("{} ", error.exception_type()),
        }
        [a.is_zero(), a.is_identity(), a.is_equal(&b), a == a, a != b]
            .into_iter()
            .for_each(flag);
        flag($matrix::<$t>::zero().is_zero());
        flag($matrix::<$t>::identity().is_identity());
        [a.get_row(1), a.get_column(2), a.get_diagonal()]
            .into_iter()
            .for_each($vout);
        c = a;
        c.set_row(0, v);
        c.set_column(1, v);
        c.set_diagonal(v);
        $out(c);
        c = a;
        c.init_zero();
        $out(c);
        c.init_identity();
        $out(c);
        json(|d| a.dump_json(d));
    }};
}
macro_rules! typed_matrices {
    ($module:ident, $t:ty) => {
        mod $module {
            use super::*;
            pub fn request(raw: &[f64]) {
                let a = Mat4::<$t>::from_data(std::array::from_fn(|i| raw[i] as $t));
                let b = Mat4::<$t>::from_data(std::array::from_fn(|i| raw[16 + i] as $t));
                let v = Vec4::<$t>::from_array(std::array::from_fn(|i| raw[32 + i] as $t));
                let s = raw[36] as $t;
                matrix_common!(a, b, v, s, Mat4, $t, mat4, vec4);
                Printed::print(a.determinant_mat3());
                mat3(a.get_mat3());
                let mut c = a;
                c.translate(v.xyz());
                mat4(c);
                c = a;
                c.set_row3(2, v.xyz());
                c.set_column3(3, v.yzx());
                c.set_diagonal3(v.zxy());
                mat4(c);
                mat4(Mat4::<f32>::from_other(&a));
                mat4(Mat4::<f64>::from_other(&a));
                c.convert_from(&Mat4::<f32>::from_other(&b));
                mat4(c);
                let (a3, b3): (Mat3<$t>, Mat3<$t>) = (a.get_mat3(), b.get_mat3());
                matrix_common!(a3, b3, v.xyz(), s, Mat3, $t, mat3, vec3);
                Printed::print(a3.determinant());
                println!();
            }
        }
    };
}
fn mat4<T: Printed>(m: Mat4<T>) {
    (0..16).for_each(|i| m.get_value(i / 4, i % 4).print());
}
fn mat3<T: Printed>(m: Mat3<T>) {
    (0..9).for_each(|i| m.get_value(i / 3, i % 3).print());
}
typed_matrices!(float_kind, f32);
typed_matrices!(double_kind, f64);
pub fn ncollection_mat_case(values: &[f64]) {
    let raw: &[f64] = &values[1..];
    if values[0] == 0.0 {
        float_kind::request(raw);
    } else {
        double_kind::request(raw);
    }
}
