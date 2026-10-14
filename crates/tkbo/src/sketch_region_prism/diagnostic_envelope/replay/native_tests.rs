// SPDX-License-Identifier: GPL-2.0-only
//! Independent field bytes pin native payload layout without formatting or hierarchy inference.
use super::native;
use rustycad_tkernel::standard::{Failure, exceptions::out_of_memory_bytes};
#[test]
fn typed_native_encoding_matches_independent_literal_bytes() {
    let value = Failure::with_type_bytes("X", &[255, 17], &[254]).with_exception_parent("P");
    let expected = [
        1, 0, 0, 0, 0, 0, 0, 0, b'X', 1, 1, 0, 0, 0, 0, 0, 0, 0, b'P', 2, 0, 0, 0, 0, 0, 0, 0, 255,
        17, 1, 0, 0, 0, 0, 0, 0, 0, 254, 0,
    ];
    let mut actual = Vec::new();
    native::append(&value, &mut actual);
    assert_eq!(actual, expected);
}
#[test]
fn typed_native_encoding_keeps_none_empty_and_base_versus_derived_messages() {
    let absent = Failure::with_type("Standard_OutOfMemory", "", "");
    let mut empty = out_of_memory_bytes(&[]);
    empty.set_stack_string("");
    let mut derived = empty.clone();
    derived.assign_base(&Failure::with_type_bytes("Base", &[255], &[254]));
    derived.set_message_bytes(&[253]).unwrap();
    assert_ne!(encode(&absent), encode(&empty));
    assert_ne!(encode(&empty), encode(&derived));
    assert_eq!(
        encode(&empty).strip_suffix(&[1, 0, 0, 0, 0, 0, 0, 0, 0]),
        Some(&encode(&absent)[..encode(&absent).len() - 1])
    );
}
fn encode(value: &Failure) -> Vec<u8> {
    let mut bytes = Vec::new();
    native::append(value, &mut bytes);
    bytes
}
