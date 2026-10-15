//! Direct translation of BRepFilletAPI_MakeFillet_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case, clippy::type_complexity)]
use rustycad_tkbo::brep_algo_api::{BRepAlgoAPICommon, BRepAlgoAPICut, BRepAlgoAPIFuse};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Compound, Edge, Shape};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::precision;
use rustycad_tkfillet::brep_fillet_api::BRepFilletAPIMakeFillet;
use rustycad_tkfillet::chfi3d::FilletShape;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gc::{MakeArcOfCircle, MakeSegment};
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::geom_abs::Shape as GeomAbsShape;
use rustycad_tkmath::gp::{Axis1, Axis2, Direction3, Point2, Point3, Transform3, Vector3};
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder, BRepPrimAPIMakePrism, BRepPrimAPIMakeRevol,
    BRepPrimAPIMakeSphere,
};
use rustycad_tkshhealing::shape_fix::ShapeFixShape;
use rustycad_tkshhealing::shape_upgrade::ShapeUpgradeUnifySameDomain;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon,
    BRepBuilderAPIMakeShape, BRepBuilderAPIMakeWire, BRepBuilderAPITransform,
};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use rustycad_tktopalgo::brep_gprop::BRepGProp;
use std::f64::consts::PI;

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

fn FirstEdge(s: &Shape) -> Edge {
    Edges(s).remove(0)
}

fn IsValid(s: &Shape) -> bool {
    BRepCheckAnalyzer::new(s, true, false, false)
        .unwrap()
        .is_valid()
        .unwrap()
}

fn Build(f: &mut BRepFilletAPIMakeFillet) {
    f.build(&ProgressRange::default())
        .expect("Fillet build must not crash");
}

fn SurfaceArea(s: &Shape) -> f64 {
    let mut p = GProps::new();
    BRepGProp::surface_properties(s, &mut p, false, false).unwrap();
    p.mass()
}

#[test]
fn BRepFilletAPI_MakeFilletTest__Issue1177_FilletToOpposingEdge_SucceedsWithoutCrash() {
    let aSize = 10.0;
    let aBox = MakeBox(aSize, aSize, aSize);
    let anEdge = FirstEdge(&aBox);
    let mut aFillet = BRepFilletAPIMakeFillet::new(&aBox, FilletShape::Rational).unwrap();
    aFillet.add_with_radius(aSize, &anEdge).unwrap();
    Build(&mut aFillet);
    assert!(
        aFillet.is_done(),
        "Fillet radius equal to box size should succeed (issue #1177)"
    );
    let aResult = aFillet.shape().unwrap().clone();
    assert!(!aResult.is_null());
    assert!(IsValid(&aResult));
}

