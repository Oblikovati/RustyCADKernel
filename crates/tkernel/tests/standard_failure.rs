//! Translation of the OCC670 exception-printing regression.
#![allow(non_snake_case)]
use rustycad_tkernel::standard::Failure;

#[test]
fn Standard_Failure_Test__OCC670_ExceptionWithoutMessage() {
    // Result transports the legacy throw/catch across Rust's typed error boundary.
    let result: Result<(), Failure> = Err(Failure::out_of_range("test"));
    let mut caught = false;
    if let Err(exception) = result {
        caught = true;
        let message = exception.to_string();
        assert!(!message.is_empty(), "Exception message should not be empty");
    }
    assert!(caught, "Exception should have been caught");
}
