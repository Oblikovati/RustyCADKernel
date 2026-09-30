//! Behavior tests of the ShapeExtend status flags and their bit codes.
use rustycad_tkshhealing::shape_extend::{ShapeExtend, ShapeExtendStatus};

/// The bits of every enumerator, in native order, as the native switch spells them.
const NATIVE_BITS: [i32; 19] = [
    0x0000, 0x0001, 0x0002, 0x0004, 0x0008, 0x0010, 0x0020, 0x0040, 0x0080, 0x00ff, 0x0100, 0x0200,
    0x0400, 0x0800, 0x1000, 0x2000, 0x4000, 0x8000, 0xff00,
];

#[test]
fn encode_status_gives_the_native_bits_of_every_enumerator() {
    for (status, bits) in ShapeExtendStatus::ALL.iter().zip(NATIVE_BITS) {
        assert_eq!(ShapeExtend::encode_status(*status), bits, "{status:?}");
    }
}

#[test]
fn from_index_follows_the_native_order() {
    for (index, status) in ShapeExtendStatus::ALL.iter().enumerate() {
        assert_eq!(ShapeExtendStatus::from_index(index as i32), Some(*status));
        assert_eq!(*status as usize, index);
    }
    assert_eq!(ShapeExtendStatus::from_index(-1), None);
    assert_eq!(ShapeExtendStatus::from_index(19), None);
}

#[test]
fn decode_status_tests_one_bit_or_no_bit_for_ok() {
    let flag = ShapeExtend::encode_status(ShapeExtendStatus::Done2)
        | ShapeExtend::encode_status(ShapeExtendStatus::Fail8);
    assert!(ShapeExtend::decode_status(flag, ShapeExtendStatus::Done2));
    assert!(ShapeExtend::decode_status(flag, ShapeExtendStatus::Done));
    assert!(ShapeExtend::decode_status(flag, ShapeExtendStatus::Fail));
    assert!(!ShapeExtend::decode_status(flag, ShapeExtendStatus::Done1));
    assert!(!ShapeExtend::decode_status(flag, ShapeExtendStatus::Ok));
    assert!(ShapeExtend::decode_status(0, ShapeExtendStatus::Ok));
    // Bits above the 16 the flags use are never decoded, but OK still sees them.
    assert!(!ShapeExtend::decode_status(
        0x10000,
        ShapeExtendStatus::Done
    ));
    assert!(!ShapeExtend::decode_status(0x10000, ShapeExtendStatus::Ok));
}
