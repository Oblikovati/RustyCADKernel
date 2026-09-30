//! Direct translation of AdvApp2Var_Network_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::EXPECT_DOUBLE_EQ;
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::Handle;
use rustycad_tkgeombase::adv_app2var::{AdvApp2VarNetwork, AdvApp2VarPatch};

// `new AdvApp2Var_Patch(U0, U1, V0, V1, iu, iv)`.
fn patch(aDomain: [f64; 4], iu: i32, iv: i32) -> Handle<AdvApp2VarPatch> {
    Handle::new(AdvApp2VarPatch::with_domain(aDomain, iu, iv))
}

fn parameters(aValues: &[f64]) -> Sequence<f64> {
    let mut aParams = Sequence::default();
    for aValue in aValues {
        aParams.append(*aValue);
    }
    aParams
}

#[test]
fn AdvApp2Var_NetworkTest__UpdateInV_PreservesPatchOrdersPerColumn() {
    let mut aNet = Sequence::default();
    aNet.append(patch([0.0, 1.0, 0.0, 1.0], 0, 1));
    aNet.append(patch([1.0, 2.0, 0.0, 1.0], 2, 3));

    let aUParams = parameters(&[0.0, 1.0, 2.0]);
    let aVParams = parameters(&[0.0, 1.0]);

    let mut aNetwork = AdvApp2VarNetwork::new(aNet, aUParams, aVParams);
    aNetwork.update_in_v(0.5).unwrap();

    assert_eq!(aNetwork.nb_patch(), 4);
    assert_eq!(aNetwork.nb_patch_in_u(), 2);
    assert_eq!(aNetwork.nb_patch_in_v(), 2);
    EXPECT_DOUBLE_EQ(aNetwork.v_parameter(2).unwrap(), 0.5);

    let aBottomLeft = aNetwork.patch(1, 1).unwrap();
    let aBottomRight = aNetwork.patch(2, 1).unwrap();
    let aTopLeft = aNetwork.patch(1, 2).unwrap();
    let aTopRight = aNetwork.patch(2, 2).unwrap();

    assert_eq!(aBottomLeft.u_order(), 0);
    assert_eq!(aBottomLeft.v_order(), 1);
    assert_eq!(aBottomRight.u_order(), 2);
    assert_eq!(aBottomRight.v_order(), 3);
    assert_eq!(aTopLeft.u_order(), 0);
    assert_eq!(aTopLeft.v_order(), 1);
    assert_eq!(aTopRight.u_order(), 2);
    assert_eq!(aTopRight.v_order(), 3);

    EXPECT_DOUBLE_EQ(aBottomLeft.v0(), 0.0);
    EXPECT_DOUBLE_EQ(aBottomLeft.v1(), 0.5);
    EXPECT_DOUBLE_EQ(aTopLeft.v0(), 0.5);
    EXPECT_DOUBLE_EQ(aTopLeft.v1(), 1.0);
}

#[test]
fn AdvApp2Var_NetworkTest__UpdateInU_PreservesPatchOrdersPerRow() {
    let mut aNet = Sequence::default();
    aNet.append(patch([0.0, 1.0, 0.0, 1.0], 0, 1));
    aNet.append(patch([1.0, 2.0, 0.0, 1.0], 2, 3));
    aNet.append(patch([0.0, 1.0, 1.0, 2.0], 4, 5));
    aNet.append(patch([1.0, 2.0, 1.0, 2.0], 6, 7));

    let aUParams = parameters(&[0.0, 1.0, 2.0]);
    let aVParams = parameters(&[0.0, 1.0, 2.0]);

    let mut aNetwork = AdvApp2VarNetwork::new(aNet, aUParams, aVParams);
    aNetwork.update_in_u(0.5).unwrap();

    assert_eq!(aNetwork.nb_patch(), 6);
    assert_eq!(aNetwork.nb_patch_in_u(), 3);
    assert_eq!(aNetwork.nb_patch_in_v(), 2);
    EXPECT_DOUBLE_EQ(aNetwork.u_parameter(2).unwrap(), 0.5);

    let aBottomLeft = aNetwork.patch(1, 1).unwrap();
    let aBottomMiddle = aNetwork.patch(2, 1).unwrap();
    let aBottomRight = aNetwork.patch(3, 1).unwrap();
    let aTopLeft = aNetwork.patch(1, 2).unwrap();
    let aTopMiddle = aNetwork.patch(2, 2).unwrap();
    let aTopRight = aNetwork.patch(3, 2).unwrap();

    assert_eq!(aBottomLeft.u_order(), 0);
    assert_eq!(aBottomLeft.v_order(), 1);
    assert_eq!(aBottomMiddle.u_order(), 0);
    assert_eq!(aBottomMiddle.v_order(), 1);
    assert_eq!(aBottomRight.u_order(), 2);
    assert_eq!(aBottomRight.v_order(), 3);

    assert_eq!(aTopLeft.u_order(), 4);
    assert_eq!(aTopLeft.v_order(), 5);
    assert_eq!(aTopMiddle.u_order(), 4);
    assert_eq!(aTopMiddle.v_order(), 5);
    assert_eq!(aTopRight.u_order(), 6);
    assert_eq!(aTopRight.v_order(), 7);

    EXPECT_DOUBLE_EQ(aBottomLeft.u0(), 0.0);
    EXPECT_DOUBLE_EQ(aBottomLeft.u1(), 0.5);
    EXPECT_DOUBLE_EQ(aBottomMiddle.u0(), 0.5);
    EXPECT_DOUBLE_EQ(aBottomMiddle.u1(), 1.0);
}