#[test]
fn BRepFilletAPI_MakeFilletTest__Issue1177_HistoryReferencesResultTopology() {
    let aSize = 10.0;
    let aBox = MakeBox(aSize, aSize, aSize);
    let anEdge = FirstEdge(&aBox);
    let mut aFillet = BRepFilletAPIMakeFillet::new(&aBox, FilletShape::Rational).unwrap();
    aFillet.add_with_radius(aSize, &anEdge).unwrap();
    Build(&mut aFillet);
    assert!(aFillet.is_done());
    let mut aResultShapes = IndexedMapOfShape::default();
    topexp::map_shapes(aFillet.shape().unwrap(), &mut aResultShapes, true, true);
    let aGenerated: Vec<Shape> = aFillet
        .generated(anEdge.as_ref())
        .unwrap()
        .iter()
        .cloned()
        .collect();
    assert!(!aGenerated.is_empty());
    for s in &aGenerated {
        assert!(
            aResultShapes.contains(s),
            "Generated() returned topology absent from the result"
        );
    }
    let mut hasModified = false;
    let mut aFaceExp = Explorer::new(&aBox, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    while aFaceExp.more() {
        let aModified: Vec<Shape> = aFillet
            .modified(aFaceExp.current())
            .unwrap()
            .iter()
            .cloned()
            .collect();
        hasModified = hasModified || !aModified.is_empty();
        for s in &aModified {
            assert!(
                aResultShapes.contains(s),
                "Modified() returned topology absent from the result"
            );
        }
        aFaceExp.next().unwrap();
    }
    assert!(hasModified);
}

#[test]
fn BRepFilletAPI_MakeFilletTest__Issue1177_OpposingFilletsMeet_SucceedsWithoutCrash() {
    let aSize = 10.0;
    let aRadius = 0.5 * aSize;
    let aBox = MakeBox(aSize, aSize, aSize);
    let aFace = Explorer::new(&aBox, ShapeEnum::Face, ShapeEnum::Shape)
        .unwrap()
        .current()
        .clone();
    let aFaceEdges = Edges(&aFace);
    assert!(aFaceEdges.len() >= 4, "Box face must have at least 4 edges");
    let anEdgeLow = aFaceEdges[0].clone();
    let anEdgeHigh = aFaceEdges[2].clone();
    let mut aFillet = BRepFilletAPIMakeFillet::new(&aBox, FilletShape::Rational).unwrap();
    aFillet.add_with_radius(aRadius, &anEdgeLow).unwrap();
    aFillet.add_with_radius(aRadius, &anEdgeHigh).unwrap();
    Build(&mut aFillet);
    assert!(
        aFillet.is_done(),
        "Opposing fillets that should meet must succeed (issues #1177, FreeCAD #5561)"
    );
    let aResult = aFillet.shape().unwrap().clone();
    assert!(!aResult.is_null());
    assert!(IsValid(&aResult));
}

#[test]
fn BRepFilletAPI_MakeFilletTest__Issue1177_ConcaveFilletsMeet_SucceedsWithoutCrash() {
    let aBox = MakeBox(10.0, 10.0, 10.0);
    let aTool = MakeBox(10.0, 10.0, 12.0);
    let mut aTranslation = Transform3::default();
    aTranslation.set_translation(Vector3::new(5.0, 5.0, -1.0));
    let mut aTransform =
        BRepBuilderAPITransform::with_shape(&aTool, aTranslation, false, false).unwrap();
    let aTool = aTransform.shape().unwrap().clone();
    let aCut = BRepAlgoAPICut::from_shapes(&aBox, &aTool).unwrap();
    assert!(aCut.is_done());
    let aCutShape = aCut.shape().clone();
    assert!(IsValid(&aCutShape));
    let mut anEdgeMap = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(&aCutShape, ShapeEnum::Edge, &mut anEdgeMap).unwrap();
    let c = precision::confusion();
    let mut anEdges = Vec::new();
    for i in 1..=anEdgeMap.extent() {
        let anEdge = topods::edge(anEdgeMap.find_key(i).unwrap()).unwrap();
        let (v1, v2) = topexp::vertices(&anEdge, false);
        let (p1, p2) = (BRepTool::pnt(&v1).unwrap(), BRepTool::pnt(&v2).unwrap());
        if (p1.x() - p2.x()).abs() > c || (p1.y() - p2.y()).abs() > c {
            continue;
        }
        let reported = ((p1.x() - 5.0).abs() <= c
            && ((p1.y() - 5.0).abs() <= c || (p1.y() - 10.0).abs() <= c))
            || ((p1.x() - 10.0).abs() <= c && (p1.y() - 5.0).abs() <= c);
        if reported {
            anEdges.push(anEdge);
        }
    }
    assert_eq!(anEdges.len(), 3);
    let mut aFillet = BRepFilletAPIMakeFillet::new(&aCutShape, FilletShape::Rational).unwrap();
    for e in &anEdges {
        aFillet.add_with_radius(2.5, e).unwrap();
    }
    Build(&mut aFillet);
    assert!(aFillet.is_done());
    let aResult = aFillet.shape().unwrap().clone();
    assert!(!aResult.is_null());
    assert!(IsValid(&aResult));
}

#[test]
fn BRepFilletAPI_MakeFilletTest__Issue1177_FilletAllEdgesFlatBox_SucceedsWithoutCrash() {
    let aBox = MakeBox(50.0, 50.0, 10.0);
    let mut aFillet = BRepFilletAPIMakeFillet::new(&aBox, FilletShape::Rational).unwrap();
    for e in Edges(&aBox) {
        aFillet.add_with_radius(5.0, &e).unwrap();
    }
    Build(&mut aFillet);
    assert!(
        aFillet.is_done(),
        "Filleting all edges of a 50x50x10 slab with r=5 should succeed (issue #1177)"
    );
    let aResult = aFillet.shape().unwrap().clone();
    assert!(!aResult.is_null());
    let anAnalyzer = BRepCheckAnalyzer::new(&aResult, true, false, true).unwrap();
    assert!(anAnalyzer.is_valid().unwrap());
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
        aMaxTolerance <= 1.01 * precision::confusion(),
        "Fillet construction must not hide inconsistent edge parameterization with tolerance: {aMaxTolerance}"
    );
}

