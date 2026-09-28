#[test]
fn BVH_ToolsTest__BoxBoxSquareDistance2D() {
    let mut aBox1 = BvhBox::from_corners(BvhVec2d::new(0.0, 0.0), BvhVec2d::new(1.0, 1.0));
    let mut aBox2 = BvhBox::from_corners(BvhVec2d::new(3.0, 0.0), BvhVec2d::new(4.0, 1.0));

    // Distance = 2
    let mut aDist: f64 = box_box_square_distance(&aBox1, &aBox2);
    assert_near(aDist, 4.0, precision::confusion());
}

#[test]
fn BVH_ToolsTest__PointBoxProjection2D() {
    let mut aBox = BvhBox::from_corners(BvhVec2d::new(0.0, 0.0), BvhVec2d::new(1.0, 1.0));

    // Point outside
    let mut aProj = point_box_projection(&BvhVec2d::new(2.0, 2.0), &aBox);
    assert_near(aProj.x, 1.0, precision::confusion());
    assert_near(aProj.y, 1.0, precision::confusion());
}

#[test]
fn BVH_ToolsTest__PointBoxSquareDistanceAtVertex() {
    let mut aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));

    // Point on vertex
    let mut aDist: f64 = point_box_square_distance(&BvhVec3d::new(0.0, 0.0, 0.0), &aBox);
    assert_near(aDist, 0.0, precision::confusion());
}

#[test]
fn BVH_ToolsTest__PointBoxSquareDistanceAtEdge() {
    let mut aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));

    // Point on edge
    let mut aDist: f64 = point_box_square_distance(&BvhVec3d::new(0.5, 0.0, 0.0), &aBox);
    assert_near(aDist, 0.0, precision::confusion());
}

#[test]
fn BVH_ToolsTest__PointBoxSquareDistanceFloat() {
    let mut aBox = BvhBox::from_corners(BvhVec3f::new(0.0, 0.0, 0.0), BvhVec3f::new(1.0, 1.0, 1.0));

    // Point inside box
    let mut aDist1: f32 = point_box_square_distance(&BvhVec3f::new(0.5, 0.5, 0.5), &aBox);
    assert_near_f32(aDist1, 0.0, 1e-5);

    // Point outside box
    let mut aDist2: f32 = point_box_square_distance(&BvhVec3f::new(2.0, 0.5, 0.5), &aBox);
    assert_near_f32(aDist2, 1.0, 1e-5);
}

#[test]
fn BVH_ToolsTest__RayBoxIntersectionFloat() {
    let mut aBox = BvhBox::from_corners(BvhVec3f::new(0.0, 0.0, 0.0), BvhVec3f::new(1.0, 1.0, 1.0));

    let (mut aTimeEnter, mut aTimeLeave) = (f32::NAN, f32::NAN);

    // Ray hitting the box
    let mut aHit = RayBoxIntersection(
        BvhVec3f::new(-1.0, 0.5, 0.5),
        BvhVec3f::new(1.0, 0.0, 0.0),
        &aBox,
        &mut aTimeEnter,
        &mut aTimeLeave,
    );
    assert!(aHit);
    assert_near_f32(aTimeEnter, 1.0, 1e-5);
    assert_near_f32(aTimeLeave, 2.0, 1e-5);
}

#[test]
fn BVH_ToolsTest__RayBoxIntersectionBehindRay() {
    let mut aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));

    let (mut aTimeEnter, mut aTimeLeave) = (f64::NAN, f64::NAN);

    // Ray pointing away from box
    let mut aHit = RayBoxIntersection(
        BvhVec3d::new(2.0, 0.5, 0.5),
        BvhVec3d::new(1.0, 0.0, 0.0),
        &aBox,
        &mut aTimeEnter,
        &mut aTimeLeave,
    );
    // The box is behind the ray origin
    assert!(aTimeLeave < 0.0 || !aHit);
}

#[test]
fn BVH_ToolsTest__RayBoxIntersectionYAxis() {
    let mut aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));

    let (mut aTimeEnter, mut aTimeLeave) = (f64::NAN, f64::NAN);

    // Ray along Y axis
    let mut aHit = RayBoxIntersection(
        BvhVec3d::new(0.5, -1.0, 0.5),
        BvhVec3d::new(0.0, 1.0, 0.0),
        &aBox,
        &mut aTimeEnter,
        &mut aTimeLeave,
    );
    assert!(aHit);
    assert_near(aTimeEnter, 1.0, precision::confusion());
    assert_near(aTimeLeave, 2.0, precision::confusion());
}

