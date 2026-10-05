//! Compiler-kind mapping proof: getters copy, public field borrows mutate the same component.
use rustycad_tkernel::ncollection::{Vec2, Vec3, Vec4};

#[test]
fn planar_field_borrows_match_mutable_native_accessors() {
    let mut vector = Vec2::new(1_i32, 2);
    assert_eq!(std::mem::replace(&mut vector.x, 11), 1);
    assert_eq!(std::mem::replace(&mut vector.y, 12), 2);
    assert_eq!([vector.x(), vector.y()], [11, 12]);
}

#[test]
fn spatial_field_borrows_match_mutable_native_accessors() {
    let mut vector = Vec3::new(3_i32, 4, 5);
    assert_eq!(std::mem::replace(&mut vector.x, 13), 3);
    assert_eq!(std::mem::replace(&mut vector.y, 14), 4);
    assert_eq!(std::mem::replace(&mut vector.z, 15), 5);
    assert_eq!([vector.x(), vector.y(), vector.z()], [13, 14, 15]);
}

#[test]
fn homogeneous_field_borrows_match_mutable_native_accessors() {
    let mut vector = Vec4::new(6_i32, 7, 8, 9);
    assert_eq!(std::mem::replace(&mut vector.x, 16), 6);
    assert_eq!(std::mem::replace(&mut vector.y, 17), 7);
    assert_eq!(std::mem::replace(&mut vector.z, 18), 8);
    assert_eq!(std::mem::replace(&mut vector.w, 19), 9);
    assert_eq!(
        [vector.x(), vector.y(), vector.z(), vector.w()],
        [16, 17, 18, 19]
    );
}
