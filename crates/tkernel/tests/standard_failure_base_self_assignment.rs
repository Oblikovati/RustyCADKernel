//! Safe adapter for the native Standard_Failure self-assignment branch.
use rustycad_tkernel::standard::{Failure, exceptions};

#[test]
fn self_assignment_keeps_identity_buffers_and_dynamic_type() {
    let mut failure = exceptions::out_of_memory("reserved");
    let address = &failure as *const Failure;
    let message_pointer = failure.message().as_ptr();
    let stack_pointer = failure.stack_string().as_ptr();
    let derived_buffer = failure.what().as_ptr();

    assert_eq!(failure.assign_base_self() as *const Failure, address);
    assert_eq!(failure.message().as_ptr(), message_pointer);
    assert_eq!(failure.stack_string().as_ptr(), stack_pointer);
    assert_eq!(failure.what().as_ptr(), derived_buffer);
    assert_eq!(failure.exception_type(), "Standard_OutOfMemory");
    assert_eq!(failure.what(), b"reserved");
}
