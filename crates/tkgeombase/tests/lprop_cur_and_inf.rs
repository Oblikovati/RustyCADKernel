//! Direct translation of LProp_CurAndInf_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::{EXPECT_DOUBLE_EQ, EXPECT_THROW};
use rustycad_tkgeombase::lprop::{CIType, CurAndInf};

#[test]
fn LProp_CurAndInfTest__DefaultConstructor_IsEmpty() {
    let aResult = CurAndInf::new();
    assert!(aResult.is_empty());
    assert_eq!(aResult.nb_points(), 0);
}

#[test]
fn LProp_CurAndInfTest__AddInflection() {
    let mut aResult = CurAndInf::new();
    aResult.add_inflection(1.5);

    assert!(!aResult.is_empty());
    assert_eq!(aResult.nb_points(), 1);
    EXPECT_DOUBLE_EQ(aResult.parameter(1).unwrap(), 1.5);
    assert_eq!(aResult.type_of(1).unwrap(), CIType::Inflection);
}

#[test]
fn LProp_CurAndInfTest__AddExtCur_Minimum() {
    let mut aResult = CurAndInf::new();
    aResult.add_ext_cur(2.0, true);

    assert_eq!(aResult.nb_points(), 1);
    EXPECT_DOUBLE_EQ(aResult.parameter(1).unwrap(), 2.0);
    assert_eq!(aResult.type_of(1).unwrap(), CIType::MinCur);
}

#[test]
fn LProp_CurAndInfTest__AddExtCur_Maximum() {
    let mut aResult = CurAndInf::new();
    aResult.add_ext_cur(3.0, false);

    assert_eq!(aResult.nb_points(), 1);
    EXPECT_DOUBLE_EQ(aResult.parameter(1).unwrap(), 3.0);
    assert_eq!(aResult.type_of(1).unwrap(), CIType::MaxCur);
}

#[test]
fn LProp_CurAndInfTest__MultiplePoints_SortedByParameter() {
    let mut aResult = CurAndInf::new();
    // Add in non-sorted order
    aResult.add_inflection(5.0);
    aResult.add_ext_cur(1.0, true);
    aResult.add_ext_cur(3.0, false);
    aResult.add_inflection(7.0);

    assert_eq!(aResult.nb_points(), 4);

    // Verify sorted order
    for i in 1..aResult.nb_points() {
        assert!(aResult.parameter(i).unwrap() <= aResult.parameter(i + 1).unwrap());
    }
}

#[test]
fn LProp_CurAndInfTest__MultiplePoints_CorrectTypes() {
    let mut aResult = CurAndInf::new();
    aResult.add_ext_cur(1.0, true); // MinCur
    aResult.add_inflection(3.0); // Inflection
    aResult.add_ext_cur(5.0, false); // MaxCur

    assert_eq!(aResult.nb_points(), 3);
    assert_eq!(aResult.type_of(1).unwrap(), CIType::MinCur);
    assert_eq!(aResult.type_of(2).unwrap(), CIType::Inflection);
    assert_eq!(aResult.type_of(3).unwrap(), CIType::MaxCur);
}

#[test]
fn LProp_CurAndInfTest__Clear() {
    let mut aResult = CurAndInf::new();
    aResult.add_inflection(1.0);
    aResult.add_ext_cur(2.0, true);

    assert_eq!(aResult.nb_points(), 2);

    aResult.clear();
    assert!(aResult.is_empty());
    assert_eq!(aResult.nb_points(), 0);
}

#[test]
fn LProp_CurAndInfTest__Clear_ThenRefill() {
    let mut aResult = CurAndInf::new();
    aResult.add_inflection(1.0);
    aResult.clear();

    aResult.add_ext_cur(5.0, false);
    assert_eq!(aResult.nb_points(), 1);
    EXPECT_DOUBLE_EQ(aResult.parameter(1).unwrap(), 5.0);
    assert_eq!(aResult.type_of(1).unwrap(), CIType::MaxCur);
}

#[test]
fn LProp_CurAndInfTest__Parameter_OutOfRange_Throws() {
    let mut aResult = CurAndInf::new();
    aResult.add_inflection(1.0);

    EXPECT_THROW(aResult.parameter(0), "Standard_OutOfRange");
    EXPECT_THROW(aResult.parameter(2), "Standard_OutOfRange");
}

#[test]
fn LProp_CurAndInfTest__Type_OutOfRange_Throws() {
    let mut aResult = CurAndInf::new();
    aResult.add_inflection(1.0);

    EXPECT_THROW(aResult.type_of(0), "Standard_OutOfRange");
    EXPECT_THROW(aResult.type_of(2), "Standard_OutOfRange");
}
