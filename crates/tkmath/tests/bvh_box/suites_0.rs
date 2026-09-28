#[test]
fn BVH_BoxTest__DefaultConstructor() {
    let mut aBox = BvhBox::<BvhVec3d>::default();
    assert!(!aBox.is_valid());
}

#[test]
fn BVH_BoxTest__ConstructorWithCorners() {
    let mut aMin = BvhVec3d::new(0.0, 0.0, 0.0);
    let mut aMax = BvhVec3d::new(1.0, 2.0, 3.0);

    let mut aBox = BvhBox::from_corners(aMin, aMax);

    assert!(aBox.is_valid());
    assert_near(aBox.corner_min().x, 0.0, precision::confusion());
    assert_near(aBox.corner_min().y, 0.0, precision::confusion());
    assert_near(aBox.corner_min().z, 0.0, precision::confusion());
    assert_near(aBox.corner_max().x, 1.0, precision::confusion());
    assert_near(aBox.corner_max().y, 2.0, precision::confusion());
    assert_near(aBox.corner_max().z, 3.0, precision::confusion());
}

#[test]
fn BVH_BoxTest__Add() {
    let mut aBox = BvhBox::<BvhVec3d>::default();

    aBox.add(&BvhVec3d::new(1.0, 2.0, 3.0));
    assert!(aBox.is_valid());

    aBox.add(&BvhVec3d::new(-1.0, -2.0, -3.0));

    assert_near(aBox.corner_min().x, -1.0, precision::confusion());
    assert_near(aBox.corner_min().y, -2.0, precision::confusion());
    assert_near(aBox.corner_min().z, -3.0, precision::confusion());
    assert_near(aBox.corner_max().x, 1.0, precision::confusion());
    assert_near(aBox.corner_max().y, 2.0, precision::confusion());
    assert_near(aBox.corner_max().z, 3.0, precision::confusion());
}

#[test]
fn BVH_BoxTest__Combine() {
    let mut aBox1 = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));
    let mut aBox2 = BvhBox::from_corners(BvhVec3d::new(2.0, 2.0, 2.0), BvhVec3d::new(3.0, 3.0, 3.0));

    aBox1.combine(&aBox2);

    assert_near(aBox1.corner_min().x, 0.0, precision::confusion());
    assert_near(aBox1.corner_min().y, 0.0, precision::confusion());
    assert_near(aBox1.corner_min().z, 0.0, precision::confusion());
    assert_near(aBox1.corner_max().x, 3.0, precision::confusion());
    assert_near(aBox1.corner_max().y, 3.0, precision::confusion());
    assert_near(aBox1.corner_max().z, 3.0, precision::confusion());
}

#[test]
fn BVH_BoxTest__Size() {
    let mut aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(2.0, 3.0, 4.0));

    let mut aSize = aBox.size();
    assert_near(aSize.x, 2.0, precision::confusion());
    assert_near(aSize.y, 3.0, precision::confusion());
    assert_near(aSize.z, 4.0, precision::confusion());
}

#[test]
fn BVH_BoxTest__Center() {
    let mut aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(2.0, 4.0, 6.0));

    let mut aCenter = aBox.center();
    assert_near(aCenter.x, 1.0, precision::confusion());
    assert_near(aCenter.y, 2.0, precision::confusion());
    assert_near(aCenter.z, 3.0, precision::confusion());
}

#[test]
fn BVH_BoxTest__Area() {
    let mut aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 2.0, 3.0));

    // Surface area = 2 * (1*2 + 2*3 + 1*3) = 2 * (2 + 6 + 3) = 22
    let mut anArea: f64 = aBox.area();
    assert_near(anArea, 22.0, precision::confusion());
}

#[test]
fn BVH_BoxTest__IsOut() {
    let mut aBox1 = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));
    let mut aBox2 = BvhBox::from_corners(BvhVec3d::new(2.0, 2.0, 2.0), BvhVec3d::new(3.0, 3.0, 3.0));
    let mut aBox3 = BvhBox::from_corners(BvhVec3d::new(0.5, 0.5, 0.5), BvhVec3d::new(1.5, 1.5, 1.5));

    assert!(aBox1.is_out(&aBox2));
    assert!(!aBox1.is_out(&aBox3));
}

#[test]
fn BVH_BoxTest__Clear() {
    let mut aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));
    assert!(aBox.is_valid());

    aBox.clear();
    assert!(!aBox.is_valid());
}

