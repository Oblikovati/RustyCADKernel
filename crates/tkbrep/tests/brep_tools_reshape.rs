//! Direct translation of BRepTools_ReShape_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! BRepBuilderAPI_MakeVertex, BRepBuilderAPI_MakeEdge and BRepPrimAPI_MakeBox are the
//! BRep fakes of `support` (TKTopAlgo and TKPrim are not ported).
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_tools::{BRepToolsReShape, ReShape};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Compound, Edge, Shape, ShapeIterator, Vertex};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use support::{edge_between, make_box, make_box_at};

fn MakeVertex(theX: f64, theY: f64, theZ: f64) -> Vertex {
    support::make_vertex(Point3::new(theX, theY, theZ))
}

fn MakeEdge(theV1: &Vertex, theV2: &Vertex) -> Edge {
    edge_between(theV1, theV2)
}

fn new_compound() -> Compound {
    let mut aCompound = Compound::default();
    BRepBuilder::default().make_compound(&mut aCompound);
    aCompound
}

fn add(theParent: &mut Compound, theChild: &Shape) {
    BRepBuilder::default().add(theParent, theChild).unwrap();
}

// Value() returns the direct replacement only (one hop); ValueLeaf() follows the
// chain through intermediate replacements to its terminal shape.
#[test]
fn BRepTools_ReShapeTest__ValueLeaf_FollowsChainToLeaf() {
    let aA = MakeVertex(0.0, 0.0, 0.0);
    let aB = MakeVertex(1.0, 0.0, 0.0);
    let aC = MakeVertex(2.0, 0.0, 0.0);

    let aReShape = BRepToolsReShape::new();
    aReShape.replace(&aA, &aB).unwrap();
    aReShape.replace(&aB, &aC).unwrap();

    assert!(
        aReShape.value(&aA).unwrap().is_same(&aB),
        "Value() must yield only the direct replacement"
    );
    assert!(
        aReShape.value_leaf(&aA).unwrap().is_same(&aC),
        "ValueLeaf() must walk the full chain"
    );
    assert!(aReShape.value_leaf(&aB).unwrap().is_same(&aC));
    assert!(
        aReShape.value_leaf(&aC).unwrap().is_same(&aC),
        "Terminal element is a fixpoint"
    );
}

// ValueLeaf() must terminate on an unrecorded shape without touching the map.
#[test]
fn BRepTools_ReShapeTest__ValueLeaf_UnrecordedShapeIsIdentity() {
    let aA = MakeVertex(0.0, 0.0, 0.0);

    let aReShape = BRepToolsReShape::new();
    assert!(aReShape.value_leaf(&aA).unwrap().is_same(&aA));
}

// A chain ending in a removal (null replacement) must surface as a null shape.
#[test]
fn BRepTools_ReShapeTest__ValueLeaf_ChainEndingInRemoveReturnsNull() {
    let aA = MakeVertex(0.0, 0.0, 0.0);
    let aB = MakeVertex(1.0, 0.0, 0.0);

    let aReShape = BRepToolsReShape::new();
    aReShape.replace(&aA, &aB).unwrap();
    aReShape.remove(&aB).unwrap();

    assert!(aReShape.value_leaf(&aA).unwrap().is_null());
}

// A direct A->B then B->A creates a cycle in the replacement map; both bindings
// are recorded, and the DFS in-flight guard inside Apply() handles termination.
#[test]
fn BRepTools_ReShapeTest__Replace_DirectCycleHandledByApply() {
    let aA = MakeVertex(0.0, 0.0, 0.0);
    let aB = MakeVertex(1.0, 0.0, 0.0);

    let aReShape = BRepToolsReShape::new();
    aReShape.replace(&aA, &aB).unwrap();
    aReShape.replace(&aB, &aA).unwrap();

    assert!(aReShape.value(&aA).unwrap().is_same(&aB));
    assert!(aReShape.value(&aB).unwrap().is_same(&aA));

    // Apply() must terminate (DFS in-flight guard) and return a defined shape.
    let aResult = aReShape.apply(&aA, ShapeEnum::Shape).unwrap();
    assert!(!aResult.is_null(), "Apply must terminate via the DFS guard");
}

