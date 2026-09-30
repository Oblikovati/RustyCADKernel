//! Direct BVH_Ray_Test.cxx translation at pinned OCCT 3d097a0.
#![allow(non_snake_case, non_upper_case_globals, unused_mut)]
use rustycad_tkmath::bvh::{BvhRay, BvhVec2d, BvhVec3d, BvhVec3f};

// =======================================================================================
// Tests for BVH_Ray
// =======================================================================================

#[test]
fn BVH_RayTest__ConstructorBasic() {
    let mut aOrigin = BvhVec3d::new(1.0, 2.0, 3.0);
    let mut aDirection = BvhVec3d::new(0.0, 1.0, 0.0);

    let mut aRay = BvhRay::new(aOrigin, aDirection);

    assert_eq!(aRay.origin.x, 1.0);
    assert_eq!(aRay.origin.y, 2.0);
    assert_eq!(aRay.origin.z, 3.0);

    assert_eq!(aRay.direct.x, 0.0);
    assert_eq!(aRay.direct.y, 1.0);
    assert_eq!(aRay.direct.z, 0.0);
}

#[test]
fn BVH_RayTest__DirectionStorage() {
    let mut aOrigin = BvhVec3d::new(0.0, 0.0, 0.0);
    let mut aDirection = BvhVec3d::new(2.0, 4.0, 8.0);

    let mut aRay = BvhRay::new(aOrigin, aDirection);

    // Verify direction is stored correctly
    assert_eq!(aRay.direct.x, 2.0);
    assert_eq!(aRay.direct.y, 4.0);
    assert_eq!(aRay.direct.z, 8.0);
}

#[test]
fn BVH_RayTest__NegativeDirection() {
    let mut aOrigin = BvhVec3d::new(0.0, 0.0, 0.0);
    let mut aDirection = BvhVec3d::new(-2.0, -4.0, -1.0);

    let mut aRay = BvhRay::new(aOrigin, aDirection);

    assert_eq!(aRay.direct.x, -2.0);
    assert_eq!(aRay.direct.y, -4.0);
    assert_eq!(aRay.direct.z, -1.0);
}

#[test]
fn BVH_RayTest__ZeroComponent() {
    let mut aOrigin = BvhVec3d::new(0.0, 0.0, 0.0);
    let mut aDirection = BvhVec3d::new(1.0, 0.0, 1.0); // Zero Y component

    let mut aRay = BvhRay::new(aOrigin, aDirection);

    assert_eq!(aRay.direct.x, 1.0);
    assert_eq!(aRay.direct.y, 0.0);
    assert_eq!(aRay.direct.z, 1.0);
}

#[test]
fn BVH_RayTest__AllZeroDirection() {
    let mut aOrigin = BvhVec3d::new(0.0, 0.0, 0.0);
    let mut aDirection = BvhVec3d::new(0.0, 0.0, 0.0);

    let mut aRay = BvhRay::new(aOrigin, aDirection);

    // All components should be zero
    assert_eq!(aRay.direct.x, 0.0);
    assert_eq!(aRay.direct.y, 0.0);
    assert_eq!(aRay.direct.z, 0.0);
}

#[test]
fn BVH_RayTest__DefaultConstructor() {
    let mut aRay = BvhRay::<BvhVec3d>::default();

    // Default constructed ray should have zero origin and direction
    assert_eq!(aRay.origin.x, 0.0);
    assert_eq!(aRay.origin.y, 0.0);
    assert_eq!(aRay.origin.z, 0.0);

    assert_eq!(aRay.direct.x, 0.0);
    assert_eq!(aRay.direct.y, 0.0);
    assert_eq!(aRay.direct.z, 0.0);
}

#[test]
fn BVH_RayTest__Ray2D() {
    let mut aOrigin = BvhVec2d::new(1.0, 2.0);
    let mut aDirection = BvhVec2d::new(3.0, 4.0);

    let mut aRay = BvhRay::new(aOrigin, aDirection);

    assert_eq!(aRay.origin.x, 1.0);
    assert_eq!(aRay.origin.y, 2.0);

    assert_eq!(aRay.direct.x, 3.0);
    assert_eq!(aRay.direct.y, 4.0);
}

#[test]
fn BVH_RayTest__NormalizedDirection() {
    let mut aOrigin = BvhVec3d::new(0.0, 0.0, 0.0);
    let mut aDirection = BvhVec3d::new(1.0, 0.0, 0.0); // Unit vector along X

    let mut aRay = BvhRay::new(aOrigin, aDirection);

    assert_eq!(aRay.direct.x, 1.0);
    assert_eq!(aRay.direct.y, 0.0);
    assert_eq!(aRay.direct.z, 0.0);
}

#[test]
fn BVH_RayTest__VerySmallDirection() {
    let mut aOrigin = BvhVec3d::new(0.0, 0.0, 0.0);
    let mut aDirection = BvhVec3d::new(1e-20, 1e-20, 1e-20);

    let mut aRay = BvhRay::new(aOrigin, aDirection);

    // Should handle very small values correctly
    assert_eq!(aRay.direct.x, 1e-20);
    assert_eq!(aRay.direct.y, 1e-20);
    assert_eq!(aRay.direct.z, 1e-20);
}

