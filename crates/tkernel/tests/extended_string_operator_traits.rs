//! Native-backed ExtendedString comparison and hash trait behavior.
use rustycad_tkernel::ncollection::HashKey;
use rustycad_tkernel::tcollection::ExtendedString;

const HASH_FIXTURES: &[(&[u16], usize)] = &[
    (&[], 14_695_981_039_346_656_037),
    (&[0x00e9], 17_841_711_379_143_803_052),
    (&[97, 98], 7_301_104_965_762_792_192),
    (&[65, 0, 66], 12_438_028_569_082_509_017),
    (&[65, 0x0100], 2_500_931_821_420_542_387),
];

#[test]
fn object_operators_keep_embedded_zero_prefix_and_unsigned_order() {
    let embedded = ExtendedString::from_raw(&[65, 0, 66]);
    let different = ExtendedString::from_raw(&[65, 0, 67]);
    let longer = ExtendedString::from_raw(&[65, 0, 66, 0]);
    let high = ExtendedString::from_raw(&[65, 0x0100]);
    let low = ExtendedString::from_raw(&[65, 0x00ff]);

    assert_ne!(embedded, different);
    assert!(embedded.is_different(&different));
    assert!(embedded < different);
    assert!(embedded.is_less(&different));
    assert!(different > embedded);
    assert!(different.is_greater(&embedded));
    assert!(embedded < longer);
    assert!(high > low);
}

#[test]
#[cfg(target_pointer_width = "64")]
fn hash_trait_matches_hash_code_for_native_boundary_values() {
    for (units, expected) in HASH_FIXTURES {
        let value = ExtendedString::from_raw(units);
        assert_eq!(value.hash_code(), *expected);
        assert_eq!(value.occt_hash(), *expected);
    }
}
