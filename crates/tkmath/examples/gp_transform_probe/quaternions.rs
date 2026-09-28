//! gp_Quaternion and interpolator requests.
use super::{mat, quat, read_quat};
use crate::elementary_probe::{Fields, attempt, end, flag, real, xyz};
use rustycad_tkmath::gp::{Quaternion, QuaternionNLerp, QuaternionSLerp, Vector3};

pub fn gp3_quat_case(values: &[f64]) {
    let mut f = Fields::new(values);
    let (a, b) = (read_quat(&mut f), read_quat(&mut f));
    let (u, v, h) = (
        Vector3::from(f.xyz()),
        Vector3::from(f.xyz()),
        Vector3::from(f.xyz()),
    );
    let (angle, t) = (f.real(), f.real());
    flag(a.is_equal(&b));
    flag(a.is_equal(&a));
    quat(Quaternion::from_vectors(u, v));
    quat(Quaternion::from_vectors_with_help(u, v, h));
    let mut c = Quaternion::default();
    attempt(|| {
        c.set_vector_and_angle(u, angle)?;
        quat(c);
        Ok(())
    });
    let (axis, got) = a.vector_and_angle();
    xyz(*axis.xyz());
    real(got);
    mat(a.matrix());
    c.set_matrix(a.matrix());
    quat(c);
    quat(Quaternion::from_matrix(b.matrix()));
    arithmetic(a, b, t);
    in_place(a, b, t);
    real(a.dot(b));
    real(a.rotation_angle());
    xyz(*a.multiply_vector(u).xyz());
    xyz(*(a * u).xyz());
    interpolation(a, b, t);
    end();
}
fn arithmetic(a: Quaternion, b: Quaternion, t: f64) {
    quat(a.reversed());
    quat(a.inverted());
    real(a.square_norm());
    real(a.norm());
    [a.scaled(t), a * t].into_iter().for_each(quat);
    let mut c = a;
    c.stabilize_length();
    quat(c);
    quat(a.normalized());
    c = a;
    c.normalize();
    quat(c);
    [
        a.negated(),
        -a,
        a.added(b),
        a + b,
        a.subtracted(b),
        a - b,
        a.multiplied(b),
        a * b,
    ]
    .into_iter()
    .for_each(quat);
}
fn in_place(a: Quaternion, b: Quaternion, t: f64) {
    let mut c = a;
    c += b;
    quat(c);
    c = a;
    c -= b;
    quat(c);
    c = a;
    c.multiply(b);
    quat(c);
    c = a;
    c *= b;
    quat(c);
    c = a;
    c *= t;
    quat(c);
    c = a;
    c.scale(t);
    quat(c);
    c = a;
    c.reverse();
    quat(c);
    c = a;
    c.invert();
    quat(c);
    c = a;
    c.set_ident();
    quat(c);
    c.set_from(b);
    quat(c);
}
fn interpolation(a: Quaternion, b: Quaternion, t: f64) {
    quat(QuaternionNLerp::interpolated(a, b, t));
    quat(QuaternionSLerp::interpolated(a, b, t));
    let mut linear = QuaternionNLerp::default();
    linear.init_from_unit(a, b);
    quat(linear.interpolate(t));
    let mut spherical = QuaternionSLerp::default();
    spherical.init_from_unit(a, b);
    quat(spherical.interpolate(t));
}
