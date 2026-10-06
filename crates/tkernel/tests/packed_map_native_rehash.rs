//! Public capability availability and borrowed custom-word fallback remain valid across crates.
#[path = "packed_map_native_rehash/borrowed.rs"]
mod borrowed;
use rustycad_tkernel::ncollection::PackedWord;
#[test]
fn primitive_words_expose_only_their_typed_kernel_reader() {
    assert!(<u32 as PackedWord>::NATIVE_REHASH_READER.is_some());
    assert!(<u64 as PackedWord>::NATIVE_REHASH_READER.is_some());
}
#[test]
fn nonstatic_custom_words_keep_the_transactional_fallback() {
    let lifetime = 7_u8;
    borrowed::verify(&lifetime);
}
#[path = "packed_map_native_rehash/oracle.rs"]
mod oracle;
#[path = "packed_map_native_rehash/trace.rs"]
mod trace;
macro_rules! widths {
    ($($name:ident: $ty:ty),+) => { $(#[test] fn $name() { oracle::compare::<$ty>(stringify!($ty)); })+ };
}
widths!(no_scratch_i8: i8, no_scratch_u8: u8, no_scratch_i16: i16, no_scratch_u16: u16, no_scratch_i32: i32, no_scratch_u32: u32, no_scratch_i64: i64, no_scratch_u64: u64, no_scratch_usize: usize, no_scratch_isize: isize, no_scratch_bool: bool);
#[path = "packed_map_native_rehash/key_callbacks.rs"]
mod key_callbacks;
