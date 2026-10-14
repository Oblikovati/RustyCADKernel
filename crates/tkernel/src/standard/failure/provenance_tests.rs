// SPDX-License-Identifier: GPL-2.0-only
//! New accessors observe the unchanged stored fields pinned by provenance_baseline_tests.
use super::*;
#[test]
fn explicit_parent_accessor_does_not_replace_absence_with_inferred_hierarchy() {
    let value = Failure::with_type("Standard_OutOfRange", "", "");
    let explicit = value.clone().with_exception_parent("Standard_ProgramError");
    assert_eq!(value.explicit_exception_parent(), None);
    assert!(value.is_kind("Standard_DomainError"));
    assert_eq!(
        explicit.explicit_exception_parent(),
        Some("Standard_ProgramError")
    );
    assert_eq!(
        explicit.explicit_exception_parent(),
        explicit.exception_parent
    );
}
#[test]
fn derived_message_accessor_preserves_absent_empty_and_non_utf8_buffers() {
    let ordinary = Failure::with_type("Standard_OutOfMemory", "", "");
    assert_eq!(ordinary.derived_message_bytes(), None);
    for bytes in [&[][..], &[255, 17][..]] {
        let value = out_of_memory_bytes(bytes);
        assert_eq!(value.derived_message_bytes(), Some(bytes));
        assert_eq!(value.derived_message_bytes(), value.own_buffer.as_deref());
        assert!(value.message_bytes().is_empty());
        assert_eq!(value.what(), bytes);
    }
}
#[test]
fn derived_accessor_preserves_separate_base_assignment_and_cloned_storage() {
    let mut value = out_of_memory_bytes(&[255, 17]);
    value.assign_base(&Failure::with_type_bytes("Base", &[254, 29], &[253, 31]));
    let original = value.clone();
    assert_eq!(value.derived_message_bytes(), Some([255, 17].as_slice()));
    assert_eq!(value.message_bytes(), &[254, 29]);
    assert_eq!(value.stack_bytes(), &[253, 31]);
    assert_eq!(value, original);
    value.set_message_bytes(&[252, 33]).unwrap();
    assert_eq!(original.derived_message_bytes(), Some([255, 17].as_slice()));
}
