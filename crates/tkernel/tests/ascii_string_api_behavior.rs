//! Behavior of the complete TCollection_AsciiString, TCollection_HAsciiString and
//! TCollection API, including the native quirks the differential probe confirmed.
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::{
    AsciiString, ExtendedString, HAsciiString, next_prime_for_map,
};

fn bytes(text: &AsciiString) -> &[u8] {
    text.as_bytes()
}

fn class(failure: Failure) -> &'static str {
    failure.exception_type()
}

include!("ascii_string_api_behavior/construction.rs");
include!("ascii_string_api_behavior/editing.rs");
include!("ascii_string_api_behavior/queries.rs");
include!("ascii_string_api_behavior/handles.rs");
