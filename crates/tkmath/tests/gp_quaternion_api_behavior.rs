//! Supplemental gp_Quaternion, gp_QuaternionNLerp and gp_QuaternionSLerp behavior.
use rustycad_tkmath::gp::{Quaternion, QuaternionNLerp, QuaternionSLerp, Vector3};

fn close(a: Quaternion, b: Quaternion) -> bool {
    a.components()
        .iter()
        .zip(b.components())
        .all(|(x, y)| (x - y).abs() < 1e-14)
}

#[test]
fn conjugate_inverse_and_length_helpers() {
    let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    assert_eq!(q.reversed(), Quaternion::new(-1.0, -2.0, -3.0, 4.0));
    assert!(close(q * q.inverted(), Quaternion::default()));
    let mut stable = Quaternion::new(1.0, -1.0, 0.0, 2.0);
    stable.stabilize_length();
    assert_eq!(stable, Quaternion::new(0.25, -0.25, 0.0, 0.5));
    let mut zero = Quaternion::new(0.0, 0.0, 0.0, 0.0);
    zero.stabilize_length();
    assert_eq!(zero, Quaternion::default(), "a zero sum resets to identity");
    let mut tiny = Quaternion::new(0.0, 0.0, 0.0, 0.0);
    tiny.normalize();
    assert_eq!(
        tiny,
        Quaternion::default(),
        "a null norm is stabilized first"
    );
    assert!((q.normalized().norm() - 1.0).abs() < 1e-15);
    assert!(q.is_equal(&Quaternion::new(1.0, 2.0, 3.0, 4.0)));
    assert!(!q.is_equal(&Quaternion::new(1.0, 2.0, 3.0, 4.000000000000001)));
}

#[test]
fn operators_match_named_members() {
    let a = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let b = Quaternion::new(-0.5, 0.25, 2.0, 1.0);
    assert_eq!(a + b, a.added(b));
    assert_eq!(a - b, a.subtracted(b));
    assert_eq!(a * b, a.multiplied(b));
    assert_eq!(-a, a.negated());
    assert_eq!(a * 2.0, a.scaled(2.0));
    let mut c = a;
    c *= b;
    assert_eq!(c, a * b);
    c = a;
    c.multiply(b);
    assert_eq!(c, a * b);
    assert_eq!(a.dot(b), -0.5 + 0.5 + 6.0 + 4.0);
    let rotation = Quaternion::from_axis_angle(Vector3::new(0.0, 0.0, 1.0), 0.75).unwrap();
    assert!((rotation.rotation_angle() - 0.75).abs() < 1e-15);
    let turned = rotation * Vector3::new(1.0, 0.0, 0.0);
    assert!((turned.x() - 0.75_f64.cos()).abs() < 1e-15);
}

#[test]
fn rotation_setters() {
    let from = Vector3::new(1.0, 0.0, 0.0);
    let to = Vector3::new(0.0, 1.0, 0.0);
    let q = Quaternion::from_vectors(from, to);
    let (axis, angle) = q.vector_and_angle();
    assert!((angle - std::f64::consts::FRAC_PI_2).abs() < 1e-15 && axis.z() > 0.99);
    // Opposite vectors turn about from x help = (1,0,0) x (0,0,1) = -Y.
    let helped = Quaternion::from_vectors_with_help(
        from,
        Vector3::new(-1.0, 0.0, 0.0),
        Vector3::new(0.0, 0.0, 1.0),
    );
    let (axis, angle) = helped.vector_and_angle();
    assert!((angle - std::f64::consts::PI).abs() < 1e-15 && axis.y() < -0.99);
    let mut set = Quaternion::default();
    assert!(
        set.set_vector_and_angle(Vector3::new(0.0, 0.0, 0.0), 1.0)
            .is_err()
    );
    set.set_matrix(q.matrix());
    assert!(close(set, q));
    set.set_ident();
    assert_eq!(set, Quaternion::default());
}

#[test]
fn interpolators_take_the_shorter_arc() {
    let start = Quaternion::default();
    let end = Quaternion::from_axis_angle(Vector3::new(0.0, 0.0, 1.0), 1.0).unwrap();
    let n = QuaternionNLerp::new(start, -end);
    assert!(
        close(n.interpolate(1.0), end),
        "a negative dot flips the end"
    );
    assert_eq!(QuaternionNLerp::interpolated(start, end, 0.0), start);
    let s = QuaternionSLerp::new(start, end * 3.0);
    let half = s.interpolate(0.5);
    assert!((half.rotation_angle() - 0.5).abs() < 1e-14);
    assert_eq!(QuaternionSLerp::interpolated(start, end, 0.5), half);
    // Identical ends are clamped to acos(0.9999), so the path stays defined.
    let same = QuaternionSLerp::new(end, end).interpolate(0.5);
    assert!(same.components().iter().all(|c| c.is_finite()));
}
