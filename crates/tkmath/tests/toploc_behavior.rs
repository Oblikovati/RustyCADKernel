//! Supplemental TopLoc regressions: composition, powers, hashing, lists and dumps.
use rustycad_tkernel::standard::{Handle, dump::DumpStream};
use rustycad_tkmath::{
    gp::{Point3, Transform3},
    toploc::{Datum3, ItemLocation, Location, MapOfLocation, SListOfItemLocation},
};
fn datum(x: f64) -> Handle<Datum3> {
    Handle::new(Datum3::new(Transform3::translation([x, 0.0, 0.0])))
}
fn origin_of(location: &Location) -> f64 {
    location.transformation().apply(Point3::default()).x()
}
#[test]
fn empty_location_reports_no_such_object_and_is_its_own_tail() {
    let identity = Location::default();
    assert_eq!(
        identity.first_datum().err().unwrap().exception_type(),
        "Standard_NoSuchObject"
    );
    assert!(identity.first_power().is_err());
    assert!(identity.next_location().is_identity());
    assert_eq!(identity.hash_code(), 0);
    assert!(identity.transformation().is_identity());
    assert_eq!(Location::scale_prec(), 1e-14);
}
#[test]
fn powers_follow_the_native_single_and_chain_paths() {
    let a = Location::from_datum(datum(1.0));
    let b = Location::from_datum(datum(10.0));
    assert_eq!(a.powered(3).unwrap().first_power().unwrap(), 3);
    assert_eq!(a.powered(-2).unwrap().first_power().unwrap(), -2);
    assert!(a.powered(0).unwrap().is_identity());
    let chain = a.multiplied(&b).unwrap();
    assert_eq!(origin_of(&chain.powered(2).unwrap()), 22.0);
    assert_eq!(origin_of(&chain.powered(-2).unwrap()), -22.0);
    assert_eq!(
        chain.powered(i32::MIN).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
}
#[test]
fn division_and_predivision_cancel_shared_datums() {
    let a = Location::from_datum(datum(1.0));
    let b = Location::from_datum(datum(10.0));
    let chain = a.multiplied(&b).unwrap();
    assert_eq!(chain.divided(&b).unwrap(), a);
    assert!(chain.predivided(&chain).unwrap().is_identity());
    assert_eq!(b.predivided(&Location::default()).unwrap(), b);
    assert_eq!(
        Location::default().predivided(&b).unwrap(),
        b.inverted().unwrap()
    );
    assert_eq!(chain.predivided(&a).unwrap(), b);
    assert_eq!((&a * &b).unwrap(), chain);
    assert_eq!((&chain / &b).unwrap(), a);
}
#[test]
fn equality_and_hash_follow_datum_identity() {
    let shared = datum(2.0);
    let first = Location::from_datum(shared.clone());
    let second = Location::from_datum(shared);
    let copy = Location::new(Transform3::translation([2.0, 0.0, 0.0]));
    assert!(first.is_equal(&second));
    assert_eq!(first.hash_code(), second.hash_code());
    assert!(first.is_different(&copy));
    let mut map = MapOfLocation::default();
    assert!(map.add(first.clone()));
    assert!(!map.add(second));
    assert!(map.add(copy));
    let address = first.first_datum().unwrap().as_ptr() as usize;
    assert_eq!(first.hash_code(), Location::hash_items(&[(address, 1)]));
}
#[test]
fn item_list_caches_the_tail_product() {
    let head = ItemLocation::new(datum(1.0), 2).unwrap();
    let tail_item = ItemLocation::new(datum(5.0), 1).unwrap();
    let tail = SListOfItemLocation::new(tail_item, &SListOfItemLocation::default());
    let mut list = SListOfItemLocation::new(head, &tail);
    assert!(list.more());
    assert!(list.tail().tail().is_empty());
    list.next();
    assert!(list.tail().is_empty());
    list.to_tail();
    assert!(list.is_empty());
    assert!(list.value().is_err());
    let singular = Handle::new(Datum3::new(
        Transform3::scaling(Point3::default(), 1.0).unwrap(),
    ));
    assert!(ItemLocation::new(singular, -1).is_ok());
}
#[test]
fn dumps_use_native_field_names() {
    let location = Location::from_datum(datum(1.5));
    let mut json = DumpStream::default();
    location.dump_json(&mut json, -1).unwrap();
    assert!(
        json.text()
            .starts_with("\"className\": \"TopLoc_Location\", \"Transformation\": {")
    );
    assert!(json.text().ends_with("\"IsIdentity\": 0"));
    let mut text = Vec::new();
    Location::default().shallow_dump(&mut text).unwrap();
    assert_eq!(
        String::from_utf8(text).unwrap(),
        "TopLoc_Location : Identity\n\n"
    );
    let mut text = Vec::new();
    location.shallow_dump(&mut text).unwrap();
    let text = String::from_utf8(text).unwrap();
    assert!(text.contains("       Exponent : 1\n TopLoc_Datum3D 0x"));
    assert!(text.contains("  (          1,         0,         0,       1.5)\n"));
}
