#[test]
fn BVH_ToolsTest__PointBoxSquareDistance() {
    let mut aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));

    // Point inside box
    let mut aDist1: f64 = point_box_square_distance(&BvhVec3d::new(0.5, 0.5, 0.5), &aBox);
    assert_near(aDist1, 0.0, CONFUSION);

    // Point on face
    let mut aDist2: f64 = point_box_square_distance(&BvhVec3d::new(0.5, 0.5, 1.0), &aBox);
    assert_near(aDist2, 0.0, CONFUSION);

    // Point outside box (distance = 1)
    let mut aDist3: f64 = point_box_square_distance(&BvhVec3d::new(2.0, 0.5, 0.5), &aBox);
    assert_near(aDist3, 1.0, CONFUSION);

    // Point at corner outside (distance^2 = 1 + 1 + 1 = 3)
    let mut aDist4: f64 = point_box_square_distance(&BvhVec3d::new(2.0, 2.0, 2.0), &aBox);
    assert_near(aDist4, 3.0, CONFUSION);
}

#[test]
fn BVH_ToolsTest__BoxBoxSquareDistance() {
    let mut aBox1 = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));
    let mut aBox2 = BvhBox::from_corners(BvhVec3d::new(2.0, 0.0, 0.0), BvhVec3d::new(3.0, 1.0, 1.0));
    let mut aBox3 = BvhBox::from_corners(BvhVec3d::new(0.5, 0.5, 0.5), BvhVec3d::new(1.5, 1.5, 1.5));

    // Separated boxes
    let mut aDist1: f64 = box_box_square_distance(&aBox1, &aBox2);
    assert_near(aDist1, 1.0, CONFUSION);

    // Overlapping boxes
    let mut aDist2: f64 = box_box_square_distance(&aBox1, &aBox3);
    assert_near(aDist2, 0.0, CONFUSION);
}

#[test]
fn BVH_ToolsTest__PointBoxProjection() {
    let mut aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));

    // Point inside - should return same point
    let mut aProj1 = point_box_projection(&BvhVec3d::new(0.5, 0.5, 0.5), &aBox);
    assert_near(aProj1.x, 0.5, CONFUSION);
    assert_near(aProj1.y, 0.5, CONFUSION);
    assert_near(aProj1.z, 0.5, CONFUSION);

    // Point outside - should clamp to box
    let mut aProj2 = point_box_projection(&BvhVec3d::new(2.0, 0.5, 0.5), &aBox);
    assert_near(aProj2.x, 1.0, CONFUSION);
    assert_near(aProj2.y, 0.5, CONFUSION);
    assert_near(aProj2.z, 0.5, CONFUSION);
}

#[test]
fn BVH_ToolsTest__RayBoxIntersection() {
    let mut aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));

    let (mut aTimeEnter, mut aTimeLeave) = (f64::NAN, f64::NAN);

    // Ray hitting the box
    let mut aHit1 = RayBoxIntersection(
        BvhVec3d::new(-1.0, 0.5, 0.5),
        BvhVec3d::new(1.0, 0.0, 0.0),
        &aBox,
        &mut aTimeEnter,
        &mut aTimeLeave,
    );
    assert!(aHit1);
    assert_near(aTimeEnter, 1.0, CONFUSION);
    assert_near(aTimeLeave, 2.0, CONFUSION);

    // Ray missing the box
    let mut aHit2 = RayBoxIntersection(
        BvhVec3d::new(-1.0, 5.0, 0.5),
        BvhVec3d::new(1.0, 0.0, 0.0),
        &aBox,
        &mut aTimeEnter,
        &mut aTimeLeave,
    );
    assert!(!aHit2);

    // Ray starting inside box
    let mut aHit3 = RayBoxIntersection(
        BvhVec3d::new(0.5, 0.5, 0.5),
        BvhVec3d::new(1.0, 0.0, 0.0),
        &aBox,
        &mut aTimeEnter,
        &mut aTimeLeave,
    );
    assert!(aHit3);
    assert!(aTimeEnter <= 0.0);
    assert_near(aTimeLeave, 0.5, CONFUSION);
}

