// Copyright (c) 2026 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! Direct translation of TopoDS_Builder_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! BRepBuilderAPI_MakeVertex and BRepBuilderAPI_MakeEdge are the BRep fakes of
//! `support` (TKTopAlgo is not ported).
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::topods::{Builder, CompSolid, Compound, ShapeIterator, Shell, Solid, Wire};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use support::{make_edge, make_vertex};

fn gp_Pnt(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

fn count_children(shape: &rustycad_tkbrep::topods::Shape) -> i32 {
    let mut aCount = 0;
    let mut anIt = ShapeIterator::new(shape, true, true);
    while anIt.more() {
        aCount += 1;
        anIt.next();
    }
    aCount
}

#[test]
fn TopoDS_Builder_Test__MakeWire() {
    let aBuilder = Builder;
    let mut aWire = Wire::default();

    aBuilder.make_wire(&mut aWire);

    assert!(!aWire.is_null(), "Wire should not be null after MakeWire");
    assert_eq!(
        aWire.shape_type().unwrap(),
        ShapeEnum::Wire,
        "Shape type should be WIRE"
    );
    assert!(aWire.free().unwrap(), "Newly created wire should be free");
}

#[test]
fn TopoDS_Builder_Test__MakeShell() {
    let aBuilder = Builder;
    let mut aShell = Shell::default();

    aBuilder.make_shell(&mut aShell);

    assert!(!aShell.is_null(), "Shell should not be null");
    assert_eq!(aShell.shape_type().unwrap(), ShapeEnum::Shell);
}

#[test]
fn TopoDS_Builder_Test__MakeSolid() {
    let aBuilder = Builder;
    let mut aSolid = Solid::default();

    aBuilder.make_solid(&mut aSolid);

    assert!(!aSolid.is_null(), "Solid should not be null");
    assert_eq!(aSolid.shape_type().unwrap(), ShapeEnum::Solid);
}

#[test]
fn TopoDS_Builder_Test__MakeCompSolid() {
    let aBuilder = Builder;
    let mut aCompSolid = CompSolid::default();

    aBuilder.make_comp_solid(&mut aCompSolid);

    assert!(!aCompSolid.is_null(), "CompSolid should not be null");
    assert_eq!(aCompSolid.shape_type().unwrap(), ShapeEnum::CompSolid);
}

#[test]
fn TopoDS_Builder_Test__MakeCompound() {
    let aBuilder = Builder;
    let mut aCompound = Compound::default();

    aBuilder.make_compound(&mut aCompound);

    assert!(!aCompound.is_null(), "Compound should not be null");
    assert_eq!(aCompound.shape_type().unwrap(), ShapeEnum::Compound);
}

#[test]
fn TopoDS_Builder_Test__Add_VerticesToCompound() {
    let aBuilder = Builder;
    let mut aCompound = Compound::default();
    aBuilder.make_compound(&mut aCompound);

    // Create and add vertices
    let aV1 = make_vertex(gp_Pnt(0.0, 0.0, 0.0));
    let aV2 = make_vertex(gp_Pnt(1.0, 0.0, 0.0));
    let aV3 = make_vertex(gp_Pnt(0.0, 1.0, 0.0));

    aBuilder.add(&mut aCompound, &aV1).unwrap();
    aBuilder.add(&mut aCompound, &aV2).unwrap();
    aBuilder.add(&mut aCompound, &aV3).unwrap();

    // Count children using iterator
    let aCount = count_children(&aCompound);

    assert_eq!(aCount, 3, "Compound should have 3 vertices");
}

#[test]
fn TopoDS_Builder_Test__Add_EdgesToWire() {
    let aBuilder = Builder;
    let mut aWire = Wire::default();
    aBuilder.make_wire(&mut aWire);

    // Create edges forming a triangle
    let aE1 = make_edge(gp_Pnt(0.0, 0.0, 0.0), gp_Pnt(1.0, 0.0, 0.0));
    let aE2 = make_edge(gp_Pnt(1.0, 0.0, 0.0), gp_Pnt(0.5, 1.0, 0.0));
    let aE3 = make_edge(gp_Pnt(0.5, 1.0, 0.0), gp_Pnt(0.0, 0.0, 0.0));

    aBuilder.add(&mut aWire, &aE1).unwrap();
    aBuilder.add(&mut aWire, &aE2).unwrap();
    aBuilder.add(&mut aWire, &aE3).unwrap();

    // Count children
    let mut aCount = 0;
    let mut anIt = ShapeIterator::new(&aWire, true, true);
    while anIt.more() {
        assert_eq!(anIt.value().unwrap().shape_type().unwrap(), ShapeEnum::Edge);
        aCount += 1;
        anIt.next();
    }

    assert_eq!(aCount, 3, "Wire should have 3 edges");
}

#[test]
fn TopoDS_Builder_Test__Remove_FromCompound() {
    let aBuilder = Builder;
    let mut aCompound = Compound::default();
    aBuilder.make_compound(&mut aCompound);

    // Add vertices
    let aV1 = make_vertex(gp_Pnt(0.0, 0.0, 0.0));
    let aV2 = make_vertex(gp_Pnt(1.0, 0.0, 0.0));
    let aV3 = make_vertex(gp_Pnt(0.0, 1.0, 0.0));

    aBuilder.add(&mut aCompound, &aV1).unwrap();
    aBuilder.add(&mut aCompound, &aV2).unwrap();
    aBuilder.add(&mut aCompound, &aV3).unwrap();

    // Remove middle vertex
    aBuilder.remove(&mut aCompound, &aV2).unwrap();

    // Count remaining children
    let aCount = count_children(&aCompound);

    assert_eq!(aCount, 2, "Compound should have 2 vertices after removal");
}

#[test]
fn TopoDS_Builder_Test__Add_ManyShapes() {
    let aBuilder = Builder;
    let mut aCompound = Compound::default();

    aBuilder.make_compound(&mut aCompound);

    // Add 500 vertices
    for i in 0..500 {
        let aV = make_vertex(gp_Pnt(f64::from(i), 0.0, 0.0));
        aBuilder.add(&mut aCompound, &aV).unwrap();
    }

    // Verify count
    let aCount = count_children(&aCompound);

    assert_eq!(aCount, 500, "Compound should have 500 vertices");
}

#[test]
fn TopoDS_Builder_Test__TShapeFlags() {
    let aBuilder = Builder;
    let mut aCompound = Compound::default();
    aBuilder.make_compound(&mut aCompound);

    // Test initial flags
    assert!(aCompound.free().unwrap(), "New shape should be free");
    assert!(
        aCompound.modified().unwrap(),
        "New shape should be modified"
    );

    // Test setting flags
    aCompound.set_checked(true).unwrap();
    assert!(aCompound.checked().unwrap());

    aCompound.set_closed(true).unwrap();
    assert!(aCompound.closed().unwrap());

    aCompound.set_infinite(true).unwrap();
    assert!(aCompound.infinite().unwrap());

    aCompound.set_convex(true).unwrap();
    assert!(aCompound.convex().unwrap());
}

#[test]
fn TopoDS_Builder_Test__Remove_FirstChild() {
    let aBuilder = Builder;
    let mut aCompound = Compound::default();
    aBuilder.make_compound(&mut aCompound);

    let aV1 = make_vertex(gp_Pnt(1.0, 0.0, 0.0));
    let aV2 = make_vertex(gp_Pnt(2.0, 0.0, 0.0));
    let aV3 = make_vertex(gp_Pnt(3.0, 0.0, 0.0));

    aBuilder.add(&mut aCompound, &aV1).unwrap();
    aBuilder.add(&mut aCompound, &aV2).unwrap();
    aBuilder.add(&mut aCompound, &aV3).unwrap();

    // Remove first vertex
    aBuilder.remove(&mut aCompound, &aV1).unwrap();

    // Should have 2 remaining
    let aCount = count_children(&aCompound);
    assert_eq!(aCount, 2);
}

#[test]
fn TopoDS_Builder_Test__Remove_LastChild() {
    let aBuilder = Builder;
    let mut aCompound = Compound::default();
    aBuilder.make_compound(&mut aCompound);

    let aV1 = make_vertex(gp_Pnt(1.0, 0.0, 0.0));
    let aV2 = make_vertex(gp_Pnt(2.0, 0.0, 0.0));
    let aV3 = make_vertex(gp_Pnt(3.0, 0.0, 0.0));

    aBuilder.add(&mut aCompound, &aV1).unwrap();
    aBuilder.add(&mut aCompound, &aV2).unwrap();
    aBuilder.add(&mut aCompound, &aV3).unwrap();

    // Remove last vertex
    aBuilder.remove(&mut aCompound, &aV3).unwrap();

    // Should have 2 remaining
    let aCount = count_children(&aCompound);
    assert_eq!(aCount, 2);
}

#[test]
fn TopoDS_Builder_Test__Remove_AllChildren() {
    let aBuilder = Builder;
    let mut aCompound = Compound::default();
    aBuilder.make_compound(&mut aCompound);

    let aV1 = make_vertex(gp_Pnt(1.0, 0.0, 0.0));
    let aV2 = make_vertex(gp_Pnt(2.0, 0.0, 0.0));
    let aV3 = make_vertex(gp_Pnt(3.0, 0.0, 0.0));

    aBuilder.add(&mut aCompound, &aV1).unwrap();
    aBuilder.add(&mut aCompound, &aV2).unwrap();
    aBuilder.add(&mut aCompound, &aV3).unwrap();

    // Remove all
    aBuilder.remove(&mut aCompound, &aV1).unwrap();
    aBuilder.remove(&mut aCompound, &aV2).unwrap();
    aBuilder.remove(&mut aCompound, &aV3).unwrap();

    // Should be empty
    let anIt = ShapeIterator::new(&aCompound, true, true);
    assert!(
        !anIt.more(),
        "Compound should be empty after removing all children"
    );
}

#[test]
fn TopoDS_Builder_Test__Remove_FromWire() {
    let aBuilder = Builder;
    let mut aWire = Wire::default();
    aBuilder.make_wire(&mut aWire);

    let aE1 = make_edge(gp_Pnt(0.0, 0.0, 0.0), gp_Pnt(1.0, 0.0, 0.0));
    let aE2 = make_edge(gp_Pnt(1.0, 0.0, 0.0), gp_Pnt(1.0, 1.0, 0.0));
    let aE3 = make_edge(gp_Pnt(1.0, 1.0, 0.0), gp_Pnt(0.0, 0.0, 0.0));

    aBuilder.add(&mut aWire, &aE1).unwrap();
    aBuilder.add(&mut aWire, &aE2).unwrap();
    aBuilder.add(&mut aWire, &aE3).unwrap();

    // Remove middle edge
    aBuilder.remove(&mut aWire, &aE2).unwrap();

    // Should have 2 edges
    let aCount = count_children(&aWire);
    assert_eq!(aCount, 2);
}

#[test]
fn TopoDS_Builder_Test__NestedCompounds() {
    let aBuilder = Builder;
    let (mut aOuter, mut aInner) = (Compound::default(), Compound::default());
    aBuilder.make_compound(&mut aOuter);
    aBuilder.make_compound(&mut aInner);

    // Add vertices to inner
    aBuilder
        .add(&mut aInner, &make_vertex(gp_Pnt(0.0, 0.0, 0.0)))
        .unwrap();
    aBuilder
        .add(&mut aInner, &make_vertex(gp_Pnt(1.0, 0.0, 0.0)))
        .unwrap();

    // Add inner to outer
    aBuilder.add(&mut aOuter, &aInner).unwrap();

    // Outer should have 1 child (the inner compound)
    let mut aOuterCount = 0;
    let mut anIt = ShapeIterator::new(&aOuter, true, true);
    while anIt.more() {
        assert_eq!(
            anIt.value().unwrap().shape_type().unwrap(),
            ShapeEnum::Compound
        );
        aOuterCount += 1;
        anIt.next();
    }
    assert_eq!(aOuterCount, 1);

    // Inner should still have 2 vertices
    assert_eq!(aInner.tshape().nb_children(), 2);
}

#[test]
fn TopoDS_Builder_Test__Add_SetsModifiedFlag() {
    let aBuilder = Builder;
    let mut aCompound = Compound::default();
    aBuilder.make_compound(&mut aCompound);

    // Clear modified flag
    aCompound.tshape().set_modified(false);
    assert!(!aCompound.modified().unwrap());

    // Add should set modified
    aBuilder
        .add(&mut aCompound, &make_vertex(gp_Pnt(0.0, 0.0, 0.0)))
        .unwrap();
    assert!(
        aCompound.modified().unwrap(),
        "Add should set Modified flag"
    );
}

#[test]
fn TopoDS_Builder_Test__Remove_SetsModifiedFlag() {
    let aBuilder = Builder;
    let mut aCompound = Compound::default();
    aBuilder.make_compound(&mut aCompound);

    let aV = make_vertex(gp_Pnt(0.0, 0.0, 0.0));
    aBuilder.add(&mut aCompound, &aV).unwrap();

    // Clear modified flag
    aCompound.tshape().set_modified(false);
    assert!(!aCompound.modified().unwrap());

    // Remove should set modified
    aBuilder.remove(&mut aCompound, &aV).unwrap();
    assert!(
        aCompound.modified().unwrap(),
        "Remove should set Modified flag"
    );
}

#[test]
fn TopoDS_Builder_Test__AllTypes() {
    let aBuilder = Builder;

    // Wire
    let mut aWire = Wire::default();
    aBuilder.make_wire(&mut aWire);
    assert!(!aWire.is_null());
    assert_eq!(aWire.shape_type().unwrap(), ShapeEnum::Wire);
    assert!(aWire.free().unwrap());

    // Shell
    let mut aShell = Shell::default();
    aBuilder.make_shell(&mut aShell);
    assert!(!aShell.is_null());
    assert_eq!(aShell.shape_type().unwrap(), ShapeEnum::Shell);
    assert!(aShell.free().unwrap());

    // Solid
    let mut aSolid = Solid::default();
    aBuilder.make_solid(&mut aSolid);
    assert!(!aSolid.is_null());
    assert_eq!(aSolid.shape_type().unwrap(), ShapeEnum::Solid);
    assert!(aSolid.free().unwrap());

    // CompSolid
    let mut aCompSolid = CompSolid::default();
    aBuilder.make_comp_solid(&mut aCompSolid);
    assert!(!aCompSolid.is_null());
    assert_eq!(aCompSolid.shape_type().unwrap(), ShapeEnum::CompSolid);
    assert!(aCompSolid.free().unwrap());

    // Compound
    let mut aCompound = Compound::default();
    aBuilder.make_compound(&mut aCompound);
    assert!(!aCompound.is_null());
    assert_eq!(aCompound.shape_type().unwrap(), ShapeEnum::Compound);
    assert!(aCompound.free().unwrap());
}

#[test]
fn TopoDS_Builder_Test__MixedShapesInCompound() {
    let aBuilder = Builder;
    let mut aCompound = Compound::default();
    aBuilder.make_compound(&mut aCompound);

    // Add different shape types
    aBuilder
        .add(&mut aCompound, &make_vertex(gp_Pnt(0.0, 0.0, 0.0)))
        .unwrap();
    aBuilder
        .add(
            &mut aCompound,
            &make_edge(gp_Pnt(0.0, 0.0, 0.0), gp_Pnt(1.0, 0.0, 0.0)),
        )
        .unwrap();

    let mut aWire = Wire::default();
    aBuilder.make_wire(&mut aWire);
    aBuilder.add(&mut aCompound, &aWire).unwrap();

    let mut aShell = Shell::default();
    aBuilder.make_shell(&mut aShell);
    aBuilder.add(&mut aCompound, &aShell).unwrap();

    let mut aInner = Compound::default();
    aBuilder.make_compound(&mut aInner);
    aBuilder.add(&mut aCompound, &aInner).unwrap();

    assert_eq!(aCompound.tshape().nb_children(), 5);

    // Verify types via iteration
    let (mut aVertexCount, mut aEdgeCount, mut aWireCount, mut aShellCount, mut aCompoundCount) =
        (0, 0, 0, 0, 0);
    let mut anIt = ShapeIterator::new(&aCompound, true, true);
    while anIt.more() {
        match anIt.value().unwrap().shape_type().unwrap() {
            ShapeEnum::Vertex => aVertexCount += 1,
            ShapeEnum::Edge => aEdgeCount += 1,
            ShapeEnum::Wire => aWireCount += 1,
            ShapeEnum::Shell => aShellCount += 1,
            ShapeEnum::Compound => aCompoundCount += 1,
            _ => {}
        }
        anIt.next();
    }

    assert_eq!(aVertexCount, 1);
    assert_eq!(aEdgeCount, 1);
    assert_eq!(aWireCount, 1);
    assert_eq!(aShellCount, 1);
    assert_eq!(aCompoundCount, 1);
}
