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

//! Direct translation of BRepGraph_RefsIterator_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871. `BRepPrimAPI_MakeBox` and
//! `BRepBuilderAPI_MakeEdge` are the fakes of the test support module (TKPrim and
//! TKTopAlgo are not ported).
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_graph::refs_iterator::*;
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkbrep::topods::{Edge, Face, Vertex, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Axis3, Point3};
use rustycad_tkmath::toploc::Location;
use support::{make_box, make_edge};

/// `countIterator(theIterator)` over the native `More`/`Next` protocol.
macro_rules! countIterator {
    ($iterator:expr) => {{
        let mut theIterator = $iterator;
        let mut aCount: u32 = 0;
        while theIterator.more() {
            aCount += 1;
            theIterator.next();
        }
        aCount
    }};
}

fn makeEdgeWithInternalVertex() -> Edge {
    let aBuilder = BRepBuilder::default();
    let mut anEdge = make_edge(Point3::new(0.0, 0.0, 0.0), Point3::new(10.0, 0.0, 0.0));

    let mut anInternalVertex = Vertex::default();
    aBuilder
        .make_vertex_at(
            &mut anInternalVertex,
            Point3::new(5.0, 0.0, 0.0),
            precision::confusion(),
        )
        .unwrap();
    aBuilder
        .add(
            &mut anEdge,
            &anInternalVertex.oriented(Orientation::Internal),
        )
        .unwrap();
    anEdge
}

fn wrapEdgeInFace(theEdge: &Edge) -> Face {
    let aBuilder = BRepBuilder::default();
    let aPlane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).into_cast();
    let mut aFace = Face::default();
    aBuilder
        .make_face_on_surface(&mut aFace, &aPlane, precision::confusion())
        .unwrap();
    let mut aWire = Wire::default();
    aBuilder.make_wire(&mut aWire);
    aBuilder.add(&mut aWire, theEdge).unwrap();
    aBuilder.add(&mut aFace, &aWire).unwrap();
    aFace
}

/// The `BRepGraph_RefsIteratorTest` fixture: a graph of a 10x20x30 box.
fn SetUp() -> BRepGraph {
    let mut myGraph = BRepGraph::new();
    myGraph.clear().unwrap();
    let _aBuildRes1 = myGraph
        .shapes_mut()
        .add(&make_box(10.0, 20.0, 30.0))
        .unwrap();
    myGraph
}

#[test]
fn BRepGraph_RefsIteratorTest__BoxHierarchy_YieldsReferenceIds() {
    let myGraph = SetUp();
    assert_eq!(
        countIterator!(RefsShellOfSolid::new(&myGraph, SolidId::start())),
        1
    );
    assert_eq!(
        countIterator!(RefsFaceOfShell::new(&myGraph, ShellId::start())),
        6
    );
    assert_eq!(
        countIterator!(RefsWireOfFace::new(&myGraph, FaceId::start())),
        1
    );
    assert_eq!(
        countIterator!(CoEdgesOfWire::new(&myGraph, WireId::start())),
        4
    );
    assert_eq!(
        countIterator!(RefsVertexOfEdge::new(&myGraph, EdgeId::start())),
        2
    );
}

#[test]
fn BRepGraph_RefsIteratorTest__CurrentId_ResolvesToExpectedEntry() {
    let myGraph = SetUp();
    let anIt = RefsWireOfFace::new(&myGraph, FaceId::start());
    assert!(anIt.more());

    let aWireRef = myGraph.refs().wires().entry(anIt.current_id());
    assert!(
        aWireRef
            .child_wire_id
            .is_valid_for(myGraph.topo().wires().nb())
    );
}

#[test]
fn BRepGraph_RefsIteratorTestStandalone__VertexOfEdge_ExposesBoundaryVertexRefsOnly() {
    let mut aGraph = BRepGraph::new();
    aGraph.clear().unwrap();
    let _aBuildRes2 = aGraph
        .shapes_mut()
        .add(&wrapEdgeInFace(&makeEdgeWithInternalVertex()))
        .unwrap();

    let mut aFoundInternal = false;
    let mut aCount: u32 = 0;
    let mut anIt = RefsVertexOfEdge::new(&aGraph, EdgeId::start());
    while anIt.more() {
        aCount += 1;
        let aVertexRef = aGraph.refs().vertices().entry(anIt.current_id());
        if aVertexRef.orientation.orientation() == Orientation::Internal {
            aFoundInternal = true;
        }
        anIt.next();
    }

    assert_eq!(aCount, 2);
    assert!(!aFoundInternal);
}