#[test]
fn BVH_ToolsTest__RayBoxIntersectionParallelRay() {
    let mut aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));

    let (mut aTimeEnter, mut aTimeLeave) = (f64::NAN, f64::NAN);

    // Ray parallel to X-axis, passing through box
    let mut aHit1 = RayBoxIntersection(
        BvhVec3d::new(-1.0, 0.5, 0.5),
        BvhVec3d::new(1.0, 0.0, 0.0),
        &aBox,
        &mut aTimeEnter,
        &mut aTimeLeave,
    );
    assert!(aHit1);

    // Ray parallel to X-axis, missing box (Y out of range)
    let mut aHit2 = RayBoxIntersection(
        BvhVec3d::new(-1.0, 2.0, 0.5),
        BvhVec3d::new(1.0, 0.0, 0.0),
        &aBox,
        &mut aTimeEnter,
        &mut aTimeLeave,
    );
    assert!(!aHit2);
}

#[test]
fn BVH_ToolsTest__PointTriangleProjection() {
    let mut aNode0 = BvhVec3d::new(0.0, 0.0, 0.0);
    let mut aNode1 = BvhVec3d::new(1.0, 0.0, 0.0);
    let mut aNode2 = BvhVec3d::new(0.0, 1.0, 0.0);

    // Point projects to vertex
    let mut aProj1 =
        PointTriangleProjection(BvhVec3d::new(-1.0, -1.0, 0.0), aNode0, aNode1, aNode2);
    assert_near(aProj1.x, 0.0, CONFUSION);
    assert_near(aProj1.y, 0.0, CONFUSION);
    assert_near(aProj1.z, 0.0, CONFUSION);

    // Point projects to edge
    let mut aProj2 = PointTriangleProjection(BvhVec3d::new(0.5, -1.0, 0.0), aNode0, aNode1, aNode2);
    assert_near(aProj2.x, 0.5, CONFUSION);
    assert_near(aProj2.y, 0.0, CONFUSION);
    assert_near(aProj2.z, 0.0, CONFUSION);

    // Point projects inside triangle
    let mut aProj3 =
        PointTriangleProjection(BvhVec3d::new(0.25, 0.25, 1.0), aNode0, aNode1, aNode2);
    assert_near(aProj3.x, 0.25, CONFUSION);
    assert_near(aProj3.y, 0.25, CONFUSION);
    assert_near(aProj3.z, 0.0, CONFUSION);
}

#[test]
fn BVH_ToolsTest__PointBoxSquareDistance2D() {
    let mut aBox = BvhBox::from_corners(BvhVec2d::new(0.0, 0.0), BvhVec2d::new(1.0, 1.0));

    // Point inside box
    let mut aDist1: f64 = point_box_square_distance(&BvhVec2d::new(0.5, 0.5), &aBox);
    assert_near(aDist1, 0.0, CONFUSION);

    // Point outside box
    let mut aDist2: f64 = point_box_square_distance(&BvhVec2d::new(2.0, 0.5), &aBox);
    assert_near(aDist2, 1.0, CONFUSION);
}

#[test]
fn BVH_ToolsTest__RayBoxIntersectionDiagonalRay() {
    let mut aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));

    let (mut aTimeEnter, mut aTimeLeave) = (f64::NAN, f64::NAN);

    // Diagonal ray through box center
    let mut aDir = BvhVec3d::new(1.0, 1.0, 1.0);
    let mut aNorm: f64 = 3.0_f64.sqrt();
    aDir = BvhVec3d::new(aDir.x / aNorm, aDir.y / aNorm, aDir.z / aNorm);

    let mut aHit = RayBoxIntersection(
        BvhVec3d::new(-1.0, -1.0, -1.0),
        aDir,
        &aBox,
        &mut aTimeEnter,
        &mut aTimeLeave,
    );
    assert!(aHit);
    assert!(aTimeEnter > 0.0);
    assert!(aTimeLeave > aTimeEnter);
}

