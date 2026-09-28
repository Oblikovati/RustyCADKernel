// Copyright (c) 2025 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

#[test]
fn Bnd_RangeTest__Shift() {
    let mut aRange = Range1::new(3.0, 7.0).unwrap();
    aRange.shift(10.0);
    let mut aMin: f64 = 0.0;
    let mut aMax: f64 = 0.0;
    assert!(aRange.get_bounds(&mut aMin, &mut aMax));
    assert_eq!(aMin, 13.0);
    assert_eq!(aMax, 17.0);
}

#[test]
fn Bnd_RangeTest__TrimFrom() {
    let mut aRange = Range1::new(3.0, 10.0).unwrap();
    aRange.trim_from(5.0);
    let mut aMin: f64 = 0.0;
    let mut aMax: f64 = 0.0;
    assert!(aRange.get_bounds(&mut aMin, &mut aMax));
    assert_eq!(aMin, 5.0);
    assert_eq!(aMax, 10.0);
}

#[test]
fn Bnd_RangeTest__TrimFrom_MakesVoid() {
    let mut aRange = Range1::new(3.0, 10.0).unwrap();
    aRange.trim_from(15.0);
    assert!(aRange.is_void());
}

#[test]
fn Bnd_RangeTest__TrimTo() {
    let mut aRange = Range1::new(3.0, 10.0).unwrap();
    aRange.trim_to(7.0);
    let mut aMin: f64 = 0.0;
    let mut aMax: f64 = 0.0;
    assert!(aRange.get_bounds(&mut aMin, &mut aMax));
    assert_eq!(aMin, 3.0);
    assert_eq!(aMax, 7.0);
}

#[test]
fn Bnd_RangeTest__TrimTo_MakesVoid() {
    let mut aRange = Range1::new(3.0, 10.0).unwrap();
    aRange.trim_to(1.0);
    assert!(aRange.is_void());
}

#[test]
fn Bnd_RangeTest__IsOut_Value() {
    let mut aRange = Range1::new(3.0, 10.0).unwrap();
    assert!(!(aRange.is_out(5.0)));
    assert!(!(aRange.is_out(3.0)));
    assert!(!(aRange.is_out(10.0)));
    assert!(aRange.is_out(1.0));
    assert!(aRange.is_out(15.0));
}

#[test]
fn Bnd_RangeTest__IsOut_Void() {
    let mut aRange = Range1::default();
    assert!(aRange.is_out(5.0));
}

#[test]
fn Bnd_RangeTest__IsOut_Range() {
    let mut aRange = Range1::new(3.0, 10.0).unwrap();
    assert!(!(aRange.is_out_range(Range1::new(5.0, 8.0).unwrap())));
    assert!(!(aRange.is_out_range(Range1::new(1.0, 5.0).unwrap())));
    assert!(aRange.is_out_range(Range1::new(11.0, 15.0).unwrap()));
}

#[test]
fn Bnd_RangeTest__IsOut_Range_Void() {
    let mut aRange = Range1::new(3.0, 10.0).unwrap();
    let mut aVoid = Range1::default();
    assert!(aRange.is_out_range(aVoid));
    assert!(aVoid.is_out_range(aRange));
}

#[test]
fn Bnd_RangeTest__Equality() {
    let mut aRange1 = Range1::new(3.0, 10.0).unwrap();
    let mut aRange2 = Range1::new(3.0, 10.0).unwrap();
    let mut aRange3 = Range1::new(3.0, 11.0).unwrap();
    assert!(aRange1 == aRange2);
    assert!(!(aRange1 == aRange3));
}

#[test]
fn Bnd_RangeTest__Contains_Value() {
    let mut aRange = Range1::new(3.0, 10.0).unwrap();
    assert!(aRange.contains(5.0));
    assert!(aRange.contains(3.0));
    assert!(aRange.contains(10.0));
    assert!(!(aRange.contains(2.0)));
    assert!(!(aRange.contains(11.0)));

    let mut aVoid = Range1::default();
    assert!(!(aVoid.contains(5.0)));
}

#[test]
fn Bnd_RangeTest__Intersects_Range() {
    let mut aRange1 = Range1::new(3.0, 10.0).unwrap();
    let mut aRange2 = Range1::new(5.0, 15.0).unwrap();
    let mut aRange3 = Range1::new(11.0, 20.0).unwrap();
    assert!(aRange1.intersects(aRange2));
    assert!(!(aRange1.intersects(aRange3)));

    let mut aVoid = Range1::default();
    assert!(!(aRange1.intersects(aVoid)));
}

#[test]
fn Bnd_RangeTest__Min_Max_DirectAccess() {
    let mut aRange = Range1::new(3.0, 10.0).unwrap();
    assert!(aRange.min().is_some());
    assert!(aRange.max().is_some());
    assert_eq!(aRange.min().unwrap(), 3.0);
    assert_eq!(aRange.max().unwrap(), 10.0);
}

#[test]
fn Bnd_RangeTest__Min_Max_NulloptOnVoid() {
    let mut aVoid = Range1::default();
    assert!(!(aVoid.min().is_some()));
    assert!(!(aVoid.max().is_some()));
}

#[test]
fn Bnd_RangeTest__Get_StructuredBindings() {
    let mut aRange = Range1::new(3.0, 10.0).unwrap();
    let aBounds = aRange.get();
    assert!(aBounds.is_some());
    let [aMin, aMax] = aBounds.unwrap();
    assert_eq!(aMin, 3.0);
    assert_eq!(aMax, 10.0);
}

#[test]
fn Bnd_RangeTest__Get_NulloptOnVoid() {
    let mut aVoid = Range1::default();
    assert!(!(aVoid.get().is_some()));
}

#[test]
fn Bnd_RangeTest__Center() {
    let mut aRange = Range1::new(2.0, 8.0).unwrap();
    assert!(aRange.center().is_some());
    assert_eq!(aRange.center().unwrap(), 5.0);

    let mut aPoint = Range1::new(3.0, 3.0).unwrap();
    assert!(aPoint.center().is_some());
    assert_eq!(aPoint.center().unwrap(), 3.0);
}

#[test]
fn Bnd_RangeTest__Center_NulloptOnVoid() {
    let mut aVoid = Range1::default();
    assert!(!(aVoid.center().is_some()));
}
