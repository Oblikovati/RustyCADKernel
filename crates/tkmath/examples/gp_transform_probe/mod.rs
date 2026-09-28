//! gp_Trsf, gp_GTrsf, gp_Quaternion and interpolator oracle counterpart; prints the same token
//! stream as tools/oracle/tkmath_gp_transform_probe.cxx.
use crate::elementary_probe::{Fields, attempt, end, flag, real, xyz};
use rustycad_tkernel::ncollection::Mat4;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkmath::gp::{
    Axis1, Axis2, Axis2d, Axis3, Direction2, Direction3, Matrix3, Point2, Point3, Quaternion,
    Transform2, Transform3, Vector3, Xyz,
};
mod general;
mod quaternions;
pub use general::gp3_gtrsf_case;
pub use quaternions::gp3_quat_case;

pub fn mat(m: Matrix3) {
    m.coefficients().iter().flatten().copied().for_each(real);
}
pub fn quat(q: Quaternion) {
    q.components().into_iter().for_each(real);
}
pub fn mat4<T: Copy + Into<f64>>(m: Mat4<T>) {
    (0..16).for_each(|i| real(m.get_value(i / 4, i % 4).into()));
}
pub fn read_quat(f: &mut Fields) -> Quaternion {
    Quaternion::new(f.real(), f.real(), f.real(), f.real())
}
pub fn read_mat(f: &mut Fields) -> Matrix3 {
    Matrix3::new(std::array::from_fn(|_| std::array::from_fn(|_| f.real())))
}
/// The native `gp3_trsf` fields, read before anything is built.
struct Spec {
    kind: i32,
    p: [f64; 12],
    scale: Option<f64>,
    translation: Option<Xyz>,
    rotation: Option<Quaternion>,
}
fn read_spec(f: &mut Fields) -> Spec {
    let kind = f.int();
    let p = std::array::from_fn(|_| f.real());
    let (scale_kind, scale) = (f.int(), f.real());
    let (translation_kind, translation) = (f.int(), f.xyz());
    let (rotation_kind, rotation) = (f.int(), read_quat(f));
    Spec {
        kind,
        p,
        scale: (scale_kind != 0).then_some(scale),
        translation: (translation_kind != 0).then_some(translation),
        rotation: (rotation_kind != 0).then_some(rotation),
    }
}
fn frames(p: [f64; 12]) -> Result<(Axis3, Axis3), Failure> {
    let from = Axis3::from_normal(
        Point3::new(p[0], p[1], p[2]),
        Direction3::new(p[3], p[4], p[5])?,
    )?;
    let to = Axis3::from_normal(
        Point3::new(p[6], p[7], p[8]),
        Direction3::new(p[5], p[3], p[4])?,
    )?;
    Ok((from, to))
}
fn built(p: [f64; 12], kind: i32) -> Result<Transform3, Failure> {
    let a = Point3::new(p[0], p[1], p[2]);
    let direction = || Direction3::new(p[3], p[4], p[5]);
    Ok(match kind {
        1 => Transform3::translation_vector(Vector3::from(a.xyz())),
        2 => Transform3::translation_between(a, Point3::new(p[3], p[4], p[5])),
        3 => Transform3::rotation(Axis1::new(a, direction()?), p[6])?,
        4 => Transform3::mirror_point(a),
        5 => Transform3::mirror_axis(Axis1::new(a, direction()?)),
        6 => Transform3::mirror_plane(Axis2::new(a, direction()?)?),
        7 => Transform3::scaling(a, p[3])?,
        8 => Transform3::transformation_to(Axis3::from_normal(a, direction()?)?),
        9 => {
            let (from, to) = frames(p)?;
            Transform3::transformation_between(from, to)
        }
        10 => {
            let (from, to) = frames(p)?;
            Transform3::displacement(from, to)
        }
        11 => values_transform(p)?,
        12 => Transform3::from_quaternion(Quaternion::new(p[0], p[1], p[2], p[3])),
        13 => Transform3::rotation_translation(
            Quaternion::new(p[0], p[1], p[2], p[3]),
            Vector3::new(p[4], p[5], p[6]),
        ),
        14 => Transform3::from(planar(p)?),
        _ => Transform3::default(),
    })
}
fn values_transform(p: [f64; 12]) -> Result<Transform3, Failure> {
    let mut t = Transform3::default();
    t.set_values([
        [p[0], p[1], p[2], p[3]],
        [p[4], p[5], p[6], p[7]],
        [p[8], p[9], p[10], p[11]],
    ])?;
    Ok(t)
}
fn planar(p: [f64; 12]) -> Result<Transform2, Failure> {
    if p[4] > 0.0 {
        let axis = Axis2d::new(Point2::new(p[0], p[1]), Direction2::new(p[2], p[3])?);
        return Ok(Transform2::mirror_axis(axis));
    }
    Ok(Transform2::rotation(Point2::new(p[0], p[1]), p[2]))
}
fn build(spec: &Spec) -> Result<Transform3, Failure> {
    let mut t = built(spec.p, spec.kind)?;
    if let Some(scale) = spec.scale {
        t.set_scale_factor(scale)?;
    }
    if let Some(offset) = spec.translation {
        t.set_translation_part(Vector3::from(offset));
    }
    if let Some(rotation) = spec.rotation {
        t.set_rotation_part(rotation);
    }
    Ok(t)
}
/// `GP3_TRY(t=gp3_trsf(r))`: a failed build prints the exception and keeps an identity.
pub fn read_transform(f: &mut Fields) -> Transform3 {
    let spec = read_spec(f);
    let mut t = Transform3::default();
    attempt(|| {
        t = build(&spec)?;
        Ok(())
    });
    t
}
pub fn trsf_state(t: Transform3) {
    print!("{} ", t.form() as i32);
    real(t.scale_factor());
    flag(t.is_negative());
    for row in 1..=3 {
        for col in 1..=4 {
            real(t.value(row, col).unwrap());
        }
    }
    mat(t.vectorial_part());
    mat(t.matrix());
    xyz(t.translation_part());
    quat(t.rotation_quaternion());
    let (axis, angle) = t.rotation_axis_angle();
    flag(true);
    xyz(axis);
    real(angle);
    let mut double_matrix = Mat4::<f64>::default();
    t.get_mat4(&mut double_matrix);
    mat4(double_matrix);
    let mut float_matrix = Mat4::<f32>::default();
    t.get_mat4(&mut float_matrix);
    mat4(float_matrix);
    let mut dump = DumpStream::default();
    t.dump_json(&mut dump).unwrap();
    print!("{} | ", dump.text());
}
pub fn gp3_trsf_case(values: &[f64]) {
    let mut f = Fields::new(values);
    let (a, b) = (read_transform(&mut f), read_transform(&mut f));
    let (p, d, n) = (f.point(), f.xyz(), f.int());
    [a, b, a.multiplied(b), a * b]
        .into_iter()
        .for_each(trsf_state);
    let mut c = a;
    c.pre_multiply(b);
    trsf_state(c);
    c = a;
    c *= b;
    trsf_state(c);
    attempt(|| a.inverted().map(trsf_state));
    attempt(|| a.powered(n).map(trsf_state));
    c = a;
    attempt(|| c.invert());
    trsf_state(c);
    c = a;
    attempt(|| c.power(n));
    trsf_state(c);
    objects(a, p, d);
    json_round_trip(a);
    end();
}
fn objects(a: Transform3, p: Point3, d: Xyz) {
    xyz(p.transformed(a).xyz());
    xyz(a.apply_coordinates(p.xyz()));
    let (x, y, z) = a.transforms_coordinates(p.x(), p.y(), p.z());
    [x, y, z].into_iter().for_each(real);
    attempt(|| {
        let direction = Direction3::from_xyz(d)?;
        xyz(Xyz::from(direction.transformed(a)));
        xyz(Xyz::from(
            Axis1::new(p, direction).transformed(a).direction(),
        ));
        Ok(())
    });
}
fn json_round_trip(a: Transform3) {
    let mut dump = DumpStream::default();
    a.dump_json(&mut dump).unwrap();
    let mut parsed = Transform3::default();
    let mut position: i32 = 1;
    attempt(|| {
        flag(parsed.init_from_json(dump.text(), &mut position)?);
        Ok(())
    });
    print!("{position} ");
    trsf_state(parsed);
}