#[test]
fn BVH_BoxTest__Box2D() {
    let mut aBox = BvhBox::<BvhVec2d>::default();

    aBox.add(&BvhVec2d::new(0.0, 0.0));
    aBox.add(&BvhVec2d::new(1.0, 1.0));

    assert!(aBox.is_valid());
    assert_near(aBox.corner_min().x, 0.0, precision::confusion());
    assert_near(aBox.corner_min().y, 0.0, precision::confusion());
    assert_near(aBox.corner_max().x, 1.0, precision::confusion());
    assert_near(aBox.corner_max().y, 1.0, precision::confusion());

    // Area in 2D is width * height (actual area)
    let mut anArea: f64 = aBox.area();
    assert_near(anArea, 1.0, precision::confusion()); // 1 * 1 = 1
}

#[test]
fn BVH_BoxTest__Box4D() {
    let mut aBox = BvhBox::<BvhVec4d>::default();

    aBox.add(&BvhVec4d::new(0.0, 0.0, 0.0, 0.0));
    aBox.add(&BvhVec4d::new(1.0, 2.0, 3.0, 4.0));

    assert!(aBox.is_valid());
    assert_near(aBox.corner_min().x, 0.0, precision::confusion());
    assert_near(aBox.corner_min().y, 0.0, precision::confusion());
    assert_near(aBox.corner_min().z, 0.0, precision::confusion());
    assert_near(aBox.corner_min().w, 0.0, precision::confusion());
    assert_near(aBox.corner_max().x, 1.0, precision::confusion());
    assert_near(aBox.corner_max().y, 2.0, precision::confusion());
    assert_near(aBox.corner_max().z, 3.0, precision::confusion());
    assert_near(aBox.corner_max().w, 4.0, precision::confusion());
}

#[test]
fn BVH_BoxTest__FloatPrecision() {
    let mut aBox = BvhBox::<BvhVec3f>::default();

    aBox.add(&BvhVec3f::new(0.0, 0.0, 0.0));
    aBox.add(&BvhVec3f::new(1.0, 2.0, 3.0));

    assert!(aBox.is_valid());
    assert_near_f32(aBox.corner_min().x, 0.0, 1e-5);
    assert_near_f32(aBox.corner_max().x, 1.0, 1e-5);
    assert_near_f32(aBox.corner_max().y, 2.0, 1e-5);
    assert_near_f32(aBox.corner_max().z, 3.0, 1e-5);
}

#[test]
fn BVH_BoxTest__SinglePointBox() {
    let mut aBox = BvhBox::<BvhVec3d>::default();

    aBox.add(&BvhVec3d::new(5.0, 5.0, 5.0));

    assert!(aBox.is_valid());
    assert_near(aBox.corner_min().x, 5.0, precision::confusion());
    assert_near(aBox.corner_max().x, 5.0, precision::confusion());

    // Size should be zero
    let mut aSize = aBox.size();
    assert_near(aSize.x, 0.0, precision::confusion());
    assert_near(aSize.y, 0.0, precision::confusion());
    assert_near(aSize.z, 0.0, precision::confusion());

    // Area should be zero
    assert_near(aBox.area(), 0.0, precision::confusion());
}

#[test]
fn BVH_BoxTest__NegativeCoordinates() {
    let mut aBox = BvhBox::from_corners(
        BvhVec3d::new(-10.0, -20.0, -30.0),
        BvhVec3d::new(-5.0, -10.0, -15.0),
    );

    assert!(aBox.is_valid());

    let mut aCenter = aBox.center();
    assert_near(aCenter.x, -7.5, precision::confusion());
    assert_near(aCenter.y, -15.0, precision::confusion());
    assert_near(aCenter.z, -22.5, precision::confusion());

    let mut aSize = aBox.size();
    assert_near(aSize.x, 5.0, precision::confusion());
    assert_near(aSize.y, 10.0, precision::confusion());
    assert_near(aSize.z, 15.0, precision::confusion());
}

#[test]
fn BVH_BoxTest__LargeValues() {
    let mut aBox = BvhBox::from_corners(
        BvhVec3d::new(1e10, 1e10, 1e10),
        BvhVec3d::new(1e10 + 1.0, 1e10 + 2.0, 1e10 + 3.0),
    );

    assert!(aBox.is_valid());

    let mut aSize = aBox.size();
    assert_near(aSize.x, 1.0, 1e-5);
    assert_near(aSize.y, 2.0, 1e-5);
    assert_near(aSize.z, 3.0, 1e-5);
}