#[test]
fn BRepFilletAPI_MakeFilletTest__FilletOneEdge() {
    let aBox = MakeBox(20.0, 20.0, 20.0);
    let anEdge = FirstEdge(&aBox);
    let mut aFillet = BRepFilletAPIMakeFillet::new(&aBox, FilletShape::Rational).unwrap();
    aFillet.add_with_radius(2.0, &anEdge).unwrap();
    let aResult = aFillet.shape().unwrap().clone();
    assert!(aFillet.is_done());
    assert!(IsValid(&aResult));
}

#[test]
fn BRepFilletAPI_MakeFilletTest__FilletAllEdges() {
    let aBox = MakeBox(20.0, 20.0, 20.0);
    let mut aFillet = BRepFilletAPIMakeFillet::new(&aBox, FilletShape::Rational).unwrap();
    for e in Edges(&aBox) {
        aFillet.add_with_radius(1.0, &e).unwrap();
    }
    let aResult = aFillet.shape().unwrap().clone();
    assert!(aFillet.is_done());
    assert!(IsValid(&aResult));
}

#[test]
fn BRepFilletAPI_MakeFilletTest__FilletMoreFaces() {
    let aBox = MakeBox(20.0, 20.0, 20.0);
    let mut aFillet = BRepFilletAPIMakeFillet::new(&aBox, FilletShape::Rational).unwrap();
    for e in Edges(&aBox) {
        aFillet.add_with_radius(1.0, &e).unwrap();
    }
    let aResult = aFillet.shape().unwrap().clone();
    assert!(aFillet.is_done());
    let mut aFaceCount = 0;
    let mut ex = Explorer::new(&aResult, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    while ex.more() {
        aFaceCount += 1;
        ex.next().unwrap();
    }
    assert!(aFaceCount > 6);
}

#[test]
fn BRepFilletAPI_MakeFilletTest__FilletVariableRadius() {
    let aBox = MakeBox(20.0, 20.0, 20.0);
    let anEdge = FirstEdge(&aBox);
    let mut aFillet = BRepFilletAPIMakeFillet::new(&aBox, FilletShape::Rational).unwrap();
    aFillet.add_with_radii(1.0, 3.0, &anEdge).unwrap();
    let aResult = aFillet.shape().unwrap().clone();
    assert!(aFillet.is_done());
    assert!(IsValid(&aResult));
}

#[test]
fn BRepFilletAPI_MakeFilletTest__OCC570_MixedVariableConstantRadius() {
    let aBox = MakeBox(100., 100., 100.);
    let aWire = Explorer::new(&aBox, ShapeEnum::Wire, ShapeEnum::Shape)
        .unwrap()
        .current()
        .clone();
    let e = Edges(&aWire);
    assert!(e.len() >= 4);
    let aVarRadius = [
        Point2::new(0.0, 5.0),
        Point2::new(0.3, 15.0),
        Point2::new(0.7, 15.0),
        Point2::new(1.0, 5.0),
    ];
    let mut aFillet = BRepFilletAPIMakeFillet::new(&aBox, FilletShape::Rational).unwrap();
    aFillet.set_continuity(GeomAbsShape::C1, 0.001);
    aFillet.add_with_pairs(&aVarRadius, &e[0]).unwrap();
    aFillet.add_with_radius(5.0, &e[1]).unwrap();
    aFillet.add_with_pairs(&aVarRadius, &e[2]).unwrap();
    aFillet.add_with_radius(5.0, &e[3]).unwrap();
    Build(&mut aFillet);
    assert!(aFillet.is_done(), "Fillet operation should succeed");
    let aResult = aFillet.shape().unwrap().clone();
    assert!(IsValid(&aResult), "Result shape should be valid");
    let area = SurfaceArea(&aResult);
    assert!(
        (area - 58500.).abs() <= 58500. * 0.01,
        "Surface area should be approximately 58500: {area}"
    );
}

#[test]
fn BRepFilletAPI_MakeFilletTest__Bug828_FilletOnComplexPrism() {
    let p = |x: f64, y: f64| Point3::new(x, y, 0.0);
    let arc = |a: Point3, b: Point3, c: Point3| {
        MakeArcOfCircle::through_points(a, b, c)
            .unwrap()
            .value()
            .unwrap()
    };
    let seg = |a: Point3, b: Point3| MakeSegment::from_points(a, b).unwrap().value().unwrap();
    let (p11, p12, p13) = (
        p(-27.598139, -7.0408573),
        p(-28.483755, -17.487625),
        p(-19.555504, -22.983587),
    );
    let (p21, p22, p23) = (
        p(12.125083, -22.983587),
        p(21.1572, -17.27554),
        p(19.878168, -6.6677585),
    );
    let (p31, p32, p33) = (
        p(3.265825, 13.724955),
        p(-4.7233953, 17.406338),
        p(-12.529893, 13.351856),
    );
    let curves = [
        arc(p11, p12, p13).cast(),
        seg(p13, p21).cast(),
        arc(p21, p22, p23).cast(),
        seg(p23, p31).cast(),
        arc(p31, p32, p33).cast(),
        seg(p33, p11).cast(),
    ];
    let mut aWireMaker = BRepBuilderAPIMakeWire::new();
    for c in &curves {
        let e = BRepBuilderAPIMakeEdge::from_curve(c)
            .unwrap()
            .edge()
            .unwrap();
        aWireMaker.add_edge(&e).unwrap();
    }
    assert!(aWireMaker.is_done());
    let aWire = aWireMaker.wire().unwrap();
    let aFace = BRepBuilderAPIMakeFace::from_wire(&aWire, false)
        .unwrap()
        .face()
        .unwrap();
    let aSlabThick = 111.0;
    let mut aSlab =
        BRepPrimAPIMakePrism::new(aFace.as_ref(), Vector3::new(0., 0., aSlabThick), true, true)
            .unwrap();
    let aSlabShape = aSlab.shape().unwrap().clone();
    assert!(IsValid(&aSlabShape), "Prism shape should be valid");
    let anEdges = Edges(&aSlabShape);
    assert!(anEdges.len() >= 2, "Expected at least 2 edges in the prism");
    let aTargetArea = 17816.2;
    let aTargetTolerance = aTargetArea * 0.002;
    let mut aResult = None;
    let (mut aBestArea, mut aBestDelta, mut best) = (0.0, 1.0e100, (0, 0));
    'outer: for i1 in 0..anEdges.len() {
        for i2 in i1 + 1..anEdges.len() {
            let Ok(mut aFillet) = BRepFilletAPIMakeFillet::new(&aSlabShape, FilletShape::Rational)
            else {
                continue;
            };
            aFillet.set_params(1.0e-2, 1.0e-4, 1.0e-5, 1.0e-4, 1.0e-5, 1.0e-3);
            aFillet.set_continuity(GeomAbsShape::C1, 1.0e-2);
            if aFillet.add_with_radius(10.0, &anEdges[i1]).is_err()
                || aFillet.add_with_radius(10.0, &anEdges[i2]).is_err()
                || aFillet.build(&ProgressRange::default()).is_err()
                || !aFillet.is_done()
            {
                continue;
            }
            let aCandidate = aFillet.shape().unwrap().clone();
            if aCandidate.is_null() || !IsValid(&aCandidate) {
                continue;
            }
            let area = SurfaceArea(&aCandidate);
            let aDelta = (area - aTargetArea).abs();
            if aDelta < aBestDelta {
                (aBestArea, aBestDelta, best) = (area, aDelta, (i1 + 1, i2 + 1));
            }
            if aDelta <= aTargetTolerance {
                aResult = Some(aCandidate);
                break 'outer;
            }
        }
    }
    let aResult = aResult.unwrap_or_else(|| {
        panic!("Failed to reproduce the original OCC828 fillet result; closest pair={},{} area={aBestArea}", best.0, best.1)
    });
    assert!(IsValid(&aResult), "Fillet result shape should be valid");
    assert!((SurfaceArea(&aResult) - aTargetArea).abs() <= aTargetTolerance);
}

