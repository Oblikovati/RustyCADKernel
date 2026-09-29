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

//! Direct translation of BRepGraph_ParentExplorer_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871. `BRepPrimAPI_MakeBox` is the box of the
//! test support module (TKPrim is not ported). The native constructor overloads are
//! the named `ParentExplorer` constructors (docs/topology-design.md section 11.9).
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_graph::parent_explorer::{LinkKind, ParentExplorer, TraversalMode};
use rustycad_tkbrep::brep_graph::validate::{self, Options};
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkbrep::topods::{Compound, Shape};
use rustycad_tkernel::precision;
use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Transform3, Vector3};
use rustycad_tkmath::toploc::Location;
use support::make_box;

fn translationLocation(theX: f64, theY: f64, theZ: f64) -> Location {
    Location::new(Transform3::translation_vector(Vector3::new(
        theX, theY, theZ,
    )))
}

fn rotationZLocation(theAngle: f64) -> Location {
    let mut aTrsf = Transform3::default();
    aTrsf
        .set_rotation(
            Axis1::new(Point3::new(0.0, 0.0, 0.0), Direction3::Z),
            theAngle,
        )
        .unwrap();
    Location::new(aTrsf)
}

fn locationsEquivalent(theActual: &Location, theExpected: &Location) -> bool {
    let aProbe = Point3::new(3.0, 5.0, 7.0);
    let anActualPoint = aProbe.transformed(theActual.transformation());
    let anExpectedPoint = aProbe.transformed(theExpected.transformation());
    anActualPoint.distance(&anExpectedPoint) <= precision::confusion()
}

fn expectLocationsEquivalent(theActual: &Location, theExpected: &Location) {
    let aProbe = Point3::new(3.0, 5.0, 7.0);
    let anActualPoint = aProbe.transformed(theActual.transformation());
    let anExpectedPoint = aProbe.transformed(theExpected.transformation());
    assert!((anActualPoint.x() - anExpectedPoint.x()).abs() <= precision::confusion());
    assert!((anActualPoint.y() - anExpectedPoint.y()).abs() <= precision::confusion());
    assert!((anActualPoint.z() - anExpectedPoint.z()).abs() <= precision::confusion());
}

/// `BRepGraph aGraph; aGraph.Clear(); aGraph.Shapes().Add(BRepPrimAPI_MakeBox(10, 20, 30))`.
fn boxGraph() -> BRepGraph {
    let mut aGraph = BRepGraph::new();
    aGraph.clear().unwrap();
    let _aBuildRes = aGraph
        .shapes_mut()
        .add(&make_box(10.0, 20.0, 30.0))
        .unwrap();
    assert!(!aGraph.is_empty());
    aGraph
}

fn firstFace() -> NodeId {
    FaceId::start().into()
}

fn firstSolid() -> NodeId {
    SolidId::start().into()
}

/// `Products().Add(theRoot)` then `AppendDocumentRoot`.
fn addRootPart(aGraph: &mut BRepGraph, theRoot: NodeId) -> ProductId {
    let aPart = aGraph
        .editor()
        .products()
        .add(theRoot, &Location::default());
    aGraph
        .editor()
        .products()
        .append_document_root(aPart)
        .unwrap();
    aPart
}

/// `Products().Add()` then `AppendDocumentRoot`.
fn addRootAssembly(aGraph: &mut BRepGraph) -> ProductId {
    let anAssembly = aGraph.editor().products().add_empty();
    aGraph
        .editor()
        .products()
        .append_document_root(anAssembly)
        .unwrap();
    anAssembly
}

/// `Products().Append(theParent, theChild, theLoc)`.
fn appendOccurrence(
    aGraph: &mut BRepGraph,
    theParent: ProductId,
    theChild: ProductId,
    theLoc: &Location,
) -> OccurrenceId {
    let aDefault = OccurrenceId::default();
    let mut aProducts = aGraph.editor().products();
    aProducts
        .append(theParent, theChild, theLoc, aDefault)
        .unwrap()
        .0
}

/// `Compounds().Add({theChild})`.
fn addCompound(aGraph: &mut BRepGraph, theChild: NodeId) -> CompoundId {
    aGraph.editor().compounds().add(&[theChild])
}

