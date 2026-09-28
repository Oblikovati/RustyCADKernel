//! Standard_Failure messages, printing and copies, and Standard_OutOfMemory's buffer.
use rustycad_tkernel::standard::{Failure, exceptions};

#[test]
fn what_is_the_message_for_ordinary_failures() {
    let failure = Failure::new("head\0tail");
    assert_eq!(b"head", failure.what());
    assert_eq!("head", failure.message());
    assert_eq!(b"", Failure::default().what());
    assert_eq!("Standard_Failure: head", failure.to_string());
    assert_eq!("Standard_Failure", Failure::default().to_string());
}
#[test]
fn explicit_stack_is_printed_after_the_message() {
    let failure = Failure::with_type("Standard_Failure", "bad", "\ntrace");
    assert_eq!("\ntrace", failure.stack_string());
    assert_eq!("Standard_Failure: bad\ntrace", failure.to_string());
    let copy = failure.clone();
    assert_eq!(failure, copy);
    assert_eq!("\ntrace", copy.stack_string());
}
#[test]
fn out_of_memory_keeps_its_message_out_of_print() {
    let failure = exceptions::out_of_memory("no memory");
    assert_eq!("Standard_OutOfMemory", failure.exception_type());
    assert_eq!(b"no memory", failure.what());
    assert_eq!("", failure.message());
    assert_eq!("Standard_OutOfMemory", failure.to_string());
    assert!(failure.is_kind("Standard_ProgramError"));
    assert_eq!(b"", exceptions::out_of_memory("").what());
}
#[test]
fn out_of_memory_truncates_at_1023_bytes() {
    let long = "x".repeat(2000);
    assert_eq!(1023, exceptions::out_of_memory(&long).what().len());
    // 1022 ASCII bytes and a two-byte character: the cut splits the character.
    let split = format!("{}é", "a".repeat(1022));
    let failure = exceptions::out_of_memory(&split);
    assert_eq!(1023, failure.what().len());
    assert_eq!(0xc3, failure.what()[1022]);
    assert_eq!(b"cut", exceptions::out_of_memory("cut\0off").what());
}
#[test]
fn set_message_string_only_applies_to_out_of_memory() {
    let mut failure = exceptions::out_of_memory("first");
    let copy = failure.clone();
    failure.set_message_string("second").unwrap();
    assert_eq!(b"second", failure.what());
    assert_eq!(b"first", copy.what());
    failure.set_message_string("").unwrap();
    assert_eq!(b"", failure.what());
    let refused = Failure::new("plain").set_message_string("x").unwrap_err();
    assert_eq!("Standard_TypeMismatch", refused.exception_type());
    assert!(refused.message().contains("Standard_Failure"));
}
