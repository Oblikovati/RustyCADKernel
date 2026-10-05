//! Base copy construction slices derived exception identity and private storage.
use rustycad_tkernel::standard::{Failure, exceptions};

#[test]
fn failure_base_copy_slices_type_and_shares_base_strings() {
    let source = Failure::with_type("Standard_DomainError", "message", "stack");
    let copy = Failure::copy_base(&source);
    assert_eq!(copy.exception_type(), "Standard_Failure");
    assert_eq!(copy.message().as_ptr(), source.message().as_ptr());
    assert_eq!(copy.stack_string().as_ptr(), source.stack_string().as_ptr());
    assert_eq!(copy.to_string(), "Standard_Failure: messagestack");
    assert_eq!(source.exception_type(), "Standard_DomainError");
}

#[test]
fn failure_base_copy_omits_out_of_memory_private_buffer() {
    let mut source = exceptions::out_of_memory("private");
    source.assign_base(&Failure::with_type("Standard_Failure", "", "trace"));
    let copy = Failure::copy_base(&source);
    assert_eq!(copy.what(), b"");
    assert_eq!(copy.stack_string().as_ptr(), source.stack_string().as_ptr());
    assert_eq!(copy.exception_type(), "Standard_Failure");
    assert_eq!(source.what(), b"private");
}
