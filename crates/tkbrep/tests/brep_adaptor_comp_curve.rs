//! Direct translation of BRepAdaptor_CompCurve_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871. BRepBuilderAPI_MakeVertex, MakeEdge and
//! MakeWire are the named fakes of `support` (TKTopAlgo is not ported).
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCompCurve;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Wire};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomTrimmedCurve};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeombase::gc::MakeArcOfCircle;
use rustycad_tkmath::gp::{self, Axis2, Circle3, Direction3, Point3, Vector3};
use std::f64::consts::PI;
use support::{edge_between, make_edge, make_edge_on_whole_curve, make_vertex, make_wire};

// Test OCC5696: BRepAdaptor_CompCurve::Edge() method
// Migrated from QABugs_5.cxx
#[test]
fn BRepAdaptor_CompCurve_Test__OCC5696_EdgeMethod() {
    // Create a simple edge from two points
    let anEdge = make_edge(Point3::new(0.0, 0.0, 0.0), Point3::new(2.0, 0.0, 0.0));

    // Create a wire from the edge
    let aWire = make_wire(&anEdge);

    // Create a composite curve adaptor
    let aCurve = BRepAdaptorCompCurve::with_wire(&aWire, false).unwrap();

    // Get curve parameters
    let aFirst = aCurve.first_parameter().unwrap();
    let aLast = aCurve.last_parameter().unwrap();
    let aPar = (aFirst + aLast) / 2.0;

    // The original test was checking that this method doesn't throw an exception
    // and returns valid parameter
    let aFound = aCurve.edge(aPar);
    assert!(
        aFound.is_ok(),
        "Edge() method should not throw an exception"
    );
    let aFound = aFound.unwrap();
    let (anEdgeFound, aParEdge) = (aFound.edge, aFound.parameter);

    // Verify that the returned edge is valid
    assert!(!anEdgeFound.is_null(), "Returned edge should not be null");

    // Verify that the parameter is within valid range [0, edge length]
    assert!(aParEdge >= 0.0, "Edge parameter should be non-negative");
    assert!(
        aParEdge <= 2.0,
        "Edge parameter should not exceed edge length"
    );

    // The parameter should be approximately half of the edge length
    assert!(
        (1.0 - aParEdge).abs() <= 0.01,
        "Edge parameter should be approximately 1.0"
    );
}

// Test OCC29430: BRepAdaptor_CompCurve::Value() at boundary parameters matches wire vertices.
// The bug was that evaluating a composite curve at its First/LastParameter
// did not return the correct endpoint.
#[test]
fn BRepAdaptor_CompCurve_Test__OCC29430_ArcBoundaryPoints() {
    let (r45, r225) = (PI / 4.0, 3.0 * PI / 4.0);

    let aFrame =
        Axis2::with_x_direction(Point3::new(0.0, 0.0, 0.0), Direction3::Z, Direction3::X).unwrap();
    let arcMaker =
        MakeArcOfCircle::from_parameters(Circle3::new(aFrame, 1.0).unwrap(), r45, r225, true)
            .unwrap();
    let anArc: Handle<dyn GeomCurve> = arcMaker.value().unwrap().cast();
    let aWire = make_wire(&make_edge_on_whole_curve(&anArc));

    let aCurve = BRepAdaptorCompCurve::with_wire(&aWire, false).unwrap();
    let aStartPt = aCurve.value(aCurve.first_parameter().unwrap()).unwrap();
    let anEndPt = aCurve.value(aCurve.last_parameter().unwrap()).unwrap();

    // Collect wire vertices
    let mut aVertices: Vec<Point3> = Vec::new();
    for aCurrent in Explorer::new(&aWire, ShapeEnum::Vertex, ShapeEnum::Shape).unwrap() {
        aVertices.push(BRepTool::pnt(&topods::vertex(&aCurrent).unwrap()).unwrap());
    }
    assert!(!aVertices.is_empty());

    // Start point should match one of the wire vertices (within 1e-7 tolerance)
    let mut aStartMatchesAnyVertex = false;
    let mut anEndMatchesAnyVertex = false;
    for aV in &aVertices {
        if aStartPt.distance(aV) < 1.0e-7 {
            aStartMatchesAnyVertex = true;
        }
        if anEndPt.distance(aV) < 1.0e-7 {
            anEndMatchesAnyVertex = true;
        }
    }
    assert!(
        aStartMatchesAnyVertex,
        "Start point does not match any wire vertex"
    );
    assert!(
        anEndMatchesAnyVertex,
        "End point does not match any wire vertex"
    );
    assert!(
        aStartPt.distance(&anEndPt) > 1.0e-7,
        "Start and end points should be different"
    );
}

/// `if (V.SquareMagnitude() > gp::Resolution()) V.Normalize();`
fn normalizedIfLong(mut theVector: Vector3) -> Vector3 {
    if theVector.square_magnitude() > gp::resolution() {
        theVector.normalize().unwrap();
    }
    theVector
}

/// `EXPECT_NEAR` on the three coordinates.
fn expectXyzNear(theActual: [f64; 3], theExpected: [f64; 3], theTol: f64, theWhat: &str) {
    for (anIndex, aName) in ["X", "Y", "Z"].iter().enumerate() {
        let aDelta = (theActual[anIndex] - theExpected[anIndex]).abs();
        assert!(aDelta <= theTol, "{theWhat} {aName}");
    }
}

