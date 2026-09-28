// =======================================================================================
// Tests for Single-Point Constructor
// =======================================================================================

#[test]
fn BVH_BoxTest__SinglePointConstructor() {
    let mut aPoint = BvhVec3d::new(5.0, 10.0, 15.0);
    let mut aBox = BvhBox::from_point(aPoint);

    assert!(aBox.is_valid());
    assert_near(aBox.corner_min().x, 5.0, precision::confusion());
    assert_near(aBox.corner_min().y, 10.0, precision::confusion());
    assert_near(aBox.corner_min().z, 15.0, precision::confusion());
    assert_near(aBox.corner_max().x, 5.0, precision::confusion());
    assert_near(aBox.corner_max().y, 10.0, precision::confusion());
    assert_near(aBox.corner_max().z, 15.0, precision::confusion());

    // Size should be zero
    let mut aSize = aBox.size();
    assert_near(aSize.x, 0.0, precision::confusion());
    assert_near(aSize.y, 0.0, precision::confusion());
    assert_near(aSize.z, 0.0, precision::confusion());
}

#[test]
fn BVH_BoxTest__SinglePointConstructor2D() {
    let mut aPoint = BvhVec2d::new(3.0, 7.0);
    let mut aBox = BvhBox::from_point(aPoint);

    assert!(aBox.is_valid());
    assert_near(aBox.corner_min().x, 3.0, precision::confusion());
    assert_near(aBox.corner_min().y, 7.0, precision::confusion());
    assert_near(aBox.corner_max().x, 3.0, precision::confusion());
    assert_near(aBox.corner_max().y, 7.0, precision::confusion());
}

#[test]
fn BVH_BoxTest__SinglePointConstructorOrigin() {
    let mut aOrigin = BvhVec3d::new(0.0, 0.0, 0.0);
    let mut aBox = BvhBox::from_point(aOrigin);

    assert!(aBox.is_valid());
    assert_near(aBox.center().x, 0.0, precision::confusion());
    assert_near(aBox.center().y, 0.0, precision::confusion());
    assert_near(aBox.center().z, 0.0, precision::confusion());
}

// =======================================================================================
// Tests for Center(axis) method
// =======================================================================================

#[test]
fn BVH_BoxTest__CenterByAxis() {
    let mut aBox =
        BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(10.0, 20.0, 30.0));

    assert_near(aBox.center_axis(0), 5.0, precision::confusion()); // X axis
    assert_near(aBox.center_axis(1), 10.0, precision::confusion()); // Y axis
    assert_near(aBox.center_axis(2), 15.0, precision::confusion()); // Z axis
}

#[test]
fn BVH_BoxTest__CenterByAxis2D() {
    let mut aBox = BvhBox::from_corners(BvhVec2d::new(2.0, 4.0), BvhVec2d::new(8.0, 12.0));

    assert_near(aBox.center_axis(0), 5.0, precision::confusion()); // X axis
    assert_near(aBox.center_axis(1), 8.0, precision::confusion()); // Y axis
}

#[test]
fn BVH_BoxTest__CenterByAxisNegative() {
    let mut aBox = BvhBox::from_corners(
        BvhVec3d::new(-10.0, -20.0, -30.0),
        BvhVec3d::new(10.0, 20.0, 30.0),
    );

    assert_near(aBox.center_axis(0), 0.0, precision::confusion());
    assert_near(aBox.center_axis(1), 0.0, precision::confusion());
    assert_near(aBox.center_axis(2), 0.0, precision::confusion());
}

#[test]
fn BVH_BoxTest__CenterByAxis4D() {
    let mut aBox = BvhBox::from_corners(
        BvhVec4d::new(0.0, 0.0, 0.0, 0.0),
        BvhVec4d::new(4.0, 6.0, 8.0, 10.0),
    );

    assert_near(aBox.center_axis(0), 2.0, precision::confusion());
    assert_near(aBox.center_axis(1), 3.0, precision::confusion());
    assert_near(aBox.center_axis(2), 4.0, precision::confusion());
}

// =======================================================================================
// Tests for IsOut(point) method
// =======================================================================================

#[test]
fn BVH_BoxTest__IsOutPoint_Inside() {
    let mut aBox =
        BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(10.0, 10.0, 10.0));

    // Point inside
    assert!(!aBox.is_out_point(&BvhVec3d::new(5.0, 5.0, 5.0)));
    assert!(!aBox.is_out_point(&BvhVec3d::new(1.0, 1.0, 1.0)));
    assert!(!aBox.is_out_point(&BvhVec3d::new(9.0, 9.0, 9.0)));
}

