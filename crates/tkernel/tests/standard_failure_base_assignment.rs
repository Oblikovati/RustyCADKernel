//! Base assignment preserves the dynamic exception identity and derived storage.
use rustycad_tkernel::standard::{Failure, exceptions};

#[test]
fn failure_base_assignment_preserves_type_and_shares_strings() {
    let source = Failure::with_type("Standard_Failure", "new", "trace");
    let mut target = Failure::with_type("Standard_DomainError", "old", "old trace");
    let address = &target as *const Failure;
    assert_eq!(target.assign_base(&source) as *const Failure, address);
    assert_eq!(target.exception_type(), "Standard_DomainError");
    assert_eq!(target.message().as_ptr(), source.message().as_ptr());
    assert_eq!(
        target.stack_string().as_ptr(),
        source.stack_string().as_ptr()
    );
    assert_eq!(target.to_string(), "Standard_DomainError: newtrace");
}

#[test]
fn failure_base_assignment_preserves_out_of_memory_buffer() {
    let source = Failure::with_type("Standard_Failure", "base", "stack");
    let mut target = exceptions::out_of_memory("reserved");
    target.assign_base(&source);
    assert_eq!(target.what(), b"reserved");
    assert_eq!(target.exception_type(), "Standard_OutOfMemory");
    assert_eq!(target.to_string(), "Standard_OutOfMemory: basestack");
}

#[test]
fn failure_base_assignment_reads_base_state_of_derived_source() {
    let source = exceptions::out_of_memory("reserved");
    let mut target = Failure::with_type("Standard_DomainError", "old", "trace");
    target.assign_base(&source);
    assert!(target.message().is_empty());
    assert_eq!(target.what(), b"");
    assert_eq!(target.stack_string(), source.stack_string());
    assert_eq!(target.exception_type(), "Standard_DomainError");
    assert_eq!(source.what(), b"reserved");
}