#[test]
fn BVH_RayTest__MixedZeroNonZero() {
    let mut aOrigin = BvhVec3d::new(1.0, 2.0, 3.0);
    let mut aDirection = BvhVec3d::new(0.0, 2.0, 0.0);

    let mut aRay = BvhRay::new(aOrigin, aDirection);

    assert_eq!(aRay.direct.x, 0.0);
    assert_eq!(aRay.direct.y, 2.0);
    assert_eq!(aRay.direct.z, 0.0);
}

#[test]
fn BVH_RayTest__FloatPrecision() {
    let mut aOrigin = BvhVec3f::new(1.0, 2.0, 3.0);
    let mut aDirection = BvhVec3f::new(2.0, 4.0, 8.0);

    let mut aRay = BvhRay::new(aOrigin, aDirection);

    assert_eq!(aRay.direct.x, 2.0_f32);
    assert_eq!(aRay.direct.y, 4.0_f32);
    assert_eq!(aRay.direct.z, 8.0_f32);
}

// Upstream also declares the ray `constexpr`. The Rust constructor computes the
// reciprocal through generic trait arithmetic, which cannot run in const context;
// the vectors are const items and the ray is built at test time.
#[test]
fn BVH_RayTest__ConstexprConstructor() {
    // This test verifies that the constructor is truly constexpr
    const aOrigin: BvhVec3d = BvhVec3d::new(1.0, 2.0, 3.0);
    const aDirection: BvhVec3d = BvhVec3d::new(1.0, 1.0, 1.0);
    let aRay = BvhRay::new(aOrigin, aDirection);

    // If this compiles, constexpr works
    assert_eq!(aRay.origin.x, 1.0);
}

#[test]
fn BVH_RayTest__DiagonalRay() {
    let mut aOrigin = BvhVec3d::new(0.0, 0.0, 0.0);
    let mut aDirection = BvhVec3d::new(1.0, 1.0, 1.0);

    let mut aRay = BvhRay::new(aOrigin, aDirection);

    assert_eq!(aRay.direct.x, 1.0);
    assert_eq!(aRay.direct.y, 1.0);
    assert_eq!(aRay.direct.z, 1.0);
}

#[test]
fn BVH_RayTest__NegativeOrigin() {
    let mut aOrigin = BvhVec3d::new(-10.0, -20.0, -30.0);
    let mut aDirection = BvhVec3d::new(1.0, 2.0, 3.0);

    let mut aRay = BvhRay::new(aOrigin, aDirection);

    assert_eq!(aRay.origin.x, -10.0);
    assert_eq!(aRay.origin.y, -20.0);
    assert_eq!(aRay.origin.z, -30.0);

    assert_eq!(aRay.direct.x, 1.0);
    assert_eq!(aRay.direct.y, 2.0);
    assert_eq!(aRay.direct.z, 3.0);
}

#[test]
fn BVH_RayTest__AxisAlignedX() {
    let mut aOrigin = BvhVec3d::new(0.0, 0.0, 0.0);
    let mut aDirection = BvhVec3d::new(1.0, 0.0, 0.0);

    let mut aRay = BvhRay::new(aOrigin, aDirection);

    assert_eq!(aRay.direct.x, 1.0);
    assert_eq!(aRay.direct.y, 0.0);
    assert_eq!(aRay.direct.z, 0.0);
}

#[test]
fn BVH_RayTest__AxisAlignedY() {
    let mut aOrigin = BvhVec3d::new(0.0, 0.0, 0.0);
    let mut aDirection = BvhVec3d::new(0.0, 1.0, 0.0);

    let mut aRay = BvhRay::new(aOrigin, aDirection);

    assert_eq!(aRay.direct.x, 0.0);
    assert_eq!(aRay.direct.y, 1.0);
    assert_eq!(aRay.direct.z, 0.0);
}

#[test]
fn BVH_RayTest__AxisAlignedZ() {
    let mut aOrigin = BvhVec3d::new(0.0, 0.0, 0.0);
    let mut aDirection = BvhVec3d::new(0.0, 0.0, 1.0);

    let mut aRay = BvhRay::new(aOrigin, aDirection);

    assert_eq!(aRay.direct.x, 0.0);
    assert_eq!(aRay.direct.y, 0.0);
    assert_eq!(aRay.direct.z, 1.0);
}

#[test]
fn BVH_RayTest__LargeValues() {
    let mut aOrigin = BvhVec3d::new(1e6, 2e6, 3e6);
    let mut aDirection = BvhVec3d::new(1e3, 2e3, 4e3);

    let mut aRay = BvhRay::new(aOrigin, aDirection);

    assert_eq!(aRay.direct.x, 1e3);
    assert_eq!(aRay.direct.y, 2e3);
    assert_eq!(aRay.direct.z, 4e3);
}

#[test]
fn BVH_RayTest__OriginAndDirectionPreservation() {
    let mut aOrigin = BvhVec3d::new(1.0, 2.0, 3.0);
    let mut aDirection = BvhVec3d::new(2.0, 4.0, 8.0);

    let mut aRay = BvhRay::new(aOrigin, aDirection);

    // Verify that both origin and direction are preserved exactly
    assert_eq!(aRay.origin.x, aOrigin.x);
    assert_eq!(aRay.origin.y, aOrigin.y);
    assert_eq!(aRay.origin.z, aOrigin.z);

    assert_eq!(aRay.direct.x, aDirection.x);
    assert_eq!(aRay.direct.y, aDirection.y);
    assert_eq!(aRay.direct.z, aDirection.z);
}