// Longer cycles A -> B -> C -> A are accepted at Replace() time and broken at
// traversal time by the DFS in-flight guard inside Apply(). This avoids
// over-rejection in IGES/STEP healing pipelines where shapes legitimately
// alias TShapes across stages.
#[test]
fn BRepTools_ReShapeTest__Replace_LongerCycleHandledByApply() {
    let aA = MakeVertex(0.0, 0.0, 0.0);
    let aB = MakeVertex(1.0, 0.0, 0.0);
    let aC = MakeVertex(2.0, 0.0, 0.0);

    let aReShape = BRepToolsReShape::new();
    aReShape.replace(&aA, &aB).unwrap();
    aReShape.replace(&aB, &aC).unwrap();
    aReShape.replace(&aC, &aA).unwrap();

    assert!(aReShape.value(&aA).unwrap().is_same(&aB));
    assert!(aReShape.value(&aB).unwrap().is_same(&aC));
    assert!(aReShape.value(&aC).unwrap().is_same(&aA));

    // Apply() must terminate (DFS in-flight guard) and return a defined shape.
    let aResult = aReShape.apply(&aA, ShapeEnum::Shape).unwrap();
    assert!(!aResult.is_null(), "Apply must terminate via the DFS guard");
}

// Apply() must not stack-overflow when a shape's replacement is a compound that
// transitively contains the original shape as a sub-shape. This is the pattern
// that crashed in shape healing on non-orientable / shared-edge inputs.
#[test]
fn BRepTools_ReShapeTest__Apply_HandlesStructuralContainmentWithoutCrash() {
    let aV1 = MakeVertex(0.0, 0.0, 0.0);
    let aV2 = MakeVertex(1.0, 0.0, 0.0);
    let aV3 = MakeVertex(2.0, 0.0, 0.0);
    let anE = MakeEdge(&aV1, &aV2);

    // Build a compound that contains the edge (plus another shape for flavour).
    let mut aContainer = new_compound();
    add(&mut aContainer, &anE);
    add(&mut aContainer, &aV3);

    let aReShape = BRepToolsReShape::new();
    aReShape.replace(&anE, &aContainer).unwrap(); // e -> compound{e, v3}

    let aResult = aReShape.apply(&anE, ShapeEnum::Shape).unwrap();
    assert!(!aResult.is_null());
}

// Legitimate diamond sharing (the same sub-shape reached via different parent
// paths) must continue to process normally; the DFS guard must not mistake it
// for a cycle. Two edges sharing a vertex form the minimal diamond.
#[test]
fn BRepTools_ReShapeTest__Apply_DiamondSharingIsProcessedCorrectly() {
    let aV1 = MakeVertex(0.0, 0.0, 0.0);
    let aV2 = MakeVertex(1.0, 0.0, 0.0);
    let aV3 = MakeVertex(2.0, 0.0, 0.0);
    let aV2Repl = MakeVertex(1.5, 0.0, 0.0);

    // aE1 and aE2 both reference aV2 as an endpoint - that's the diamond.
    let aE1 = MakeEdge(&aV1, &aV2);
    let aE2 = MakeEdge(&aV2, &aV3);

    let mut aPair = new_compound();
    add(&mut aPair, &aE1);
    add(&mut aPair, &aE2);

    let aReShape = BRepToolsReShape::new();
    aReShape.replace(&aV2, &aV2Repl).unwrap();

    let aResult = aReShape.apply(&aPair, ShapeEnum::Shape).unwrap();
    assert!(!aResult.is_null());

    // aV2 must have been substituted everywhere it appeared; aV2Repl must appear;
    // the original aV2 TShape must no longer be present in the result.
    let mut aNbReplVertices = 0;
    for aCurrent in Explorer::new(&aResult, ShapeEnum::Vertex, ShapeEnum::Shape).unwrap() {
        assert!(
            aCurrent.tshape() != aV2.tshape(),
            "Original v2 must have been replaced on every diamond arm"
        );
        if aCurrent.tshape() == aV2Repl.tshape() {
            aNbReplVertices += 1;
        }
    }
    assert!(
        aNbReplVertices >= 2,
        "Replacement must appear on both arms of the diamond"
    );
}

// Clear() must drop every binding and reset the map to identity behaviour.
#[test]
fn BRepTools_ReShapeTest__Clear_DropsAllBindings() {
    let aA = MakeVertex(0.0, 0.0, 0.0);
    let aB = MakeVertex(1.0, 0.0, 0.0);

    let aReShape = BRepToolsReShape::new();
    aReShape.replace(&aA, &aB).unwrap();
    assert!(aReShape.value(&aA).unwrap().is_same(&aB));

    aReShape.clear();
    assert!(aReShape.value(&aA).unwrap().is_same(&aA));
    assert!(aReShape.value_leaf(&aA).unwrap().is_same(&aA));
}