#[test]
fn BVH_ToolsTest__RayBoxIntersectionNegativeDirection() {
    let mut aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));

    let (mut aTimeEnter, mut aTimeLeave) = (f64::NAN, f64::NAN);

    // Ray going in negative X direction
    let mut aHit = RayBoxIntersection(
        BvhVec3d::new(2.0, 0.5, 0.5),
        BvhVec3d::new(-1.0, 0.0, 0.0),
        &aBox,
        &mut aTimeEnter,
        &mut aTimeLeave,
    );
    assert!(aHit);
    assert_near(aTimeEnter, 1.0, CONFUSION);
    assert_near(aTimeLeave, 2.0, CONFUSION);
}

#[test]
fn BVH_ToolsTest__RayBoxIntersectionTouchingEdge() {
    let mut aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));

    let (mut aTimeEnter, mut aTimeLeave) = (f64::NAN, f64::NAN);

    // Ray touching edge of box
    let mut aHit = RayBoxIntersection(
        BvhVec3d::new(-1.0, 0.0, 0.0),
        BvhVec3d::new(1.0, 0.0, 0.0),
        &aBox,
        &mut aTimeEnter,
        &mut aTimeLeave,
    );
    assert!(aHit);
}

#[test]
fn BVH_ToolsTest__RayBoxIntersectionTouchingCorner() {
    let mut aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));

    let (mut aTimeEnter, mut aTimeLeave) = (f64::NAN, f64::NAN);

    // Ray through corner
    let mut aDir = BvhVec3d::new(1.0, 1.0, 1.0);
    let mut aNorm: f64 = 3.0_f64.sqrt();
    aDir = BvhVec3d::new(aDir.x / aNorm, aDir.y / aNorm, aDir.z / aNorm);

    let mut aHit = RayBoxIntersection(
        BvhVec3d::new(-1.0, -1.0, -1.0),
        aDir,
        &aBox,
        &mut aTimeEnter,
        &mut aTimeLeave,
    );
    assert!(aHit);
}

#[test]
fn BVH_ToolsTest__BoxBoxSquareDistanceTouching() {
    let mut aBox1 = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));
    let mut aBox2 = BvhBox::from_corners(BvhVec3d::new(1.0, 0.0, 0.0), BvhVec3d::new(2.0, 1.0, 1.0));

    // Touching boxes (sharing a face)
    let mut aDist: f64 = box_box_square_distance(&aBox1, &aBox2);
    assert_near(aDist, 0.0, CONFUSION);
}

#[test]
fn BVH_ToolsTest__BoxBoxSquareDistanceOneInsideOther() {
    let mut aBox1 =
        BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(10.0, 10.0, 10.0));
    let mut aBox2 = BvhBox::from_corners(BvhVec3d::new(2.0, 2.0, 2.0), BvhVec3d::new(3.0, 3.0, 3.0));

    // Small box inside large box
    let mut aDist: f64 = box_box_square_distance(&aBox1, &aBox2);
    assert_near(aDist, 0.0, CONFUSION);
}

#[test]
fn BVH_ToolsTest__BoxBoxSquareDistanceCornerToCorner() {
    let mut aBox1 = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));
    let mut aBox2 = BvhBox::from_corners(BvhVec3d::new(2.0, 2.0, 2.0), BvhVec3d::new(3.0, 3.0, 3.0));

    // Distance from corner to corner: sqrt(1^2 + 1^2 + 1^2) = sqrt(3), squared = 3
    let mut aDist: f64 = box_box_square_distance(&aBox1, &aBox2);
    assert_near(aDist, 3.0, CONFUSION);
}

#[test]
fn BVH_ToolsTest__PointBoxProjectionNegativeCoords() {
    let mut aBox =
        BvhBox::from_corners(BvhVec3d::new(-1.0, -1.0, -1.0), BvhVec3d::new(1.0, 1.0, 1.0));

    // Point far outside in negative direction
    let mut aProj = point_box_projection(&BvhVec3d::new(-5.0, -5.0, -5.0), &aBox);
    assert_near(aProj.x, -1.0, CONFUSION);
    assert_near(aProj.y, -1.0, CONFUSION);
    assert_near(aProj.z, -1.0, CONFUSION);
}

