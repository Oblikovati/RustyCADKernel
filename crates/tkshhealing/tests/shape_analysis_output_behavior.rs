//! Regressions for caller-owned outputs that OCCT leaves unchanged on early return.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{Plane3, Point2, Vector2};
use rustycad_tkmath::toploc::Location;
use rustycad_tkshhealing::shape_analysis::{EdgeTangent2d, ShapeAnalysisEdge};
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;

fn empty_analysis_edge() -> Edge {
    let mut edge = Edge::default();
    BRepBuilder::default().make_edge(&mut edge).unwrap();
    edge
}

#[test]
fn absent_pcurve_retains_point_but_clears_tangent() {
    let edge = empty_analysis_edge();
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomPlane::from_pln(Plane3::default())).cast();
    let mut outputs = EdgeTangent2d {
        point: Point2::new(17.0, -23.0),
        tangent: Vector2::new(3.0, -5.0),
    };
    let analyzer = ShapeAnalysisEdge::new();
    let found = analyzer
        .get_end_tangent_2d_on_surface_into(
            &edge,
            &surface,
            &Location::default(),
            false,
            0.0,
            &mut outputs,
        )
        .unwrap();
    assert!(!found);
    assert_eq!(outputs.point, Point2::new(17.0, -23.0));
    assert_eq!(outputs.tangent, Vector2::new(0.0, 0.0));
}

#[test]
fn absent_vertices_leave_both_tolerances_untouched() {
    let edge = empty_analysis_edge();
    let face = BRepBuilderAPIMakeFace::from_plane(Plane3::default())
        .unwrap()
        .face()
        .unwrap();
    let mut analyzer = ShapeAnalysisEdge::new();
    let mut tolerances = (17.0, -23.0);
    assert!(
        !analyzer
            .check_vertex_tolerance_into(&edge, &mut tolerances)
            .unwrap()
    );
    assert_eq!(tolerances, (17.0, -23.0));
    assert!(analyzer.status_flag(ShapeExtendStatus::Fail1));
    assert!(
        !analyzer
            .check_vertex_tolerance_on_face_into(&edge, &face, &mut tolerances)
            .unwrap()
    );
    assert_eq!(tolerances, (17.0, -23.0));
}

#[test]
fn degenerate_edge_retains_the_caller_deviation() {
    let edge = empty_analysis_edge();
    BRepBuilder::default().degenerated(&edge, true).unwrap();
    let mut analyzer = ShapeAnalysisEdge::new();
    let mut deviation = 17.0;
    assert!(
        !analyzer
            .check_same_parameter_into(&edge, 23, &mut deviation)
            .unwrap()
    );
    assert_eq!(deviation, 17.0);
    assert!(analyzer.status_flag(ShapeExtendStatus::Ok));
}
