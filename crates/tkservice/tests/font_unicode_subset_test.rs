//! Direct translation of Font_UnicodeSubset_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkservice::font::FontUnicodeSubset;

#[test]
fn Font_UnicodeSubsetTest__CountIncludesEveryDeclaredSubset() {
    assert_eq!(4, FontUnicodeSubset::Nb as i32);
    assert!(FontUnicodeSubset::Western < FontUnicodeSubset::Nb);
    assert!(FontUnicodeSubset::Korean < FontUnicodeSubset::Nb);
    assert!(FontUnicodeSubset::Cjk < FontUnicodeSubset::Nb);
    assert!(FontUnicodeSubset::Arabic < FontUnicodeSubset::Nb);
}