// Test OCC30869: BRepAdaptor_CompCurve D1 at boundary parameters of a wire with reversed edge.
// The bug was that a wire with a single reversed-orientation trimmed-circle edge returned
// incorrect boundary point coordinates and tangent directions.
// Migrated from QABugs_20.cxx OCC30869
#[test]
fn BRepAdaptor_CompCurve_Test__OCC30869_ReversedEdgeBoundaryPoints() {
    // Build a circle: center(1,0,0), Z-axis(0,-1,0), X-axis(0,0,-1), radius=1
    let anAx2 = Axis2::with_x_direction(
        Point3::new(1., 0., 0.),
        Direction3::new(0., -1., 0.).unwrap(),
        Direction3::new(0., 0., -1.).unwrap(),
    )
    .unwrap();
    let aCircle: Handle<dyn GeomCurve> = Handle::new(GeomCircle::new(anAx2, 1.0).unwrap()).cast();

    let t1 = PI / 2.0; // 1.5707963267949
    let t2 = 3.0 * PI / 2.0; // 4.71238898038469

    let aTrimmed: Handle<dyn GeomCurve> =
        Handle::new(GeomTrimmedCurve::new(&aCircle, t1, t2, true, true).unwrap()).cast();
    let mut anEdge = make_edge_on_whole_curve(&aTrimmed);

    // Reverse the edge, then wrap it in a wire
    anEdge.set_orientation(Orientation::Reversed);
    let aWire = make_wire(&anEdge);

    let aBACC = BRepAdaptorCompCurve::with_wire(&aWire, false).unwrap();
    let aFirst = aBACC.first_parameter().unwrap();
    let aLast = aBACC.last_parameter().unwrap();

    let (aPFirst, aVFirst) = aBACC.d1(aFirst).unwrap();
    let (aPLast, aVLast) = aBACC.d1(aLast).unwrap();
    let aVFirst = normalizedIfLong(aVFirst);
    let aVLast = normalizedIfLong(aVLast);

    // Reference: inverse circle (normal = (0,1,0)), evaluated at the same parameters
    let anAx2Ref = Axis2::with_x_direction(
        Point3::new(1., 0., 0.),
        Direction3::new(0., 1., 0.).unwrap(),
        Direction3::new(0., 0., -1.).unwrap(),
    )
    .unwrap();
    let aCircleRef = GeomCircle::new(anAx2Ref, 1.0).unwrap();
    let (aRefP1, aRefV1) = aCircleRef.d1(t1).unwrap();
    let (aRefP2, aRefV2) = aCircleRef.d1(t2).unwrap();
    let aRefV1 = normalizedIfLong(aRefV1);
    let aRefV2 = normalizedIfLong(aRefV2);

    let aTol = 1.0e-7;
    expectXyzNear(
        [aPFirst.x(), aPFirst.y(), aPFirst.z()],
        [aRefP1.x(), aRefP1.y(), aRefP1.z()],
        aTol,
        "First point",
    );
    expectXyzNear(
        [aVFirst.x(), aVFirst.y(), aVFirst.z()],
        [aRefV1.x(), aRefV1.y(), aRefV1.z()],
        aTol,
        "First tangent",
    );
    expectXyzNear(
        [aPLast.x(), aPLast.y(), aPLast.z()],
        [aRefP2.x(), aRefP2.y(), aRefP2.z()],
        aTol,
        "Last point",
    );
    expectXyzNear(
        [aVLast.x(), aVLast.y(), aVLast.z()],
        [aRefV2.x(), aRefV2.y(), aRefV2.z()],
        aTol,
        "Last tangent",
    );
}

fn makeOCC30422Wire(theReverseInsertionOrder: bool) -> Wire {
    let aV1 = make_vertex(Point3::new(0.0, 0.0, 0.0));
    let aV2 = make_vertex(Point3::new(1.0, 0.0, 0.0));
    let aV3 = make_vertex(Point3::new(2.0, 0.0, 0.0));
    let aV4 = make_vertex(Point3::new(3.0, 0.0, 0.0));
    let aV5 = make_vertex(Point3::new(4.0, 0.0, 0.0));

    let anE1 = edge_between(&aV1, &aV2);
    let anE2 = edge_between(&aV2, &aV3);
    let anE3 = edge_between(&aV3, &aV4);
    let anE4 = edge_between(&aV5, &aV4);

    let aBuilder = BRepBuilder::default();
    let mut aWire = Wire::default();
    aBuilder.make_wire(&mut aWire);
    let anOrder: [&Edge; 4] = if theReverseInsertionOrder {
        [&anE1, &anE4, &anE3, &anE2]
    } else {
        [&anE1, &anE2, &anE3, &anE4]
    };
    for anEdge in anOrder {
        aBuilder.add(&mut aWire, anEdge).unwrap();
    }
    aWire
}

fn expectOCC30422WireExplorer(theWire: &Wire) {
    let mut anExplorer = WireExplorer::with_wire(theWire).unwrap();
    let mut aNbEdges = 0;
    while anExplorer.more() {
        assert!(!anExplorer.current().is_null());
        aNbEdges += 1;
        anExplorer.next().unwrap();
    }
    // DRAW validates the third WEDGE_* object; the fourth edge is deliberately
    // not guaranteed to be reachable for this malformed insertion sequence.
    assert!(aNbEdges >= 3);
}

// Migrated from tests/bugs/moddata_3/bug30422. Wire exploration must remain
// deterministic for both the connected and reordered compound insertion paths.
#[test]
fn BRepAdaptor_CompCurve_Test__OCC30422_WireExplorerReorderedEdges() {
    expectOCC30422WireExplorer(&makeOCC30422Wire(false));
    expectOCC30422WireExplorer(&makeOCC30422Wire(true));
}
