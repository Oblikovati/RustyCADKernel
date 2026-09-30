//! Direct translation of BSplSLib_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkmath::bsplslib;

// GoogleTest EXPECT_DOUBLE_EQ uses a four-ULP comparison.
fn EXPECT_DOUBLE_EQ(a: f64, b: f64) {
    assert!(!a.is_nan() && !b.is_nan());
    assert!(ordered_bits(a).abs_diff(ordered_bits(b)) <= 4, "{a} != {b}");
}
fn ordered_bits(value: f64) -> u64 {
    let bits: u64 = value.to_bits();
    if bits >> 63 != 0 {
        (!bits).wrapping_add(1)
    } else {
        bits | (1 << 63)
    }
}

// NCollection_Array2::ColLength and RowLength.
fn col_length(array: &Array2<'_, f64>) -> i32 {
    array.nb_rows() as i32
}
fn row_length(array: &Array2<'_, f64>) -> i32 {
    array.nb_columns() as i32
}

#[test]
fn BSplSLibTest__UnitWeights_SmallSurface_ReturnsNonOwning() {
    let aNbU: i32 = 4;
    let aNbV: i32 = 5;
    let aWeights: Array2<'static, f64> = bsplslib::unit_weights(aNbU, aNbV).unwrap();

    assert_eq!(col_length(&aWeights), aNbU);
    assert_eq!(row_length(&aWeights), aNbV);
    assert_eq!(aWeights.len() as i32, aNbU * aNbV);
    assert!(!aWeights.is_deletable());

    for i in 1..=aNbU {
        for j in 1..=aNbV {
            EXPECT_DOUBLE_EQ(aWeights[(i, j)], 1.0);
        }
    }
}

#[test]
fn BSplSLibTest__UnitWeights_MaxBezierSize_ReturnsNonOwning() {
    // Max Bezier: 26x26 = 676 poles, well within limit
    let aNbU: i32 = 26;
    let aNbV: i32 = 26;
    let aWeights: Array2<'static, f64> = bsplslib::unit_weights(aNbU, aNbV).unwrap();

    assert_eq!(aWeights.len() as i32, aNbU * aNbV);
    assert!(!aWeights.is_deletable());
    EXPECT_DOUBLE_EQ(aWeights[(1, 1)], 1.0);
    EXPECT_DOUBLE_EQ(aWeights[(aNbU, aNbV)], 1.0);
}

#[test]
fn BSplSLibTest__UnitWeights_AtMaxLimit_ReturnsNonOwning() {
    // Exactly at the limit: 2049 total poles
    let aNbU: i32 = 3;
    let aNbV: i32 = 683;
    let aWeights: Array2<'static, f64> = bsplslib::unit_weights(aNbU, aNbV).unwrap();

    assert_eq!(aWeights.len() as i32, aNbU * aNbV);
    assert!(!aWeights.is_deletable());
    EXPECT_DOUBLE_EQ(aWeights[(1, 1)], 1.0);
    EXPECT_DOUBLE_EQ(aWeights[(aNbU, aNbV)], 1.0);
}

#[test]
fn BSplSLibTest__UnitWeights_OverMaxLimit_ReturnsOwning() {
    // Over the limit: needs allocation
    let aNbU: i32 = 50;
    let aNbV: i32 = 50;
    let aWeights: Array2<'static, f64> = bsplslib::unit_weights(aNbU, aNbV).unwrap();

    assert_eq!(col_length(&aWeights), aNbU);
    assert_eq!(row_length(&aWeights), aNbV);
    assert_eq!(aWeights.len() as i32, aNbU * aNbV);
    assert!(aWeights.is_deletable());

    for i in 1..=aNbU {
        for j in 1..=aNbV {
            EXPECT_DOUBLE_EQ(aWeights[(i, j)], 1.0);
        }
    }
}

#[test]
fn BSplSLibTest__UnitWeights_SingleElement() {
    let aWeights: Array2<'static, f64> = bsplslib::unit_weights(1, 1).unwrap();

    assert_eq!(aWeights.len(), 1);
    assert!(!aWeights.is_deletable());
    EXPECT_DOUBLE_EQ(aWeights[(1, 1)], 1.0);
}
