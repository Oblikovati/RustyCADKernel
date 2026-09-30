//! Direct translation of TopExp_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! BRepPrimAPI_MakeBox is the BRep fake of `support` (TKPrim is not ported; the
//! unchanged upstream file runs natively against the shared oracle build, which has
//! TKPrim).
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape};
use rustycad_tkg3d::top_abs::ShapeEnum;
use support::make_box;

/// `BRepPrimAPI_MakeBox aBoxMaker(10.0, 20.0, 30.0); aBoxMaker.Shape()`; the fake is
/// always done.
fn box_shape() -> Shape {
    make_box(10.0, 20.0, 30.0).into_shape()
}

#[test]
fn TopExp_Test__MapShapes_Faces() {
    let aBox = box_shape();

    let mut aFaceMap = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(&aBox, ShapeEnum::Face, &mut aFaceMap).unwrap();

    assert_eq!(aFaceMap.extent(), 6, "A box should have exactly 6 faces");
}

#[test]
fn TopExp_Test__MapShapes_Edges() {
    let aBox = box_shape();

    let mut anEdgeMap = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(&aBox, ShapeEnum::Edge, &mut anEdgeMap).unwrap();

    assert_eq!(anEdgeMap.extent(), 12, "A box should have exactly 12 edges");
}

#[test]
fn TopExp_Test__MapShapes_Vertices() {
    let aBox = box_shape();

    let mut aVertexMap = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(&aBox, ShapeEnum::Vertex, &mut aVertexMap).unwrap();

    assert_eq!(
        aVertexMap.extent(),
        8,
        "A box should have exactly 8 vertices"
    );
}

#[test]
fn TopExp_Test__FirstVertex_LastVertex() {
    let aBox = box_shape();

    let anExp = Explorer::new(&aBox, ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
    assert!(anExp.more());

    let anEdge = topods::edge(anExp.current()).unwrap();

    let aFirstVertex = topexp::first_vertex(&anEdge, false);
    let aLastVertex = topexp::last_vertex(&anEdge, false);

    assert!(!aFirstVertex.is_null(), "First vertex should not be null");
    assert!(!aLastVertex.is_null(), "Last vertex should not be null");
    assert!(
        !aFirstVertex.is_same(&aLastVertex),
        "First and last vertices of a box edge should be distinct"
    );
}

#[test]
fn TopExp_Test__Vertices() {
    let aBox = box_shape();

    let anExp = Explorer::new(&aBox, ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
    assert!(anExp.more());

    let anEdge = topods::edge(anExp.current()).unwrap();

    let (aV1, aV2) = topexp::vertices(&anEdge, false);

    assert!(
        !aV1.is_null(),
        "First vertex from Vertices() should not be null"
    );
    assert!(
        !aV2.is_null(),
        "Second vertex from Vertices() should not be null"
    );
}

#[test]
fn TopExp_Test__CommonVertex() {
    let aBox = box_shape();

    let mut anEdgeMap = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(&aBox, ShapeEnum::Edge, &mut anEdgeMap).unwrap();
    assert!(anEdgeMap.extent() >= 2);

    // Find two adjacent edges (sharing a common vertex)
    let mut isFound = false;
    let mut i = 1;
    while i <= anEdgeMap.extent() && !isFound {
        let anEdge1 = topods::edge(&anEdgeMap[i]).unwrap();
        let mut j = i + 1;
        while j <= anEdgeMap.extent() && !isFound {
            let anEdge2 = topods::edge(&anEdgeMap[j]).unwrap();
            if let Some(aCommonVertex) = topexp::common_vertex(&anEdge1, &anEdge2) {
                assert!(!aCommonVertex.is_null(), "Common vertex should not be null");
                isFound = true;
            }
            j += 1;
        }
        i += 1;
    }

    assert!(
        isFound,
        "Should find at least one pair of adjacent edges in a box"
    );
}

#[test]
fn TopExp_Test__Explorer_NestedFaceEdge_Terminates() {
    let aBox = box_shape();

    // Reproduce the pattern from issue #1194: nested TopExp_Explorer
    // iterating edges within faces must terminate.
    let mut aTotalEdgeRefs = 0;
    let mut aFaceCount = 0;
    let mut aFaceExp = Explorer::new(&aBox, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    while aFaceExp.more() {
        aFaceCount += 1;
        let mut anEdgeCount = 0;
        let aFace = topods::face(aFaceExp.current()).unwrap();
        let mut anEdgeExp = Explorer::new(&aFace, ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
        while anEdgeExp.more() {
            assert!(!anEdgeExp.current().is_null());
            anEdgeCount += 1;
            // Guard against infinite loop
            assert!(
                anEdgeCount < 100,
                "Edge iteration appears infinite on face {aFaceCount}"
            );
            anEdgeExp.next().unwrap();
        }
        assert_eq!(
            anEdgeCount, 4,
            "Each box face should reference exactly 4 edges"
        );
        aTotalEdgeRefs += anEdgeCount;
        aFaceExp.next().unwrap();
    }
    assert_eq!(aFaceCount, 6);
    assert_eq!(
        aTotalEdgeRefs, 24,
        "6 faces * 4 edges each = 24 edge references"
    );
}

#[test]
fn TopExp_Test__Explorer_ReInit_Terminates() {
    let aBox = box_shape();

    // Test that re-initializing an explorer (Clear + Init cycle) works correctly.
    let mut anExp = Explorer::default();
    let mut aFaceExp = Explorer::new(&aBox, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    while aFaceExp.more() {
        let aFace = topods::face(aFaceExp.current()).unwrap();
        anExp
            .init(&aFace, ShapeEnum::Edge, ShapeEnum::Shape)
            .unwrap();
        let mut anEdgeCount = 0;
        while anExp.more() {
            anEdgeCount += 1;
            assert!(
                anEdgeCount < 100,
                "Edge iteration appears infinite (reused explorer)"
            );
            anExp.next().unwrap();
        }
        assert_eq!(anEdgeCount, 4);
        aFaceExp.next().unwrap();
    }
}

#[test]
fn TopExp_Test__MapShapesAndAncestors() {
    let aBox = box_shape();

    let mut anEdgeFaceMap = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(&aBox, ShapeEnum::Edge, ShapeEnum::Face, &mut anEdgeFaceMap)
        .unwrap();

    assert_eq!(
        anEdgeFaceMap.extent(),
        12,
        "A box should have 12 edges in the ancestor map"
    );

    // Each edge of a box is shared by exactly 2 faces
    for i in 1..=anEdgeFaceMap.extent() {
        let aFaceList = &anEdgeFaceMap[i];
        assert_eq!(
            aFaceList.extent(),
            2,
            "Each box edge should be shared by exactly 2 faces"
        );
    }
}