#[test]
fn BRepGraph_ParentExplorerTest__FaceParents_All_CountAndOrder() {
    let aGraph = boxGraph();

    let mut anExp = ParentExplorer::new(&aGraph, firstFace()).unwrap();
    assert!(anExp.more());
    assert_eq!(anExp.current().def_id, NodeId::from(ShellId::start()));

    anExp.next().unwrap();
    assert!(anExp.more());
    assert_eq!(anExp.current().def_id, firstSolid());

    anExp.next().unwrap();
    assert!(anExp.more());
    assert_eq!(anExp.current().def_id.kind, NodeKind::OCCURRENCE);

    anExp.next().unwrap();
    assert!(anExp.more());
    assert_eq!(anExp.current().def_id.kind, NodeKind::PRODUCT);

    anExp.next().unwrap();
    assert!(!anExp.more());
}

#[test]
fn BRepGraph_ParentExplorerTest__FaceParents_TypedSolid_OneResult() {
    let aGraph = boxGraph();

    let mut anExp = ParentExplorer::with_target(&aGraph, firstFace(), NodeKind::SOLID).unwrap();
    assert!(anExp.more());
    assert_eq!(anExp.current().def_id, firstSolid());

    anExp.next().unwrap();
    assert!(!anExp.more());
}

#[test]
fn BRepGraph_ParentExplorerTest__FaceParents_DirectParents_StopsAtImmediateShell() {
    let aGraph = boxGraph();

    let mut anExp =
        ParentExplorer::with_mode(&aGraph, firstFace(), TraversalMode::DirectParents).unwrap();
    assert!(anExp.more());
    assert_eq!(anExp.current().def_id, NodeId::from(ShellId::start()));

    anExp.next().unwrap();
    assert!(!anExp.more());
}

#[test]
fn BRepGraph_ParentExplorerTest__FaceParents_DirectParents_ExposeChildAndRef() {
    let aGraph = boxGraph();

    let anExp =
        ParentExplorer::with_mode(&aGraph, firstFace(), TraversalMode::DirectParents).unwrap();
    assert!(anExp.more());
    assert_eq!(anExp.current().def_id, NodeId::from(ShellId::start()));
    assert_eq!(anExp.current_child(), firstFace());
    assert_eq!(anExp.current_link_kind(), LinkKind::Reference);

    let aFaceRefId = aGraph.refs().faces().ids_of(ShellId::start())[0];
    assert_eq!(anExp.current_ref(), RefId::from(aFaceRefId));
}

#[test]
fn BRepGraph_ParentExplorerTest__AvoidKind_Solid_PrunesProducts() {
    let aGraph = boxGraph();

    let anExp = ParentExplorer::with_target_avoid(
        &aGraph,
        firstFace(),
        NodeKind::PRODUCT,
        Some(NodeKind::SOLID),
        false,
        TraversalMode::Recursive,
    )
    .unwrap();
    assert!(!anExp.more());
}

#[test]
fn BRepGraph_ParentExplorerTest__AvoidKind_EmitBoundary_ReturnsSolidInsteadOfProducts() {
    let aGraph = boxGraph();

    let mut anExp = ParentExplorer::with_target_avoid(
        &aGraph,
        firstFace(),
        NodeKind::PRODUCT,
        Some(NodeKind::SOLID),
        true,
        TraversalMode::Recursive,
    )
    .unwrap();
    assert!(anExp.more());
    assert_eq!(anExp.current().def_id, firstSolid());

    anExp.next().unwrap();
    assert!(!anExp.more());
}

#[test]
fn BRepGraph_ParentExplorerTest__AvoidKind_SameAsTarget_IsIgnored() {
    let aGraph = boxGraph();

    let mut anExp = ParentExplorer::with_target_avoid(
        &aGraph,
        firstFace(),
        NodeKind::SOLID,
        Some(NodeKind::SOLID),
        false,
        TraversalMode::Recursive,
    )
    .unwrap();
    assert!(anExp.more());
    assert_eq!(anExp.current().def_id, firstSolid());

    anExp.next().unwrap();
    assert!(!anExp.more());
}

#[test]
fn BRepGraph_ParentExplorerTest__AllParents_AvoidSolid_PrunesProducts() {
    let aGraph = boxGraph();

    let mut anExp = ParentExplorer::with_avoid(
        &aGraph,
        firstFace(),
        Some(NodeKind::SOLID),
        false,
        TraversalMode::Recursive,
    )
    .unwrap();
    assert!(anExp.more());
    assert_eq!(anExp.current().def_id, NodeId::from(ShellId::start()));

    anExp.next().unwrap();
    assert!(!anExp.more());
}