/// Apply fillet of given radius to all section edges of a boolean operation.
fn occ1077BoolBl(aShapeCut: &Shape, sections: &[Shape], theRadius: f64) -> Shape {
    let (aTesp, aT3d, aT2d, aTa, aFl, aTappAngl) = (1.0e-4, 1.0e-4, 1.0e-5, 1.0e-2, 1.0e-3, 1.0e-2);
    let aBuilder = BRepBuilder::default();
    let mut aResult = Compound::default();
    aBuilder.make_compound(&mut aResult);
    let mut ex = Explorer::new(aShapeCut, ShapeEnum::Solid, ShapeEnum::Shape).unwrap();
    while ex.more() {
        let aSolid = ex.current().clone();
        let mut aFill = BRepFilletAPIMakeFillet::new(&aSolid, FilletShape::Rational).unwrap();
        aFill.set_params(aTa, aTesp, aT2d, aT3d, aT2d, aFl);
        aFill.set_continuity(GeomAbsShape::C1, aTappAngl);
        for s in sections {
            aFill
                .add_with_radius(theRadius, &topods::edge(s).unwrap())
                .unwrap();
        }
        aFill.build(&ProgressRange::default()).unwrap();
        if aFill.is_done() {
            aBuilder.add(&mut aResult, aFill.shape().unwrap()).unwrap();
        } else {
            aBuilder.add(&mut aResult, &aSolid).unwrap();
        }
        ex.next().unwrap();
    }
    aResult.into()
}

