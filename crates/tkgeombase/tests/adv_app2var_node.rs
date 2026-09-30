//! Direct translation of AdvApp2Var_Node_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::EXPECT_DOUBLE_EQ;
use rustycad_tkgeombase::adv_app2var::AdvApp2VarNode;

#[test]
fn AdvApp2Var_NodeTest__Constructor_InitializesValuesToZero() {
    let aNode = AdvApp2VarNode::with_orders(1, 2);
    for aUOrder in 0..=1 {
        for aVOrder in 0..=2 {
            let aPoint = aNode.point(aUOrder, aVOrder).unwrap();
            EXPECT_DOUBLE_EQ(aPoint.x(), 0.0);
            EXPECT_DOUBLE_EQ(aPoint.y(), 0.0);
            EXPECT_DOUBLE_EQ(aPoint.z(), 0.0);
            EXPECT_DOUBLE_EQ(aNode.error(aUOrder, aVOrder).unwrap(), 0.0);
        }
    }
}