#[test]
fn BVH_BoxTest__IsOutPoint_Outside() {
    let mut aBox =
        BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(10.0, 10.0, 10.0));

    // Points outside
    assert!(aBox.is_out_point(&BvhVec3d::new(11.0, 5.0, 5.0))); // Outside X
    assert!(aBox.is_out_point(&BvhVec3d::new(5.0, 11.0, 5.0))); // Outside Y
    assert!(aBox.is_out_point(&BvhVec3d::new(5.0, 5.0, 11.0))); // Outside Z
    assert!(aBox.is_out_point(&BvhVec3d::new(-1.0, 5.0, 5.0))); // Outside negative X
    assert!(aBox.is_out_point(&BvhVec3d::new(15.0, 15.0, 15.0))); // Far outside
}

#[test]
fn BVH_BoxTest__IsOutPoint_OnBoundary() {
    let mut aBox =
        BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(10.0, 10.0, 10.0));

    // Points on boundary (should be considered inside)
    assert!(!aBox.is_out_point(&BvhVec3d::new(0.0, 5.0, 5.0))); // On min X face
    assert!(!aBox.is_out_point(&BvhVec3d::new(10.0, 5.0, 5.0))); // On max X face
    assert!(!aBox.is_out_point(&BvhVec3d::new(5.0, 0.0, 5.0))); // On min Y face
    assert!(!aBox.is_out_point(&BvhVec3d::new(5.0, 10.0, 5.0))); // On max Y face
    assert!(!aBox.is_out_point(&BvhVec3d::new(0.0, 0.0, 0.0))); // At corner
    assert!(!aBox.is_out_point(&BvhVec3d::new(10.0, 10.0, 10.0))); // At opposite corner
}

#[test]
fn BVH_BoxTest__IsOutPoint_InvalidBox() {
    let mut aBox = BvhBox::<BvhVec3d>::default(); // Invalid box

    assert!(aBox.is_out_point(&BvhVec3d::new(0.0, 0.0, 0.0)));
    assert!(aBox.is_out_point(&BvhVec3d::new(5.0, 5.0, 5.0)));
}

#[test]
fn BVH_BoxTest__IsOutPoint_2D() {
    let mut aBox = BvhBox::from_corners(BvhVec2d::new(0.0, 0.0), BvhVec2d::new(5.0, 5.0));

    assert!(!aBox.is_out_point(&BvhVec2d::new(2.5, 2.5))); // Inside
    assert!(aBox.is_out_point(&BvhVec2d::new(6.0, 2.5))); // Outside X
    assert!(aBox.is_out_point(&BvhVec2d::new(2.5, 6.0))); // Outside Y
    assert!(!aBox.is_out_point(&BvhVec2d::new(0.0, 0.0))); // On corner
}

#[test]
fn BVH_BoxTest__IsOutPoint_NegativeCoords() {
    let mut aBox =
        BvhBox::from_corners(BvhVec3d::new(-5.0, -5.0, -5.0), BvhVec3d::new(5.0, 5.0, 5.0));

    assert!(!aBox.is_out_point(&BvhVec3d::new(0.0, 0.0, 0.0))); // At center
    assert!(!aBox.is_out_point(&BvhVec3d::new(-3.0, -3.0, -3.0))); // Inside negative
    assert!(aBox.is_out_point(&BvhVec3d::new(-6.0, 0.0, 0.0))); // Outside negative X
}

// =======================================================================================
// Tests for Contains() methods
// =======================================================================================

#[test]
fn BVH_BoxTest__Contains_FullyContained() {
    let mut aBox1 =
        BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(10.0, 10.0, 10.0));
    let mut aBox2 = BvhBox::from_corners(BvhVec3d::new(2.0, 2.0, 2.0), BvhVec3d::new(8.0, 8.0, 8.0));

    let (isContained, hasOverlap) = aBox1.contains(&aBox2);

    assert!(isContained);
    assert!(hasOverlap);
}

#[test]
fn BVH_BoxTest__Contains_NotContained() {
    let mut aBox1 =
        BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(10.0, 10.0, 10.0));
    let mut aBox2 =
        BvhBox::from_corners(BvhVec3d::new(8.0, 8.0, 8.0), BvhVec3d::new(12.0, 12.0, 12.0));

    let (isContained, hasOverlap) = aBox1.contains(&aBox2);

    assert!(!isContained);
    assert!(hasOverlap); // They overlap but box2 is not fully contained
}

