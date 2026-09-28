//! `gp3d_vec`: every gp_Vec member, mirroring `gpc_vec_case`.
use crate::elementary_probe::{Fields, attempt, end, flag, real, reals, text, vec};
use crate::gp_transform_probe::read_transform;
use rustycad_tkernel::standard::{Failure, dump::DumpStream};
use rustycad_tkmath::gp::{Axis1, Axis2, Direction3, Point3, Transform3, Vector3, Xyz};

pub fn gp3_vec_case(values: &[f64]) {
    let mut f = Fields::new(values);
    let [a, b, c, d] = std::array::from_fn(|_| Vector3::from(f.xyz()));
    let (s, t, idx) = (f.real(), f.real(), f.int());
    let (o, n, angle) = (f.point(), f.xyz(), f.real());
    let transform = read_transform(&mut f);
    access(a, b, c, (s, t), idx);
    metrics(a, b, c, (s, t));
    algebra([a, b, c, d], s, t);
    attempt(|| motions(a, b, (o, n, angle), transform));
    end();
}
fn access(a: Vector3, b: Vector3, c: Vector3, (s, t): (f64, f64), idx: i32) {
    let mut v = a;
    attempt(|| v.set_coord(idx, s));
    vec(v);
    attempt(|| {
        real(a.coord(idx)?);
        Ok(())
    });
    v.set_coordinates(s, t, s);
    vec(v);
    v.set_x(t);
    v.set_y(s);
    v.set_z(t);
    vec(v);
    v.set_xyz(*b.xyz());
    vec(v);
    reals(a.xyz().coordinates());
    vec(Vector3::between(
        Point3::from(*a.xyz()),
        Point3::from(*b.xyz()),
    ));
    attempt(|| {
        vec(Vector3::from(Direction3::from_xyz(*c.xyz())?));
        Ok(())
    });
}
fn metrics(a: Vector3, b: Vector3, c: Vector3, (s, t): (f64, f64)) {
    real(a.magnitude());
    real(a.square_magnitude());
    type Predicate = fn(Vector3, Vector3, f64) -> Result<bool, Failure>;
    attempt(|| {
        flag(a.is_equal(b, s, t)?);
        Ok(())
    });
    let predicates: [Predicate; 3] = [
        Vector3::is_normal,
        Vector3::is_opposite,
        Vector3::is_parallel,
    ];
    for predicate in predicates {
        attempt(|| {
            flag(predicate(a, b, t)?);
            Ok(())
        });
    }
    attempt(|| {
        real(a.angle(b)?);
        Ok(())
    });
    attempt(|| {
        real(a.angle_with_ref(b, c)?);
        Ok(())
    });
    attempt(|| {
        vec(a.normalized()?);
        Ok(())
    });
    let mut v = a;
    attempt(|| v.normalize());
    vec(v);
}
fn algebra([a, b, c, d]: [Vector3; 4], s: f64, t: f64) {
    [a + b, a - b, a * s, a / s, a ^ b]
        .into_iter()
        .for_each(vec);
    real(a * b);
    vec(-a);
    vec(s * a);
    let mut v = a;
    v += b;
    v -= c;
    v *= s;
    v /= t;
    v ^= d;
    vec(v);
    vec(v);
    [a + b, a - b, a * s, a / s, a.crossed(b)]
        .into_iter()
        .for_each(vec);
    reals(&[a.cross_magnitude(b), a.cross_square_magnitude(b)]);
    vec(a.cross_crossed(b, c));
    v = a;
    v.cross_cross(b, c);
    vec(v);
    reals(&[a.dot(b), a.dot_cross(b, c)]);
    vec(a.reversed());
    v = a;
    v.reverse();
    vec(v);
    linear_forms(v, [a, b, c, d], s, t);
    vec(a.scaled(s));
    v = a;
    v.scale(t);
    vec(v);
}
/// The six SetLinearForm overloads applied in sequence to one vector.
fn linear_forms(mut v: Vector3, [a, b, c, d]: [Vector3; 4], s: f64, t: f64) {
    v.set_linear_form_three_offset([s, t, s], [a, b, c], d);
    vec(v);
    v.set_linear_form_three([s, t, s], [a, b, c]);
    vec(v);
    v.set_linear_form_offset(s, a, t, b, c);
    vec(v);
    v.set_linear_form(s, a, t, b);
    vec(v);
    v.set_scaled_sum(s, a, b);
    vec(v);
    v.set_sum(a, b);
    vec(v);
}
fn motions(
    a: Vector3,
    b: Vector3,
    (o, n, angle): (Point3, Xyz, f64),
    transform: Transform3,
) -> Result<(), Failure> {
    let axis = Axis1::new(o, Direction3::from_xyz(n)?);
    let plane = Axis2::new(o, Direction3::from_xyz(n)?)?;
    [
        a.mirrored_vector(b),
        a.mirrored_axis(axis),
        a.mirrored_plane(plane),
    ]
    .into_iter()
    .for_each(vec);
    let mut v = a;
    v.mirror_vector(b);
    v.mirror_axis(axis);
    v.mirror_plane(plane);
    vec(v);
    vec(a.rotated(axis, angle)?);
    v = a;
    v.rotate(axis, angle)?;
    vec(v);
    vec(a.transformed(transform));
    v = a;
    v.transform(transform);
    vec(v);
    let mut stream = DumpStream::default();
    a.dump_json(&mut stream).unwrap();
    text(stream.text());
    Ok(())
}
