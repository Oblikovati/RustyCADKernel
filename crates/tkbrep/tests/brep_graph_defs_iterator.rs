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

//! Direct translation of BRepGraph_DefsIterator_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871. `BRepPrimAPI_MakeBox` and
//! `BRepBuilderAPI_MakeEdge` are the fakes of the test support module (TKPrim and
//! TKTopAlgo are not ported).
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_graph::defs_iterator::*;
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
        let mut aCount: usize = 0;
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

/// The `BRepGraph_DefsIteratorTest` fixture: a graph of a 10x20x30 box.
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
fn BRepGraph_DefsIteratorTest__BoxHierarchy_TraversesSingleLevelChildren() {
    let myGraph = SetUp();
    assert_eq!(
        countIterator!(DefsShellOfSolid::new(&myGraph, SolidId::start())),
        1
    );
    assert_eq!(
        countIterator!(DefsFaceOfShell::new(&myGraph, ShellId::start())),
        6
    );
    assert_eq!(
        countIterator!(DefsWireOfFace::new(&myGraph, FaceId::start())),
        1
    );
    assert_eq!(
        countIterator!(DefsEdgeOfWire::new(&myGraph, WireId::start())),
        4
    );
    assert_eq!(
        countIterator!(DefsCoEdgeOfWire::new(&myGraph, WireId::start())),
        4
    );
    assert_eq!(
        countIterator!(DefsVertexOfEdge::new(&myGraph, EdgeId::start())),
        2
    );
}

#[test]
fn BRepGraph_DefsIteratorTest__EdgeOfWire_YieldsEdgeDefinitions() {
    let myGraph = SetUp();
    let anIt = DefsEdgeOfWire::new(&myGraph, WireId::start());
    assert!(anIt.more());
    let anEdgeId = anIt.current_id();
    let aCoEdgeId = anIt.current_ref_id();
    assert!(anEdgeId.is_valid_for(myGraph.topo().edges().nb()));
    assert!(aCoEdgeId.is_valid_for(myGraph.topo().co_edges().nb()));
    assert_eq!(
        myGraph
            .topo()
            .co_edges()
            .definition(aCoEdgeId)
            .child_edge_id,
        anEdgeId
    );
    assert!(anIt.current().start_vertex_ref_id.is_valid());
    assert!(anIt.current().end_vertex_ref_id.is_valid());
}

#[test]
fn BRepGraph_DefsIteratorTest__CoEdgeOfWire_YieldsCoEdgeDefinitions() {
    let myGraph = SetUp();
    let anIt = DefsCoEdgeOfWire::new(&myGraph, WireId::start());
    assert!(anIt.more());
    assert!(
        anIt.current_id()
            .is_valid_for(myGraph.topo().co_edges().nb())
    );
    assert_eq!(anIt.current_ref_id(), anIt.current_id());
    assert!(
        anIt.current()
            .child_edge_id
            .is_valid_for(myGraph.topo().edges().nb())
    );
}

#[test]
fn BRepGraph_DefsIteratorTest__VertexOfEdge_ExposesBoundaryVertexRef() {
    let myGraph = SetUp();
    let anIt = DefsVertexOfEdge::new(&myGraph, EdgeId::start());
    assert!(anIt.more());
    let aVertexId = anIt.current_id();
    let aRefId = anIt.current_ref_id();
    assert!(aVertexId.is_valid_for(myGraph.topo().vertices().nb()));
    assert!(aRefId.is_valid_for(myGraph.refs().vertices().nb()));
    assert_eq!(
        myGraph.refs().vertices().entry(aRefId).child_vertex_id,
        aVertexId
    );
}

#[test]
fn BRepGraph_DefsIteratorTestStandalone__VertexOfEdge_EnumeratesBoundaryVerticesOnly() {
    let mut aGraph = BRepGraph::new();
    aGraph.clear().unwrap();
    let _aBuildRes2 = aGraph
        .shapes_mut()
        .add(&wrapEdgeInFace(&makeEdgeWithInternalVertex()))
        .unwrap();

    let mut aFoundSplitVertex = false;
    let mut aCount: usize = 0;
    let mut anIt = DefsVertexOfEdge::new(&aGraph, EdgeId::start());
    while anIt.more() {
        aCount += 1;
        let aPoint = anIt.current().point;
        if (aPoint.x() - 5.0).abs() <= precision::confusion() {
            aFoundSplitVertex = true;
        }
        anIt.next();
    }

    assert_eq!(aCount, 2);
    assert!(!aFoundSplitVertex);
}

#[test]
fn BRepGraph_DefsIteratorTest__ChildOfCompound_EnumeratesHeterogeneousChildren() {
    let mut myGraph = SetUp();
    let aLooseVertex = myGraph
        .editor()
        .vertices()
        .add(Point3::new(1.0, 2.0, 3.0), 0.01);
    assert!(aLooseVertex.is_valid());

    let aChildren: Vec<NodeId> = vec![SolidId::start().into(), aLooseVertex.into()];
    let aCompound = myGraph.editor().compounds().add(&aChildren);

    assert!(aCompound.is_valid());
    let mut anIt = DefsChildOfCompound::new(&myGraph, aCompound);
    assert!(anIt.more());
    assert_eq!(anIt.current_id().kind, NodeKind::SOLID);
    anIt.next();
    assert!(anIt.more());
    assert_eq!(anIt.current_id().kind, NodeKind::VERTEX);
}

#[test]
fn BRepGraph_DefsIteratorTest__ChildOfCompound_SkipsOutOfRangeChildNode() {
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
    let mut anIt = DefsChildOfCompound::new(&myGraph, aCompound);
    while anIt.more() {
        assert!(myGraph.topo().generic().is_active(anIt.current_id()));
        aCount += 1;
        anIt.next();
    }
    assert_eq!(aCount, 2);
}

#[test]
fn BRepGraph_DefsIteratorTest__ChildOfCompound_SkipsRemovedChildNode() {
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

    assert!(DefsChildOfCompound::new(&myGraph, aCompound).more());
    assert_eq!(
        countIterator!(DefsChildOfCompound::new(&myGraph, aCompound)),
        1
    );
}

#[test]
fn BRepGraph_DefsIteratorTest__SolidOfCompSolid_EnumeratesDirectSolids() {
    let mut myGraph = SetUp();
    let aSolidA = myGraph.editor().solids().add();
    let aSolidB = myGraph.editor().solids().add();
    assert!(aSolidA.is_valid());
    assert!(aSolidB.is_valid());

    let aSolids = vec![aSolidA, aSolidB];
    let aCompSolid = myGraph.editor().comp_solids().add(&aSolids);

    assert!(aCompSolid.is_valid());
    assert_eq!(
        countIterator!(DefsSolidOfCompSolid::new(&myGraph, aCompSolid)),
        2
    );
}

#[test]
fn BRepGraph_DefsIteratorTest__OccurrenceOfProduct_EnumeratesDirectOccurrences() {
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
        countIterator!(DefsOccurrenceOfProduct::new(&myGraph, anAssembly)),
        2
    );
}

#[test]
fn BRepGraph_DefsIteratorTest__RemovedWireRef_IsSkipped() {
    let mut myGraph = SetUp();
    let aWireRefs = myGraph.refs().wires().ids_of(FaceId::start()).to_vec();
    assert_eq!(aWireRefs.len(), 1);

    myGraph
        .editor()
        .generic()
        .remove_ref(aWireRefs[0].into())
        .unwrap();

    assert_eq!(
        countIterator!(DefsWireOfFace::new(&myGraph, FaceId::start())),
        0
    );
}
