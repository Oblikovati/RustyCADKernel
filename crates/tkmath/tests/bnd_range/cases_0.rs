#[test]
fn Bnd_RangeTest__DefaultConstructor_IsVoid() {
    let mut aRange = Range1::default();
    assert!(aRange.is_void());
    assert!(aRange.delta() < 0.0);
}

#[test]
fn Bnd_RangeTest__ParameterizedConstructor() {
    let mut aRange = Range1::new(3.0, 15.0).unwrap();
    assert!(!(aRange.is_void()));
    assert_eq!(aRange.delta(), 12.0);
}

#[test]
fn Bnd_RangeTest__ParameterizedConstructor_InvalidRange() {
    assert_eq!(
        Range1::new(10.0, 5.0).unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
}

#[test]
fn Bnd_RangeTest__ParameterizedConstructor_PointRange() {
    let mut aRange = Range1::new(5.0, 5.0).unwrap();
    assert!(!(aRange.is_void()));
    assert_eq!(aRange.delta(), 0.0);
}

#[test]
fn Bnd_RangeTest__GetMin_GetMax_GetBounds() {
    let mut aRange = Range1::new(2.0, 8.0).unwrap();
    let mut aMin: f64 = 0.0;
    let mut aMax: f64 = 0.0;
    assert!(aRange.get_min(&mut aMin));
    assert_eq!(aMin, 2.0);
    assert!(aRange.get_max(&mut aMax));
    assert_eq!(aMax, 8.0);

    let mut aFirst: f64 = 0.0;
    let mut aLast: f64 = 0.0;
    assert!(aRange.get_bounds(&mut aFirst, &mut aLast));
    assert_eq!(aFirst, 2.0);
    assert_eq!(aLast, 8.0);
}

#[test]
fn Bnd_RangeTest__GetMin_GetMax_Void() {
    let mut aRange = Range1::default();
    let mut aVal: f64 = 0.0;
    assert!(!(aRange.get_min(&mut aVal)));
    assert!(!(aRange.get_max(&mut aVal)));

    let mut aFirst: f64 = 0.0;
    let mut aLast: f64 = 0.0;
    assert!(!(aRange.get_bounds(&mut aFirst, &mut aLast)));
}

#[test]
fn Bnd_RangeTest__GetIntermediatePoint() {
    let mut aRange = Range1::new(10.0, 20.0).unwrap();
    let mut aPar: f64 = 0.0;
    assert!(aRange.get_intermediate_point(0.0, &mut aPar));
    assert_eq!(aPar, 10.0);
    assert!(aRange.get_intermediate_point(0.5, &mut aPar));
    assert_eq!(aPar, 15.0);
    assert!(aRange.get_intermediate_point(1.0, &mut aPar));
    assert_eq!(aPar, 20.0);
}

#[test]
fn Bnd_RangeTest__GetIntermediatePoint_Void() {
    let mut aRange = Range1::default();
    let mut aPar: f64 = 0.0;
    assert!(!(aRange.get_intermediate_point(0.5, &mut aPar)));
}

#[test]
fn Bnd_RangeTest__SetVoid() {
    let mut aRange = Range1::new(1.0, 5.0).unwrap();
    assert!(!(aRange.is_void()));
    aRange.set_void();
    assert!(aRange.is_void());
}

#[test]
fn Bnd_RangeTest__Add_Double_ToVoid() {
    let mut aRange = Range1::default();
    aRange.add(5.0);
    assert!(!(aRange.is_void()));
    let mut aMin: f64 = 0.0;
    let mut aMax: f64 = 0.0;
    assert!(aRange.get_bounds(&mut aMin, &mut aMax));
    assert_eq!(aMin, 5.0);
    assert_eq!(aMax, 5.0);
}

#[test]
fn Bnd_RangeTest__Add_Double_Extends() {
    let mut aRange = Range1::new(3.0, 7.0).unwrap();
    aRange.add(1.0);
    aRange.add(10.0);
    let mut aMin: f64 = 0.0;
    let mut aMax: f64 = 0.0;
    assert!(aRange.get_bounds(&mut aMin, &mut aMax));
    assert_eq!(aMin, 1.0);
    assert_eq!(aMax, 10.0);
}

#[test]
fn Bnd_RangeTest__Add_Range_ToVoid() {
    let mut aRange = Range1::default();
    let mut anOther = Range1::new(3.0, 7.0).unwrap();
    aRange.add_range(anOther);
    let mut aMin: f64 = 0.0;
    let mut aMax: f64 = 0.0;
    assert!(aRange.get_bounds(&mut aMin, &mut aMax));
    assert_eq!(aMin, 3.0);
    assert_eq!(aMax, 7.0);
}

#[test]
fn Bnd_RangeTest__Add_VoidRange() {
    let mut aRange = Range1::new(3.0, 7.0).unwrap();
    let mut aVoid = Range1::default();
    aRange.add_range(aVoid);
    // Should remain unchanged
    let mut aMin: f64 = 0.0;
    let mut aMax: f64 = 0.0;
    assert!(aRange.get_bounds(&mut aMin, &mut aMax));
    assert_eq!(aMin, 3.0);
    assert_eq!(aMax, 7.0);
}

#[test]
fn Bnd_RangeTest__Add_Range_VoidToVoid() {
    let mut aRange = Range1::default();
    let mut aVoid = Range1::default();
    aRange.add_range(aVoid);
    assert!(aRange.is_void());
}

#[test]
fn Bnd_RangeTest__Add_Range_Extends() {
    let mut aRange = Range1::new(3.0, 7.0).unwrap();
    let mut anOther = Range1::new(1.0, 10.0).unwrap();
    aRange.add_range(anOther);
    let mut aMin: f64 = 0.0;
    let mut aMax: f64 = 0.0;
    assert!(aRange.get_bounds(&mut aMin, &mut aMax));
    assert_eq!(aMin, 1.0);
    assert_eq!(aMax, 10.0);
}

#[test]
fn Bnd_RangeTest__Common_Overlapping() {
    let mut aRange = Range1::new(1.0, 10.0).unwrap();
    let mut anOther = Range1::new(5.0, 15.0).unwrap();
    aRange.common(anOther);
    let mut aMin: f64 = 0.0;
    let mut aMax: f64 = 0.0;
    assert!(aRange.get_bounds(&mut aMin, &mut aMax));
    assert_eq!(aMin, 5.0);
    assert_eq!(aMax, 10.0);
}

#[test]
fn Bnd_RangeTest__Common_NoOverlap() {
    let mut aRange = Range1::new(1.0, 5.0).unwrap();
    let mut anOther = Range1::new(7.0, 10.0).unwrap();
    aRange.common(anOther);
    assert!(aRange.is_void());
}

#[test]
fn Bnd_RangeTest__Common_WithVoid() {
    let mut aRange = Range1::new(1.0, 5.0).unwrap();
    let mut aVoid = Range1::default();
    aRange.common(aVoid);
    assert!(aRange.is_void());
}