#[test]
fn BVH_BoxTest__Contains_NoOverlap() {
    let mut aBox1 = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(5.0, 5.0, 5.0));
    let mut aBox2 =
        BvhBox::from_corners(BvhVec3d::new(10.0, 10.0, 10.0), BvhVec3d::new(15.0, 15.0, 15.0));

    let (isContained, hasOverlap) = aBox1.contains(&aBox2);

    assert!(!isContained);
    assert!(!hasOverlap);
}

#[test]
fn BVH_BoxTest__Contains_SameBox() {
    let mut aBox1 =
        BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(10.0, 10.0, 10.0));
    let mut aBox2 =
        BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(10.0, 10.0, 10.0));

    let (isContained, hasOverlap) = aBox1.contains(&aBox2);

    assert!(isContained);
    assert!(hasOverlap);
}

#[test]
fn BVH_BoxTest__Contains_TouchingBoundary() {
    let mut aBox1 =
        BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(10.0, 10.0, 10.0));
    let mut aBox2 = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(5.0, 5.0, 5.0));

    let (isContained, hasOverlap) = aBox1.contains(&aBox2);

    assert!(isContained);
    assert!(hasOverlap);
}

#[test]
fn BVH_BoxTest__Contains_InvalidBox() {
    let mut aBox1 =
        BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(10.0, 10.0, 10.0));
    let mut aBox2 = BvhBox::<BvhVec3d>::default(); // Invalid

    let (isContained, hasOverlap) = aBox1.contains(&aBox2);

    assert!(!isContained);
    assert!(!hasOverlap);
}

#[test]
fn BVH_BoxTest__Contains_InvalidContainer() {
    let mut aBox1 = BvhBox::<BvhVec3d>::default(); // Invalid
    let mut aBox2 =
        BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(10.0, 10.0, 10.0));

    let (isContained, hasOverlap) = aBox1.contains(&aBox2);

    assert!(!isContained);
    assert!(!hasOverlap);
}

#[test]
fn BVH_BoxTest__Contains_PartialOverlapOneAxis() {
    let mut aBox1 =
        BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(10.0, 10.0, 10.0));
    let mut aBox2 =
        BvhBox::from_corners(BvhVec3d::new(5.0, 5.0, 5.0), BvhVec3d::new(15.0, 8.0, 8.0));

    let (isContained, hasOverlap) = aBox1.contains(&aBox2);

    assert!(!isContained); // Not fully contained (extends in X)
    assert!(hasOverlap);
}

#[test]
fn BVH_BoxTest__Contains_2D() {
    let mut aBox1 = BvhBox::from_corners(BvhVec2d::new(0.0, 0.0), BvhVec2d::new(10.0, 10.0));
    let mut aBox2 = BvhBox::from_corners(BvhVec2d::new(2.0, 2.0), BvhVec2d::new(8.0, 8.0));

    let (isContained, hasOverlap) = aBox1.contains(&aBox2);

    assert!(isContained);
    assert!(hasOverlap);
}

#[test]
fn BVH_BoxTest__ContainsByCorners_FullyContained() {
    let mut aBox =
        BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(10.0, 10.0, 10.0));

    let (isContained, hasOverlap) =
        aBox.contains_corners(&BvhVec3d::new(2.0, 2.0, 2.0), &BvhVec3d::new(8.0, 8.0, 8.0));

    assert!(isContained);
    assert!(hasOverlap);
}

#[test]
fn BVH_BoxTest__ContainsByCorners_NotContained() {
    let mut aBox =
        BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(10.0, 10.0, 10.0));

    let (isContained, hasOverlap) =
        aBox.contains_corners(&BvhVec3d::new(8.0, 8.0, 8.0), &BvhVec3d::new(12.0, 12.0, 12.0));

    assert!(!isContained);
    assert!(hasOverlap);
}

// =======================================================================================
// Tests for Transform() and Transformed() methods
// =======================================================================================

#[test]
fn BVH_BoxTest__Transform_Identity() {
    let mut aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));

    let mut aIdentity = Mat4::<f64>::default();
    aIdentity.init_identity();

    aBox.transform(&aIdentity);

    assert_near(aBox.corner_min().x, 0.0, precision::confusion());
    assert_near(aBox.corner_max().x, 1.0, precision::confusion());
}

