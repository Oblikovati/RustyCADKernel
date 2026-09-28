//! Stable slot addresses and absence boundaries in sparse storage.
use rustycad_tkernel::ncollection::SparseArray;
#[test]
fn assignment_reuses_compatible_blocks_and_clears_missing_slots() {
    let mut source = SparseArray::new(4);
    source.set_value(1, 42);
    let mut destination = SparseArray::new(4);
    destination.set_value(1, 17);
    destination.set_value(2, 99);
    let address = &destination[1] as *const i32;
    destination.assign(&source);
    assert_eq!(&destination[1] as *const i32, address);
    assert_eq!(destination[1], 42);
    assert!(!destination.has_value(2));
    destination[1] = 7;
    let copy = destination.clone();
    assert_eq!(copy[1], 7);
    assert!(!destination.unset_value(1000));
    assert!(!destination.unset_value(2));
}
#[test]
fn character_classification_c_locale_boundaries() {
    use rustycad_tkernel::standard::character::*;
    assert!(is_equal(b'A', b'A'));
    assert!(!is_equal(b'A', b'B'));
    assert!(is_space(11));
    assert!(is_control(127));
    assert!(!is_printable(127));
    assert!(is_printable(b' '));
    assert!(!is_graphic(b' '));
    assert!(is_xdigit(b'F'));
    assert!(is_punctuation(b'!'));
    for byte in 128..=255 {
        assert!(!is_alphabetic(byte));
        assert!(!is_printable(byte));
        assert_eq!(upper_case(byte), byte);
        assert_eq!(lower_case(byte), byte);
    }
}
