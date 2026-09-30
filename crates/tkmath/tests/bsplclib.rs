//! Direct translation of BSplCLib_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkmath::bsplclib;

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

#[test]
fn BSplCLibTest__UnitWeights_SmallSize_ReturnsNonOwning() {
    let aNbElems: i32 = 10;
    let aWeights: Array1<'static, f64> = bsplclib::unit_weights(aNbElems).unwrap();

    assert_eq!(aWeights.lower(), 1);
    assert_eq!(aWeights.upper(), aNbElems);
    assert_eq!(aWeights.len() as i32, aNbElems);
    assert!(!aWeights.is_deletable());

    for i in 1..=aNbElems {
        EXPECT_DOUBLE_EQ(aWeights[i], 1.0);
    }
}

#[test]
fn BSplCLibTest__UnitWeights_MaxSize_ReturnsNonOwning() {
    let aNbElems: i32 = bsplclib::max_unit_weights_size();
    let aWeights: Array1<'static, f64> = bsplclib::unit_weights(aNbElems).unwrap();

    assert_eq!(aWeights.len() as i32, aNbElems);
    assert!(!aWeights.is_deletable());

    EXPECT_DOUBLE_EQ(aWeights[1], 1.0);
    EXPECT_DOUBLE_EQ(aWeights[aNbElems], 1.0);
}

#[test]
fn BSplCLibTest__UnitWeights_OverMaxSize_ReturnsOwning() {
    let aNbElems: i32 = bsplclib::max_unit_weights_size() + 1;
    let aWeights: Array1<'static, f64> = bsplclib::unit_weights(aNbElems).unwrap();

    assert_eq!(aWeights.lower(), 1);
    assert_eq!(aWeights.upper(), aNbElems);
    assert_eq!(aWeights.len() as i32, aNbElems);
    assert!(aWeights.is_deletable());

    for i in 1..=aNbElems {
        EXPECT_DOUBLE_EQ(aWeights[i], 1.0);
    }
}

#[test]
fn BSplCLibTest__UnitWeights_SingleElement() {
    let aWeights: Array1<'static, f64> = bsplclib::unit_weights(1).unwrap();

    assert_eq!(aWeights.len(), 1);
    assert!(!aWeights.is_deletable());
    EXPECT_DOUBLE_EQ(aWeights[1], 1.0);
}

#[test]
fn BSplCLibTest__MaxUnitWeightsSize_IsPositive() {
    assert!(bsplclib::max_unit_weights_size() > 0);
    assert_eq!(bsplclib::max_unit_weights_size(), 2049);
}