#[test]
fn BVH_BoxTest__Transform_Translation() {
    let mut aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));

    let mut aTransform = Mat4::<f64>::default();
    aTransform.init_identity();
    aTransform.set_column3(3, Vec3::<f64>::new(5.0, 10.0, 15.0));

    aBox.transform(&aTransform);

    assert_near(aBox.corner_min().x, 5.0, precision::confusion());
    assert_near(aBox.corner_min().y, 10.0, precision::confusion());
    assert_near(aBox.corner_min().z, 15.0, precision::confusion());
    assert_near(aBox.corner_max().x, 6.0, precision::confusion());
    assert_near(aBox.corner_max().y, 11.0, precision::confusion());
    assert_near(aBox.corner_max().z, 16.0, precision::confusion());
}

#[test]
fn BVH_BoxTest__Transform_Scale() {
    let mut aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));

    let mut aTransform = Mat4::<f64>::default();
    aTransform.init_identity();
    aTransform.set_value(0, 0, 2.0); // Scale X by 2
    aTransform.set_value(1, 1, 3.0); // Scale Y by 3
    aTransform.set_value(2, 2, 4.0); // Scale Z by 4

    aBox.transform(&aTransform);

    assert_near(aBox.corner_min().x, 0.0, precision::confusion());
    assert_near(aBox.corner_max().x, 2.0, precision::confusion());
    assert_near(aBox.corner_max().y, 3.0, precision::confusion());
    assert_near(aBox.corner_max().z, 4.0, precision::confusion());
}

#[test]
fn BVH_BoxTest__Transformed_Identity() {
    let mut aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));

    let mut aIdentity = Mat4::<f64>::default();
    aIdentity.init_identity();

    let mut aTransformed = aBox.transformed(&aIdentity);

    // Original should be unchanged
    assert_near(aBox.corner_min().x, 0.0, precision::confusion());
    assert_near(aBox.corner_max().x, 1.0, precision::confusion());

    // Transformed should be the same
    assert_near(aTransformed.corner_min().x, 0.0, precision::confusion());
    assert_near(aTransformed.corner_max().x, 1.0, precision::confusion());
}

#[test]
fn BVH_BoxTest__Transformed_Translation() {
    let mut aBox = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 1.0, 1.0));

    let mut aTransform = Mat4::<f64>::default();
    aTransform.init_identity();
    aTransform.set_column3(3, Vec3::<f64>::new(10.0, 20.0, 30.0));

    let mut aTransformed = aBox.transformed(&aTransform);

    // Original should be unchanged
    assert_near(aBox.corner_min().x, 0.0, precision::confusion());
    assert_near(aBox.corner_max().x, 1.0, precision::confusion());

    // Transformed should be translated
    assert_near(aTransformed.corner_min().x, 10.0, precision::confusion());
    assert_near(aTransformed.corner_min().y, 20.0, precision::confusion());
    assert_near(aTransformed.corner_min().z, 30.0, precision::confusion());
    assert_near(aTransformed.corner_max().x, 11.0, precision::confusion());
    assert_near(aTransformed.corner_max().y, 21.0, precision::confusion());
    assert_near(aTransformed.corner_max().z, 31.0, precision::confusion());
}

#[test]
fn BVH_BoxTest__Transform_InvalidBox() {
    let mut aBox = BvhBox::<BvhVec3d>::default(); // Invalid box

    let mut aTransform = Mat4::<f64>::default();
    aTransform.init_identity();
    aTransform.set_column3(3, Vec3::<f64>::new(10.0, 20.0, 30.0));

    aBox.transform(&aTransform);

    // Should remain invalid
    assert!(!aBox.is_valid());
}

#[test]
fn BVH_BoxTest__Transformed_InvalidBox() {
    let mut aBox = BvhBox::<BvhVec3d>::default(); // Invalid box

    let mut aTransform = Mat4::<f64>::default();
    aTransform.init_identity();
    aTransform.set_column3(3, Vec3::<f64>::new(10.0, 20.0, 30.0));

    let mut aTransformed = aBox.transformed(&aTransform);

    // Should remain invalid
    assert!(!aTransformed.is_valid());
}

// =======================================================================================
// Tests for Corner Modification
// =======================================================================================