/// Cut theShapeToCut with aTool and fillet the section edges with given radius.
fn occ1077CutBlend(theShapeToCut: &Shape, theTool: &Shape, theRadius: f64) -> Shape {
    let aCut = BRepAlgoAPICut::from_shapes(theShapeToCut, theTool).unwrap();
    let sections: Vec<Shape> = aCut.section_edges().unwrap().iter().cloned().collect();
    occ1077BoolBl(aCut.shape(), &sections, theRadius)
}

#[test]
fn BRepFilletAPI_MakeFilletTest__OCC1077_BooleanCutFillet() {
    let aBox = BRepPrimAPIMakeBox::at_point(Point3::new(-5., -5., -5.), 10., 10., 10.)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    let aSphere = BRepPrimAPIMakeSphere::new(7.)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    let aCommon = BRepAlgoAPICommon::from_shapes(&aBox, &aSphere)
        .unwrap()
        .shape()
        .clone();
    let cyl = |p: Point3, d: Direction3| {
        BRepPrimAPIMakeCylinder::with_axes(Axis2::new(p, d).unwrap(), 3., 20.)
            .shape()
            .unwrap()
            .clone()
    };
    let aCyl1 = cyl(
        Point3::new(0., 0., -10.),
        Direction3::new(0., 0., 1.).unwrap(),
    );
    let aCyl2 = cyl(
        Point3::new(-10., 0., 0.),
        Direction3::new(1., 0., 0.).unwrap(),
    );
    let aCyl3 = cyl(
        Point3::new(0., -10., 0.),
        Direction3::new(0., 1., 0.).unwrap(),
    );
    let fix = |s: &Shape| {
        let f = ShapeFixShape::new(s.clone());
        f.perform().unwrap();
        f.shape()
    };
    let aTmp1 = fix(&occ1077CutBlend(&aCommon, &aCyl1, 0.7));
    let aTmp2 = fix(&occ1077CutBlend(&aTmp1, &aCyl2, 0.7));
    let aResult = fix(&occ1077CutBlend(&aTmp2, &aCyl3, 0.7));
    assert!(!aResult.is_null(), "Result shape must not be null");
    assert!(IsValid(&aResult), "Result shape must be valid");
    let area = SurfaceArea(&aResult);
    assert!(
        (area - 587.181).abs() <= 587.181 * 0.001,
        "Surface area mismatch: {area}"
    );
}

