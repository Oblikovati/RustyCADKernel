//! Wall-face paths from BRepOffset_MakeSimpleOffset::BuildWallFace.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::{BRepToolsModification, Modifier};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Axis2, Circle3, Direction3, Plane3, Point2, Point3};
use rustycad_tkoffset::brep_offset::{BRepOffsetSimpleOffset, build_simple_offset_wall};
use rustycad_tktopalgo::brep_lib::{BRepLibMakeEdge, BRepLibMakeFace};

#[test]
fn planar_walls_share_connecting_edges() {
    let face = BRepLibMakeFace::from_plane_bounds(Plane3::default(), (0., 2., 0., 3.))
        .unwrap()
        .face()
        .unwrap();
    let modification: Handle<dyn BRepToolsModification> =
        Handle::new(BRepOffsetSimpleOffset::new(&face, 1., 1e-7).unwrap()).cast();
    let modifier = Modifier::with_modification(&face, &modification).unwrap();
    let mut vertex_edges = DataMap::<Vertex, Edge>::default();
    let mut edges = Explorer::new(&face, ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
    let expected_points = [
        Point3::new(0.0, 1.0, 0.75),
        Point3::new(1.5, 0.0, 0.75),
        Point3::new(2.0, 1.0, 0.75),
        Point3::new(1.5, 3.0, 0.75),
    ];
    let mut wall_index = 0;
    while edges.more() {
        wall_index += 1;
        let original = Edge::unchecked(edges.current().clone());
        let offset = Edge::unchecked(modifier.modified_shape(&original).unwrap());
        let wall = build_simple_offset_wall(&original, &offset, &mut vertex_edges).unwrap();
        assert_eq!(surface_kind(&wall), "Geom_Plane");
        let point = BRepTool::surface(&wall).unwrap().value(0.25, 0.5).unwrap();
        assert!(point.distance(&expected_points[wall_index - 1]) < 1.0e-13);
        check_planar_wall_edges(&wall);
        edges.next().unwrap();
    }
    assert_eq!(vertex_edges.len(), 4);
}

#[test]
fn coincident_vertices_fail_wall_construction() {
    let edge = BRepLibMakeEdge::from_points(Point3::new(0., 0., 0.), Point3::new(1., 0., 0.))
        .unwrap()
        .edge()
        .unwrap();
    let mut vertex_edges = DataMap::default();
    let wall = build_simple_offset_wall(&edge, &edge, &mut vertex_edges).unwrap();
    assert!(wall.is_null());
    assert!(vertex_edges.is_empty());
}

#[test]
fn nonplanar_open_arcs_use_ruled_surface() {
    check_ruled_wall(std::f64::consts::PI, 2);
}

#[test]
fn closed_circles_use_one_seam_connecting_edge() {
    check_ruled_wall(std::f64::consts::TAU, 1);
}

fn check_ruled_wall(end: f64, expected_edges: usize) {
    let original = circle_edge(0., end);
    let offset = circle_edge(2., end);
    let mut vertex_edges = DataMap::default();
    let wall = build_simple_offset_wall(&original, &offset, &mut vertex_edges).unwrap();
    assert_eq!(surface_kind(&wall), "Geom_BSplineSurface");
    assert_eq!(vertex_edges.len(), expected_edges);
    let surface = BRepTool::surface(&wall).unwrap();
    let expected_bounds = (0.0, end, 0.0, 1.0);
    assert_eq!(surface.bounds(), expected_bounds);
    let expected_point = if expected_edges == 1 {
        Point3::new(
            2.9067371922853966,
            0.742_211_884_537_079_3,
            0.999_999_999_999_988_8,
        )
    } else {
        Point3::new(
            2.9067372829339573,
            0.742_211_877_152_640_3,
            1.0000000000000013,
        )
    };
    assert!(surface.value(0.25, 0.5).unwrap().distance(&expected_point) < 1.0e-12);
    check_ruled_wall_edges(&wall, end, expected_edges == 1);
}

fn check_planar_wall_edges(wall: &Face) {
    let mut edges = Explorer::new(
        &wall.clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )
    .unwrap();
    let mut count = 0;
    while edges.more() {
        count += 1;
        let edge = Edge::unchecked(edges.current().clone());
        let (curve, first, last) = BRepTool::curve(&edge).unwrap();
        let pcurve = BRepTool::curve_on_face(&edge, wall).unwrap();
        assert_eq!(curve.get().unwrap().dynamic_type().name(), "Geom_Line");
        assert_eq!(
            pcurve.pcurve.get().unwrap().dynamic_type().name(),
            "Geom2d_Line"
        );
        assert_eq!((first, last), (pcurve.first, pcurve.last));
        assert!(BRepTool::same_range(&edge).unwrap());
        assert!(BRepTool::same_parameter(&edge).unwrap());
        assert_eq!(BRepTool::edge_tolerance(&edge).unwrap(), 1.0e-7);
        edges.next().unwrap();
    }
    assert_eq!(count, 4);
}

fn check_ruled_wall_edges(wall: &Face, end: f64, closed: bool) {
    let expected_uv = [
        (Point2::new(0.0, 0.0), Point2::new(end, 0.0)),
        (Point2::new(end, 0.0), Point2::new(end, 1.0)),
        (Point2::new(0.0, 1.0), Point2::new(end, 1.0)),
        (Point2::new(0.0, 0.0), Point2::new(0.0, 1.0)),
    ];
    let mut edges = Explorer::new(
        &wall.clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )
    .unwrap();
    let mut index = 0;
    while edges.more() {
        check_ruled_wall_edge(wall, &edges, &expected_uv[index], end, closed, index);
        index += 1;
        edges.next().unwrap();
    }
    assert_eq!(index, 4);
}

fn check_ruled_wall_edge(
    wall: &Face,
    edges: &Explorer,
    expected_uv: &(Point2, Point2),
    end: f64,
    closed: bool,
    index: usize,
) {
    let edge = Edge::unchecked(edges.current().clone());
    let (_, first, last) = BRepTool::curve(&edge).unwrap();
    let expected_range = if index == 0 || index == 2 {
        (0.0, end)
    } else {
        (0.0, 1.0)
    };
    let pcurve = BRepTool::curve_on_face(&edge, wall).unwrap();
    let uv_first = pcurve.pcurve.value(pcurve.first).unwrap();
    let uv_last = pcurve.pcurve.value(pcurve.last).unwrap();
    assert_eq!((first, last), expected_range);
    assert_eq!((pcurve.first, pcurve.last), expected_range);
    assert!(uv_first.distance(expected_uv.0) < 1.0e-13);
    assert!(uv_last.distance(expected_uv.1) < 1.0e-13);
    assert_eq!(
        BRepTool::same_range(&edge).unwrap(),
        closed || index.is_multiple_of(2)
    );
    assert_eq!(
        BRepTool::same_parameter(&edge).unwrap(),
        closed || index.is_multiple_of(2)
    );
    assert_eq!(
        BRepTool::is_closed_on_face(&edge, wall).unwrap(),
        closed && index % 2 == 1
    );
}

fn circle_edge(z: f64, end: f64) -> Edge {
    let axis = Axis2::new(Point3::new(0., 0., z), Direction3::new(0., 0., 1.).unwrap()).unwrap();
    let circle = Circle3::new(axis, 3.).unwrap();
    BRepLibMakeEdge::from_circ_params(circle, 0., end)
        .unwrap()
        .edge()
        .unwrap()
}

fn surface_kind(face: &Face) -> &'static str {
    let surface = BRepTool::surface(face).unwrap();
    surface.get().unwrap().dynamic_type().name()
}
