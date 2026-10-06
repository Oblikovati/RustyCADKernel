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