#[test]
fn BRepGraph_RefsIteratorTest__ChildOfCompound_EnumeratesChildRefs() {
    let mut myGraph = SetUp();
    let aLooseVertex = myGraph
        .editor()
        .vertices()
        .add(Point3::new(1.0, 2.0, 3.0), 0.01);
    assert!(aLooseVertex.is_valid());

    let aChildren: Vec<NodeId> = vec![SolidId::start().into(), aLooseVertex.into()];
    let aCompound = myGraph.editor().compounds().add(&aChildren);

    assert!(aCompound.is_valid());
    let mut anIt = RefsChildOfCompound::new(&myGraph, aCompound);
    assert!(anIt.more());
    assert_eq!(
        myGraph
            .refs()
            .children()
            .entry(anIt.current_id())
            .child_node_id
            .kind,
        NodeKind::SOLID
    );
    anIt.next();
    assert!(anIt.more());
    assert_eq!(
        myGraph
            .refs()
            .children()
            .entry(anIt.current_id())
            .child_node_id
            .kind,
        NodeKind::VERTEX
    );
}

#[test]
fn BRepGraph_RefsIteratorTest__ChildOfCompound_SkipsOutOfRangeChildNode() {
    let mut myGraph = SetUp();
    let aLooseVertex = myGraph
        .editor()
        .vertices()
        .add(Point3::new(1.0, 2.0, 3.0), 0.01);
    assert!(aLooseVertex.is_valid());

    let aChildren: Vec<NodeId> = vec![
        SolidId::start().into(),
        aLooseVertex.into(),
        aLooseVertex.into(),
    ];
    let aCompound = myGraph.editor().compounds().add(&aChildren);
    assert!(aCompound.is_valid());

    let aChildRefs = myGraph
        .topo()
        .compounds()
        .relations(aCompound)
        .child_ref_ids
        .clone();
    assert_eq!(aChildRefs.len(), 3);
    {
        let aNbVertices = myGraph.topo().vertices().nb();
        let mut aBadRef = myGraph
            .editor()
            .generic()
            .mutate_child_ref(aChildRefs[1])
            .unwrap();
        aBadRef.internal().child_node_id = NodeId::new(NodeKind::VERTEX, aNbVertices);
    }

    let mut aCount: u32 = 0;
    let mut anIt = RefsChildOfCompound::new(&myGraph, aCompound);
    while anIt.more() {
        let aChildNode = myGraph
            .refs()
            .children()
            .entry(anIt.current_id())
            .child_node_id;
        assert!(myGraph.topo().generic().is_active(aChildNode));
        aCount += 1;
        anIt.next();
    }
    assert_eq!(aCount, 2);
}

#[test]
fn BRepGraph_RefsIteratorTest__ChildOfCompound_SkipsRemovedChildNode() {
    let mut myGraph = SetUp();
    let aLooseVertex = myGraph
        .editor()
        .vertices()
        .add(Point3::new(1.0, 2.0, 3.0), 0.01);
    assert!(aLooseVertex.is_valid());

    let aChildren: Vec<NodeId> = vec![SolidId::start().into(), aLooseVertex.into()];
    let aCompound = myGraph.editor().compounds().add(&aChildren);
    assert!(aCompound.is_valid());

    myGraph
        .editor()
        .generic()
        .remove_node(aLooseVertex.into())
        .unwrap();

    assert!(RefsChildOfCompound::new(&myGraph, aCompound).more());
    assert_eq!(
        countIterator!(RefsChildOfCompound::new(&myGraph, aCompound)),
        1
    );
}

#[test]
fn BRepGraph_RefsIteratorTest__OccurrenceOfProduct_EnumeratesOccurrenceRefs() {
    let mut myGraph = SetUp();
    let aPart = myGraph
        .editor()
        .products()
        .add(SolidId::start().into(), &Location::default());
    myGraph
        .editor()
        .products()
        .append_document_root(aPart)
        .unwrap();
    let anAssembly = myGraph.editor().products().add_empty();
    myGraph
        .editor()
        .products()
        .append_document_root(anAssembly)
        .unwrap();
    assert!(aPart.is_valid());
    assert!(anAssembly.is_valid());

    let aLoc = Location::default();
    let anOcc = OccurrenceId::default();
    assert!(
        myGraph
            .editor()
            .products()
            .append(anAssembly, aPart, &aLoc, anOcc)
            .unwrap()
            .0
            .is_valid()
    );
    assert!(
        myGraph
            .editor()
            .products()
            .append(anAssembly, aPart, &aLoc, anOcc)
            .unwrap()
            .0
            .is_valid()
    );

    assert_eq!(
        countIterator!(RefsOccurrenceOfProduct::new(&myGraph, anAssembly)),
        2
    );
}

#[test]
fn BRepGraph_RefsIteratorTest__RemovedWireRef_IsSkipped() {
    let mut myGraph = SetUp();
    let aWireRefs = myGraph.refs().wires().ids_of(FaceId::start()).to_vec();
    assert_eq!(aWireRefs.len(), 1);

    myGraph
        .editor()
        .generic()
        .remove_ref(aWireRefs[0].into())
        .unwrap();

    assert_eq!(
        countIterator!(RefsWireOfFace::new(&myGraph, FaceId::start())),
        0
    );
}
