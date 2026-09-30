//! Direct translation of AdvApp2Var_Context_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::EXPECT_DOUBLE_EQ;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkgeombase::adv_app2var::{AdvApp2VarContext, FrontierTolerances, SubspaceTolerances};

fn tolerance(value: f64) -> Array1<'static, f64> {
    let mut aTol = Array1::with_bounds(1, 1).unwrap();
    aTol.set_value(1, value).unwrap();
    aTol
}

fn frontier() -> Array2<'static, f64> {
    let mut aTof = Array2::with_bounds(1, 1, 1, 4).unwrap();
    aTof.init(100.0);
    aTof
}

#[test]
fn AdvApp2Var_ContextTest__Tolerances_AreAggregatedAndRescaledForBorderConstraints() {
    let aTol1D = tolerance(6.0);
    let aTol2D = tolerance(12.0);
    let aTol3D = tolerance(18.0);

    let aTof1D = frontier();
    let aTof2D = frontier();
    let aTof3D = frontier();

    let aTolerances = SubspaceTolerances {
        one_d: Some(&aTol1D),
        two_d: Some(&aTol2D),
        three_d: Some(&aTol3D),
    };
    let aFrontiers = FrontierTolerances {
        one_d: Some(&aTof1D),
        two_d: Some(&aTof2D),
        three_d: Some(&aTof3D),
    };
    let aContext =
        AdvApp2VarContext::new(0, 0, 0, 2, 2, 0, [1, 1, 1], aTolerances, aFrontiers).unwrap();

    assert_eq!(aContext.total_number_ssp(), 3);
    assert_eq!(aContext.total_dimension(), 6);

    let anITol = aContext.i_toler();
    assert!(anITol.is_some());
    let anITol = anITol.unwrap();
    EXPECT_DOUBLE_EQ(*anITol.value(1).unwrap(), 3.0);
    EXPECT_DOUBLE_EQ(*anITol.value(2).unwrap(), 6.0);
    EXPECT_DOUBLE_EQ(*anITol.value(3).unwrap(), 9.0);

    let aFTol = aContext.f_toler();
    let aCTol = aContext.c_toler();
    assert!(aFTol.is_some());
    assert!(aCTol.is_some());
    let (aFTol, aCTol) = (aFTol.unwrap(), aCTol.unwrap());

    for aTolIndex in 1..=4 {
        EXPECT_DOUBLE_EQ(*aFTol.value(1, aTolIndex).unwrap(), 1.0);
        EXPECT_DOUBLE_EQ(*aFTol.value(2, aTolIndex).unwrap(), 2.0);
        EXPECT_DOUBLE_EQ(*aFTol.value(3, aTolIndex).unwrap(), 3.0);

        EXPECT_DOUBLE_EQ(*aCTol.value(1, aTolIndex).unwrap(), 1.0);
        EXPECT_DOUBLE_EQ(*aCTol.value(2, aTolIndex).unwrap(), 2.0);
        EXPECT_DOUBLE_EQ(*aCTol.value(3, aTolIndex).unwrap(), 3.0);
    }
}
