//! Regression for `FillVertexData` on free edges with no offset-edge map entries.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Shape, Vertex};
use rustycad_tkmath::gp::Point3;
use rustycad_tkoffset::brep_offset::{
    OffsetEdgeGeometry, OffsetVertexIncidentEdge, build_simple_offset_vertex,
    build_simple_offset_vertices,
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

#[test]
fn free_edge_vertices_keep_native_empty_sample_results() {
    let edge =
        BRepBuilderAPIMakeEdge::from_points(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0))
            .unwrap()
            .edge()
            .unwrap();
    let results = build_simple_offset_vertices(&Shape::from(edge), &[]).unwrap();

    assert_eq!(results.len(), 2);
    for result in results {
        assert!(result.geometry.point.x().is_nan());
        assert!(result.geometry.point.y().is_nan());
        assert!(result.geometry.point.z().is_nan());
        assert_eq!(result.geometry.tolerance, 0.0);
    }
}

#[test]
fn bound_null_offset_curve_is_not_treated_as_an_empty_map_entry() {
    let maker =
        BRepBuilderAPIMakeEdge::from_points(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0))
            .unwrap();
    let edge = maker.edge().unwrap();
    let vertex = maker.vertex1().unwrap().clone();
    let incident = OffsetVertexIncidentEdge {
        edge,
        geometry: Some(OffsetEdgeGeometry::default()),
    };

    let failure = match build_simple_offset_vertex(&vertex, &[incident]) {
        Err(failure) => failure,
        Ok(_) => panic!("bound null offset curve should be rejected"),
    };
    assert!(failure.message().contains("null offset curve"));
}

#[test]
fn null_source_curve_skips_before_bound_offset_curve_access() {
    let (vertex, edge) = degenerated_edge_fixture();
    assert!(BRepTool::curve(&edge).unwrap().0.is_null());
    let incident = OffsetVertexIncidentEdge {
        edge,
        geometry: Some(OffsetEdgeGeometry::default()),
    };
    let geometry = build_simple_offset_vertex(&vertex, &[incident])
        .unwrap()
        .unwrap();

    assert!(geometry.point.x().is_nan());
    assert!(geometry.point.y().is_nan());
    assert!(geometry.point.z().is_nan());
    assert_eq!(geometry.tolerance, 0.0);
}

fn degenerated_edge_fixture() -> (Vertex, Edge) {
    let builder = BRepBuilder::default();
    let mut vertex = Vertex::default();
    builder
        .make_vertex_at(&mut vertex, Point3::new(3.0, 4.0, 5.0), 1.0e-7)
        .unwrap();
    let mut edge = Edge::default();
    builder.make_edge(&mut edge).unwrap();
    builder.degenerated(&edge, true).unwrap();
    builder
        .add(&mut edge, &Shape::from(vertex.clone()))
        .unwrap();
    builder
        .add(&mut edge, &Shape::from(vertex.clone()))
        .unwrap();
    (vertex, edge)
}