#[test]
fn BVH_BoxTest__CombineWithInvalid() {
    let mut aBox1 = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));
    let mut aBox2 = BvhBox::<BvhVec3d>::default(); // Invalid box

    aBox1.combine(&aBox2);

    // Box1 should remain unchanged when combining with invalid box
    assert!(aBox1.is_valid());
    assert_near(aBox1.corner_min().x, 0.0, precision::confusion());
    assert_near(aBox1.corner_max().x, 1.0, precision::confusion());
}

#[test]
fn BVH_BoxTest__AddToInvalid() {
    let mut aBox1 = BvhBox::<BvhVec3d>::default(); // Invalid box
    let mut aBox2 = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));

    aBox1.combine(&aBox2);

    // Box1 should now be valid and equal to aBox2
    assert!(aBox1.is_valid());
    assert_near(aBox1.corner_min().x, 0.0, precision::confusion());
    assert_near(aBox1.corner_max().x, 1.0, precision::confusion());
}

#[test]
fn BVH_BoxTest__IsOutTouchingBoxes() {
    // Boxes that touch at a face
    let mut aBox1 = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));
    let mut aBox2 = BvhBox::from_corners(BvhVec3d::new(1.0, 0.0, 0.0), BvhVec3d::new(2.0, 1.0, 1.0));

    // Touching boxes should NOT be "out"
    assert!(!aBox1.is_out(&aBox2));
}

#[test]
fn BVH_BoxTest__IsOutTouchingAtEdge() {
    // Boxes that touch at an edge
    let mut aBox1 = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));
    let mut aBox2 = BvhBox::from_corners(BvhVec3d::new(1.0, 1.0, 0.0), BvhVec3d::new(2.0, 2.0, 1.0));

    // Touching at edge should NOT be "out"
    assert!(!aBox1.is_out(&aBox2));
}

#[test]
fn BVH_BoxTest__IsOutTouchingAtCorner() {
    // Boxes that touch at a corner
    let mut aBox1 = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));
    let mut aBox2 = BvhBox::from_corners(BvhVec3d::new(1.0, 1.0, 1.0), BvhVec3d::new(2.0, 2.0, 2.0));

    // Touching at corner should NOT be "out"
    assert!(!aBox1.is_out(&aBox2));
}

#[test]
fn BVH_BoxTest__AreaUnitCube() {
    let mut aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));

    // Surface area of unit cube = 6 faces * 1 = 6
    // Formula: 2 * (xy + yz + xz) = 2 * (1 + 1 + 1) = 6
    assert_near(aBox.area(), 6.0, precision::confusion());
}

#[test]
fn BVH_BoxTest__CenterAtOrigin() {
    let mut aBox =
        BvhBox::from_corners(BvhVec3d::new(-1.0, -1.0, -1.0), BvhVec3d::new(1.0, 1.0, 1.0));

    let mut aCenter = aBox.center();
    assert_near(aCenter.x, 0.0, precision::confusion());
    assert_near(aCenter.y, 0.0, precision::confusion());
    assert_near(aCenter.z, 0.0, precision::confusion());
}

#[test]
fn BVH_BoxTest__MultipleAdds() {
    let mut aBox = BvhBox::<BvhVec3d>::default();

    // Add points in random order
    aBox.add(&BvhVec3d::new(5.0, 3.0, 1.0));
    aBox.add(&BvhVec3d::new(-2.0, 7.0, 4.0));
    aBox.add(&BvhVec3d::new(1.0, -1.0, 8.0));
    aBox.add(&BvhVec3d::new(0.0, 2.0, -3.0));

    assert_near(aBox.corner_min().x, -2.0, precision::confusion());
    assert_near(aBox.corner_min().y, -1.0, precision::confusion());
    assert_near(aBox.corner_min().z, -3.0, precision::confusion());
    assert_near(aBox.corner_max().x, 5.0, precision::confusion());
    assert_near(aBox.corner_max().y, 7.0, precision::confusion());
    assert_near(aBox.corner_max().z, 8.0, precision::confusion());
}

#[test]
fn BVH_BoxTest__FlatBox2D() {
    // Box with zero extent in Z (flat)
    let mut aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 5.0), BvhVec3d::new(3.0, 4.0, 5.0));

    assert!(aBox.is_valid());

    let mut aSize = aBox.size();
    assert_near(aSize.x, 3.0, precision::confusion());
    assert_near(aSize.y, 4.0, precision::confusion());
    assert_near(aSize.z, 0.0, precision::confusion());

    // Area = 2 * (3*4 + 4*0 + 3*0) = 24
    assert_near(aBox.area(), 24.0, precision::confusion());
}

