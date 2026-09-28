//! TColStd_HPackedMapOfInteger construction, sharing and RTTI.
use rustycad_tkernel::ncollection::PackedMap;
use rustycad_tkernel::standard::{Handle, StandardTransient};
use rustycad_tkernel::tcol_std::HPackedMapOfInteger;

#[test]
fn bucket_counts_below_one_mean_one() {
    assert_eq!(1, HPackedMapOfInteger::new(0).map().bucket_count());
    assert_eq!(1, HPackedMapOfInteger::new(-5).map().bucket_count());
    assert_eq!(1, HPackedMapOfInteger::new(i32::MIN).map().bucket_count());
    assert_eq!(10, HPackedMapOfInteger::new(10).map().bucket_count());
    assert_eq!(1, HPackedMapOfInteger::default().map().bucket_count());
    assert!(HPackedMapOfInteger::new(10).map().is_empty());
}
#[test]
fn copying_constructor_is_independent_of_its_source() {
    let mut source: PackedMap<i32> = PackedMap::new(1);
    source.add(4);
    let copy = HPackedMapOfInteger::from_map_ref(&source);
    source.add(9);
    assert!(copy.map().contains(4));
    assert!(!copy.map().contains(9));
    let moved = HPackedMapOfInteger::from_map(source);
    assert_eq!(2, moved.map().len());
}
#[test]
fn edits_through_one_handle_are_seen_by_every_holder() {
    let handle = Handle::new(HPackedMapOfInteger::new(1));
    let base = handle.cast::<StandardTransient>();
    let other = Handle::<HPackedMapOfInteger>::down_cast(&base);
    assert!(handle.change_map().add(-7));
    assert!(other.map().contains(-7));
    assert!(handle.is_kind_name("Standard_Transient"));
    assert!(handle.is_instance_name("TColStd_HPackedMapOfInteger"));
}