#[test]
fn BVH_ToolsTest__RayBoxIntersectionZAxis() {
    let mut aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));

    let (mut aTimeEnter, mut aTimeLeave) = (f64::NAN, f64::NAN);

    // Ray along Z axis
    let mut aHit = RayBoxIntersection(
        BvhVec3d::new(0.5, 0.5, -1.0),
        BvhVec3d::new(0.0, 0.0, 1.0),
        &aBox,
        &mut aTimeEnter,
        &mut aTimeLeave,
    );
    assert!(aHit);
    assert_near(aTimeEnter, 1.0, precision::confusion());
    assert_near(aTimeLeave, 2.0, precision::confusion());
}

#[test]
fn BVH_ToolsTest__BoxBoxSquareDistanceFloat() {
    let mut aBox1 = BvhBox::from_corners(BvhVec3f::new(0.0, 0.0, 0.0), BvhVec3f::new(1.0, 1.0, 1.0));
    let mut aBox2 = BvhBox::from_corners(BvhVec3f::new(2.0, 0.0, 0.0), BvhVec3f::new(3.0, 1.0, 1.0));

    let mut aDist: f32 = box_box_square_distance(&aBox1, &aBox2);
    assert_near_f32(aDist, 1.0, 1e-5);
}

#[test]
fn BVH_ToolsTest__PointBoxProjectionFloat() {
    let mut aBox = BvhBox::from_corners(BvhVec3f::new(0.0, 0.0, 0.0), BvhVec3f::new(1.0, 1.0, 1.0));

    let mut aProj = point_box_projection(&BvhVec3f::new(2.0, 0.5, 0.5), &aBox);
    assert_near_f32(aProj.x, 1.0, 1e-5);
    assert_near_f32(aProj.y, 0.5, 1e-5);
    assert_near_f32(aProj.z, 0.5, 1e-5);
}

#[test]
fn BVH_ToolsTest__PointBoxSquareDistanceNegativeBox() {
    let mut aBox =
        BvhBox::from_corners(BvhVec3d::new(-2.0, -2.0, -2.0), BvhVec3d::new(-1.0, -1.0, -1.0));

    // Point at origin
    let mut aDist: f64 = point_box_square_distance(&BvhVec3d::new(0.0, 0.0, 0.0), &aBox);
    // Distance = sqrt(1^2 + 1^2 + 1^2) = sqrt(3), squared = 3
    assert_near(aDist, 3.0, precision::confusion());
}

#[test]
fn BVH_ToolsTest__BoxBoxSquareDistanceEdgeToEdge() {
    let mut aBox1 = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));
    let mut aBox2 = BvhBox::from_corners(BvhVec3d::new(2.0, 2.0, 0.0), BvhVec3d::new(3.0, 3.0, 1.0));

    // Closest points are on edges, distance = sqrt(1^2 + 1^2) = sqrt(2), squared = 2
    let mut aDist: f64 = box_box_square_distance(&aBox1, &aBox2);
    assert_near(aDist, 2.0, precision::confusion());
}

#[test]
fn BVH_ToolsTest__PointTriangleProjectionFloat() {
    let mut aNode0 = BvhVec3f::new(0.0, 0.0, 0.0);
    let mut aNode1 = BvhVec3f::new(1.0, 0.0, 0.0);
    let mut aNode2 = BvhVec3f::new(0.0, 1.0, 0.0);

    // Point projects inside triangle
    let mut aProj =
        PointTriangleProjection(BvhVec3f::new(0.25, 0.25, 1.0), aNode0, aNode1, aNode2);
    assert_near_f32(aProj.x, 0.25, 1e-5);
    assert_near_f32(aProj.y, 0.25, 1e-5);
    assert_near_f32(aProj.z, 0.0, 1e-5);
}

#[test]
fn BVH_ToolsTest__PointBoxSquareDistanceLargeBox() {
    let mut aBox = BvhBox::from_corners(
        BvhVec3d::new(0.0, 0.0, 0.0),
        BvhVec3d::new(1000.0, 1000.0, 1000.0),
    );

    // Point inside
    let mut aDist1: f64 = point_box_square_distance(&BvhVec3d::new(500.0, 500.0, 500.0), &aBox);
    assert_near(aDist1, 0.0, precision::confusion());

    // Point outside
    let mut aDist2: f64 = point_box_square_distance(&BvhVec3d::new(1001.0, 500.0, 500.0), &aBox);
    assert_near(aDist2, 1.0, precision::confusion());
}