#[test]
fn BVH_BoxTest__FlatBox1D() {
    // Box with zero extent in Y and Z (line)
    let mut aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 5.0, 5.0), BvhVec3d::new(10.0, 5.0, 5.0));

    assert!(aBox.is_valid());

    let mut aSize = aBox.size();
    assert_near(aSize.x, 10.0, precision::confusion());
    assert_near(aSize.y, 0.0, precision::confusion());
    assert_near(aSize.z, 0.0, precision::confusion());

    // For degenerate box (line), Area returns length of longest dimension
    // This is an implementation detail - the surface area formula gives 0,
    // but Area() returns the length (10) for degenerate cases
    let mut anArea: f64 = aBox.area();
    assert!(anArea >= 0.0);
}

#[test]
fn BVH_BoxTest__CombineMultiple() {
    let mut aBox1 = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));
    let mut aBox2 = BvhBox::from_corners(BvhVec3d::new(2.0, 0.0, 0.0), BvhVec3d::new(3.0, 1.0, 1.0));
    let mut aBox3 = BvhBox::from_corners(BvhVec3d::new(0.0, 2.0, 0.0), BvhVec3d::new(1.0, 3.0, 1.0));

    aBox1.combine(&aBox2);
    aBox1.combine(&aBox3);

    assert_near(aBox1.corner_min().x, 0.0, precision::confusion());
    assert_near(aBox1.corner_min().y, 0.0, precision::confusion());
    assert_near(aBox1.corner_min().z, 0.0, precision::confusion());
    assert_near(aBox1.corner_max().x, 3.0, precision::confusion());
    assert_near(aBox1.corner_max().y, 3.0, precision::confusion());
    assert_near(aBox1.corner_max().z, 1.0, precision::confusion());
}

#[test]
fn BVH_BoxTest__IsOutPartialOverlap() {
    // Boxes that partially overlap
    let mut aBox1 = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(2.0, 2.0, 2.0));
    let mut aBox2 = BvhBox::from_corners(BvhVec3d::new(1.0, 1.0, 1.0), BvhVec3d::new(3.0, 3.0, 3.0));

    assert!(!aBox1.is_out(&aBox2));
    assert!(!aBox2.is_out(&aBox1));
}

#[test]
fn BVH_BoxTest__IsOutContained() {
    // One box completely inside another
    let mut aBox1 =
        BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(10.0, 10.0, 10.0));
    let mut aBox2 = BvhBox::from_corners(BvhVec3d::new(2.0, 2.0, 2.0), BvhVec3d::new(3.0, 3.0, 3.0));

    assert!(!aBox1.is_out(&aBox2));
    assert!(!aBox2.is_out(&aBox1));
}

#[test]
fn BVH_BoxTest__IsOutFloat() {
    let mut aBox1 = BvhBox::from_corners(BvhVec3f::new(0.0, 0.0, 0.0), BvhVec3f::new(1.0, 1.0, 1.0));
    let mut aBox2 = BvhBox::from_corners(BvhVec3f::new(2.0, 2.0, 2.0), BvhVec3f::new(3.0, 3.0, 3.0));
    let mut aBox3 = BvhBox::from_corners(BvhVec3f::new(0.5, 0.5, 0.5), BvhVec3f::new(1.5, 1.5, 1.5));

    assert!(aBox1.is_out(&aBox2));
    assert!(!aBox1.is_out(&aBox3));
}

#[test]
fn BVH_BoxTest__Box2DIsOut() {
    let mut aBox1 = BvhBox::from_corners(BvhVec2d::new(0.0, 0.0), BvhVec2d::new(1.0, 1.0));
    let mut aBox2 = BvhBox::from_corners(BvhVec2d::new(2.0, 2.0), BvhVec2d::new(3.0, 3.0));
    let mut aBox3 = BvhBox::from_corners(BvhVec2d::new(0.5, 0.5), BvhVec2d::new(1.5, 1.5));

    assert!(aBox1.is_out(&aBox2));
    assert!(!aBox1.is_out(&aBox3));
}

#[test]
fn BVH_BoxTest__Box2DCenter() {
    let mut aBox = BvhBox::from_corners(BvhVec2d::new(0.0, 0.0), BvhVec2d::new(4.0, 6.0));

    let mut aCenter = aBox.center();
    assert_near(aCenter.x, 2.0, precision::confusion());
    assert_near(aCenter.y, 3.0, precision::confusion());
}