#[test]
fn BRepGraph_ParentExplorerTest__AllParents_AvoidSolidEmitBoundary_ReturnsShellAndSolid() {
    let aGraph = boxGraph();

    let mut anExp = ParentExplorer::with_avoid(
        &aGraph,
        firstFace(),
        Some(NodeKind::SOLID),
        true,
        TraversalMode::Recursive,
    )
    .unwrap();
    assert!(anExp.more());
    assert_eq!(anExp.current().def_id, NodeId::from(ShellId::start()));

    anExp.next().unwrap();
    assert!(anExp.more());
    assert_eq!(anExp.current().def_id, firstSolid());

    anExp.next().unwrap();
    assert!(!anExp.more());
}

#[test]
fn BRepGraph_ParentExplorerTest__SharedProduct_ProductParentsKeepDistinctContexts() {
    let mut aGraph = boxGraph();

    let aPart = addRootPart(&mut aGraph, firstSolid());
    let anAssembly = addRootAssembly(&mut aGraph);
    assert!(aPart.is_valid());
    assert!(anAssembly.is_valid());

    let aT1 = Transform3::translation_vector(Vector3::new(10.0, 0.0, 0.0));
    let aT2 = Transform3::translation_vector(Vector3::new(25.0, 0.0, 0.0));
    assert!(appendOccurrence(&mut aGraph, anAssembly, aPart, &Location::new(aT1)).is_valid());
    assert!(appendOccurrence(&mut aGraph, anAssembly, aPart, &Location::new(aT2)).is_valid());

    let mut aPartCount = 0;
    let mut aLoc1 = Location::default();
    let mut aLoc2 = Location::default();
    let mut anExp = ParentExplorer::with_target(&aGraph, firstSolid(), NodeKind::PRODUCT).unwrap();
    while anExp.more() {
        if anExp.current().def_id == NodeId::from(aPart) {
            if aPartCount == 0 {
                aLoc1 = anExp.current().location;
            } else if aPartCount == 1 {
                aLoc2 = anExp.current().location;
            }
            aPartCount += 1;
        }
        anExp.next().unwrap();
    }

    assert_eq!(aPartCount, 2);
    assert!(!aLoc1.is_equal(&aLoc2));
}

#[test]
fn BRepGraph_ParentExplorerTest__DeepProductOccurrenceChain_ComposesToTopProduct() {
    let mut aGraph = boxGraph();

    let aPart = ProductId::start();
    let mut aChildProduct = aPart;
    let mut anExpectedX = 0.0;
    let mut aTopProduct = ProductId::default();
    for aLevel in 1..=6 {
        aTopProduct = addRootAssembly(&mut aGraph);
        assert!(aTopProduct.is_valid());

        let aTrsf = Transform3::translation_vector(Vector3::new(f64::from(aLevel), 0.0, 0.0));
        let aLoc = Location::new(aTrsf);
        assert!(appendOccurrence(&mut aGraph, aTopProduct, aChildProduct, &aLoc).is_valid());
        anExpectedX += f64::from(aLevel);
        aChildProduct = aTopProduct;
    }

    let mut foundTop = false;
    let mut anExp = ParentExplorer::with_target(&aGraph, firstSolid(), NodeKind::PRODUCT).unwrap();
    while anExp.more() {
        if anExp.current().def_id == NodeId::from(aTopProduct) {
            foundTop = true;
            let aX = anExp
                .leaf_location()
                .transformation()
                .translation_part()
                .x();
            assert!((aX - anExpectedX).abs() <= precision::confusion());
        }
        anExp.next().unwrap();
    }
    assert!(foundTop);
}

