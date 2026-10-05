//! Standard_GUID constructors, conversions, operators, hashing and failure classes.
use rustycad_tkernel::standard::{Guid, Uuid};
use std::collections::HashSet;

const TEXT: &[u8] = b"12345678-1234-5678-9ABC-DEF012345678";

#[test]
fn invalid_text_raises_standard_range_error() {
    for text in [
        &b""[..],
        b"12345678-1234-5678-9abc-def01234567",
        b" 2345678-1234-5678-9abc-def012345678",
    ] {
        let failure = Guid::from_ascii(text).unwrap_err();
        assert_eq!(failure.exception_type(), "Standard_RangeError");
        assert_eq!(failure.message(), "Invalid format of GUID");
        assert!(failure.is_kind("Standard_DomainError"));
    }
}

#[test]
fn utf16_keeps_low_bytes_and_rejects_short_buffers() {
    let units: Vec<u16> = TEXT.iter().map(|&b| u16::from(b) | 0x2000).collect();
    assert_eq!(
        Guid::from_utf16(&units).unwrap(),
        Guid::from_ascii(TEXT).unwrap()
    );
    let failure = Guid::from_utf16(&units[..35]).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_RangeError");
    assert!(failure.message().contains("35 UTF-16 units"));
}

#[test]
fn std_hash_uses_the_packed_field_image() {
    assert_eq!(Guid::new().hash_code() as u64, 0x7e37_c433_4630_b6d9);
    let guid = Guid::from_ascii(TEXT).unwrap();
    assert_eq!(guid.hash_code() as u64, 0x299a_4c73_5585_28e1);
    let set: HashSet<Guid> = [guid, guid, Guid::new()].into_iter().collect();
    assert_eq!(set.len(), 2);
}

#[test]
fn components_uuid_and_assignment_round_trip() {
    let guid = Guid::from_components(i32::MIN, 1, 2, 0x0102, 3, 4, 5, 6, 7, 8);
    assert_eq!(guid.to_string(), "80000000-0001-0002-0102-030405060708");
    let uuid: Uuid = guid.into();
    assert_eq!(uuid.data4, [1, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!(Guid::from(uuid), guid);
    let mut other = Guid::new();
    other.assign_uuid(uuid);
    assert!(other.is_same(guid) && !other.is_not_same(guid));
    assert_eq!(
        other.to_ext_string()[..8],
        "80000000".encode_utf16().collect::<Vec<_>>()[..]
    );
}

#[test]
fn assignment_adapters_return_the_assigned_guid_for_chaining() {
    let mut guid = Guid::new();
    let source = Guid::from_components(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
    let returned_address = guid.assign_returning(source) as *mut Guid;
    assert_eq!(returned_address, &mut guid as *mut Guid);
    let uuid = source.to_uuid();
    let returned_address = guid.assign_uuid_returning(uuid) as *mut Guid;
    assert_eq!(returned_address, &mut guid as *mut Guid);
    guid.assign_returning(Guid::new())
        .assign_uuid_returning(uuid);
    assert_eq!(guid.to_uuid(), uuid);
}
