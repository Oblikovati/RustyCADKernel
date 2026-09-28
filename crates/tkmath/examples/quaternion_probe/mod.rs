//! Exact quaternion and Euler oracle counterpart.
use super::{matrix, xyz};
use rustycad_tkmath::gp::{
    Axis3, Direction3, EulerSequence, Point3, Quaternion, Transform3, Vector3,
};
fn quaternion_values(quaternion: Quaternion) {
    for value in quaternion.components() {
        print!("{} ", value.to_bits());
    }
}
fn euler_order(index: usize) -> EulerSequence {
    if index == 0 {
        return EulerSequence::EulerAngles;
    }
    if index == 1 {
        return EulerSequence::YawPitchRoll;
    }
    EulerSequence::ALL[index - 2]
}
pub(super) fn quaternion_case(v: &[f64]) {
    let quaternion = Quaternion::new(v[0], v[1], v[2], v[3]);
    matrix(quaternion.matrix());
    let order = euler_order(v[10] as usize);
    for angle in quaternion.euler_angles(order) {
        print!("{} ", angle.to_bits());
    }
    let euler = Quaternion::from_euler_angles(order, v[4], v[5], v[6]);
    quaternion_values(euler);
    matrix(euler.matrix());
    quaternion_values(Quaternion::from_matrix(quaternion.matrix()));
    let rotation = Quaternion::from_axis_angle(Vector3::new(v[7], v[8], v[9]), v[4]).unwrap();
    quaternion_values(rotation);
    xyz(*rotation
        .multiply_vector(Vector3::new(v[0], v[1], v[2]))
        .xyz());
    displacement_values(v);
    println!();
}
fn displacement_values(v: &[f64]) {
    let from = Axis3::from_normal(
        Point3::new(v[0], v[1], v[2]),
        Direction3::new(v[7], v[8], v[9]).unwrap(),
    )
    .unwrap();
    let to = Axis3::from_normal(
        Point3::new(v[4], v[5], v[6]),
        Direction3::new(v[4] + 0.1, v[5] + 0.2, v[6] + 0.3).unwrap(),
    )
    .unwrap();
    let displacement = Transform3::displacement(from, to);
    let point = Point3::new(v[0], v[1], v[2]);
    print!(
        "{} {} ",
        displacement.form() as i32,
        displacement.scale_factor().to_bits()
    );
    xyz(displacement.translation_part());
    matrix(displacement.matrix());
    xyz(point.transformed(displacement).xyz());
    quaternion_values(displacement.rotation_quaternion());
}