#[test]
fn BVH_ToolsTest__PointTriangleProjectionOnEdge01() {
    let mut aNode0 = BvhVec3d::new(0.0, 0.0, 0.0);
    let mut aNode1 = BvhVec3d::new(2.0, 0.0, 0.0);
    let mut aNode2 = BvhVec3d::new(0.0, 2.0, 0.0);

    // Point projects onto edge between Node0 and Node1
    let mut aProj = PointTriangleProjection(BvhVec3d::new(1.0, -1.0, 0.0), aNode0, aNode1, aNode2);
    assert_near(aProj.x, 1.0, CONFUSION);
    assert_near(aProj.y, 0.0, CONFUSION);
    assert_near(aProj.z, 0.0, CONFUSION);
}

#[test]
fn BVH_ToolsTest__PointTriangleProjectionOnEdge12() {
    let mut aNode0 = BvhVec3d::new(0.0, 0.0, 0.0);
    let mut aNode1 = BvhVec3d::new(2.0, 0.0, 0.0);
    let mut aNode2 = BvhVec3d::new(0.0, 2.0, 0.0);

    // Point projects onto edge between Node1 and Node2
    let mut aProj = PointTriangleProjection(BvhVec3d::new(2.0, 2.0, 0.0), aNode0, aNode1, aNode2);
    assert_near(aProj.x, 1.0, CONFUSION);
    assert_near(aProj.y, 1.0, CONFUSION);
    assert_near(aProj.z, 0.0, CONFUSION);
}

#[test]
fn BVH_ToolsTest__PointTriangleProjectionOnEdge20() {
    let mut aNode0 = BvhVec3d::new(0.0, 0.0, 0.0);
    let mut aNode1 = BvhVec3d::new(2.0, 0.0, 0.0);
    let mut aNode2 = BvhVec3d::new(0.0, 2.0, 0.0);

    // Point projects onto edge between Node2 and Node0
    let mut aProj = PointTriangleProjection(BvhVec3d::new(-1.0, 1.0, 0.0), aNode0, aNode1, aNode2);
    assert_near(aProj.x, 0.0, CONFUSION);
    assert_near(aProj.y, 1.0, CONFUSION);
    assert_near(aProj.z, 0.0, CONFUSION);
}

#[test]
fn BVH_ToolsTest__PointTriangleProjectionVertex1() {
    let mut aNode0 = BvhVec3d::new(0.0, 0.0, 0.0);
    let mut aNode1 = BvhVec3d::new(2.0, 0.0, 0.0);
    let mut aNode2 = BvhVec3d::new(0.0, 2.0, 0.0);

    // Point projects to Node1
    let mut aProj = PointTriangleProjection(BvhVec3d::new(3.0, -1.0, 0.0), aNode0, aNode1, aNode2);
    assert_near(aProj.x, 2.0, CONFUSION);
    assert_near(aProj.y, 0.0, CONFUSION);
    assert_near(aProj.z, 0.0, CONFUSION);
}

#[test]
fn BVH_ToolsTest__PointTriangleProjectionVertex2() {
    let mut aNode0 = BvhVec3d::new(0.0, 0.0, 0.0);
    let mut aNode1 = BvhVec3d::new(2.0, 0.0, 0.0);
    let mut aNode2 = BvhVec3d::new(0.0, 2.0, 0.0);

    // Point projects to Node2
    let mut aProj = PointTriangleProjection(BvhVec3d::new(-1.0, 3.0, 0.0), aNode0, aNode1, aNode2);
    assert_near(aProj.x, 0.0, CONFUSION);
    assert_near(aProj.y, 2.0, CONFUSION);
    assert_near(aProj.z, 0.0, CONFUSION);
}

#[test]
fn BVH_ToolsTest__PointTriangleProjection3D() {
    // Triangle in 3D space (not in XY plane)
    let mut aNode0 = BvhVec3d::new(0.0, 0.0, 0.0);
    let mut aNode1 = BvhVec3d::new(1.0, 0.0, 0.0);
    let mut aNode2 = BvhVec3d::new(0.0, 1.0, 1.0);

    // Point above triangle center
    let mut aProj = PointTriangleProjection(BvhVec3d::new(0.3, 0.3, 0.8), aNode0, aNode1, aNode2);

    // Should project inside triangle
    assert!(aProj.x >= 0.0);
    assert!(aProj.y >= 0.0);
    assert!(aProj.x + aProj.y <= 1.0 + CONFUSION);
}