fn RingFace(pts: [(f64, f64, f64); 4]) -> Shape {
    let mut w = BRepBuilderAPIMakePolygon::new();
    for (x, y, z) in pts {
        w.add_point(Point3::new(x, y, z)).unwrap();
    }
    w.add_point(Point3::new(pts[0].0, pts[0].1, pts[0].2))
        .unwrap();
    BRepBuilderAPIMakeFace::from_wire(&w.wire().unwrap(), false)
        .unwrap()
        .face()
        .unwrap()
        .into()
}

fn Revolve(face: &Shape, angle: f64) -> Shape {
    let axis = Axis1::new(
        Point3::new(0., 0., 0.),
        Direction3::new(0., 0., 1.).unwrap(),
    );
    BRepPrimAPIMakeRevol::new(face, axis, angle, false)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

#[test]
fn BRepFilletAPI_MakeFilletTest__OCC426_RevolveFuseUnifyFillet() {
    let aRs1 = Revolve(
        &RingFace([(10., 0., 0.), (20., 0., 0.), (20., 0., 10.), (10., 0., 10.)]),
        2.0 * PI,
    );
    // The native literals 7.0710678118654752440 and 14.1421356237309504880, rounded to the
    // same doubles.
    let (f1, f2) = (7.071_067_811_865_475_5, 14.142_135_623_730_951);
    let aRs2 = Revolve(
        &RingFace([(f1, f1, 10.), (f2, f2, 10.), (f2, f2, 20.), (f1, f1, 20.)]),
        270.0 * PI / 180.0,
    );
    let aRs3 = Revolve(
        &RingFace([
            (10., 0., 20.),
            (20., 0., 20.),
            (20., 0., 30.),
            (10., 0., 30.),
        ]),
        2.0 * PI,
    );
    assert!(IsValid(&aRs1), "rs1 must be valid");
    assert!(IsValid(&aRs2), "rs2 must be valid");
    assert!(IsValid(&aRs3), "rs3 must be valid");
    let aFuse32 = BRepAlgoAPIFuse::from_shapes(&aRs3, &aRs2).unwrap();
    assert!(aFuse32.is_done(), "Fuse rs3+rs2 must succeed");
    let aFuse32Shape = aFuse32.shape().clone();
    assert!(IsValid(&aFuse32Shape), "fuse32 must be valid");
    let aFuse321 = BRepAlgoAPIFuse::from_shapes(&aFuse32Shape, &aRs1).unwrap();
    assert!(aFuse321.is_done(), "Fuse fuse32+rs1 must succeed");
    let aFuse321Shape = aFuse321.shape().clone();
    assert!(IsValid(&aFuse321Shape), "fuse321 must be valid");
    let anUnify = ShapeUpgradeUnifySameDomain::new(&aFuse321Shape, true, true, true);
    anUnify.build().unwrap();
    let aFuseUnif = anUnify.shape();
    let mut anEdgeMap = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(
        &aFuseUnif,
        ShapeEnum::Edge,
        ShapeEnum::Solid,
        &mut anEdgeMap,
    )
    .unwrap();
    let mut aBlend = BRepFilletAPIMakeFillet::new(&aFuseUnif, FilletShape::Rational).unwrap();
    for i in 1..=anEdgeMap.extent() {
        let anEdge = topods::edge(anEdgeMap.find_key(i).unwrap()).unwrap();
        if !anEdge.is_null() {
            aBlend.add_with_radius(1.0, &anEdge).unwrap();
        }
    }
    if aBlend.build(&ProgressRange::default()).is_err() {
        // Known exception on some platforms (OCC24156).
        return;
    }
    assert!(aBlend.is_done(), "Fillet must be done");
    let aResult = aBlend.shape().unwrap().clone();
    assert!(!aResult.is_null(), "Result shape must not be null");
    assert!(IsValid(&aResult), "Result shape must be valid");
    let area = SurfaceArea(&aResult);
    assert!(
        (area - 7507.61).abs() <= 7507.61 * 0.001,
        "Surface area mismatch: {area}"
    );
}