#[test]
fn BVH_BoxTest__ModifyCornerMin() {
    let mut aBox =
        BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(10.0, 10.0, 10.0));

    // Modify via non-const reference
    *aBox.corner_min_mut() = BvhVec3d::new(-5.0, -5.0, -5.0);

    assert_near(aBox.corner_min().x, -5.0, precision::confusion());
    assert_near(aBox.corner_min().y, -5.0, precision::confusion());
    assert_near(aBox.corner_min().z, -5.0, precision::confusion());
    assert_near(aBox.corner_max().x, 10.0, precision::confusion());
}

#[test]
fn BVH_BoxTest__ModifyCornerMax() {
    let mut aBox =
        BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(10.0, 10.0, 10.0));

    // Modify via non-const reference
    *aBox.corner_max_mut() = BvhVec3d::new(20.0, 30.0, 40.0);

    assert_near(aBox.corner_max().x, 20.0, precision::confusion());
    assert_near(aBox.corner_max().y, 30.0, precision::confusion());
    assert_near(aBox.corner_max().z, 40.0, precision::confusion());
    assert_near(aBox.corner_min().x, 0.0, precision::confusion());
}

#[test]
fn BVH_BoxTest__ModifyCornerComponents() {
    let mut aBox =
        BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(10.0, 10.0, 10.0));

    // Modify individual components
    aBox.corner_min_mut().x = -1.0;
    aBox.corner_min_mut().y = -2.0;
    aBox.corner_max_mut().z = 15.0;

    assert_near(aBox.corner_min().x, -1.0, precision::confusion());
    assert_near(aBox.corner_min().y, -2.0, precision::confusion());
    assert_near(aBox.corner_min().z, 0.0, precision::confusion());
    assert_near(aBox.corner_max().z, 15.0, precision::confusion());
}

// =======================================================================================
// Additional Edge Cases
// =======================================================================================

#[test]
fn BVH_BoxTest__Area_DegenerateBox2D() {
    // 2D box with zero area should return perimeter
    let mut aBox = BvhBox::from_corners(BvhVec2d::new(0.0, 0.0), BvhVec2d::new(5.0, 0.0));

    let mut anArea: f64 = aBox.area();
    // For degenerate case, returns sum of dimensions
    assert!(anArea >= 0.0);
    assert_near(anArea, 5.0, precision::confusion()); // Width + height = 5 + 0
}

#[test]
fn BVH_BoxTest__IsOut_TwoCorners_Overlapping() {
    let mut aBox =
        BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(10.0, 10.0, 10.0));

    // Test with overlapping region
    assert!(
        !aBox.is_out_corners(&BvhVec3d::new(5.0, 5.0, 5.0), &BvhVec3d::new(15.0, 15.0, 15.0))
    );
}

#[test]
fn BVH_BoxTest__IsOut_TwoCorners_Disjoint() {
    let mut aBox =
        BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(10.0, 10.0, 10.0));

    // Test with disjoint region
    assert!(
        aBox.is_out_corners(&BvhVec3d::new(20.0, 20.0, 20.0), &BvhVec3d::new(30.0, 30.0, 30.0))
    );
}

#[test]
fn BVH_BoxTest__IsOut_TwoCorners_Contained() {
    let mut aBox =
        BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(10.0, 10.0, 10.0));

    // Test with fully contained region
    assert!(!aBox.is_out_corners(&BvhVec3d::new(2.0, 2.0, 2.0), &BvhVec3d::new(8.0, 8.0, 8.0)));
}

// Upstream checks these with `static_assert`. Rust generic trait methods such as
// `is_valid` cannot run in const context, so the `const` items prove the
// constructors are `const fn` and the validity check runs at test time.
#[test]
fn BVH_BoxTest__Constexpr_Construction() {
    // Test that constexpr constructors work at compile time
    const aMin: BvhVec3d = BvhVec3d::new(0.0, 0.0, 0.0);
    const aMax: BvhVec3d = BvhVec3d::new(1.0, 1.0, 1.0);
    const aBox: BvhBox<BvhVec3d> = BvhBox::from_corners(aMin, aMax);

    assert!(aBox.is_valid(), "Constexpr box should be valid");
}

#[test]
fn BVH_BoxTest__Constexpr_SinglePoint() {
    const aPoint: BvhVec3d = BvhVec3d::new(5.0, 5.0, 5.0);
    const aBox: BvhBox<BvhVec3d> = BvhBox::from_point(aPoint);

    assert!(aBox.is_valid(), "Constexpr single-point box should be valid");
}