#[test]
fn BVH_ToolsTest__RayBoxIntersectionLargeBox() {
    let mut aBox = BvhBox::from_corners(
        BvhVec3d::new(0.0, 0.0, 0.0),
        BvhVec3d::new(1000.0, 1000.0, 1000.0),
    );

    let (mut aTimeEnter, mut aTimeLeave) = (f64::NAN, f64::NAN);

    let mut aHit = RayBoxIntersection(
        BvhVec3d::new(-100.0, 500.0, 500.0),
        BvhVec3d::new(1.0, 0.0, 0.0),
        &aBox,
        &mut aTimeEnter,
        &mut aTimeLeave,
    );
    assert!(aHit);
    assert_near(aTimeEnter, 100.0, precision::confusion());
    assert_near(aTimeLeave, 1100.0, precision::confusion());
}

#[test]
fn BVH_ToolsTest__PointTriangleProjectionCentroid() {
    let mut aNode0 = BvhVec3d::new(0.0, 0.0, 0.0);
    let mut aNode1 = BvhVec3d::new(3.0, 0.0, 0.0);
    let mut aNode2 = BvhVec3d::new(0.0, 3.0, 0.0);

    // Point directly above centroid
    let mut aProj = PointTriangleProjection(BvhVec3d::new(1.0, 1.0, 5.0), aNode0, aNode1, aNode2);
    assert_near(aProj.x, 1.0, precision::confusion());
    assert_near(aProj.y, 1.0, precision::confusion());
    assert_near(aProj.z, 0.0, precision::confusion());
}

#[test]
fn BVH_ToolsTest__BoxBoxSquareDistanceSameBox() {
    let mut aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));

    // Distance to itself should be 0
    let mut aDist: f64 = box_box_square_distance(&aBox, &aBox);
    assert_near(aDist, 0.0, precision::confusion());
}

#[test]
fn BVH_ToolsTest__PointBoxProjectionAllCorners() {
    let mut aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));

    // Test projection from each octant
    let mut aProj1 = point_box_projection(&BvhVec3d::new(-1.0, -1.0, -1.0), &aBox);
    assert_near(aProj1.x, 0.0, precision::confusion());
    assert_near(aProj1.y, 0.0, precision::confusion());
    assert_near(aProj1.z, 0.0, precision::confusion());

    let mut aProj2 = point_box_projection(&BvhVec3d::new(2.0, 2.0, 2.0), &aBox);
    assert_near(aProj2.x, 1.0, precision::confusion());
    assert_near(aProj2.y, 1.0, precision::confusion());
    assert_near(aProj2.z, 1.0, precision::confusion());
}

// =======================================================================================
// Tests for improved RayBoxIntersection (dimension-independent, early exit)
// =======================================================================================

#[test]
fn BVH_ToolsTest__RayBoxIntersection2D() {
    // Test 2D ray-box intersection (old code was hardcoded for 3D)
    let mut aBox = BvhBox::from_corners(BvhVec2d::new(0.0, 0.0), BvhVec2d::new(1.0, 1.0));

    let (mut aTimeEnter, mut aTimeLeave) = (f64::NAN, f64::NAN);

    // Ray hitting the box
    let mut aHit1 = RayBoxIntersection(
        BvhVec2d::new(-1.0, 0.5),
        BvhVec2d::new(1.0, 0.0),
        &aBox,
        &mut aTimeEnter,
        &mut aTimeLeave,
    );
    assert!(aHit1);
    assert_near(aTimeEnter, 1.0, precision::confusion());
    assert_near(aTimeLeave, 2.0, precision::confusion());

    // Ray missing the box
    let mut aHit2 = RayBoxIntersection(
        BvhVec2d::new(-1.0, 2.0),
        BvhVec2d::new(1.0, 0.0),
        &aBox,
        &mut aTimeEnter,
        &mut aTimeLeave,
    );
    assert!(!aHit2);

    // Ray parallel to X axis, inside Y bounds
    let mut aHit3 = RayBoxIntersection(
        BvhVec2d::new(-1.0, 0.5),
        BvhVec2d::new(1.0, 0.0),
        &aBox,
        &mut aTimeEnter,
        &mut aTimeLeave,
    );
    assert!(aHit3);

    // Ray parallel to Y axis, inside X bounds
    let mut aHit4 = RayBoxIntersection(
        BvhVec2d::new(0.5, -1.0),
        BvhVec2d::new(0.0, 1.0),
        &aBox,
        &mut aTimeEnter,
        &mut aTimeLeave,
    );
    assert!(aHit4);
    assert_near(aTimeEnter, 1.0, precision::confusion());
    assert_near(aTimeLeave, 2.0, precision::confusion());
}