#[test]
fn BRepGraph_ParentExplorerTest__DeepProductOccurrenceChain_ReachesNestedOccurrences() {
    let mut aGraph = BRepGraph::new();
    aGraph.clear().unwrap();

    let aPart = aGraph.editor().products().add_empty();
    let aSubAssembly = aGraph.editor().products().add_empty();
    let aTopAssembly = aGraph.editor().products().add_empty();
    assert!(aPart.is_valid());
    assert!(aSubAssembly.is_valid());
    assert!(aTopAssembly.is_valid());

    let aPartOccurrence = appendOccurrence(&mut aGraph, aSubAssembly, aPart, &Location::default());
    let aSubOccurrence = appendOccurrence(
        &mut aGraph,
        aTopAssembly,
        aSubAssembly,
        &Location::default(),
    );
    assert!(aPartOccurrence.is_valid());
    assert!(aSubOccurrence.is_valid());

    let mut hasPartOccurrence = false;
    let mut hasSubOccurrence = false;
    let mut aCount = 0;
    let mut anIt =
        ParentExplorer::with_target(&aGraph, aPart.into(), NodeKind::OCCURRENCE).unwrap();
    while anIt.more() {
        hasPartOccurrence |= anIt.current().def_id == NodeId::from(aPartOccurrence);
        hasSubOccurrence |= anIt.current().def_id == NodeId::from(aSubOccurrence);
        aCount += 1;
        anIt.next().unwrap();
    }

    assert_eq!(aCount, 2);
    assert!(hasPartOccurrence);
    assert!(hasSubOccurrence);
}

#[test]
fn BRepGraph_ParentExplorerTest__DeepCompoundChain_ReachesTopCompound() {
    let mut aGraph = boxGraph();

    let mut aChild = firstSolid();
    let mut aTopCompound = CompoundId::default();
    for _aLevel in 0..8 {
        aTopCompound = addCompound(&mut aGraph, aChild);
        assert!(aTopCompound.is_valid());
        aChild = aTopCompound.into();
    }

    let mut aCompoundCount = 0;
    let mut foundTop = false;
    let mut anExp = ParentExplorer::with_target(&aGraph, firstSolid(), NodeKind::COMPOUND).unwrap();
    while anExp.more() {
        aCompoundCount += 1;
        foundTop |= anExp.current().def_id == NodeId::from(aTopCompound);
        anExp.next().unwrap();
    }
    assert_eq!(aCompoundCount, 8);
    assert!(foundTop);
}

#[test]
fn BRepGraph_ParentExplorerTest__VertexChildOfDeepCompoundChain_ReachesTopCompound() {
    let mut aGraph = BRepGraph::new();
    aGraph.clear().unwrap();
    let aVertex = aGraph
        .editor()
        .vertices()
        .add(Point3::new(1.0, 2.0, 3.0), 1.0e-7);
    assert!(aVertex.is_valid());

    let mut aChild: NodeId = aVertex.into();
    let mut aTopCompound = CompoundId::default();
    let mut anExpectedX = 0.0;
    for aLevel in 1..=4 {
        aTopCompound = addCompound(&mut aGraph, aChild);
        assert!(aTopCompound.is_valid());

        let aStepLoc = translationLocation(f64::from(aLevel), 0.0, 0.0);
        let aChildRefs = aGraph.refs().children().ids_of(aTopCompound).to_vec();
        assert_eq!(aChildRefs.len(), 1);
        aGraph
            .editor()
            .generic()
            .set_child_ref_local_location(aChildRefs[0], &aStepLoc)
            .unwrap();
        anExpectedX += f64::from(aLevel);

        aChild = aTopCompound.into();
    }

    let mut aCompoundCount = 0;
    let mut foundTop = false;
    let mut anExp =
        ParentExplorer::with_target(&aGraph, aVertex.into(), NodeKind::COMPOUND).unwrap();
    while anExp.more() {
        aCompoundCount += 1;
        if anExp.current().def_id == NodeId::from(aTopCompound) {
            foundTop = true;
            let aX = anExp
                .leaf_location()
                .transformation()
                .translation_part()
                .x();
            assert!((aX - anExpectedX).abs() <= precision::confusion());
        }
        anExp.next().unwrap();
    }

    assert_eq!(aCompoundCount, 4);
    assert!(foundTop);
    assert!(aGraph.validate_relations());
}

/// `aGraph.Editor().Gen().SetChildRefLocalLocation(Refs().Children().IdsOf(c).First(), l)`.
fn setFirstChildRefLocation(aGraph: &mut BRepGraph, theCompound: CompoundId, theLoc: &Location) {
    assert_eq!(aGraph.refs().children().ids_of(theCompound).len(), 1);
    let aRef = aGraph.refs().children().ids_of(theCompound)[0];
    aGraph
        .editor()
        .generic()
        .set_child_ref_local_location(aRef, theLoc)
        .unwrap();
}

