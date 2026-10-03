//! Wall-face paths from BRepOffset_MakeSimpleOffset::BuildWallFace.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::{BRepToolsModification, Modifier};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Axis2, Circle3, Direction3, Plane3, Point3};
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
    while edges.more() {
        let original = Edge::unchecked(edges.current().clone());
        let offset = Edge::unchecked(modifier.modified_shape(&original).unwrap());
        let wall = build_simple_offset_wall(&original, &offset, &mut vertex_edges).unwrap();
        assert_eq!(surface_kind(&wall), "Geom_Plane");
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
    let (u, _, v, _) = surface.bounds();
    assert!((surface.value(u, v).unwrap().z()).abs() < 1e-12);
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
