// SPDX-License-Identifier: GPL-2.0-only
//! ADR0048 pins stored provenance separately from inferred hierarchy and virtual message bytes.
use super::*;
#[test]
fn provenance_base_bytes_and_absent_overrides_remain_distinct() {
    let value = Failure::with_type_bytes("Standard_OutOfMemory", &[255, 17], &[254, 29]);
    assert_eq!(value.exception_parent, None);
    assert!(value.own_buffer.is_none());
    assert_eq!(value.message_bytes(), &[255, 17]);
    assert_eq!(value.what(), value.message_bytes());
    assert_eq!(value.stack_bytes(), &[254, 29]);
    assert!(value.is_kind("Standard_ProgramError"));
}
#[test]
fn provenance_empty_derived_buffer_is_present_even_when_virtual_bytes_match_base() {
    let ordinary = Failure::with_type("Standard_OutOfMemory", "", "");
    let derived = owned_buffer(&[]);
    assert!(ordinary.own_buffer.is_none());
    assert_eq!(derived.own_buffer.as_deref(), Some([].as_slice()));
    assert_eq!(ordinary.what(), derived.what());
    assert_ne!(ordinary, derived);
}
#[test]
fn provenance_explicit_parent_is_not_inferred_from_exception_type() {
    let ordinary = Failure::with_type("Standard_OutOfRange", "", "");
    let explicit = ordinary
        .clone()
        .with_exception_parent("Standard_ProgramError");
    assert_eq!(ordinary.exception_parent, None);
    assert_eq!(explicit.exception_parent, Some("Standard_ProgramError"));
    assert!(ordinary.is_kind("Standard_DomainError"));
    assert!(!explicit.is_kind("Standard_DomainError"));
    assert!(explicit.is_kind("Standard_ProgramError"));
}
#[test]
fn provenance_base_assignment_preserves_derived_buffer_and_parent_override() {
    let mut derived = owned_buffer(&[255, 17]).with_exception_parent("Standard_Failure");
    let base = Failure::with_type_bytes("Base", &[254, 29], &[253, 31]);
    derived.assign_base(&base);
    assert_eq!(derived.exception_type(), "Standard_OutOfMemory");
    assert_eq!(derived.exception_parent, Some("Standard_Failure"));
    assert_eq!(derived.message_bytes(), &[254, 29]);
    assert_eq!(derived.stack_bytes(), &[253, 31]);
    assert_eq!(derived.what(), &[255, 17]);
    assert_eq!(derived.own_buffer.as_deref(), Some([255, 17].as_slice()));
}
#[test]
fn provenance_cloned_buffers_survive_all_source_mutations() {
    let mut original = owned_buffer(&[255, 17]).with_exception_parent("Standard_Failure");
    original.assign_base(&Failure::with_type_bytes("Base", &[254, 29], &[253, 31]));
    let saved = original.clone();
    original.set_message("replacement base");
    original.set_stack_string("replacement stack");
    original.set_message_bytes(&[252, 33]).unwrap();
    original = original.with_exception_parent("Standard_DomainError");
    assert_eq!(saved.message_bytes(), &[254, 29]);
    assert_eq!(saved.stack_bytes(), &[253, 31]);
    assert_eq!(saved.what(), &[255, 17]);
    assert_eq!(saved.exception_parent, Some("Standard_Failure"));
    assert_ne!(saved, original);
}
fn owned_buffer(bytes: &[u8]) -> Failure {
    let mut value = out_of_memory_bytes(bytes);
    // Explicit stack contents isolate this fixture from the legacy global stack-capture setting.
    value.set_stack_string("");
    value
}
