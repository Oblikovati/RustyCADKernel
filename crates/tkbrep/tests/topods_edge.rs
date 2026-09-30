//! Direct translation of TopoDS_Edge_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! BRepBuilderAPI_MakeEdge is the BRep fake of `support` (TKTopAlgo is not ported).
#![allow(non_snake_case)]
mod support;
use rustycad_tkmath::gp::Point3;
use support::make_edge;

// Test BUC60828: TopoDS_Edge Infinite flag getter/setter
// Migrated from QABugs_16.cxx
#[test]
fn TopoDS_Edge_Test__BUC60828_InfiniteFlag() {
    // Create a simple edge
    let anEdge = make_edge(Point3::new(0., 0., 0.), Point3::new(0., 0., 1.));

    // Check initial flag value (should be false by default)
    let anInitialValue = anEdge.infinite().unwrap();
    assert!(!anInitialValue, "Initial Infinite flag should be false");

    // Set the flag to true
    anEdge.set_infinite(true).unwrap();

    // Verify the flag was set correctly
    let aCurrentValue = anEdge.infinite().unwrap();
    assert!(aCurrentValue, "Infinite flag should be true after setting");
}
