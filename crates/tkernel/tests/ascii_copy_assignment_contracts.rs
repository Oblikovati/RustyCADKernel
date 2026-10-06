//! Supplemental assignment checks mirror the pinned native return and reuse contracts.
use rustycad_tkernel::tcollection::AsciiString;

fn verify_assignment(before: &[u8], input: &[u8]) {
    let mut destination = AsciiString::from_raw(before);
    let source = AsciiString::from_raw(input);
    let prior_storage = destination.as_bytes_with_nul().as_ptr();
    let destination_address = std::ptr::from_mut(&mut destination);
    let returned_address = std::ptr::from_mut(destination.assign_from(&source));
    assert_eq!(returned_address, destination_address);
    assert_eq!(destination.as_bytes(), input);
    assert_eq!(source.as_bytes(), input);
    if !input.is_empty() && input.len() <= before.len() {
        assert_eq!(destination.as_bytes_with_nul().as_ptr(), prior_storage);
    }
}

#[test]
fn assignment_returns_destination_and_reuses_shorter_or_equal_storage() {
    verify_assignment(b"old", b"new");
    verify_assignment(b"longer", b"x");
}

#[test]
fn assignment_copies_longer_and_embedded_zero_spans_without_mutating_source() {
    verify_assignment(b"x", b"longer");
    verify_assignment(b"longer", b"x\0y");
}

#[test]
fn assignment_handles_empty_source_and_destination() {
    verify_assignment(b"longer", b"");
    verify_assignment(b"", b"new");
}

#[test]
fn self_assignment_returns_destination_and_preserves_storage_and_bytes() {
    let mut destination = AsciiString::from_raw(b"h\0i");
    let prior_storage = destination.as_bytes_with_nul().as_ptr();
    let address = std::ptr::from_mut(&mut destination);
    assert_eq!(std::ptr::from_mut(destination.assign_self()), address);
    assert_eq!(destination.as_bytes_with_nul().as_ptr(), prior_storage);
    assert_eq!(destination.as_bytes(), b"h\0i");
}