#[test]
fn BVH_ToolsTest__RayBoxIntersectionEarlyExit() {
    // Test early exit optimization when ray misses on first axis
    let mut aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));
    let (mut aTimeEnter, mut aTimeLeave) = (f64::NAN, f64::NAN);

    // Ray clearly misses in X direction - should exit immediately
    let mut aHit = RayBoxIntersection(
        BvhVec3d::new(-2.0, 0.5, 0.5),
        BvhVec3d::new(-1.0, 0.0, 0.0),
        &aBox,
        &mut aTimeEnter,
        &mut aTimeLeave,
    );
    assert!(!aHit);

    // Ray with early mismatch in Y direction
    let mut aHit2 = RayBoxIntersection(
        BvhVec3d::new(0.5, -2.0, 0.5),
        BvhVec3d::new(0.0, -1.0, 0.0),
        &aBox,
        &mut aTimeEnter,
        &mut aTimeLeave,
    );
    assert!(!aHit2);
}

#[test]
fn BVH_ToolsTest__RayBoxIntersectionParallelRayEarlyExit() {
    // Test parallel ray that misses - should exit immediately
    let mut aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));
    let (mut aTimeEnter, mut aTimeLeave) = (f64::NAN, f64::NAN);

    // Ray parallel to X axis but Y coordinate outside box - should reject immediately
    let mut aHit = RayBoxIntersection(
        BvhVec3d::new(-1.0, 2.0, 0.5),
        BvhVec3d::new(1.0, 0.0, 0.0),
        &aBox,
        &mut aTimeEnter,
        &mut aTimeLeave,
    );
    assert!(!aHit);

    // Ray parallel to Y axis but X coordinate outside box
    let mut aHit2 = RayBoxIntersection(
        BvhVec3d::new(2.0, -1.0, 0.5),
        BvhVec3d::new(0.0, 1.0, 0.0),
        &aBox,
        &mut aTimeEnter,
        &mut aTimeLeave,
    );
    assert!(!aHit2);

    // Ray parallel to Z axis but X and Y outside
    let mut aHit3 = RayBoxIntersection(
        BvhVec3d::new(-1.0, -1.0, 0.5),
        BvhVec3d::new(0.0, 0.0, 1.0),
        &aBox,
        &mut aTimeEnter,
        &mut aTimeLeave,
    );
    assert!(!aHit3);
}

#[test]
fn BVH_ToolsTest__RayBoxIntersection2DParallelBothAxes() {
    // 2D test with ray parallel to both axes (direction = 0,0)
    let mut aBox = BvhBox::from_corners(BvhVec2d::new(0.0, 0.0), BvhVec2d::new(1.0, 1.0));
    let (mut aTimeEnter, mut aTimeLeave) = (f64::NAN, f64::NAN);

    // Ray origin inside box, direction = 0
    let mut aHit1 = RayBoxIntersection(
        BvhVec2d::new(0.5, 0.5),
        BvhVec2d::new(0.0, 0.0),
        &aBox,
        &mut aTimeEnter,
        &mut aTimeLeave,
    );
    assert!(aHit1); // Should still hit because origin is inside

    // Ray origin outside box, direction = 0
    let mut aHit2 = RayBoxIntersection(
        BvhVec2d::new(2.0, 2.0),
        BvhVec2d::new(0.0, 0.0),
        &aBox,
        &mut aTimeEnter,
        &mut aTimeLeave,
    );
    assert!(!aHit2); // Should miss because origin is outside
}

#[test]
fn BVH_ToolsTest__RayBoxIntersectionNegativeTime() {
    // Test that ray doesn't report intersection behind the origin
    let mut aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));
    let (mut aTimeEnter, mut aTimeLeave) = (f64::NAN, f64::NAN);

    // Ray origin is past the box, pointing away
    let mut aHit = RayBoxIntersection(
        BvhVec3d::new(2.0, 0.5, 0.5),
        BvhVec3d::new(1.0, 0.0, 0.0),
        &aBox,
        &mut aTimeEnter,
        &mut aTimeLeave,
    );
    assert!(!aHit);

    // Ray origin inside box, pointing away - should still hit (leave point is in front)
    let mut aHit2 = RayBoxIntersection(
        BvhVec3d::new(0.5, 0.5, 0.5),
        BvhVec3d::new(1.0, 0.0, 0.0),
        &aBox,
        &mut aTimeEnter,
        &mut aTimeLeave,
    );
    assert!(aHit2);
    assert!(aTimeLeave >= 0.0);
}
