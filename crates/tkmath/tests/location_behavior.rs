//! Supplemental numeric and ownership regressions for the OCC25545 dependency path.
use rustycad_tkmath::{
    gp::{Point3, Transform3},
    toploc::{Datum3, Location},
};
use std::sync::Arc;
#[test]
fn translation_forms_and_integer_powers() {
    let identity = Transform3::default();
    let t = Transform3::translation([0.1, -2.0, 3.0]);
    assert!(identity.is_identity());
    assert!(!Transform3::translation([0.0; 3]).is_identity());
    assert_eq!(identity.apply(Point3::default()), Point3::default());
    assert_eq!(identity.multiplied(t), t);
    assert_eq!(t.multiplied(identity), t);
    assert_eq!(t.powered(0).unwrap(), identity);
    assert_eq!(t.powered(1).unwrap(), t);
    assert_eq!(t.powered(-1).unwrap(), t.inverted().unwrap());
    assert_eq!(
        t.powered(3).unwrap().apply(Point3::default()),
        Point3::new(0.1 + 0.2, -6.0, 9.0)
    );
    assert_eq!(
        t.powered(-4).unwrap().apply(Point3::default()),
        Point3::new(-0.4, 8.0, -12.0)
    );
}
#[test]
fn datum_identity_cancellation_and_shared_tails() {
    let datum = Arc::new(Datum3::new(Transform3::translation([2.0, 3.0, 4.0])));
    let location = Location::from_datum(datum.clone());
    assert_eq!(*datum.transformation(), location.transformation());
    assert!(Arc::ptr_eq(location.first_datum().unwrap(), &datum));
    assert_eq!(location.first_power(), Some(1));
    assert!(location.next_location().unwrap().is_identity());
    assert_eq!(location.clone(), location);
    assert_ne!(location, Location::new(*datum.transformation()));
    assert!(
        location
            .multiplied(&location.inverted().unwrap())
            .unwrap()
            .is_identity()
    );
    assert_eq!(
        location.multiplied(&location).unwrap().first_power(),
        Some(2)
    );
    let chain = location
        .multiplied(&Location::new(Transform3::translation([1.0; 3])))
        .unwrap();
    assert_eq!(
        chain
            .inverted()
            .unwrap()
            .transformation()
            .apply(Point3::default()),
        Point3::new(-3.0, -4.0, -5.0)
    );
    assert!(Location::default().inverted().unwrap().is_identity());
    assert_eq!(location.multiplied(&Location::default()).unwrap(), location);
}
