//! Direct translation of src/ModelingAlgorithms/TKBO/GTests/BOPAlgo_PaveFiller_Test.cxx (3d097a0).
#![allow(non_snake_case)]

#[path = "bop_test_utilities/mod.rs"]
mod bop_test_utilities;
use bop_test_utilities::{CreateBox, CreateCone, CreateEdge, CreateVertex, GetVolume};
use rustycad_tkbo::brep_algo_api::BRepAlgoAPIFuse;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Shell, Solid, Wire};
use rustycad_tkernel::precision;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Axis2, Circle3, Direction3, Point3};
use rustycad_tkoffset::brep_offset_api::BRepOffsetAPIThruSections;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeShape, BRepBuilderAPIMakeWire,
};
use std::f64::consts::PI;

/// Every sub-shape of `kind` in explorer order.
fn explore(shape: &Shape, kind: ShapeEnum) -> Vec<Shape> {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape).unwrap();
    let mut found = Vec::new();
    while explorer.more() {
        found.push(explorer.current().clone());
        explorer.next().unwrap();
    }
    found
}

/// Create a circular wire at given height.
fn CreateCircularWire(radius: f64, z: f64) -> Wire {
    let axis = Axis2::new(
        Point3::new(0.0, 0.0, z),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let circle = Circle3::new(axis, radius).unwrap();
    let edge = BRepBuilderAPIMakeEdge::from_circ(circle)
        .unwrap()
        .edge()
        .unwrap();
    BRepBuilderAPIMakeWire::from_edge(&edge)
        .unwrap()
        .wire()
        .unwrap()
}

/// The closed wire through the corners, one straight edge per side.
fn quadrilateral_wire(corners: [(f64, f64, f64); 4]) -> Wire {
    let mut maker = BRepBuilderAPIMakeWire::new();
    for (index, &(x, y, z)) in corners.iter().enumerate() {
        let (nx, ny, nz) = corners[(index + 1) % corners.len()];
        let edge = CreateEdge(Point3::new(x, y, z), Point3::new(nx, ny, nz));
        maker.add_edge(&edge).unwrap();
    }
    maker.wire().unwrap()
}

/// The square wire of the second loft (and the top of the first) at the height.
fn square_wire(z: f64) -> Wire {
    quadrilateral_wire([
        (0.0, 0.0, z),
        (100.0, 0.0, z),
        (100.0, -100.0, z),
        (0.0, -100.0, z),
    ])
}

/// A solid ruled loft of the wires; `Build` is called explicitly as in the C++.
fn ruled_loft(wires: &[Wire]) -> BRepOffsetAPIThruSections {
    let mut loft = BRepOffsetAPIThruSections::new(true, true, 1.0e-6);
    for wire in wires {
        loft.add_wire(wire);
    }
    loft
}

fn is_degenerated(edge: &Shape) -> bool {
    BRepTool::degenerated(&topods::edge(edge).unwrap()).unwrap()
}

/// Check if shape has degenerated edges without pcurves.
fn HasDegeneratedEdgesWithoutPCurves(shape: &Shape) -> bool {
    explore(shape, ShapeEnum::Face).iter().any(|face| {
        let face = topods::face(face).unwrap();
        explore(&face, ShapeEnum::Edge).iter().any(|edge| {
            let edge = topods::edge(edge).unwrap();
            BRepTool::degenerated(&edge).unwrap()
                && BRepTool::curve_on_face(&edge, &face)
                    .unwrap()
                    .pcurve
                    .is_null()
        })
    })
}

fn fuse_with_apex_box(shape: &Shape) -> BRepAlgoAPIFuse {
    let box_shape = CreateBox(Point3::new(-5.0, -5.0, 15.0), 10.0, 10.0, 10.0);
    BRepAlgoAPIFuse::from_shapes(shape, &box_shape).unwrap()
}

// Test: Boolean fuse of loft (wire to vertex) with box
// This tests the fix for FillPaves() handling edges without 2D curves.
// A loft from a circular wire to a vertex creates a cone-like shape
// with a degenerated edge at the apex.
#[test]
fn BOPAlgo_PaveFillerTest__FuseConeLoftWithBox_DegeneratedEdge() {
    // Create a loft from a circular wire to a point (apex)
    // This creates a cone-like shape with a degenerated edge at the top
    let base_wire = CreateCircularWire(10.0, 0.0);
    let apex = CreateVertex(Point3::new(0.0, 0.0, 20.0));
    let mut loft_maker = ruled_loft(&[base_wire]);
    loft_maker.add_vertex(&apex).unwrap();
    loft_maker.build(&Default::default()).unwrap();
    assert!(loft_maker.is_done(), "Loft operation should succeed");
    let loft = loft_maker.shape().unwrap().clone();
    assert!(!loft.is_null(), "Loft shape should not be null");

    // Create a box that intersects the loft
    let box_shape = CreateBox(Point3::new(-5.0, -5.0, 5.0), 10.0, 10.0, 10.0);

    // Perform Boolean fuse - this should not crash even if there are
    // degenerated edges without pcurves
    let fuser = BRepAlgoAPIFuse::from_shapes(&loft, &box_shape).unwrap();

    // The operation should complete without crashing
    assert!(
        fuser.is_done(),
        "Boolean fuse should succeed for loft with degenerated edges"
    );
    assert!(!fuser.shape().is_null(), "Result shape should not be null");

    // Verify the result has reasonable volume (both shapes contribute)
    let loft_volume = GetVolume(&loft);
    let box_volume = GetVolume(&box_shape);
    let result_volume = GetVolume(fuser.shape());

    // Result volume should be less than sum (intersection) but more than max (union effect)
    assert!(result_volume > 0.0, "Result should have positive volume");
    assert!(
        result_volume < loft_volume + box_volume,
        "Result volume should be less than sum of inputs"
    );
}

// Test: Boolean fuse of cone (which has degenerated edge at apex) with box
// Standard cone also has a degenerated edge at the apex
#[test]
fn BOPAlgo_PaveFillerTest__FuseConeWithBox_DegeneratedEdge() {
    let cone = CreateCone(10.0, 0.0, 20.0);
    assert!(!cone.is_null(), "Cone should be created");
    let fuser = fuse_with_apex_box(&cone);
    assert!(
        fuser.is_done(),
        "Boolean fuse of cone and box should succeed"
    );
    assert!(!fuser.shape().is_null(), "Result shape should not be null");
    assert!(
        GetVolume(fuser.shape()) > 0.0,
        "Result should have positive volume"
    );
}

// Test: Two lofts fused together (original OCC10006 scenario)
// This is similar to the existing test but focuses on the robustness aspect
#[test]
fn BOPAlgo_PaveFillerTest__FuseTwoLofts_RobustnessCheck() {
    // Create first loft: quadrilateral wire to quadrilateral wire
    let bottom1 = quadrilateral_wire([
        (10.0, -10.0, 0.0),
        (100.0, -10.0, 0.0),
        (100.0, -100.0, 0.0),
        (10.0, -100.0, 0.0),
    ]);
    let mut loft1 = ruled_loft(&[bottom1, square_wire(10.0)]);
    loft1.build(&Default::default()).unwrap();
    assert!(loft1.is_done(), "First loft should succeed");

    // Create second loft
    let mut loft2 = ruled_loft(&[square_wire(10.0), square_wire(250.0)]);
    loft2.build(&Default::default()).unwrap();
    assert!(loft2.is_done(), "Second loft should succeed");

    // Fuse the two lofts - this is the operation that could crash
    let first = loft1.shape().unwrap().clone();
    let second = loft2.shape().unwrap().clone();
    let fuser = BRepAlgoAPIFuse::from_shapes(&first, &second).unwrap();

    assert!(fuser.is_done(), "Boolean fuse of two lofts should succeed");
    assert!(!fuser.shape().is_null(), "Result shape should not be null");
    assert!(
        GetVolume(fuser.shape()) > 0.0,
        "Result should have positive volume"
    );
}

/// The conical face and its degenerated apex edge, or None when the cone has none.
fn find_degenerated(cone: &Shape) -> Option<(Face, Edge)> {
    explore(cone, ShapeEnum::Face).iter().find_map(|face| {
        let edge = explore(face, ShapeEnum::Edge)
            .into_iter()
            .find(is_degenerated)?;
        Some((topods::face(face).unwrap(), topods::edge(&edge).unwrap()))
    })
}

/// A degenerated edge on the apex vertex with a [0, 2*PI] range and, on purpose, no pcurve.
fn degenerated_edge_without_pcurve(builder: &BRepBuilder, degenerated: &Edge) -> Edge {
    let vertex = explore(degenerated, ShapeEnum::Vertex).remove(0);
    let mut edge = Edge::default();
    builder.make_edge(&mut edge).unwrap();
    builder
        .add(&mut edge, &vertex.oriented(Orientation::Forward))
        .unwrap();
    builder
        .add(&mut edge, &vertex.oriented(Orientation::Reversed))
        .unwrap();
    builder.degenerated(&edge, true).unwrap();
    builder.range(&edge, 0.0, 2.0 * PI, false).unwrap();
    edge
}

/// The first wire of `face` rebuilt with `new_edge` in place of its degenerated edges.
fn rebuild_first_wire(builder: &BRepBuilder, face: &Face, new_edge: &Edge) -> Wire {
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    let Some(first) = explore(face, ShapeEnum::Wire).into_iter().next() else {
        return wire;
    };
    for edge in explore(&first, ShapeEnum::Edge) {
        let component = if is_degenerated(&edge) {
            new_edge.oriented(edge.orientation())
        } else {
            edge
        };
        builder.add(&mut wire, &component).unwrap();
    }
    wire
}

/// The cone solid with the conical face replaced by one without the apex pcurve.
fn cone_without_apex_pcurve(cone: &Shape, conical: &Face, degenerated: &Edge) -> Solid {
    let builder = BRepBuilder::default();
    let new_edge = degenerated_edge_without_pcurve(&builder, degenerated);
    let wire = rebuild_first_wire(&builder, conical, &new_edge);
    let mut face = Face::default();
    builder
        .make_face_on_surface(
            &mut face,
            &BRepTool::surface(conical).unwrap(),
            precision::confusion(),
        )
        .unwrap();
    builder.add(&mut face, &wire).unwrap();
    let mut shell = Shell::default();
    builder.make_shell(&mut shell);
    for original in explore(cone, ShapeEnum::Face) {
        let component = if original.is_same(conical) {
            face.clone().into_shape()
        } else {
            original
        };
        builder.add(&mut shell, &component).unwrap();
    }
    let mut solid = Solid::default();
    builder.make_solid(&mut solid);
    builder.add(&mut solid, &shell).unwrap();
    solid
}

// Test: Create a cone with manually removed pcurve on degenerated edge
// This directly tests the null pcurve handling in FillPaves() and CorrectWires()
#[test]
fn BOPAlgo_PaveFillerTest__FuseConeWithRemovedPCurve_NullPCurveHandling() {
    let cone = CreateCone(10.0, 0.0, 20.0);
    assert!(!cone.is_null(), "Cone should be created");
    let Some((conical, degenerated)) = find_degenerated(&cone) else {
        // Fallback: just run with regular cone if we couldn't find degenerated edge
        assert!(fuse_with_apex_box(&cone).is_done());
        return;
    };
    let solid = cone_without_apex_pcurve(&cone, &conical, &degenerated);
    assert!(
        HasDegeneratedEdgesWithoutPCurves(&solid),
        "Modified cone should have degenerated edge without pcurve"
    );
    // We expect either success or graceful failure, but NOT a crash.
    let fuser = fuse_with_apex_box(&solid);
    let _is_done = fuser.is_done();
}