#[test]
fn BRepGraph_ParentExplorerTest__MixedProductOccurrenceAndCompoundChain_PreservesLocations() {
    let mut aGraph = boxGraph();

    let aComp1Loc = translationLocation(2.0, 0.0, 0.0);
    let aComp2Loc = rotationZLocation(0.25);
    let aComp3Loc = translationLocation(0.0, 3.0, 0.0);
    let anOcc1Loc = translationLocation(0.0, 0.0, 5.0);
    let anOcc2Loc = rotationZLocation(-0.5);

    let aComp1 = addCompound(&mut aGraph, firstSolid());
    assert!(aComp1.is_valid());
    setFirstChildRefLocation(&mut aGraph, aComp1, &aComp1Loc);

    let aComp2 = addCompound(&mut aGraph, aComp1.into());
    assert!(aComp2.is_valid());
    setFirstChildRefLocation(&mut aGraph, aComp2, &aComp2Loc);

    let aComp3 = addCompound(&mut aGraph, aComp2.into());
    assert!(aComp3.is_valid());
    setFirstChildRefLocation(&mut aGraph, aComp3, &aComp3Loc);

    let aPart = addRootPart(&mut aGraph, aComp3.into());
    let aMidAssembly = addRootAssembly(&mut aGraph);
    let aTopAssembly = addRootAssembly(&mut aGraph);
    assert!(aPart.is_valid());
    assert!(aMidAssembly.is_valid());
    assert!(aTopAssembly.is_valid());

    let anOcc1 = appendOccurrence(&mut aGraph, aMidAssembly, aPart, &anOcc1Loc);
    let anOcc2 = appendOccurrence(&mut aGraph, aTopAssembly, aMidAssembly, &anOcc2Loc);
    assert!(anOcc1.is_valid());
    assert!(anOcc2.is_valid());

    let aToPart = (&anOcc2Loc * &anOcc1Loc).unwrap();
    let aToComp3 = aToPart.clone();
    let aToComp2 = (&aToComp3 * &aComp3Loc).unwrap();
    let aToComp1 = (&aToComp2 * &aComp2Loc).unwrap();
    let aToSolid = (&aToComp1 * &aComp1Loc).unwrap();

    let mut hasTopAssembly = false;
    let mut hasMidAssembly = false;
    let mut hasPart = false;
    let mut hasOcc2 = false;
    let mut hasOcc1 = false;
    let mut hasComp3 = false;
    let mut hasComp2 = false;
    let mut hasComp1 = false;

    let mut anExp = ParentExplorer::new(&aGraph, firstSolid()).unwrap();
    while anExp.more() {
        if !locationsEquivalent(&anExp.leaf_location(), &aToSolid) {
            anExp.next().unwrap();
            continue;
        }

        let aCurrent = anExp.current().def_id;
        let aLocation = anExp.current().location;
        if aCurrent == NodeId::from(aTopAssembly) {
            hasTopAssembly = true;
            expectLocationsEquivalent(&aLocation, &Location::default());
        } else if aCurrent == NodeId::from(anOcc2) {
            hasOcc2 = true;
            expectLocationsEquivalent(&aLocation, &anOcc2Loc);
        } else if aCurrent == NodeId::from(aMidAssembly) {
            hasMidAssembly = true;
            expectLocationsEquivalent(&aLocation, &anOcc2Loc);
        } else if aCurrent == NodeId::from(anOcc1) {
            hasOcc1 = true;
            expectLocationsEquivalent(&aLocation, &aToPart);
        } else if aCurrent == NodeId::from(aPart) {
            hasPart = true;
            expectLocationsEquivalent(&aLocation, &aToPart);
        } else if aCurrent == NodeId::from(aComp3) {
            hasComp3 = true;
            expectLocationsEquivalent(&aLocation, &aToComp3);
        } else if aCurrent == NodeId::from(aComp2) {
            hasComp2 = true;
            expectLocationsEquivalent(&aLocation, &aToComp2);
        } else if aCurrent == NodeId::from(aComp1) {
            hasComp1 = true;
            expectLocationsEquivalent(&aLocation, &aToComp1);
        }
        anExp.next().unwrap();
    }

    assert!(hasTopAssembly);
    assert!(hasMidAssembly);
    assert!(hasPart);
    assert!(hasOcc2);
    assert!(hasOcc1);
    assert!(hasComp3);
    assert!(hasComp2);
    assert!(hasComp1);
}

