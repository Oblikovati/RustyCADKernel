//! Geometric projection and native vertex/range mutation over real curves.
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool, access::vertex_tshape},
    topods::{self, Edge, Face, Vertex},
};
use rustycad_tkernel::{
    ncollection::{Array1, HSequence},
    precision,
    standard::Handle,
};
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkshhealing::{
    shape_analysis::{ShapeAnalysisTransferParametersProj, TransferParameters},
    shape_build::ShapeBuildEdge,
};
#[path = "divide_support/geometry.rs"]
mod geometry;
fn nonlinear_edge() -> (Edge, Face) {
    let (edge, face) = geometry::edge_with_parameterizations();
    let points = [
        Point2::new(2., 0.),
        Point2::new(2., 0.),
        Point2::new(2., 0.),
        Point2::new(5., 0.),
    ];
    let poles = Array1::from_shared(&points, 1);
    let curve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dBezierCurve::new(&poles).unwrap()).cast();
    BRepBuilder::default()
        .update_edge_pcurve_on_face(&edge, &curve, &face, 1e-7)
        .unwrap();
    BRepBuilder::default()
        .range_on_face(&edge, &face, 0., 1.)
        .unwrap();
    (edge, face)
}
fn parameter_sequence(values: &[f64]) -> Handle<HSequence<f64>> {
    let sequence = Handle::new(HSequence::<f64>::default());
    for &value in values {
        sequence.change_sequence(|s| {
            s.append(value);
        });
    }
    sequence
}
#[test]
fn default_projection_preserves_identity_and_native_tolerance() {
    let tool = ShapeAnalysisTransferParametersProj::new();
    assert!(!tool.force_projection());
    assert!(tool.is_same_range());
    assert_eq!(tool.transfer_parameter_data().with(|f| f.max_tolerance), 1.);
    assert_eq!(tool.perform_parameter(8., true).unwrap(), 8.);
    tool.set_force_projection(true);
    assert!(tool.force_projection());
    assert!(tool.is_same_range());
}
#[test]
fn force_and_tolerance_select_geometric_projection() {
    let (edge, face) = nonlinear_edge();
    let tool = ShapeAnalysisTransferParametersProj::from_edge(&edge, &face).unwrap();
    assert_eq!(
        tool.perform_parameter(3.5, true).unwrap(),
        -2. * (1. / 3.) + 3.5 * (1. / 3.)
    );
    tool.set_force_projection(true);
    assert!(!tool.is_same_range());
    assert!((tool.perform_parameter(3.5, true).unwrap() - 0.5_f64.cbrt()).abs() < 1e-7);
    assert!((tool.perform_parameter(0.5, false).unwrap() - 2.375).abs() < 1e-7);
    tool.set_force_projection(false);
    tool.set_max_tolerance(0.);
    assert!((tool.perform_parameter(0.5, false).unwrap() - 2.375).abs() < 1e-7);
}
#[test]
fn projected_sequence_clamps_bounds_and_keeps_input_owned() {
    let (edge, face) = nonlinear_edge();
    let tool = ShapeAnalysisTransferParametersProj::from_edge(&edge, &face).unwrap();
    tool.set_force_projection(true);
    let parameters = parameter_sequence(&[2., 2.375, 3.5, 5.]);
    let result = tool.perform(&parameters, true).unwrap();
    assert!(result != parameters);
    let values = result.sequence(|s| s.iter().copied().collect::<Vec<_>>());
    assert_eq!((values[0], values[3]), (0., 1.));
    assert!((values[1] - 0.5).abs() < 1e-7);
    assert_eq!(tool.perform_parameter(-3., true).unwrap(), 0.);
    assert_eq!(tool.perform_parameter(20., true).unwrap(), 1.);
}
#[test]
fn initialization_without_face_disables_projection_but_retains_force() {
    let (edge, face) = nonlinear_edge();
    let tool = ShapeAnalysisTransferParametersProj::from_edge(&edge, &face).unwrap();
    tool.set_force_projection(true);
    tool.init(&edge, &Face::default()).unwrap();
    assert!(tool.force_projection());
    assert!(tool.is_same_range());
    assert_eq!(tool.perform_parameter(3.5, true).unwrap(), 3.5);
    let mut bare = Edge::default();
    BRepBuilder::default().make_edge(&mut bare).unwrap();
    tool.init(&bare, &face).unwrap();
    assert_eq!(
        tool.transfer_parameter_data().with(|f| (f.first, f.last)),
        (0., 1.)
    );
}
#[test]
fn projected_range_copies_geometry_and_expands_collapsed_spatial_interval() {
    let (edge, face) = nonlinear_edge();
    let tool = ShapeAnalysisTransferParametersProj::from_edge(&edge, &face).unwrap();
    tool.set_force_projection(true);
    let copy = ShapeBuildEdge.copy(&edge, true).unwrap();
    tool.transfer_range(&copy, 3.5, 3.5, false).unwrap();
    let tolerance = precision::p_confusion();
    assert_eq!(
        BRepTool::range(&copy).unwrap(),
        (3.5 - tolerance, 3.5 + tolerance)
    );
    let (first, last) = BRepTool::range_on_face(&copy, &face).unwrap();
    assert!(last > first);
    assert!(!BRepTool::same_range(&copy).unwrap());
    assert_eq!(BRepTool::range(&edge).unwrap(), (2., 5.));
}
fn represented_vertex(edge: &Edge, face: &Face) -> Vertex {
    let mut vertex = Vertex::default();
    let builder = BRepBuilder::default();
    builder
        .make_vertex_at(&mut vertex, Point3::new(3.5, 0., 0.), 1e-7)
        .unwrap();
    vertex.set_orientation(Orientation::Internal);
    builder
        .update_vertex_on_edge(&vertex, 3.5, edge, 1e-7)
        .unwrap();
    builder
        .update_vertex_on_face(&vertex, 3.5, 0., face, 1e-7)
        .unwrap();
    vertex
}
#[test]
fn copied_nonmanifold_vertex_keeps_source_and_allocates_new_representations() {
    let (edge, face) = geometry::edge_with_parameterizations();
    let vertex = represented_vertex(&edge, &face);
    let target = ShapeBuildEdge.copy(&edge, true).unwrap();
    let copy =
        ShapeAnalysisTransferParametersProj::copy_nm_vertex_edge(&vertex, &target, &edge).unwrap();
    assert!(!copy.is_same(&vertex));
    assert_eq!(copy.orientation(), Orientation::Internal);
    assert_eq!(
        BRepTool::pnt(&copy).unwrap(),
        BRepTool::pnt(&vertex).unwrap()
    );
    assert_eq!(
        BRepTool::vertex_tolerance(&copy).unwrap(),
        4.750000000000001
    );
    assert_eq!(BRepTool::vertex_tolerance(&vertex).unwrap(), 1e-7);
    let old = vertex_tshape(&vertex, "test").unwrap().points();
    let new = vertex_tshape(&copy, "test").unwrap().points();
    assert_eq!(old.len(), new.len());
    assert!(new.iter().all(|p| old.iter().all(|q| p != q)));
}
#[test]
fn face_vertex_copy_retains_uv_and_ignores_manifold_vertices() {
    let (edge, face) = geometry::edge_with_parameterizations();
    let vertex = represented_vertex(&edge, &face);
    let copy =
        ShapeAnalysisTransferParametersProj::copy_nm_vertex_face(&vertex, &face, &face).unwrap();
    assert!(!copy.is_same(&vertex));
    assert_eq!(BRepTool::vertex_tolerance(&copy).unwrap(), 1e-7);
    let forward = topods::vertex(&vertex.oriented(Orientation::Forward)).unwrap();
    assert!(
        ShapeAnalysisTransferParametersProj::copy_nm_vertex_face(&forward, &face, &face)
            .unwrap()
            .is_null()
    );
    assert!(
        ShapeAnalysisTransferParametersProj::copy_nm_vertex_edge(&forward, &edge, &edge)
            .unwrap()
            .is_null()
    );
}
#[test]
fn vertex_projection_to_another_face_increases_tolerance_by_native_margin() {
    use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
    let (edge, face) = geometry::edge_with_parameterizations();
    let vertex = represented_vertex(&edge, &face);
    let plane: Handle<dyn GeomSurface> = Handle::new(
        GeomPlane::from_point(Point3::new(0., 0., 1.), rustycad_tkmath::gp::dz()).unwrap(),
    )
    .cast();
    let mut target = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut target, &plane, 1e-7)
        .unwrap();
    let copy =
        ShapeAnalysisTransferParametersProj::copy_nm_vertex_face(&vertex, &target, &face).unwrap();
    assert_eq!(
        BRepTool::vertex_tolerance(&copy).unwrap(),
        1. + 0.1 * precision::confusion()
    );
    let uv = BRepTool::parameters(&copy, &target).unwrap();
    assert_eq!(
        plane.value(uv.x(), uv.y()).unwrap(),
        Point3::new(3.5, 0., 1.)
    );
    assert_eq!(BRepTool::vertex_tolerance(&vertex).unwrap(), 1e-7);
}
#[test]
fn copying_vertex_to_changed_range_projects_its_spatial_parameter() {
    let (edge, face) = geometry::edge_with_parameterizations();
    let vertex = represented_vertex(&edge, &face);
    let target = ShapeBuildEdge.copy(&edge, true).unwrap();
    BRepBuilder::default().range(&target, 2., 6., true).unwrap();
    let copy =
        ShapeAnalysisTransferParametersProj::copy_nm_vertex_edge(&vertex, &target, &edge).unwrap();
    let points = vertex_tshape(&copy, "test").unwrap().points();
    let stored = points
        .iter()
        .find(|p| p.is_point_on_curve())
        .unwrap()
        .parameter();
    assert_eq!(stored, 3.5);
    // OCCT 3d097a0 BRep_Tool::Parameter treats an interval shorter than vertex tolerance as closed.
    assert_eq!(BRepTool::parameter(&copy, &target).unwrap(), 6.);
    assert_eq!(BRepTool::range(&edge).unwrap(), (2., 5.));
}
#[test]
fn infinite_endpoint_keeps_copied_ranges_and_clears_same_range() {
    let (edge, face) = nonlinear_edge();
    let tool = ShapeAnalysisTransferParametersProj::from_edge(&edge, &face).unwrap();
    tool.set_force_projection(true);
    let copy = ShapeBuildEdge.copy(&edge, true).unwrap();
    tool.transfer_range(&copy, 1e101, 2e101, false).unwrap();
    assert_eq!(BRepTool::range(&copy).unwrap(), (2., 5.));
    assert_eq!(BRepTool::range_on_face(&copy, &face).unwrap(), (0., 1.));
    assert!(!BRepTool::same_range(&copy).unwrap());
}
#[test]
fn pcurve_only_target_preserves_explicit_native_crash_divergence() {
    let (spatial, face) = geometry::edge_with_parameterizations();
    let planar = ShapeBuildEdge.copy(&spatial, true).unwrap();
    ShapeBuildEdge.remove_curve_3d(&planar).unwrap();
    for source in [&spatial, &planar] {
        let vertex = pcurve_vertex(source, &face);
        let failure =
            ShapeAnalysisTransferParametersProj::copy_nm_vertex_edge(&vertex, &planar, source)
                .unwrap_err();
        assert_eq!(failure.exception_type(), "Standard_NullObject");
    }
}

fn pcurve_vertex(edge: &Edge, face: &Face) -> Vertex {
    let builder = BRepBuilder::default();
    let mut vertex = Vertex::default();
    builder
        .make_vertex_at(&mut vertex, Point3::new(3.5, 0., 0.), 1e-7)
        .unwrap();
    vertex.set_orientation(Orientation::Internal);
    builder
        .update_vertex_on_edge_face(&vertex, 3.5, edge, face, 1e-7)
        .unwrap();
    vertex
}