#[test]
fn BVH_BoxTest__Constexpr_DefaultInvalid() {
    const aBox: BvhBox<BvhVec3d> = BvhBox::new();

    assert!(!aBox.is_valid(), "Default constexpr box should be invalid");
}

// =======================================================================================
// Tests for Transform/Transformed with float precision
// =======================================================================================

#[test]
fn BVH_BoxTest__Transform_Float_Identity() {
    let mut aBox = BvhBox::from_corners(BvhVec3f::new(0.0, 0.0, 0.0), BvhVec3f::new(1.0, 1.0, 1.0));

    let mut aIdentity = Mat4::<f32>::default();
    aIdentity.init_identity();

    aBox.transform(&aIdentity);

    assert_near_f32(aBox.corner_min().x, 0.0, 1e-5);
    assert_near_f32(aBox.corner_max().x, 1.0, 1e-5);
}

#[test]
fn BVH_BoxTest__Transform_Float_Translation() {
    let mut aBox = BvhBox::from_corners(BvhVec3f::new(0.0, 0.0, 0.0), BvhVec3f::new(1.0, 1.0, 1.0));

    let mut aTransform = Mat4::<f32>::default();
    aTransform.init_identity();
    aTransform.set_column3(3, Vec3::<f32>::new(5.0, 10.0, 15.0));

    aBox.transform(&aTransform);

    assert_near_f32(aBox.corner_min().x, 5.0, 1e-5);
    assert_near_f32(aBox.corner_min().y, 10.0, 1e-5);
    assert_near_f32(aBox.corner_min().z, 15.0, 1e-5);
    assert_near_f32(aBox.corner_max().x, 6.0, 1e-5);
    assert_near_f32(aBox.corner_max().y, 11.0, 1e-5);
    assert_near_f32(aBox.corner_max().z, 16.0, 1e-5);
}

#[test]
fn BVH_BoxTest__Transform_Float_Scale() {
    let mut aBox = BvhBox::from_corners(BvhVec3f::new(0.0, 0.0, 0.0), BvhVec3f::new(1.0, 1.0, 1.0));

    let mut aTransform = Mat4::<f32>::default();
    aTransform.init_identity();
    aTransform.set_value(0, 0, 2.0); // Scale X by 2
    aTransform.set_value(1, 1, 3.0); // Scale Y by 3
    aTransform.set_value(2, 2, 4.0); // Scale Z by 4

    aBox.transform(&aTransform);

    assert_near_f32(aBox.corner_min().x, 0.0, 1e-5);
    assert_near_f32(aBox.corner_max().x, 2.0, 1e-5);
    assert_near_f32(aBox.corner_max().y, 3.0, 1e-5);
    assert_near_f32(aBox.corner_max().z, 4.0, 1e-5);
}

#[test]
fn BVH_BoxTest__Transformed_Float_Translation() {
    let mut aBox = BvhBox::from_corners(BvhVec3f::new(0.0, 0.0, 0.0), BvhVec3f::new(1.0, 1.0, 1.0));

    let mut aTransform = Mat4::<f32>::default();
    aTransform.init_identity();
    aTransform.set_column3(3, Vec3::<f32>::new(10.0, 20.0, 30.0));

    let mut aTransformed = aBox.transformed(&aTransform);

    // Original should be unchanged
    assert_near_f32(aBox.corner_min().x, 0.0, 1e-5);
    assert_near_f32(aBox.corner_max().x, 1.0, 1e-5);

    // Transformed should be translated
    assert_near_f32(aTransformed.corner_min().x, 10.0, 1e-5);
    assert_near_f32(aTransformed.corner_min().y, 20.0, 1e-5);
    assert_near_f32(aTransformed.corner_min().z, 30.0, 1e-5);
    assert_near_f32(aTransformed.corner_max().x, 11.0, 1e-5);
    assert_near_f32(aTransformed.corner_max().y, 21.0, 1e-5);
    assert_near_f32(aTransformed.corner_max().z, 31.0, 1e-5);
}

#[test]
fn BVH_BoxTest__Transform_Float_InvalidBox() {
    let mut aBox = BvhBox::<BvhVec3f>::default(); // Invalid box

    let mut aTransform = Mat4::<f32>::default();
    aTransform.init_identity();
    aTransform.set_column3(3, Vec3::<f32>::new(10.0, 20.0, 30.0));

    aBox.transform(&aTransform);

    // Should remain invalid
    assert!(!aBox.is_valid());
}