#[test]
fn BRepGraph_ParentExplorerTest__ShapeRootDirectParent_IsOccurrenceNotProductShortcut() {
    let aGraph = boxGraph();

    // BRepGraph::ShapesView::Add() auto-creates a root Product for the shape root node.
    // Use that product instead of creating a duplicate.
    assert!(aGraph.topo().products().nb() > 0);
    let aPart = ProductId::new(0);
    assert_eq!(aGraph.topo().products().nb_components(aPart), 1);
    let aRootOccurrence = aGraph.topo().products().component(aPart, 0);
    assert!(aRootOccurrence.is_valid());

    let mut anExp =
        ParentExplorer::with_mode(&aGraph, firstSolid(), TraversalMode::DirectParents).unwrap();
    assert!(anExp.more());
    assert_eq!(anExp.current().def_id, NodeId::from(aRootOccurrence));
    assert_eq!(anExp.current_child(), firstSolid());
    assert_eq!(anExp.current_link_kind(), LinkKind::Structural);
    assert!(!anExp.current_ref().is_valid());

    anExp.next().unwrap();
    assert!(!anExp.more());

    let aProductExp =
        ParentExplorer::with_target(&aGraph, firstSolid(), NodeKind::PRODUCT).unwrap();
    assert!(aProductExp.more());
    assert_eq!(aProductExp.current().def_id, NodeId::from(aPart));
}

#[test]
fn BRepGraph_ParentExplorerTest__TargetProductAvoidCompound_StopsAtCompound() {
    let aBuilder = BRepBuilder::default();
    let mut aCompound = Compound::default();
    aBuilder.make_compound(&mut aCompound);
    aBuilder
        .add(&mut aCompound, &make_box(10.0, 20.0, 30.0).into())
        .unwrap();

    let mut aGraph = BRepGraph::new();
    aGraph.clear().unwrap();
    let aShape: Shape = aCompound.into();
    let _aBuildRes = aGraph.shapes_mut().add(&aShape).unwrap();
    assert!(!aGraph.is_empty());
    assert!(aGraph.topo().products().nb() > 0);

    let anExp = ParentExplorer::with_target_avoid(
        &aGraph,
        firstSolid(),
        NodeKind::PRODUCT,
        Some(NodeKind::COMPOUND),
        false,
        TraversalMode::Recursive,
    )
    .unwrap();
    assert!(!anExp.more());
}

#[test]
fn BRepGraph_ParentExplorerTest__OccurrenceParent_ExposeOccurrenceRef() {
    let mut aGraph = boxGraph();

    let aPart = addRootPart(&mut aGraph, firstSolid());
    let anAssembly = addRootAssembly(&mut aGraph);
    assert!(aPart.is_valid());
    assert!(anAssembly.is_valid());

    let anOccurrence = appendOccurrence(&mut aGraph, anAssembly, aPart, &Location::default());
    assert!(anOccurrence.is_valid());

    let anExp = ParentExplorer::with_target_mode(
        &aGraph,
        anOccurrence.into(),
        NodeKind::PRODUCT,
        TraversalMode::DirectParents,
    )
    .unwrap();
    assert!(anExp.more());
    assert_eq!(anExp.current().def_id, NodeId::from(anAssembly));
    assert_eq!(anExp.current_child(), NodeId::from(anOccurrence));
    assert_eq!(anExp.current_link_kind(), LinkKind::Reference);

    let anOccurrenceRefId = aGraph.refs().occurrences().ids_of(anAssembly)[0];
    assert_eq!(anExp.current_ref(), RefId::from(anOccurrenceRefId));
}

#[test]
fn BRepGraph_ParentExplorerTest__OccurrenceParents_SetRefOccurrenceDefRejectsSharing() {
    let mut aGraph = boxGraph();

    let aPart = addRootPart(&mut aGraph, firstSolid());
    let anAssembly1 = addRootAssembly(&mut aGraph);
    let anAssembly2 = addRootAssembly(&mut aGraph);
    assert!(aPart.is_valid());
    assert!(anAssembly1.is_valid());
    assert!(anAssembly2.is_valid());

    let aDefault = OccurrenceId::default();
    let (anOcc1, anOccRef1) = aGraph
        .editor()
        .products()
        .append(anAssembly1, aPart, &Location::default(), aDefault)
        .unwrap();
    let (anOcc2, anOccRef2) = aGraph
        .editor()
        .products()
        .append(anAssembly2, aPart, &Location::default(), aDefault)
        .unwrap();
    assert!(anOcc1.is_valid());
    assert!(anOcc2.is_valid());
    assert!(anOccRef1.is_valid());
    assert!(anOccRef2.is_valid());

    aGraph
        .editor()
        .occurrences()
        .set_ref_child_occurrence_id(anOccRef2, anOcc1)
        .unwrap();
    assert_eq!(
        aGraph
            .refs()
            .occurrences()
            .entry(anOccRef2)
            .child_occurrence_id,
        anOcc2
    );
    assert!(aGraph.validate_relations());

    let anAuditResult = validate::perform_with_options(&aGraph, &Options::audit());
    assert!(anAuditResult.is_valid());
}

