#[test]
fn Bnd_BoxTest__CornerMethods() {
    let mut aBox = BoundingBox3::default();
    aBox.update_bounds([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
    aBox.set_gap(0.1);

    let mut aCornerMin = aBox.corner_min().unwrap();
    let mut aCornerMax = aBox.corner_max().unwrap();

    assert_eq!(0.9, aCornerMin.x());
    assert_eq!(1.9, aCornerMin.y());
    assert_eq!(2.9, aCornerMin.z());

    assert_eq!(4.1, aCornerMax.x());
    assert_eq!(5.1, aCornerMax.y());
    assert_eq!(6.1, aCornerMax.z());
}

#[test]
fn Bnd_BoxTest__CornerMethodsVoidBox() {
    let mut aBox = BoundingBox3::default(); // void box

    assert_eq!(
        aBox.corner_min().unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
    assert_eq!(
        aBox.corner_max().unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
}

#[test]
fn Bnd_BoxTest__CornerMethodsOpenBox() {
    let mut aBox = BoundingBox3::default();
    aBox.update_bounds([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
    aBox.open_xmax();
    aBox.open_ymin();

    let mut aCornerMin = aBox.corner_min().unwrap();
    let mut aCornerMax = aBox.corner_max().unwrap();

    assert_eq!(-1e100, aCornerMin.y());
    assert_eq!(1e100, aCornerMax.x());
}

//=================================================================================================
// Thin Methods Tests
//=================================================================================================

#[test]
fn Bnd_BoxTest__ThinnessMethods() {
    let mut aBox = BoundingBox3::default();

    assert!(aBox.is_x_thin(1.0));
    assert!(aBox.is_y_thin(1.0));
    assert!(aBox.is_z_thin(1.0));
    assert!(aBox.is_thin(1.0));

    // Create a thin box in X direction
    aBox.update_bounds([1.0, 0.0, 0.0], [1.01, 2.0, 2.0]);

    assert!(aBox.is_x_thin(0.1));
    assert!(!(aBox.is_y_thin(0.1)));
    assert!(!(aBox.is_z_thin(0.1)));
    assert!(!(aBox.is_thin(0.1)));

    // Create a box that is thin in all dimensions
    let mut aThinBox = BoundingBox3::default();
    aThinBox.update_bounds([0.0, 0.0, 0.0], [0.01, 0.01, 0.01]);

    assert!(aThinBox.is_x_thin(0.1));
    assert!(aThinBox.is_y_thin(0.1));
    assert!(aThinBox.is_z_thin(0.1));
    assert!(aThinBox.is_thin(0.1));
}

#[test]
fn Bnd_BoxTest__ThinnessWithOpenBox() {
    let mut aBox = BoundingBox3::default();
    aBox.update_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    aBox.open_xmin();

    assert!(!(aBox.is_x_thin(0.1)));

    aBox.set_whole();
    assert!(!(aBox.is_x_thin(0.1)));
    assert!(!(aBox.is_thin(0.1)));
}

//=================================================================================================
// Transformation Tests
//=================================================================================================

#[test]
fn Bnd_BoxTest__TransformationIdentity() {
    let mut aBox = BoundingBox3::default();
    aBox.update_bounds([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
    aBox.set_gap(0.1);

    let mut anIdentity = Transform3::default();
    let mut aTransformed = aBox.transformed(anIdentity);

    let [aXmin1, aYmin1, aZmin1, aXmax1, aYmax1, aZmax1] = aBox.get().unwrap();
    let [aXmin2, aYmin2, aZmin2, aXmax2, aYmax2, aZmax2] = aTransformed.get().unwrap();

    assert_eq!(aXmin1, aXmin2);
    assert_eq!(aYmin1, aYmin2);
    assert_eq!(aZmin1, aZmin2);
    assert_eq!(aXmax1, aXmax2);
    assert_eq!(aYmax1, aYmax2);
    assert_eq!(aZmax1, aZmax2);
    assert_eq!(aBox.gap(), aTransformed.gap());
}

#[test]
fn Bnd_BoxTest__TransformationTranslation() {
    let mut aBox = BoundingBox3::default();
    aBox.update_bounds([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);

    let mut aTranslation = Transform3::default();
    aTranslation = Transform3::translation([2.0, 3.0, 4.0]);

    let mut aTransformed = aBox.transformed(aTranslation);

    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = aTransformed.get().unwrap();

    assert_eq!(2.0, aXmin);
    assert_eq!(3.0, aYmin);
    assert_eq!(4.0, aZmin);
    assert_eq!(3.0, aXmax);
    assert_eq!(4.0, aYmax);
    assert_eq!(5.0, aZmax);
}

#[test]
fn Bnd_BoxTest__TransformationVoidBox() {
    let mut aVoidBox = BoundingBox3::default();
    let mut aRotation = Transform3::default();
    aRotation = Transform3::rotation(
        Axis1::new(Point3::new(0.0, 0.0, 0.0), Direction3::Z),
        std::f64::consts::PI / 4.0,
    )
    .unwrap();

    let mut aTransformed = aVoidBox.transformed(aRotation);

    assert!(aTransformed.is_void());
}

//=================================================================================================
// Add Methods Tests
//=================================================================================================
