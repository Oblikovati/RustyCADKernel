//! Regression for `FillVertexData` on free edges with no offset-edge map entries.
use rustycad_tkbrep::topods::Shape;
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