#[test]
fn BVH_BoxTest__Box2DSize() {
    let mut aBox = BvhBox::from_corners(BvhVec2d::new(1.0, 2.0), BvhVec2d::new(4.0, 7.0));

    let mut aSize = aBox.size();
    assert_near(aSize.x, 3.0, precision::confusion());
    assert_near(aSize.y, 5.0, precision::confusion());
}

#[test]
fn BVH_BoxTest__VerySmallBox() {
    // Test with very small values
    let mut aSmall: f64 = 1e-10;
    let mut aBox =
        BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(aSmall, aSmall, aSmall));

    assert!(aBox.is_valid());

    let mut aSize = aBox.size();
    assert_near(aSize.x, aSmall, 1e-15);
    assert_near(aSize.y, aSmall, 1e-15);
    assert_near(aSize.z, aSmall, 1e-15);
}

#[test]
fn BVH_BoxTest__SymmetricBox() {
    let mut aBox =
        BvhBox::from_corners(BvhVec3d::new(-5.0, -5.0, -5.0), BvhVec3d::new(5.0, 5.0, 5.0));

    let mut aCenter = aBox.center();
    assert_near(aCenter.x, 0.0, precision::confusion());
    assert_near(aCenter.y, 0.0, precision::confusion());
    assert_near(aCenter.z, 0.0, precision::confusion());

    let mut aSize = aBox.size();
    assert_near(aSize.x, 10.0, precision::confusion());
    assert_near(aSize.y, 10.0, precision::confusion());
    assert_near(aSize.z, 10.0, precision::confusion());
}

#[test]
fn BVH_BoxTest__NonCubicBox() {
    // Box with different dimensions on each axis
    let mut aBox =
        BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 10.0, 100.0));

    let mut aSize = aBox.size();
    assert_near(aSize.x, 1.0, precision::confusion());
    assert_near(aSize.y, 10.0, precision::confusion());
    assert_near(aSize.z, 100.0, precision::confusion());

    // Area = 2 * (1*10 + 10*100 + 1*100) = 2 * (10 + 1000 + 100) = 2220
    assert_near(aBox.area(), 2220.0, precision::confusion());
}

#[test]
fn BVH_BoxTest__ClearAndReuse() {
    let mut aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));

    assert!(aBox.is_valid());

    aBox.clear();
    assert!(!aBox.is_valid());

    // Reuse the box
    aBox.add(&BvhVec3d::new(5.0, 5.0, 5.0));
    aBox.add(&BvhVec3d::new(10.0, 10.0, 10.0));

    assert!(aBox.is_valid());
    assert_near(aBox.corner_min().x, 5.0, precision::confusion());
    assert_near(aBox.corner_max().x, 10.0, precision::confusion());
}

#[test]
fn BVH_BoxTest__IsOutSameBox() {
    let mut aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));

    // Box should not be "out" of itself
    assert!(!aBox.is_out(&aBox));
}

#[test]
fn BVH_BoxTest__CombineSameBox() {
    let mut aBox1 = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));
    let mut aBox2 = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));

    aBox1.combine(&aBox2);

    // Should remain unchanged
    assert_near(aBox1.corner_min().x, 0.0, precision::confusion());
    assert_near(aBox1.corner_max().x, 1.0, precision::confusion());
}

#[test]
fn BVH_BoxTest__IsOutNearMiss() {
    // Boxes that are very close but not touching
    let mut aBox1 = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));
    let mut aBox2 =
        BvhBox::from_corners(BvhVec3d::new(1.0001, 0.0, 0.0), BvhVec3d::new(2.0, 1.0, 1.0));

    assert!(aBox1.is_out(&aBox2));
}

#[test]
fn BVH_BoxTest__AddDuplicatePoint() {
    let mut aBox = BvhBox::<BvhVec3d>::default();

    aBox.add(&BvhVec3d::new(1.0, 2.0, 3.0));
    aBox.add(&BvhVec3d::new(1.0, 2.0, 3.0));
    aBox.add(&BvhVec3d::new(1.0, 2.0, 3.0));

    // Should still be a single point box
    assert_near(aBox.corner_min().x, 1.0, precision::confusion());
    assert_near(aBox.corner_max().x, 1.0, precision::confusion());
    assert_near(aBox.size().x, 0.0, precision::confusion());
}
