//! Failure, out-of-memory, stack capture and stack trace cases (tkernel_standard_runtime_failure_probe.cxx).
use super::{bit, next_text};
use crate::common::{hex, next};
use rustycad_tkernel::standard::{Failure, exceptions, stack_trace};
use std::str::SplitWhitespace;

/// ExceptionType, what(), GetMessageString(), GetStackString() and operator<< of one failure.
fn failure_fields(failure: &Failure) -> String {
    // GetMessageString() is what(); both columns come from the same Rust method.
    format!(
        "{} {} {} {} {}",
        failure.exception_type(),
        hex(failure.what()),
        hex(failure.what()),
        hex(failure.stack_string().as_bytes()),
        hex(failure.to_string().as_bytes())
    )
}
/// "runtime_failure <hex message>".
pub fn failure_case(input: &mut SplitWhitespace<'_>) -> String {
    let failure = Failure::new(&next_text(input));
    let copy = failure.clone();
    let mut assigned = Failure::default();
    assigned.clone_from(&failure);
    format!(
        "{} {} {} {}{} {}",
        failure_fields(&failure),
        failure_fields(&copy),
        failure_fields(&assigned),
        bit(std::ptr::eq(copy.what(), failure.what())),
        bit(std::ptr::eq(assigned.what(), failure.what())),
        failure_fields(&Failure::default())
    )
}
/// "runtime_failure_stack <hex message> <hex stack>".
pub fn failure_stack_case(input: &mut SplitWhitespace<'_>) -> String {
    let message = next_text(input);
    let stack = next_text(input);
    failure_fields(&Failure::with_type("Standard_Failure", &message, &stack))
}
/// A message token: "null" (nullptr) holds the same bytes as an empty message.
fn next_message(input: &mut SplitWhitespace<'_>) -> String {
    let token = input.next().expect("message token");
    if token == "null" {
        return String::new();
    }
    String::from_utf8(crate::common::decode(token)).expect("probe texts are UTF-8")
}
/// "runtime_out_of_memory <message> <new message>".
pub fn out_of_memory_case(input: &mut SplitWhitespace<'_>) -> String {
    let mut failure = exceptions::out_of_memory(&next_message(input));
    let copy = failure.clone();
    failure
        .set_message_string(&next_message(input))
        .expect("an out-of-memory failure has its buffer");
    format!(
        "{} {} {}",
        failure_fields(&copy),
        failure_fields(&failure),
        bit(copy.is_kind("Standard_ProgramError"))
    )
}
/// "H" for the header, "F" per frame, "T" for the footer, as the native `trace_shape`.
fn trace_shape(trace: &str) -> String {
    let bytes: &[u8] = trace.as_bytes();
    let mut shape = String::new();
    let mut start: usize = 0;
    while start < bytes.len() {
        let end: usize = bytes[start + 1..]
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(bytes.len(), |at| start + 1 + at);
        shape.push(match &bytes[start..end] {
            b"\n==Backtrace==" => 'H',
            b"\n=============" => 'T',
            _ => 'F',
        });
        start = end;
    }
    if shape.is_empty() { "-".into() } else { shape }
}
/// "runtime_stack_length <n>".
pub fn stack_length_case(input: &mut SplitWhitespace<'_>) -> String {
    Failure::set_default_stack_trace_length(next(input));
    let failure = Failure::new("m");
    let range = exceptions::out_of_range("r");
    let out_of_memory = exceptions::out_of_memory("o").to_string();
    let explicit = Failure::with_type("Standard_Failure", "e", "");
    let result = format!(
        "{} {} {} {}{} {}",
        Failure::default_stack_trace_length(),
        &trace_shape(failure.stack_string())[..1],
        &trace_shape(range.stack_string())[..1],
        &out_of_memory[..20],
        &trace_shape(&out_of_memory[20..])[..1],
        hex(explicit.to_string().as_bytes())
    );
    Failure::set_default_stack_trace_length(0);
    let stack = failure.stack_string();
    format!(
        "{result} {}",
        bit(stack.len() > 28 && stack.ends_with("\n============="))
    )
}
/// "runtime_stack_trace <size> <traces> <skip> <hex prefix>".
pub fn stack_trace_case(input: &mut SplitWhitespace<'_>) -> String {
    let (size, traces, skip): (i32, i32, i32) = (next(input), next(input), next(input));
    let prefix = next_text(input);
    let mut buffer = prefix.clone();
    let done = stack_trace(&mut buffer, size, traces, skip);
    format!(
        "{} {} {}",
        bit(done),
        hex(&buffer.as_bytes()[..prefix.len()]),
        trace_shape(&buffer[prefix.len()..])
    )
}
