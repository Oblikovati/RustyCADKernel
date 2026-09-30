//! Direct translation of AdvApp2Var_Iso_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::EXPECT_DOUBLE_EQ;
use rustycad_tkgeombase::adv_app2var::AdvApp2VarIso;
use rustycad_tkmath::geom_abs::IsoType;

#[test]
fn AdvApp2Var_IsoTest__ConstructorWithOrders_AssignsOrdersByIsoType() {
    let anIsoU = AdvApp2VarIso::with_orders(IsoType::IsoU, 3, 1);
    assert_eq!(anIsoU.iso_type(), IsoType::IsoU);
    assert_eq!(anIsoU.u_order(), 3);
    assert_eq!(anIsoU.v_order(), 1);
    EXPECT_DOUBLE_EQ(anIsoU.constante(), 0.5);
    EXPECT_DOUBLE_EQ(anIsoU.u0(), 0.0);
    EXPECT_DOUBLE_EQ(anIsoU.u1(), 1.0);
    EXPECT_DOUBLE_EQ(anIsoU.v0(), 0.0);
    EXPECT_DOUBLE_EQ(anIsoU.v1(), 1.0);
    EXPECT_DOUBLE_EQ(anIsoU.t0(), 0.0);
    EXPECT_DOUBLE_EQ(anIsoU.t1(), 1.0);
    assert_eq!(anIsoU.position(), 0);

    let anIsoV = AdvApp2VarIso::with_orders(IsoType::IsoV, 3, 1);
    assert_eq!(anIsoV.iso_type(), IsoType::IsoV);
    assert_eq!(anIsoV.u_order(), 3);
    assert_eq!(anIsoV.v_order(), 1);
}