#[test]
fn BRepGraph_ParentExplorerTest__ProductParents_ImmediateOccurrence_IsStructural() {
    let mut aGraph = boxGraph();

    let aPart = addRootPart(&mut aGraph, firstSolid());
    let anAssembly = addRootAssembly(&mut aGraph);
    assert!(aPart.is_valid());
    assert!(anAssembly.is_valid());

    let anOccurrence = appendOccurrence(&mut aGraph, anAssembly, aPart, &Location::default());
    assert!(anOccurrence.is_valid());

    let anExp =
        ParentExplorer::with_mode(&aGraph, aPart.into(), TraversalMode::DirectParents).unwrap();
    assert!(anExp.more());
    assert_eq!(anExp.current().def_id, NodeId::from(anOccurrence));
    assert_eq!(anExp.current_child(), NodeId::from(aPart));
    assert_eq!(anExp.current_link_kind(), LinkKind::Structural);
    assert!(!anExp.current_ref().is_valid());
}

#[test]
fn BRepGraph_ParentExplorerTest__CoEdgeParents_ImmediateWireIsVisible() {
    let aGraph = boxGraph();

    let aWireRefIds = aGraph.refs().wires().ids_of(FaceId::start());
    assert!(!aWireRefIds.is_empty());
    let aWireId = aGraph.refs().wires().entry(aWireRefIds[0]).child_wire_id;

    let aCoEdgeIds = &aGraph.topo().wires().relations(aWireId).co_edge_ids;
    assert!(!aCoEdgeIds.is_empty());
    let aCoEdgeId = aCoEdgeIds[0];

    let mut anExp = ParentExplorer::new(&aGraph, aCoEdgeId.into()).unwrap();
    assert!(anExp.more());
    assert_eq!(anExp.current().def_id, NodeId::from(aWireId));
    assert_eq!(anExp.current_link_kind(), LinkKind::Structural);
    assert!(!anExp.current_ref().is_valid());

    anExp.next().unwrap();
    assert!(anExp.more());
    assert_eq!(anExp.current().def_id, firstFace());
}

#[test]
fn BRepGraph_ParentExplorerTest__CoEdgeChildOfCompound_ImmediateCompoundIsVisible() {
    let mut aGraph = boxGraph();

    let aFirstWireRef = aGraph.refs().wires().ids_of(FaceId::start())[0];
    let aWireId = aGraph.refs().wires().entry(aFirstWireRef).child_wire_id;
    let aCoEdgeId = aGraph.topo().wires().relations(aWireId).co_edge_ids[0];

    let aCompound = addCompound(&mut aGraph, aCoEdgeId.into());
    assert!(aCompound.is_valid());

    let mut hasWire = false;
    let mut hasCompound = false;
    let mut anExp =
        ParentExplorer::with_mode(&aGraph, aCoEdgeId.into(), TraversalMode::DirectParents).unwrap();
    while anExp.more() {
        if anExp.current().def_id == NodeId::from(aWireId) {
            hasWire = true;
        }
        if anExp.current().def_id == NodeId::from(aCompound) {
            hasCompound = true;
            assert_eq!(anExp.current_link_kind(), LinkKind::Reference);
            assert!(anExp.current_ref().is_valid());
        }
        anExp.next().unwrap();
    }

    assert!(hasWire);
    assert!(hasCompound);
}

#[test]
fn BRepGraph_ParentExplorerTest__ProductRoot_HasNoParents() {
    let mut aGraph = boxGraph();

    let aRootProduct = addRootAssembly(&mut aGraph);
    assert!(aRootProduct.is_valid());

    let anExp = ParentExplorer::new(&aGraph, aRootProduct.into()).unwrap();
    assert!(!anExp.more());
}
