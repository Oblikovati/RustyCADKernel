//! Direct translation of AdvApp2Var_Framework_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::EXPECT_DOUBLE_EQ;
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::Handle;
use rustycad_tkgeombase::adv_app2var::{
    AdvApp2VarFramework, AdvApp2VarIso, AdvApp2VarIsoStrip, AdvApp2VarNode,
};
use rustycad_tkmath::geom_abs::IsoType;
use rustycad_tkmath::gp::Xy;

type AdvApp2VarIsoGrid = Sequence<AdvApp2VarIsoStrip>;

// `new AdvApp2Var_Iso(Type, Constante, U0, U1, V0, V1, 0, 0, 0)`.
fn iso(aType: IsoType, aConstante: f64, aDomain: [f64; 4]) -> Handle<AdvApp2VarIso> {
    Handle::new(AdvApp2VarIso::with_domain(
        aType, aConstante, aDomain, 0, 0, 0,
    ))
}

fn makeUFrontier() -> AdvApp2VarIsoGrid {
    let mut aUFrontier = AdvApp2VarIsoGrid::default();

    let mut aUStrip1 = AdvApp2VarIsoStrip::default();
    aUStrip1.append(iso(IsoType::IsoV, 30.0, [0.0, 1.0, 0.0, 1.0]));
    aUStrip1.append(iso(IsoType::IsoV, 40.0, [0.0, 1.0, 0.0, 1.0]));
    aUFrontier.append(aUStrip1);

    let mut aUStrip2 = AdvApp2VarIsoStrip::default();
    aUStrip2.append(iso(IsoType::IsoV, 30.0, [1.0, 2.0, 0.0, 1.0]));
    aUStrip2.append(iso(IsoType::IsoV, 40.0, [1.0, 2.0, 0.0, 1.0]));
    aUFrontier.append(aUStrip2);

    aUFrontier
}

fn makeVFrontier() -> AdvApp2VarIsoGrid {
    let mut aVFrontier = AdvApp2VarIsoGrid::default();

    let mut aVStrip1 = AdvApp2VarIsoStrip::default();
    aVStrip1.append(iso(IsoType::IsoU, 10.0, [0.0, 1.0, 0.0, 1.0]));
    aVStrip1.append(iso(IsoType::IsoU, 20.0, [0.0, 1.0, 0.0, 1.0]));
    aVFrontier.append(aVStrip1);

    let mut aVStrip2 = AdvApp2VarIsoStrip::default();
    aVStrip2.append(iso(IsoType::IsoU, 10.0, [0.0, 1.0, 1.0, 2.0]));
    aVStrip2.append(iso(IsoType::IsoU, 20.0, [0.0, 1.0, 1.0, 2.0]));
    aVFrontier.append(aVStrip2);

    aVFrontier
}

fn makeNodeGrid() -> Sequence<Handle<AdvApp2VarNode>> {
    let mut aNodes = Sequence::default();
    for (aU, aV) in [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)] {
        aNodes.append(Handle::new(
            AdvApp2VarNode::with_coord(Xy::new(aU, aV), 0, 0).unwrap(),
        ));
    }
    aNodes
}

#[test]
fn AdvApp2Var_FrameworkTest__IsoLookup_ReturnsMatchingIso() {
    let aNodes = makeNodeGrid();
    let aFramework = AdvApp2VarFramework::new(aNodes, makeUFrontier(), makeVFrontier());

    let anIsoU = aFramework.iso_u(20.0, 0.0, 1.0).unwrap();
    assert_eq!(anIsoU.iso_type(), IsoType::IsoU);
    EXPECT_DOUBLE_EQ(anIsoU.constante(), 20.0);
    EXPECT_DOUBLE_EQ(anIsoU.t0(), 0.0);
    EXPECT_DOUBLE_EQ(anIsoU.t1(), 1.0);

    let anIsoV = aFramework.iso_v(1.0, 2.0, 40.0).unwrap();
    assert_eq!(anIsoV.iso_type(), IsoType::IsoV);
    EXPECT_DOUBLE_EQ(anIsoV.constante(), 40.0);
    EXPECT_DOUBLE_EQ(anIsoV.t0(), 1.0);
    EXPECT_DOUBLE_EQ(anIsoV.t1(), 2.0);
}

#[test]
fn AdvApp2Var_FrameworkTest__NodeIndexMapping_UsesGridLayout() {
    let aNodes = makeNodeGrid();
    let aFramework = AdvApp2VarFramework::new(aNodes, makeUFrontier(), makeVFrontier());

    assert_eq!(aFramework.first_node(IsoType::IsoU, 2, 1), 2);
    assert_eq!(aFramework.last_node(IsoType::IsoU, 2, 1), 5);

    assert_eq!(aFramework.first_node(IsoType::IsoV, 2, 1), 4);
    assert_eq!(aFramework.last_node(IsoType::IsoV, 2, 1), 5);
}

#[test]
fn AdvApp2Var_FrameworkTest__UpdateInU_InsertsCutIsoAndNodes() {
    let aNodes = makeNodeGrid();
    let mut aFramework = AdvApp2VarFramework::new(aNodes, makeUFrontier(), makeVFrontier());
    aFramework.update_in_u(0.5).unwrap();

    let aCutIso = aFramework.iso_u(0.5, 0.0, 1.0).unwrap();
    assert_eq!(aCutIso.iso_type(), IsoType::IsoU);
    EXPECT_DOUBLE_EQ(aCutIso.constante(), 0.5);
    EXPECT_DOUBLE_EQ(aCutIso.t0(), 0.0);
    EXPECT_DOUBLE_EQ(aCutIso.t1(), 1.0);

    // ASSERT_FALSE(IsNull()): a missing node is an error here, so unwrap asserts it.
    let aBottomCutNode = aFramework.node_at(0.5, 0.0).unwrap();
    let aTopCutNode = aFramework.node_at(0.5, 1.0).unwrap();
    EXPECT_DOUBLE_EQ(aBottomCutNode.coord().x(), 0.5);
    EXPECT_DOUBLE_EQ(aBottomCutNode.coord().y(), 0.0);
    EXPECT_DOUBLE_EQ(aTopCutNode.coord().x(), 0.5);
    EXPECT_DOUBLE_EQ(aTopCutNode.coord().y(), 1.0);

    assert_eq!(aFramework.first_node(IsoType::IsoU, 2, 1), 2);
    assert_eq!(aFramework.last_node(IsoType::IsoU, 2, 1), 6);
}
