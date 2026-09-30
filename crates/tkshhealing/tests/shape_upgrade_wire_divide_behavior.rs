//! Wire division preserves native topology, context sharing and parameter ranges.
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    brep_tools::ReShape,
    topexp::{self, Explorer},
    topods::{self, Edge, Face, Vertex},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Point2, Point3, dx2d};
use rustycad_tkshhealing::{
    shape_build::ShapeBuildReShape,
    shape_extend::ShapeExtendStatus,
    shape_upgrade::{ShapeUpgradeWireDivide, WireDivider},
};
#[path = "divide_support/criteria.rs"]
mod criteria;
#[path = "divide_support/geometry.rs"]
mod geometry;
fn aligned_edge() -> (Edge, Face) {
    let (mut edge, face) = geometry::edge_with_parameterizations();
    let pcurve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::from_point(Point2::default(), dx2d())).cast();
    let builder = BRepBuilder::default();
    builder
        .update_edge_pcurve_on_face(&edge, &pcurve, &face, 1e-7)
        .unwrap();
    builder.range_on_face(&edge, &face, 2., 5.).unwrap();
    add_vertex(&mut edge, 2., Orientation::Forward);
    add_vertex(&mut edge, 5., Orientation::Reversed);
    (edge, face)
}
fn add_vertex(edge: &mut Edge, parameter: f64, orientation: Orientation) -> Vertex {
    let builder = BRepBuilder::default();
    let mut vertex = Vertex::default();
    builder
        .make_vertex_at(&mut vertex, Point3::new(parameter, 0., 0.), 1e-7)
        .unwrap();
    vertex.set_orientation(orientation);
    builder.add(edge, &vertex).unwrap();
    builder
        .update_vertex_on_edge(&vertex, parameter, edge, 1e-7)
        .unwrap();
    vertex
}
fn divider(edge: &Edge, face: &Face) -> ShapeUpgradeWireDivide {
    let tool = ShapeUpgradeWireDivide::new();
    tool.set_context(&Handle::new(ShapeBuildReShape::new()));
    tool.load_edge(edge).unwrap();
    tool.set_face(face);
    tool.set_split_curve_3d_tool(&Handle::new(criteria::Midpoint3d::default()).cast());
    tool.set_split_curve_2d_tool(&Handle::new(criteria::Midpoint2d::default()).cast());
    tool
}
fn output_edges(tool: &ShapeUpgradeWireDivide) -> Vec<Edge> {
    Explorer::new(&tool.wire(), ShapeEnum::Edge, ShapeEnum::Shape)
        .unwrap()
        .map(|shape| topods::edge(&shape).unwrap())
        .collect()
}
#[test]
fn default_and_no_op_preserve_wire_identity() {
    let tool = ShapeUpgradeWireDivide::new();
    assert!(tool.wire().is_null());
    tool.perform().unwrap();
    assert!(tool.status_flag(ShapeExtendStatus::Ok));
    let (edge, face) = aligned_edge();
    tool.set_context(&Handle::new(ShapeBuildReShape::new()));
    tool.load_edge(&edge).unwrap();
    tool.set_face(&face);
    let original = tool.wire();
    tool.perform().unwrap();
    assert!(tool.wire().is_same(&original));
    assert!(tool.status_flag(ShapeExtendStatus::Ok));
}
#[test]
fn splits_aligned_curves_and_shares_intermediate_vertex() {
    let (edge, face) = aligned_edge();
    let tool = divider(&edge, &face);
    tool.perform().unwrap();
    assert!(tool.status_flag(ShapeExtendStatus::Done1));
    let pieces = output_edges(&tool);
    assert_eq!(pieces.len(), 2);
    assert_eq!(BRepTool::range(&pieces[0]).unwrap(), (2., 3.5));
    assert_eq!(BRepTool::range(&pieces[1]).unwrap(), (3.5, 5.));
    assert!(
        topexp::last_vertex(&pieces[0], false).is_same(&topexp::first_vertex(&pieces[1], false))
    );
    assert_eq!(BRepTool::range(&edge).unwrap(), (2., 5.));
    assert_eq!(
        BRepTool::range_on_face(&pieces[0], &face).unwrap(),
        (2., 3.5)
    );
}
#[test]
fn reversals_preserve_edge_orientation_and_original_endpoints() {
    let (edge, face) = aligned_edge();
    let reversed = topods::edge(&edge.reversed()).unwrap();
    let tool = divider(&reversed, &face);
    tool.perform().unwrap();
    let pieces = output_edges(&tool);
    assert_eq!(pieces.len(), 2);
    assert!(
        pieces
            .iter()
            .all(|piece| piece.orientation() == Orientation::Reversed)
    );
    assert_eq!(
        BRepTool::pnt(&topexp::first_vertex(&edge, false)).unwrap(),
        Point3::new(2., 0., 0.)
    );
}
#[test]
fn inactive_edge_mode_retains_a_previously_assigned_spatial_tool() {
    let (edge, face) = aligned_edge();
    let tool = divider(&edge, &face);
    tool.set_split_curve_2d_tool(&Handle::new(
        rustycad_tkshhealing::shape_upgrade::ShapeUpgradeSplitCurve2d::new(),
    ));
    tool.set_edge_mode(0);
    tool.perform().unwrap();
    assert_eq!(output_edges(&tool).len(), 1);
    tool.set_edge_mode(2);
    tool.perform().unwrap();
    assert_eq!(output_edges(&tool).len(), 2);
    tool.set_edge_mode(0);
    tool.perform().unwrap();
    assert_eq!(output_edges(&tool).len(), 4);
}
#[test]
fn small_pieces_are_removed_only_with_a_supporting_face() {
    let (edge, face) = aligned_edge();
    let tool = divider(&edge, &face);
    tool.set_min_tolerance(100.);
    tool.perform().unwrap();
    assert!(output_edges(&tool).is_empty());
    assert!(tool.status_flag(ShapeExtendStatus::Done1));
    assert!(
        tool.context()
            .apply(&edge, ShapeEnum::Shape)
            .unwrap()
            .is_null()
    );
    let free = divider(&edge, &Face::default());
    free.set_min_tolerance(100.);
    free.perform().unwrap();
    assert_eq!(output_edges(&free).len(), 2);
}
#[test]
fn internal_vertices_are_projected_and_replaced_in_context() {
    let (mut edge, face) = aligned_edge();
    let vertex = add_vertex(&mut edge, 2.75, Orientation::Internal);
    let tool = divider(&edge, &face);
    tool.perform().unwrap();
    let pieces = output_edges(&tool);
    let replacement = tool.context().apply(&vertex, ShapeEnum::Shape).unwrap();
    assert!(!replacement.is_same(&vertex));
    let found: Vec<_> = Explorer::new(&pieces[0], ShapeEnum::Vertex, ShapeEnum::Shape)
        .unwrap()
        .collect();
    assert!(found.iter().any(|item| item.is_same(&replacement)));
    assert_eq!(BRepTool::pnt(&vertex).unwrap(), Point3::new(2.75, 0., 0.));
}
#[test]
fn replacing_tools_preserves_identity_and_done_status() {
    use rustycad_tkshhealing::{
        shape_analysis::ShapeAnalysisTransferParameters,
        shape_upgrade::{ShapeUpgradeEdgeDivide, ShapeUpgradeFixSmallCurves},
    };
    let (edge, face) = aligned_edge();
    let tool = divider(&edge, &face);
    tool.perform().unwrap();
    let transfer = Handle::new(ShapeAnalysisTransferParameters::new());
    let criterion = Handle::new(ShapeUpgradeEdgeDivide::new());
    let small = Handle::new(ShapeUpgradeFixSmallCurves::new());
    tool.set_transfer_param_tool(&transfer);
    tool.set_edge_divide_tool(&criterion);
    tool.set_fix_small_curve_tool(&small);
    assert!(tool.transfer_param_tool() == transfer);
    assert!(tool.edge_divide_tool() == criterion);
    assert!(tool.fix_small_curve_tool() == small);
    assert!(tool.status_flag(ShapeExtendStatus::Done1));
    tool.load_edge(&edge).unwrap();
    assert!(tool.status_flag(ShapeExtendStatus::Done1));
    tool.init(&tool.wire(), &face);
    assert!(tool.status_flag(ShapeExtendStatus::Ok));
}
#[test]
fn surface_overloads_preserve_geometry_and_location() {
    use rustycad_tkmath::toploc::Location;
    let (edge, face) = aligned_edge();
    let tool = divider(&edge, &face);
    let surface = BRepTool::surface(&face).unwrap();
    let wire = tool.wire();
    tool.init_surface(&wire, &surface).unwrap();
    assert!(tool.wire().is_same(&wire));
    tool.set_surface(&surface).unwrap();
    tool.set_surface_location(&surface, &Location::default())
        .unwrap();
    tool.wire_divide_data().with(|fields| {
        assert!(BRepTool::surface(&fields.face).unwrap() == surface);
        assert_eq!(
            BRepTool::surface_with_location(&fields.face).unwrap().1,
            Location::default()
        );
    });
    tool.load_wire(&wire);
    assert!(tool.wire().is_same(&wire));
}
#[test]
fn null_context_reports_typed_divergence_without_mutating_loaded_wire() {
    let (edge, face) = aligned_edge();
    let tool = ShapeUpgradeWireDivide::new();
    tool.load_edge(&edge).unwrap();
    tool.set_face(&face);
    let wire = tool.wire();
    // Pinned OCCT dereferences myContext here; this is an explicit divergence, not parity.
    let failure = tool.perform().unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_NullObject");
    assert!(tool.wire().is_same(&wire));
    assert!(tool.status_flag(ShapeExtendStatus::Ok));
}
#[test]
fn pcurve_only_internal_vertex_preserves_crash_divergence_and_partial_status() {
    let (mut edge, face) = aligned_edge();
    rustycad_tkshhealing::shape_build::ShapeBuildEdge
        .remove_curve_3d(&edge)
        .unwrap();
    let builder = BRepBuilder::default();
    let mut vertex = Vertex::default();
    builder
        .make_vertex_at(&mut vertex, Point3::new(2.75, 0., 0.), 1e-7)
        .unwrap();
    vertex.set_orientation(Orientation::Internal);
    builder.add(&mut edge, &vertex).unwrap();
    builder
        .update_vertex_on_edge_face(&vertex, 2.75, &edge, &face, 1e-7)
        .unwrap();
    let tool = divider(&edge, &face);
    let wire = tool.wire();
    // Native CopyNMVertex dereferences the absent spatial curve at OCCT 3d097a0.
    assert_eq!(
        tool.perform().unwrap_err().exception_type(),
        "Standard_NullObject"
    );
    assert!(tool.status_flag(ShapeExtendStatus::Done1));
    assert!(tool.wire().is_same(&wire));
}
fn uneven_bezier_edge() -> (Edge, Face) {
    use rustycad_tkernel::ncollection::Array1;
    use rustycad_tkg3d::geom::GeomBezierCurve;
    let (edge, face) = aligned_edge();
    let points = [
        Point3::default(),
        Point3::default(),
        Point3::default(),
        Point3::default(),
        Point3::new(100., 0., 0.),
    ];
    let curve = Handle::new(GeomBezierCurve::new(&Array1::from_shared(&points, 1)).unwrap()).cast();
    let builder = BRepBuilder::default();
    builder.update_edge_curve(&edge, &curve, 1e-7).unwrap();
    builder
        .update_edge_pcurve_on_face(&edge, &Handle::default(), &face, 1e-7)
        .unwrap();
    builder.range(&edge, 0., 1., false).unwrap();
    let first = topexp::first_vertex(&edge, false);
    let last = topexp::last_vertex(&edge, false);
    builder
        .update_vertex_point(&first, Point3::default(), 1e-7)
        .unwrap();
    builder
        .update_vertex_point(&last, Point3::new(100., 0., 0.), 1e-7)
        .unwrap();
    builder
        .update_vertex_on_edge(&first, 0., &edge, 1e-7)
        .unwrap();
    builder
        .update_vertex_on_edge(&last, 1., &edge, 1e-7)
        .unwrap();
    (edge, face)
}
#[test]
fn small_prefix_moves_first_vertex_to_midpoint_before_retaining_long_piece() {
    let (edge, face) = uneven_bezier_edge();
    let tool = divider(&edge, &face);
    tool.set_min_tolerance(10.);
    tool.perform().unwrap();
    let pieces = output_edges(&tool);
    assert_eq!(pieces.len(), 1);
    // Geom_BezierCurve::Segment normalizes the retained piece to [0,1].
    assert_eq!(BRepTool::range(&pieces[0]).unwrap(), (0., 1.));
    let first = topexp::first_vertex(&pieces[0], false);
    assert_eq!(BRepTool::pnt(&first).unwrap(), Point3::new(3.125, 0., 0.));
    assert_eq!(
        BRepTool::pnt(&topexp::first_vertex(&edge, false)).unwrap(),
        Point3::default()
    );
}
