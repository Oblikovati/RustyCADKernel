//! Direct translation of src/ModelingAlgorithms/TKBO/GTests/BOPAlgo_PaveFiller_Test.cxx (3d097a0).
//! The two loft cases wait for BRepOffsetAPI_ThruSections.
#![allow(non_snake_case)]

#[path = "bop_test_utilities/mod.rs"]
mod bop_test_utilities;
use bop_test_utilities::{CreateBox, CreateCone, GetVolume};
use rustycad_tkbo::brep_algo_api::BRepAlgoAPIFuse;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Shell, Solid, Wire};
use rustycad_tkernel::precision;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
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
