//! Direct translation of TopoDS_TShape_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! BRepBuilderAPI_MakeVertex and BRepBuilderAPI_MakeEdge are the BRep fakes of
//! `support` (TKTopAlgo is not ported).
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::topods::{
    Builder, CompSolid, Compound, Shape, ShapeIterator, Shell, Solid, TShape, Wire,
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use support::{make_edge, make_vertex};

fn gp_Pnt(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

/// `S.TShape()`: the TShape of a non-null shape.
fn tshape_of(shape: &Shape) -> &dyn TShape {
    shape.tshape().get().unwrap()
}

//=================================================================================================
// Test TShape ShapeType() returns correct type for each shape kind
//=================================================================================================

#[test]
fn TopoDS_TShape_Test__ShapeType_AllTypes() {
    let aBuilder = Builder;

    // Create various shapes and verify ShapeType
    let aVertex = make_vertex(gp_Pnt(0.0, 0.0, 0.0));
    assert_eq!(tshape_of(&aVertex).shape_type(), ShapeEnum::Vertex);

    let anEdge = make_edge(gp_Pnt(0.0, 0.0, 0.0), gp_Pnt(1.0, 0.0, 0.0));
    assert_eq!(tshape_of(&anEdge).shape_type(), ShapeEnum::Edge);

    let mut aWire = Wire::default();
    aBuilder.make_wire(&mut aWire);
    assert_eq!(tshape_of(&aWire).shape_type(), ShapeEnum::Wire);

    let mut aShell = Shell::default();
    aBuilder.make_shell(&mut aShell);
    assert_eq!(tshape_of(&aShell).shape_type(), ShapeEnum::Shell);

    let mut aSolid = Solid::default();
    aBuilder.make_solid(&mut aSolid);
    assert_eq!(tshape_of(&aSolid).shape_type(), ShapeEnum::Solid);

    let mut aCompSolid = CompSolid::default();
    aBuilder.make_comp_solid(&mut aCompSolid);
    assert_eq!(tshape_of(&aCompSolid).shape_type(), ShapeEnum::CompSolid);

    let mut aCompound = Compound::default();
    aBuilder.make_compound(&mut aCompound);
    assert_eq!(tshape_of(&aCompound).shape_type(), ShapeEnum::Compound);
}

//=================================================================================================
// Test TShape flag setters and getters
//=================================================================================================

#[test]
fn TopoDS_TShape_Test__FlagSettersGetters() {
    let aBuilder = Builder;
    let mut aCompound = Compound::default();
    aBuilder.make_compound(&mut aCompound);

    let aTShape = tshape_of(&aCompound);

    // Test Free flag
    assert!(aTShape.free());
    aTShape.set_free(false);
    assert!(!aTShape.free());
    aTShape.set_free(true);
    assert!(aTShape.free());

    // Test Modified flag
    assert!(aTShape.modified());
    aTShape.set_modified(false);
    assert!(!aTShape.modified());

    // Test Checked flag
    assert!(!aTShape.checked());
    aTShape.set_checked(true);
    assert!(aTShape.checked());

    // Test that setting Modified(true) clears Checked
    aTShape.set_modified(true);
    assert!(
        !aTShape.checked(),
        "Modified(true) should clear Checked flag"
    );

    // Test Orientable flag
    assert!(!aTShape.orientable(), "Compound should not be orientable");

    // Test Closed flag
    assert!(!aTShape.closed());
    aTShape.set_closed(true);
    assert!(aTShape.closed());

    // Test Infinite flag
    assert!(!aTShape.infinite());
    aTShape.set_infinite(true);
    assert!(aTShape.infinite());

    // Test Convex flag
    assert!(!aTShape.convex());
    aTShape.set_convex(true);
    assert!(aTShape.convex());

    // Test Locked flag
    assert!(!aTShape.locked());
    aTShape.set_locked(true);
    assert!(aTShape.locked());
}

//=================================================================================================
// Test TShape NbChildren() for various shapes
//=================================================================================================

#[test]
fn TopoDS_TShape_Test__NbChildren() {
    let aBuilder = Builder;

    // Vertex has 0 children
    let aVertex = make_vertex(gp_Pnt(0.0, 0.0, 0.0));
    assert_eq!(tshape_of(&aVertex).nb_children(), 0);

    // Edge has 2 children (vertices)
    let anEdge = make_edge(gp_Pnt(0.0, 0.0, 0.0), gp_Pnt(1.0, 0.0, 0.0));
    assert_eq!(tshape_of(&anEdge).nb_children(), 2);

    // Empty wire has 0 children
    let mut aWire = Wire::default();
    aBuilder.make_wire(&mut aWire);
    assert_eq!(tshape_of(&aWire).nb_children(), 0);

    // Wire with edges
    aBuilder.add(&mut aWire, &anEdge).unwrap();
    assert_eq!(tshape_of(&aWire).nb_children(), 1);

    // Compound with multiple children
    let mut aCompound = Compound::default();
    aBuilder.make_compound(&mut aCompound);
    for i in 0..10 {
        aBuilder
            .add(&mut aCompound, &make_vertex(gp_Pnt(f64::from(i), 0.0, 0.0)))
            .unwrap();
    }
    assert_eq!(tshape_of(&aCompound).nb_children(), 10);
}

//=================================================================================================
// Test TShape EmptyCopy creates same type with no children
//=================================================================================================

#[test]
fn TopoDS_TShape_Test__EmptyCopy() {
    let aBuilder = Builder;

    // Create compound with children
    let mut aCompound = Compound::default();
    aBuilder.make_compound(&mut aCompound);
    aBuilder
        .add(&mut aCompound, &make_vertex(gp_Pnt(0.0, 0.0, 0.0)))
        .unwrap();
    aBuilder
        .add(&mut aCompound, &make_vertex(gp_Pnt(1.0, 0.0, 0.0)))
        .unwrap();

    assert_eq!(tshape_of(&aCompound).nb_children(), 2);

    // EmptyCopy should create same type with no children
    let aCopy = tshape_of(&aCompound).empty_copy().unwrap();

    assert_eq!(aCopy.shape_type(), ShapeEnum::Compound);
    assert_eq!(aCopy.nb_children(), 0, "EmptyCopy should have no children");
}

//=================================================================================================
// Test TShape EmptyCopy for all shape types
//=================================================================================================

#[test]
fn TopoDS_TShape_Test__EmptyCopy_AllTypes() {
    let aBuilder = Builder;

    // Test Edge EmptyCopy
    {
        let anEdge = make_edge(gp_Pnt(0.0, 0.0, 0.0), gp_Pnt(1.0, 0.0, 0.0));
        let aCopy = tshape_of(&anEdge).empty_copy().unwrap();
        assert_eq!(aCopy.shape_type(), ShapeEnum::Edge);
        assert_eq!(aCopy.nb_children(), 0);
    }

    // Test Wire EmptyCopy
    {
        let mut aWire = Wire::default();
        aBuilder.make_wire(&mut aWire);
        let aCopy = tshape_of(&aWire).empty_copy().unwrap();
        assert_eq!(aCopy.shape_type(), ShapeEnum::Wire);
    }

    // Test Shell EmptyCopy
    {
        let mut aShell = Shell::default();
        aBuilder.make_shell(&mut aShell);
        let aCopy = tshape_of(&aShell).empty_copy().unwrap();
        assert_eq!(aCopy.shape_type(), ShapeEnum::Shell);
    }

    // Test Solid EmptyCopy
    {
        let mut aSolid = Solid::default();
        aBuilder.make_solid(&mut aSolid);
        let aCopy = tshape_of(&aSolid).empty_copy().unwrap();
        assert_eq!(aCopy.shape_type(), ShapeEnum::Solid);
    }

    // Test CompSolid EmptyCopy
    {
        let mut aCompSolid = CompSolid::default();
        aBuilder.make_comp_solid(&mut aCompSolid);
        let aCopy = tshape_of(&aCompSolid).empty_copy().unwrap();
        assert_eq!(aCopy.shape_type(), ShapeEnum::CompSolid);
    }
}

//=================================================================================================
// Test TShape Orientable flag for different shape types
//=================================================================================================

#[test]
fn TopoDS_TShape_Test__Orientable_DifferentTypes() {
    let aBuilder = Builder;

    // Edge should be orientable
    let anEdge = make_edge(gp_Pnt(0.0, 0.0, 0.0), gp_Pnt(1.0, 0.0, 0.0));
    assert!(tshape_of(&anEdge).orientable(), "Edge should be orientable");

    // Compound should not be orientable
    let mut aCompound = Compound::default();
    aBuilder.make_compound(&mut aCompound);
    assert!(
        !tshape_of(&aCompound).orientable(),
        "Compound should not be orientable"
    );

    // Wire should be orientable
    let mut aWire = Wire::default();
    aBuilder.make_wire(&mut aWire);
    assert!(tshape_of(&aWire).orientable(), "Wire should be orientable");
}

//=================================================================================================
// Test TShape flags are independent
//=================================================================================================

#[test]
fn TopoDS_TShape_Test__FlagsIndependent() {
    let aBuilder = Builder;
    let mut aCompound = Compound::default();
    aBuilder.make_compound(&mut aCompound);

    let aTShape = tshape_of(&aCompound);

    // Set all flags
    aTShape.set_closed(true);
    aTShape.set_infinite(true);
    aTShape.set_convex(true);
    aTShape.set_locked(true);

    // Verify all are set
    assert!(aTShape.closed());
    assert!(aTShape.infinite());
    assert!(aTShape.convex());
    assert!(aTShape.locked());

    // Clear one flag, others should remain
    aTShape.set_closed(false);
    assert!(!aTShape.closed());
    assert!(
        aTShape.infinite(),
        "Clearing Closed should not affect Infinite"
    );
    assert!(aTShape.convex(), "Clearing Closed should not affect Convex");
    assert!(aTShape.locked(), "Clearing Closed should not affect Locked");
}

//=================================================================================================
// Test NbChildren consistency with iterator
//=================================================================================================

#[test]
fn TopoDS_TShape_Test__NbChildren_ConsistencyWithIterator() {
    let aBuilder = Builder;

    // Create wire with edge
    let mut aWire = Wire::default();
    aBuilder.make_wire(&mut aWire);
    let anEdge = make_edge(gp_Pnt(0.0, 0.0, 0.0), gp_Pnt(1.0, 0.0, 0.0));
    aBuilder.add(&mut aWire, &anEdge).unwrap();

    // Get NbChildren via TShape
    let aNbViaTShape = tshape_of(&aWire).nb_children();

    // Count via iterator
    let mut aNbViaIterator = 0;
    let mut anIt = ShapeIterator::new(&aWire, true, true);
    while anIt.more() {
        aNbViaIterator += 1;
        anIt.next();
    }

    assert_eq!(
        aNbViaTShape, aNbViaIterator,
        "NbChildren should match iterator count"
    );
    assert_eq!(aNbViaTShape, 1);
}