// Re-replacing the same shape must overwrite the previous binding.
// This is the documented behaviour and must not be blocked by the cycle guard.
#[test]
fn BRepTools_ReShapeTest__Replace_LastWins() {
    let aA = MakeVertex(0.0, 0.0, 0.0);
    let aB = MakeVertex(1.0, 0.0, 0.0);
    let aC = MakeVertex(2.0, 0.0, 0.0);

    let aReShape = BRepToolsReShape::new();
    aReShape.replace(&aA, &aB).unwrap();
    aReShape.replace(&aA, &aC).unwrap(); // overwrite

    assert!(aReShape.value(&aA).unwrap().is_same(&aC));
    assert!(aReShape.value_leaf(&aA).unwrap().is_same(&aC));
}

// Replace(X, X) is a no-op - the shape equals itself, no binding should be stored.
#[test]
fn BRepTools_ReShapeTest__Replace_SelfIsNoOp() {
    let aA = MakeVertex(0.0, 0.0, 0.0);

    let aReShape = BRepToolsReShape::new();
    aReShape.replace(&aA, &aA).unwrap();

    assert!(!aReShape.is_recorded(&aA));
    assert!(aReShape.value(&aA).unwrap().is_same(&aA));
}

// A Remove request must propagate through Apply() so that the sub-shape is
// dropped from the rebuilt parent.
#[test]
fn BRepTools_ReShapeTest__Apply_RemoveDropsSubShapeFromParent() {
    let aV1 = MakeVertex(0.0, 0.0, 0.0);
    let aV2 = MakeVertex(1.0, 0.0, 0.0);
    let aV3 = MakeVertex(2.0, 0.0, 0.0);

    let mut aParent = new_compound();
    add(&mut aParent, &aV1);
    add(&mut aParent, &aV2);
    add(&mut aParent, &aV3);

    let aReShape = BRepToolsReShape::new();
    aReShape.remove(&aV2).unwrap();

    let aResult = aReShape.apply(&aParent, ShapeEnum::Shape).unwrap();
    assert!(!aResult.is_null());

    let mut aNbVerts = 0;
    for aChild in ShapeIterator::new(&aResult, true, true) {
        assert!(
            aChild.tshape() != aV2.tshape(),
            "v2 should have been removed"
        );
        aNbVerts += 1;
    }
    assert_eq!(aNbVerts, 2);
}

// The fuller Moebius-style pattern: an edge appears in two different parents
// (simulated here by two compounds), each parent is rebuilt through Apply,
// and the edge replacement must propagate to both without triggering the
// DFS cycle guard. Regression for the original "shared sub-shape" failure.
#[test]
fn BRepTools_ReShapeTest__Apply_SharedEdgeAcrossTwoParents() {
    let aV1 = MakeVertex(0.0, 0.0, 0.0);
    let aV2 = MakeVertex(1.0, 0.0, 0.0);
    let aV3 = MakeVertex(2.0, 0.0, 0.0);
    let aE1 = MakeEdge(&aV1, &aV2);
    let aE2 = MakeEdge(&aV2, &aV3);
    let aE1New = MakeEdge(&MakeVertex(0.5, 0.0, 0.0), &aV2);

    // Build two parents both referencing aE1: parentA{aE1, aV3}, parentB{aE1, aE2}.
    let mut aParentA = new_compound();
    add(&mut aParentA, &aE1);
    add(&mut aParentA, &aV3);
    let mut aParentB = new_compound();
    add(&mut aParentB, &aE1);
    add(&mut aParentB, &aE2);

    let aReShape = BRepToolsReShape::new();
    aReShape.replace(&aE1, &aE1New).unwrap();

    let aResultA = aReShape.apply(&aParentA, ShapeEnum::Shape).unwrap();
    let aResultB = aReShape.apply(&aParentB, ShapeEnum::Shape).unwrap();
    assert!(!aResultA.is_null());
    assert!(!aResultB.is_null());

    let (mut aFoundNewInA, mut aFoundNewInB) = (false, false);
    for aCurrent in Explorer::new(&aResultA, ShapeEnum::Edge, ShapeEnum::Shape).unwrap() {
        assert!(aCurrent.tshape() != aE1.tshape());
        if aCurrent.tshape() == aE1New.tshape() {
            aFoundNewInA = true;
        }
    }
    for aCurrent in Explorer::new(&aResultB, ShapeEnum::Edge, ShapeEnum::Shape).unwrap() {
        assert!(aCurrent.tshape() != aE1.tshape());
        if aCurrent.tshape() == aE1New.tshape() {
            aFoundNewInB = true;
        }
    }
    assert!(aFoundNewInA);
    assert!(aFoundNewInB);
}

