#[test]
fn Bnd_BoxTest__UpdateExpandsCorrectly() {
    let mut aBox = BoundingBox3::default();

    // Start with a point
    aBox.add_point(Point3::new(1.0, 1.0, 1.0));

    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = aBox.get().unwrap();

    assert_eq!(1.0, aXmin);
    assert_eq!(1.0, aXmax);

    // Add another point that expands in all directions
    aBox.add_point(Point3::new(-0.5, 2.5, 0.5));
    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = aBox.get().unwrap();

    assert_eq!(-0.5, aXmin);
    assert_eq!(1.0, aXmax);
    assert_eq!(1.0, aYmin);
    assert_eq!(2.5, aYmax);
}

#[test]
fn Bnd_BoxTest__DumpJsonAndInitFromJson() {
    // Create a bounding box with specific values
    let mut aBox = BoundingBox3::new(Point3::new(0.0, 0.0, 0.0), Point3::new(10.0, 5.0, 3.0));
    aBox.set_gap(0.1);

    // Serialize to JSON
    let mut anOStream = DumpStream::default();
    aBox.dump_json(&mut anOStream).unwrap();
    let aJsonStr = anOStream.text();

    // Try to deserialize
    let anIStream = aJsonStr;
    let mut aDeserializedBox = BoundingBox3::default();
    let mut aStreamPos = 1;

    assert!(
        aDeserializedBox
            .init_from_json(anIStream, &mut aStreamPos)
            .unwrap()
    );

    // Verify the deserialized box matches the original

    let [aXmin1, aYmin1, aZmin1, aXmax1, aYmax1, aZmax1] = aBox.get().unwrap();
    let [aXmin2, aYmin2, aZmin2, aXmax2, aYmax2, aZmax2] = aDeserializedBox.get().unwrap();

    assert_eq!(aXmin1, aXmin2);
    assert_eq!(aYmin1, aYmin2);
    assert_eq!(aZmin1, aZmin2);
    assert_eq!(aXmax1, aXmax2);
    assert_eq!(aYmax1, aYmax2);
    assert_eq!(aZmax1, aZmax2);
    assert_eq!(aBox.gap(), aDeserializedBox.gap());
}

//=================================================================================================
// Regression Tests
//=================================================================================================

// Test OCC16485: Bnd_Box tolerance issue with cumulative enlargement
// Migrated from QABugs_14.cxx
#[test]
fn Bnd_BoxTest__OCC16485_CumulativeEnlargeTolerance() {
    // Create points with X coordinate varying from 0 to 1000
    // and compute cumulative bounding box by adding boxes for all the
    // points, enlarged on tolerance
    let aTol: f64 = 1e-3;
    let aNbStep: i32 = 1000;
    let mut aBox = BoundingBox3::default();

    for i in 0..=aNbStep {
        let mut aP = Point3::new(f64::from(i), 0.0, 0.0);
        let mut aB = BoundingBox3::default();
        aB.add_point(aP);
        aB.enlarge(aTol);
        aB.add_box(&aBox);
        aBox = aB; // This was causing XMin to grow each time (regression)
    }

    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = aBox.get().unwrap();

    // Verify that Xmin is approximately -tolerance (not growing with iterations)
    assert!((-aTol - aXmin).abs() <= 1e-10);
    assert!((f64::from(aNbStep) + aTol - aXmax).abs() <= 1e-10);
}

#[test]
fn Bnd_BoxTest__Contains_Point() {
    let mut aBox = BoundingBox3::new(Point3::new(0.0, 0.0, 0.0), Point3::new(10.0, 10.0, 10.0));
    assert!(aBox.contains(Point3::new(5.0, 5.0, 5.0)));
    assert!(aBox.contains(Point3::new(0.0, 0.0, 0.0)));
    assert!(!(aBox.contains(Point3::new(-1.0, 5.0, 5.0))));
    assert!(!(aBox.contains(Point3::new(5.0, 5.0, 11.0))));
}

#[test]
fn Bnd_BoxTest__Intersects_Box() {
    let mut aBox1 = BoundingBox3::new(Point3::new(0.0, 0.0, 0.0), Point3::new(10.0, 10.0, 10.0));
    let mut aBox2 = BoundingBox3::new(Point3::new(5.0, 5.0, 5.0), Point3::new(15.0, 15.0, 15.0));
    let mut aBox3 = BoundingBox3::new(Point3::new(20.0, 20.0, 20.0), Point3::new(30.0, 30.0, 30.0));
    assert!(aBox1.intersects(&aBox2));
    assert!(!(aBox1.intersects(&aBox3)));
}

#[test]
fn Bnd_BoxTest__Center() {
    let mut aBox = BoundingBox3::new(Point3::new(0.0, 0.0, 0.0), Point3::new(10.0, 20.0, 30.0));
    let aCenter = aBox.center();
    assert!(aCenter.is_some());
    assert_eq!(aCenter.unwrap().x(), 5.0);
    assert_eq!(aCenter.unwrap().y(), 10.0);
    assert_eq!(aCenter.unwrap().z(), 15.0);
}

#[test]
fn Bnd_BoxTest__Center_VoidNullopt() {
    let mut aVoid = BoundingBox3::default();
    assert!(!(aVoid.center().is_some()));
}
