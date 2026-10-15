//! Direct translation of BRepFilletAPI_MakeChamfer_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkbo::brep_algo_api::BRepAlgoAPIFuse;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkfillet::brep_fillet_api::BRepFilletAPIMakeChamfer;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;

fn MakeBox(dx: f64, dy: f64, dz: f64) -> Shape {
    BRepPrimAPIMakeBox::from_sizes(dx, dy, dz)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

fn Edges(s: &Shape) -> Vec<Edge> {
    let mut out = Vec::new();
    let mut ex = Explorer::new(s, ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
    while ex.more() {
        out.push(topods::edge(ex.current()).unwrap());
        ex.next().unwrap();
    }
    out
}

fn IsValid(s: &Shape) -> bool {
    BRepCheckAnalyzer::new(s, true, false, false)
        .unwrap()
        .is_valid()
        .unwrap()
}

fn FaceCount(s: &Shape) -> usize {
    let mut n = 0;
    let mut ex = Explorer::new(s, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    while ex.more() {
        n += 1;
        ex.next().unwrap();
    }
    n
}

#[test]
fn BRepFilletAPI_MakeChamferTest__SymmetricChamfer() {
    let aBox = MakeBox(20.0, 20.0, 20.0);
    let anEdge = Edges(&aBox).remove(0);
    let mut aChamfer = BRepFilletAPIMakeChamfer::new(&aBox).unwrap();
    aChamfer.add_with_dist(2.0, &anEdge).unwrap();
    let aResult = aChamfer.shape().unwrap().clone();
    assert!(aChamfer.is_done());
    assert!(IsValid(&aResult));
}

#[test]
fn BRepFilletAPI_MakeChamferTest__AsymmetricChamfer() {
    let aBox = MakeBox(20.0, 20.0, 20.0);
    let mut anEdgeFaceMap = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(&aBox, ShapeEnum::Edge, ShapeEnum::Face, &mut anEdgeFaceMap)
        .unwrap();
    let anEdge = Edges(&aBox).remove(0);
    let aFace = topods::face(
        anEdgeFaceMap
            .find_from_key(anEdge.as_ref())
            .unwrap()
            .first()
            .unwrap(),
    )
    .unwrap();
    let mut aChamfer = BRepFilletAPIMakeChamfer::new(&aBox).unwrap();
    aChamfer.add_with_dists(1.0, 3.0, &anEdge, &aFace).unwrap();
    let aResult = aChamfer.shape().unwrap().clone();
    assert!(aChamfer.is_done());
    assert!(IsValid(&aResult));
}

#[test]
fn BRepFilletAPI_MakeChamferTest__Issue1177_ChamferAllEdgesFlatBox_SucceedsWithoutCrash() {
    let aBox = MakeBox(50.0, 50.0, 10.0);
    let mut aChamfer = BRepFilletAPIMakeChamfer::new(&aBox).unwrap();
    for e in Edges(&aBox) {
        aChamfer.add_with_dist(5.0, &e).unwrap();
    }
    aChamfer
        .build(&ProgressRange::default())
        .expect("Chamfer build must not crash");
    assert!(
        aChamfer.is_done(),
        "Chamfering all edges of a 50x50x10 slab with d=5 should succeed (issue #1177)"
    );
    let aResult = aChamfer.shape().unwrap().clone();
    assert!(!aResult.is_null());
    assert!(
        BRepCheckAnalyzer::new(&aResult, true, false, true)
            .unwrap()
            .is_valid()
            .unwrap()
    );
    let mut aMaxTolerance: f64 = 0.0;
    let mut ex = Explorer::new(&aResult, ShapeEnum::Vertex, ShapeEnum::Shape).unwrap();
    while ex.more() {
        aMaxTolerance = aMaxTolerance
            .max(BRepTool::vertex_tolerance(&topods::vertex(ex.current()).unwrap()).unwrap());
        ex.next().unwrap();
    }
    for e in Edges(&aResult) {
        aMaxTolerance = aMaxTolerance.max(BRepTool::edge_tolerance(&e).unwrap());
    }
    assert!(
        aMaxTolerance <= 1.01e-4,
        "Chamfer construction must not hide inconsistent edge parameterization with tolerance: {aMaxTolerance}"
    );
}

#[test]
fn BRepFilletAPI_MakeChamferTest__ChamferMoreFaces() {
    let aBox = MakeBox(20.0, 20.0, 20.0);
    let anEdge = Edges(&aBox).remove(0);
    let mut aChamfer = BRepFilletAPIMakeChamfer::new(&aBox).unwrap();
    aChamfer.add_with_dist(2.0, &anEdge).unwrap();
    let aResult = aChamfer.shape().unwrap().clone();
    assert!(aChamfer.is_done());
    assert!(FaceCount(&aResult) > 6);
}

#[test]
fn BRepFilletAPI_MakeChamferTest__ChamferAfterBooleanFusion() {
    let aBox = MakeBox(10.0, 10.0, 10.0);
    let anAxis = Axis2::new(
        Point3::new(5.0, 0.0, 5.0),
        Direction3::new(0.0, 1.0, 0.0).unwrap(),
    )
    .unwrap();
    let aCyl = BRepPrimAPIMakeCylinder::with_axes(anAxis, 3.0, 10.0)
        .shape()
        .unwrap()
        .clone();
    let aFuse = BRepAlgoAPIFuse::from_shapes(&aBox, &aCyl).unwrap();
    assert!(aFuse.is_done());
    let aFused = aFuse.shape().clone();
    let mut aVtxFaceMap = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(
        &aFused,
        ShapeEnum::Vertex,
        ShapeEnum::Face,
        &mut aVtxFaceMap,
    )
    .unwrap();
    let mut anEdgeFaceMap = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(
        &aFused,
        ShapeEnum::Edge,
        ShapeEnum::Face,
        &mut anEdgeFaceMap,
    )
    .unwrap();
    let mut aChamfer = BRepFilletAPIMakeChamfer::new(&aFused).unwrap();
    let mut anEdgeCount = 0;
    for anEdge in Edges(&aFused) {
        if anEdgeCount >= 4 {
            break;
        }
        if !anEdgeFaceMap.contains(anEdge.as_ref())
            || anEdgeFaceMap
                .find_from_key(anEdge.as_ref())
                .unwrap()
                .is_empty()
        {
            continue;
        }
        let mut hasComplexVertex = false;
        let mut vx = Explorer::new(anEdge.as_ref(), ShapeEnum::Vertex, ShapeEnum::Shape).unwrap();
        while vx.more() {
            if aVtxFaceMap.contains(vx.current())
                && aVtxFaceMap.find_from_key(vx.current()).unwrap().len() >= 3
            {
                hasComplexVertex = true;
                break;
            }
            vx.next().unwrap();
        }
        if !hasComplexVertex {
            continue;
        }
        let aFace = topods::face(
            anEdgeFaceMap
                .find_from_key(anEdge.as_ref())
                .unwrap()
                .first()
                .unwrap(),
        )
        .unwrap();
        aChamfer.add_with_dists(0.5, 0.5, &anEdge, &aFace).unwrap();
        anEdgeCount += 1;
    }
    assert!(anEdgeCount > 0);
    // Must not crash; may succeed or fail gracefully with IsDone() == false (an exception
    // instead of a crash is acceptable).
    if aChamfer.build(&ProgressRange::default()).is_ok() && aChamfer.is_done() {
        let aResult = aChamfer.shape().unwrap().clone();
        assert!(IsValid(&aResult));
    }
}

#[test]
fn BRepFilletAPI_MakeChamferTest__SequentialChamferNoCrash() {
    let mut aShape = MakeBox(20.0, 20.0, 20.0);
    let mut aPrevEdges = 0;
    let mut aSuccessCount = 0;
    for i in 0..3 {
        let mut anEdgeMap = IndexedMapOfShape::default();
        topexp::map_shapes_of_type(&aShape, ShapeEnum::Edge, &mut anEdgeMap).unwrap();
        if anEdgeMap.is_empty() {
            break;
        }
        if i > 0 && aPrevEdges > 0 {
            assert_ne!(anEdgeMap.extent(), aPrevEdges);
        }
        aPrevEdges = anEdgeMap.extent();
        let anIdx = (i * 3 + 1) % anEdgeMap.extent() + 1;
        let anEdge = topods::edge(anEdgeMap.find_key(anIdx).unwrap()).unwrap();
        let mut aChamfer = BRepFilletAPIMakeChamfer::new(&aShape).unwrap();
        aChamfer.add_with_dist(1.0, &anEdge).unwrap();
        match aChamfer.build(&ProgressRange::default()) {
            Ok(()) if aChamfer.is_done() => {
                aShape = aChamfer.shape().unwrap().clone();
                aSuccessCount += 1;
            }
            Ok(()) => break,
            Err(e)
                if matches!(
                    e.exception_type(),
                    "Standard_ConstructionError" | "StdFail_NotDone"
                ) =>
            {
                break;
            }
            Err(e) => panic!("unexpected exception {e:?}"),
        }
    }
    assert!(aSuccessCount >= 1);
}
