//! Native failure strings preserve arbitrary non-NUL bytes.
use rustycad_tkernel::standard::Failure;

#[test]
fn failure_byte_message_stops_at_nul_and_preserves_non_utf8() {
    let failure = Failure::from_message_bytes(&[0xff, b'A', 0, b'B']);
    assert_eq!(failure.what(), &[0xff, b'A']);
    assert_eq!(failure.message_bytes(), &[0xff, b'A']);
    assert_eq!(failure.exception_type(), "Standard_Failure");
    assert_eq!(failure.message(), "\u{fffd}A");
}

#[test]
fn failure_byte_stack_and_copy_preserve_shared_storage() {
    let source = Failure::with_type_bytes("Standard_DomainError", &[0xff], &[0xfe, 0, 1]);
    let copy = Failure::copy_base(&source);
    assert_eq!(copy.what(), &[0xff]);
    assert_eq!(copy.stack_bytes(), &[0xfe]);
    assert_eq!(copy.what().as_ptr(), source.what().as_ptr());
    assert_eq!(copy.stack_bytes().as_ptr(), source.stack_bytes().as_ptr());
}

#[test]
fn failure_byte_print_writes_original_message_and_stack() {
    let failure = Failure::with_type_bytes("Standard_Failure", &[0xff], &[0xfe]);
    let mut output = Vec::new();
    failure.print_to(&mut output).unwrap();
    assert_eq!(
        output,
        [b"Standard_Failure: ".as_slice(), &[0xff, 0xfe]].concat()
    );
}

#[test]
fn failure_empty_byte_message_prints_type_then_stack() {
    let failure = Failure::with_type_bytes("Standard_Failure", &[0, 0xff], &[0xfe]);
    let mut output = Vec::new();
    failure.print_to(&mut output).unwrap();
    assert_eq!(output, [b"Standard_Failure".as_slice(), &[0xfe]].concat());
}

#[test]
fn failure_byte_assignment_keeps_destination_type_and_source_storage() {
    let source = Failure::with_type_bytes("Standard_Failure", &[255], &[254]);
    let mut target = Failure::with_type("Standard_DomainError", "old", "trace");
    target.assign_base(&source);
    assert_eq!(target.what().as_ptr(), source.what().as_ptr());
    assert_eq!(target.stack_bytes().as_ptr(), source.stack_bytes().as_ptr());
    assert_eq!(target.exception_type(), "Standard_DomainError");
    assert_eq!(target.stack_string(), "\u{fffd}");
}

struct RejectingFailureWriter;

impl std::io::Write for RejectingFailureWriter {
    fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::other("injected failure writer rejection"))
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn failure_byte_print_reports_writer_error_without_changing_message() {
    let failure = Failure::from_message_bytes(&[255]);
    let error = failure.print_to(&mut RejectingFailureWriter).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::Other);
    assert_eq!(failure.what(), &[255]);
}

#[test]
fn failure_byte_insertion_returns_original_stream_for_chaining() {
    let failure = Failure::with_type_bytes("Standard_Failure", &[255], &[]);
    let mut output = Vec::new();
    let pointer = &output as *const Vec<u8>;
    assert_eq!(
        failure.insert_into(&mut output).unwrap() as *const Vec<u8>,
        pointer
    );
    assert_eq!(output.last(), Some(&255));
}

#[test]
fn out_of_memory_byte_constructor_truncates_to_native_buffer() {
    let failure = rustycad_tkernel::standard::exceptions::out_of_memory_bytes(&[255; 1030]);
    assert_eq!(failure.what(), &[255; 1023]);
    assert!(failure.message_bytes().is_empty());
    assert_eq!(failure.to_string(), "Standard_OutOfMemory");
}

#[test]
fn out_of_memory_byte_setter_obeys_nul_and_preserves_other_copies() {
    let mut target = rustycad_tkernel::standard::exceptions::out_of_memory_bytes(&[255]);
    let copy = target.clone();
    target.set_message_bytes(&[254, 0, 253]).unwrap();
    assert_eq!(target.what(), &[254]);
    assert_eq!(copy.what(), &[255]);
    assert!(target.message_bytes().is_empty());
}

#[test]
fn out_of_memory_byte_setter_rejects_base_failure() {
    let mut target = Failure::new("base");
    let error = target.set_message_bytes(&[255]).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_TypeMismatch");
    assert!(error.message().contains("Standard_Failure"));
    assert_eq!(target.what(), b"base");
}
