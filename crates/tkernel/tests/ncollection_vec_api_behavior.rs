//! Supplemental NCollection_Vec2/Vec3/Vec4 API behavior pinned against native OCCT.
use rustycad_tkernel::ncollection::{ElementCast, Vec2, Vec3, Vec4};
use rustycad_tkernel::standard::dump::DumpStream;

#[test]
fn swizzles_follow_the_component_macros() {
    let v = Vec4::new(1, 2, 3, 4);
    assert_eq!(v.wx(), Vec2::new(4, 1));
    assert_eq!(v.zw(), Vec2::new(3, 4));
    assert_eq!(v.wzy(), Vec3::new(4, 3, 2));
    assert_eq!(v.ywx(), Vec3::new(2, 4, 1));
    assert_eq!(v.brg(), Vec3::new(3, 1, 2));
    assert_eq!((v.r(), v.g(), v.b(), v.a()), (1, 2, 3, 4));
    let three = Vec3::new(1, 2, 3);
    assert_eq!(three.zx(), Vec2::new(3, 1));
    assert_eq!(three.yzx(), Vec3::new(2, 3, 1));
    assert_eq!((three.r(), three.g(), three.b()), (1, 2, 3));
    assert_eq!(Vec2::new(1, 2).yx(), Vec2::new(2, 1));
    assert_eq!((three.x(), three.y(), three.z()), (1, 2, 3));
    let mut colour = Vec4::new(0, 0, 0, 0);
    *colour.a_mut() = 9;
    *colour.g_mut() = 7;
    assert_eq!(colour, Vec4::new(0, 7, 0, 9));
}

#[test]
fn data_access_constructors_and_setters() {
    let mut v = Vec3::new(1.0_f64, 2.0, 3.0);
    *v.change_data()[2] = 5.0;
    assert_eq!(v.data(), [1.0, 2.0, 5.0]);
    assert_eq!(Vec3::from_array([1, 2, 3]), Vec3::new(1, 2, 3));
    v.set_values_vec2(Vec2::new(7.0, 8.0), 9.0);
    assert_eq!(v, Vec3::new(7.0, 8.0, 9.0));
    let mut four = Vec4::from_vec2(Vec2::new(1, 2));
    assert_eq!(four, Vec4::new(1, 2, 0, 0));
    four.set_values_vec3(Vec3::new(3, 4, 5), 6);
    assert_eq!(four, Vec4::new(3, 4, 5, 6));
    assert!(four.is_equal(&Vec4::new(3, 4, 5, 6)));
    let nan = Vec2::new(f64::NAN, 0.0);
    assert!(!nan.is_equal(&nan), "NaN components are never equal");
}

#[test]
fn element_conversion_matches_static_cast_on_x86_64() {
    let narrowed = Vec3::<f32>::convert_from(Vec3::new(0.1_f64, -2.5, 1e40));
    assert_eq!(narrowed.x, 0.1_f32);
    assert!(narrowed.z.is_infinite());
    let truncated = Vec4::<i32>::convert_from(Vec4::new(2.7_f64, -2.7, 3e9, f64::NAN));
    // cvttsd2si truncates, and returns INT_MIN for out-of-range and NaN inputs.
    assert_eq!(truncated, Vec4::new(2, -2, i32::MIN, i32::MIN));
    assert_eq!(ElementCast::<i32>::cast(-2_147_483_648.0_f64), i32::MIN);
    assert_eq!(
        Vec2::<f64>::convert_from(Vec2::new(3, -4)),
        Vec2::new(3.0, -4.0)
    );
}

#[test]
fn integer_magnitudes_truncate_through_double() {
    assert_eq!(Vec2::new(1, 1).modulus(), 1, "sqrt(2) truncates");
    assert_eq!(Vec3::new(2, 3, 6).modulus(), 7);
    assert_eq!(Vec3::new(-2_i32, 3, -6).cwise_abs(), Vec3::new(2, 3, 6));
    assert_eq!(
        Vec4::new(i32::MIN, -1, 0, 1).cwise_abs(),
        Vec4::new(i32::MIN, 1, 0, 1)
    );
    assert_eq!(Vec3::new(0, 0, 5).normalized(), Vec3::new(0, 0, 1));
    assert_eq!(Vec3::new(0, 0, 0).normalized(), Vec3::new(0, 0, 0));
    assert_eq!(
        Vec3::lerp(&Vec3::new(0, 0, 0), &Vec3::new(4, 8, 2), 1),
        Vec3::new(4, 8, 2)
    );
    assert_eq!(Vec3::<i32>::dz(), Vec3::new(0, 0, 1));
    assert_eq!(Vec2::<i32>::dy(), Vec2::new(0, 1));
}

#[test]
fn float_members_and_json() {
    assert_eq!(Vec2::new(3.0_f32, 4.0).modulus(), 5.0);
    assert_eq!(
        Vec3::new(0.0_f64, 0.0, 0.0).normalized(),
        Vec3::new(0.0, 0.0, 0.0)
    );
    let mut stream = DumpStream::default();
    Vec3::new(1.0_f32, 0.5, -2.0)
        .dump_json(&mut stream)
        .unwrap();
    assert_eq!(stream.text(), "\"Vec3\": [1, 0.5, -2]");
    let mut stream = DumpStream::default();
    Vec4::new(0.1_f32, 0.0, 0.0, 1.0)
        .dump_json(&mut stream)
        .unwrap();
    // The float promotes exactly to double before the six-digit formatting.
    assert_eq!(stream.text(), "\"Vec4\": [0.1, 0, 0, 1]");
    let mut stream = DumpStream::default();
    Vec2::new(1e-7_f64, 2.0).dump_json(&mut stream).unwrap();
    assert_eq!(stream.text(), "\"Vec2\": [1e-07, 2]");
}