// Idempotence: Apply() on a shape that has no bindings must return
// the shape unchanged. Guards against accidental rebuild-on-no-change.
#[test]
fn BRepTools_ReShapeTest__Apply_NoBindingsIsIdentity() {
    let aV1 = MakeVertex(0.0, 0.0, 0.0);
    let aV2 = MakeVertex(1.0, 0.0, 0.0);
    let anE = MakeEdge(&aV1, &aV2);

    let aReShape = BRepToolsReShape::new();
    let aResult = aReShape.apply(&anE, ShapeEnum::Shape).unwrap();
    assert!(aResult.is_same(&anE));
}

// Deep structural containment: a replacement compound contains a nested
// compound that in turn contains the original shape. DFS guard must still
// break the cycle regardless of depth.
#[test]
fn BRepTools_ReShapeTest__Apply_DeepStructuralContainmentWithoutCrash() {
    let aV1 = MakeVertex(0.0, 0.0, 0.0);
    let aV2 = MakeVertex(1.0, 0.0, 0.0);
    let anExtra = MakeVertex(3.0, 0.0, 0.0);
    let anE = MakeEdge(&aV1, &aV2);

    // inner{e}, outer{inner, extra}: e is reachable two levels down.
    let mut anInner = new_compound();
    add(&mut anInner, &anE);
    let mut anOuter = new_compound();
    add(&mut anOuter, &anInner);
    add(&mut anOuter, &anExtra);

    let aReShape = BRepToolsReShape::new();
    aReShape.replace(&anE, &anOuter).unwrap(); // e -> outer{inner{e}, extra}

    let aResult = aReShape.apply(&anE, ShapeEnum::Shape).unwrap();
    assert!(!aResult.is_null());
}

// Locations must be tracked when ModeConsiderLocation is enabled.
#[test]
fn BRepTools_ReShapeTest__ValueLeaf_ConsidersLocationMode() {
    let aA = MakeVertex(0.0, 0.0, 0.0);
    let aB = MakeVertex(1.0, 0.0, 0.0);

    let aReShape = BRepToolsReShape::new();
    aReShape.set_mode_consider_location(true);
    aReShape.replace(&aA, &aB).unwrap();

    assert!(aReShape.value_leaf(&aA).unwrap().is_same(&aB));
}

// Migrated from tests/bugs/moddata_3/bug29662. Replacing a compound up to the
// compound level must preserve the solids from the untouched branch and add
// the replacement branch as one solid.
#[test]
fn BRepTools_ReShapeTest__ModDataBug_29662_ReplaceCompound() {
    let aBox1 = make_box(10.0, 10.0, 10.0).into_shape();
    let aBox2 = make_box_at(Point3::new(10.0, 0.0, 0.0), 10.0, 10.0, 10.0).into_shape();
    let aBox3 = make_box_at(Point3::new(0.0, 0.0, 10.0), 10.0, 10.0, 10.0).into_shape();
    let aBox4 = make_box_at(Point3::new(10.0, 0.0, 10.0), 10.0, 10.0, 10.0).into_shape();
    let aBox5 = make_box_at(Point3::new(20.0, 0.0, 0.0), 10.0, 10.0, 10.0).into_shape();

    let mut aC1 = new_compound();
    add(&mut aC1, &aBox1);
    add(&mut aC1, &aBox2);
    let mut aC2 = new_compound();
    add(&mut aC2, &aBox3);
    add(&mut aC2, &aBox4);
    let mut aC = new_compound();
    add(&mut aC, &aC1);
    add(&mut aC, &aC2);
    let mut aC3 = new_compound();
    add(&mut aC3, &aBox5);

    let aReShape = BRepToolsReShape::new();
    aReShape.replace(&aC2, &aC3).unwrap();
    let aResult = aReShape.apply(&aC, ShapeEnum::Compound).unwrap();
    assert!(!aResult.is_null());

    let mut aNbSolids = 0;
    for _aSolid in Explorer::new(&aResult, ShapeEnum::Solid, ShapeEnum::Shape).unwrap() {
        aNbSolids += 1;
    }
    assert_eq!(aNbSolids, 3);
}
