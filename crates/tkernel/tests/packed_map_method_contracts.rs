//! Native PackedMap template widths, assignment and iterator method contracts.
use rustycad_tkernel::ncollection::{PackedInteger, PackedMap, PackedMapIter};

fn check_narrow<T: PackedInteger>(values: [T; 4]) {
    let mut map = PackedMap::<T>::default();
    for value in values {
        assert!(map.add(value));
    }
    for value in values {
        assert!(map.contains(value));
    }
    assert_eq!(map.size(), 4);
    assert_eq!(map.length(), 4);
    assert_eq!(map.iter().count(), 4);
    assert_eq!(map.minimal(), *values.iter().min().unwrap());
    assert_eq!(map.maximal(), *values.iter().max().unwrap());
    assert!(map.remove(values[0]));
    assert!(!map.remove(values[0]));
}
#[test]
fn signed_byte_template_support() {
    check_narrow([i8::MIN, -1, 0, i8::MAX]);
}
#[test]
fn unsigned_byte_template_support() {
    check_narrow([0_u8, 31, 32, u8::MAX]);
}
#[test]
fn signed_short_template_support() {
    check_narrow([i16::MIN, -1, 0, i16::MAX]);
}
#[test]
fn unsigned_short_template_support() {
    check_narrow([0_u16, 31, 32, u16::MAX]);
}
#[test]
fn pointer_sized_signed_template_support() {
    check_narrow([isize::MIN, -1, 0, isize::MAX]);
}
#[test]
fn bool_template_preserves_native_lowest_block_maximum_quirk() {
    let mut map = PackedMap::<bool>::default();
    assert!(map.add(true));
    assert!(!map.add(true));
    assert!(map.minimal());
    assert!(!map.maximal());
    assert!(map.add(false));
    assert_eq!(map.iter().collect::<Vec<_>>(), [false, true]);
    assert!(map.remove(false));
    assert_eq!(map.size(), 1);
}

#[test]
fn returning_assignments_and_clone_from_preserve_native_bucket_rules() {
    let mut source = PackedMap::new(900);
    source.add(42);
    let mut destination = PackedMap::new(700);
    let address = std::ptr::from_mut(&mut destination);
    assert_eq!(std::ptr::from_mut(destination.assign(&source)), address);
    assert_eq!(destination.bucket_count(), 700);
    assert_eq!(std::ptr::from_mut(destination.assign_self()), address);
    destination.clone_from(&source);
    assert_eq!(destination.bucket_count(), 700);
    assert_eq!(
        std::ptr::from_mut(destination.move_assign(&mut source)),
        address
    );
    assert_eq!(destination.bucket_count(), 900);
    assert_eq!(source.bucket_count(), 1);
    assert!(source.is_empty());
}

#[test]
fn self_boolean_operations_preserve_change_flags_and_modulus() {
    let mut map = PackedMap::new(700);
    map.add(42);
    assert!(!map.unite_self());
    assert!(!map.intersect_self());
    assert_eq!(map.size(), 1);
    assert!(map.subtract_self());
    assert!(!map.subtract_self());
    assert_eq!(map.bucket_count(), 700);
    map.add(43);
    assert!(map.differ_self());
    assert!(!map.differ_self());
    assert_eq!(map.bucket_count(), 700);
}

#[test]
fn stl_end_retains_cached_block_base_but_legacy_access_raises() {
    let mut map = PackedMap::<i32>::default();
    map.add(41);
    let mut cursor = PackedMapIter::new(&map);
    assert_eq!(cursor.key_checked(), Ok(&41));
    assert_eq!(cursor.value_checked(), Ok(&41));
    assert_eq!(cursor.value_ptr(), cursor.cached_value() as *const i32);
    let pointer = cursor.value_ptr();
    let address = std::ptr::from_mut(&mut cursor);
    assert_eq!(std::ptr::from_mut(cursor.advance_returning()), address);
    assert_eq!(cursor.cached_value(), &32);
    assert_eq!(cursor.value_ptr(), pointer);
    assert_eq!(
        cursor.key_checked().unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
    assert!(cursor.value_checked().is_err());
    cursor.advance_returning();
    assert_eq!(cursor.cached_value(), &32);
    assert!(cursor == map.end());
    assert_eq!(map.end().cached_value(), &0);
}

#[test]
fn post_increment_and_cursor_assignment_keep_independent_cached_storage() {
    let mut map = PackedMap::<i32>::default();
    map.add(41);
    map.add(42);
    let mut cursor = map.iter();
    let previous = cursor.post_advance();
    assert_eq!(previous.cached_value(), &41);
    assert_eq!(cursor.cached_value(), &42);
    assert_ne!(previous.value_ptr(), cursor.value_ptr());
    assert!(!previous.is_equal(&cursor));
    let address = std::ptr::from_mut(&mut cursor);
    assert_eq!(std::ptr::from_mut(cursor.assign(&previous)), address);
    assert!(previous.is_equal(&cursor));
    let mut end = map.end();
    end.reset();
    assert!(!end.more());
    assert_eq!(end.cached_value(), &0);
}
