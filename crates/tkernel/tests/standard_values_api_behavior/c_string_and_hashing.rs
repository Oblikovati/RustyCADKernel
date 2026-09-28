//! Standard_HashUtils engines and templates, Standard_CString and Standard_CStringHasher.
//! The hash constants are native OCCT results on the x86-64 Linux oracle profile.
use rustycad_tkernel::ncollection::HashPolicy;
use rustycad_tkernel::standard::c_string::{self, CStringHasher};
use rustycad_tkernel::standard::hash_bytes;
use rustycad_tkernel::standard::hashing::{self, HashWord, fnv_hash, murmur_hash};

#[test]
fn murmur_engines_match_native_values() {
    assert_eq!(murmur_hash::murmur_hash_64a(b"", 0), 0);
    assert_eq!(murmur_hash::murmur_hash_64a(b"a", 0), 0x0717_17d2_d36b_6b11);
    assert_eq!(murmur_hash::murmur_hash_2a(b"a", 0), 0x9268_5f5e);
    assert_eq!(
        murmur_hash::murmur_hash_64a(b"hello world", 0),
        0xd3ba_2368_a832_afce
    );
    assert_eq!(murmur_hash::murmur_hash_2a(b"hello world", 0), 0x44a8_1419);
}

#[test]
fn fnv_engines_sign_extend_plain_chars() {
    assert_eq!(fnv_hash::fnv_hash_1a(b"a", 0), 0x6100_98b3);
    assert_eq!(fnv_hash::fnv_hash_64a(b"a", 0), 0x0000_6100_0000_a4d3);
    // 0xff and 0x80 enter as the signed chars -1 and -128 on the oracle profile.
    assert_eq!(fnv_hash::fnv_hash_1a(&[0xff, 0x80], 0), 0x8003_0817);
    assert_eq!(
        fnv_hash::fnv_hash_64a(&[0xff, 0x80], 0),
        0x0003_8000_0003_0f57
    );
}

#[test]
fn seeds_follow_the_word_width() {
    assert_eq!(murmur_hash::optimal_seed::<u64>(), 0xA329_F1D3_A586);
    assert_eq!(murmur_hash::optimal_seed::<u32>(), 0xc70f_6907);
    assert_eq!(fnv_hash::optimal_seed::<u64>(), 0xcbf2_9ce4_8422_2325);
    assert_eq!(fnv_hash::optimal_seed::<u32>(), 0x811c_9dc5);
    assert_eq!(usize::MURMUR_SEED as u64, 0xA329_F1D3_A586);
}

#[test]
fn templates_hash_the_object_representation() {
    let size_hash: usize = hashing::hash(&0_i32);
    assert_eq!(size_hash as u64, 0x68c6_4920_fbde_a08f);
    assert_eq!(hashing::hash::<u32, i32>(&-1), 0x162f_8dbc);
    assert_eq!(
        fnv_hash::hash_combine(&[-1_i8], 1, fnv_hash::optimal_seed::<u64>()),
        0x509c_41b3_79fe_466e
    );
    assert_eq!(hash_bytes(b"") as u64, 0x1513_eba9_820e_1cb4);
    let chained: usize = hashing::hash_combine(&[1_u8, 2, 3, 4], 2, 7);
    assert_eq!(chained, murmur_hash::hash_combine(&[1_u8, 2], 2, 7_usize));
}

#[test]
#[should_panic(expected = "hash length 5 exceeds the 4-byte object representation")]
fn hash_combine_rejects_reads_past_the_object() {
    let _: u64 = murmur_hash::hash_combine(&[0_i32], 5, 0_u64);
}

#[test]
fn c_string_hasher_switches_engine_at_four_bytes() {
    assert_eq!(CStringHasher.hash_code(b"") as u64, 0xcbf2_9ce4_8422_2325);
    assert_eq!(
        CStringHasher.hash_code(b"abc") as u64,
        0xe71f_a219_0541_574b
    );
    assert_eq!(CStringHasher.hash_code(b"abcd"), hash_bytes(b"abcd"));
    // The C string ends at the first NUL.
    assert_eq!(
        CStringHasher.hash_code(b"ab\0cd"),
        CStringHasher.hash_code(b"ab")
    );
    assert!(HashPolicy::<Vec<u8>>::equal(
        &CStringHasher,
        &b"abc\0x".to_vec(),
        &b"abc".to_vec()
    ));
    assert_eq!(
        HashPolicy::<&str>::hash(&CStringHasher, &"abcd"),
        hash_bytes(b"abcd")
    );
}

#[test]
fn c_string_equality_and_atof() {
    assert!(c_string::is_equal(b"", b"\0"));
    assert!(!c_string::is_equal(b"abc", b"abd"));
    assert_eq!(c_string::atof(" 2.5e1xyz"), 25.0);
    assert_eq!(c_string::atof("abc"), 0.0);
    assert_eq!(c_string::atof("12\x0034"), 12.0);
    assert_eq!(c_string::atof("1e400"), f64::INFINITY);
}
