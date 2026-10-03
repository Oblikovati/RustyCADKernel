//! Modification callbacks on the same bounded plane used by the native oracle.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::{BRepToolsModification, Modifier, NewCurve, NewSurface};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Plane3, Point3};
use rustycad_tkoffset::brep_offset::BRepOffsetSimpleOffset;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;

#[test]
fn prepared_face_and_edges_match_native_plane_offsets() {
    let face = plane_face();
    let modification = BRepOffsetSimpleOffset::new(&face, 10.0, 1e-7).unwrap();
    let mut output = NewSurface::default();
    assert!(modification.new_surface(&face, &mut output).unwrap());
    assert_eq!(
        output.surface.value(0.0, 0.0).unwrap(),
        Point3::new(0.0, 0.0, 10.0)
    );
    assert!(output.location.is_identity());
    assert!(!output.rev_face && !output.rev_wires);
    assert_eq!(output.tolerance, BRepTool::face_tolerance(&face).unwrap());
    assert_eq!(check_edge_callbacks(&modification, &face), 4);
}

#[test]
fn prepared_vertex_points_match_native_plane_offsets() {
    let face = plane_face();
    let modification = BRepOffsetSimpleOffset::new(&face, 10.0, 1e-7).unwrap();
    let mut ancestors = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(&face, ShapeEnum::Vertex, ShapeEnum::Edge, &mut ancestors)
        .unwrap();
    assert_eq!(ancestors.extent(), 4);
    for index in 1..=ancestors.extent() {
        check_vertex_callback(
            &modification,
            &Vertex::unchecked(ancestors.find_key(index).unwrap().clone()),
        );
    }
}

#[test]
fn missing_face_leaves_callback_outputs_unchanged() {
    let modification = empty_modification();
    let mut output = NewSurface {
        tolerance: 42.0,
        rev_face: true,
        ..Default::default()
    };
    assert!(
        !modification
            .new_surface(&Face::default(), &mut output)
            .unwrap()
    );
    assert_eq!((output.tolerance, output.rev_face), (42.0, true));
}

#[test]
fn missing_edge_leaves_callback_outputs_unchanged() {
    let modification = empty_modification();
    let mut output = NewCurve {
        tolerance: 43.0,
        ..Default::default()
    };
    assert!(
        !modification
            .new_curve(&Edge::default(), &mut output)
            .unwrap()
    );
    assert_eq!(output.tolerance, 43.0);
}

#[test]
fn missing_vertex_leaves_callback_outputs_unchanged() {
    let modification = empty_modification();
    let mut point = Point3::new(1.0, 2.0, 3.0);
    let mut tolerance = 44.0;
    assert!(
        !modification
            .new_point(&Vertex::default(), &mut point, &mut tolerance)
            .unwrap()
    );
    assert_eq!(point, Point3::new(1.0, 2.0, 3.0));
    assert_eq!(tolerance, 44.0);
}

fn empty_modification() -> BRepOffsetSimpleOffset {
    BRepOffsetSimpleOffset::new(&Shape::default(), 1.0, 1e-7).unwrap()
}

#[test]
fn reversed_edge_query_is_not_a_native_map_match() {
    let face = plane_face();
    let modification = BRepOffsetSimpleOffset::new(&face, 10.0, 1e-7).unwrap();
    let explorer = Explorer::new(&face, ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
    let mut edge = Edge::unchecked(explorer.current().clone());
    edge.reverse();
    let mut output = NewCurve {
        tolerance: 42.0,
        ..Default::default()
    };
    assert!(!modification.new_curve(&edge, &mut output).unwrap());
    assert_eq!(output.tolerance, 42.0);
}

#[test]
fn modifier_consumes_the_offset_callbacks() {
    let face = plane_face();
    let modification: Handle<dyn BRepToolsModification> =
        Handle::new(BRepOffsetSimpleOffset::new(&face, 10.0, 1e-7).unwrap()).cast();
    let modifier = Modifier::with_modification(&face, &modification).unwrap();
    assert!(modifier.is_done());
    let modified = Face::unchecked(modifier.modified_shape(&face).unwrap().clone());
    assert!(!modified.is_same(&face));
    let surface = BRepTool::surface(&modified).unwrap();
    assert_eq!(surface.value(0.0, 0.0).unwrap().z(), 10.0);
}

fn check_edge_callbacks(modification: &BRepOffsetSimpleOffset, face: &Face) -> usize {
    let mut explorer = Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
    let mut count = 0;
    while explorer.more() {
        let edge = Edge::unchecked(explorer.current().clone());
        check_edge_geometry(modification, &edge);
        check_preserved_pcurve(modification, &edge, face);
        check_preserved_parameter(modification, &edge);
        check_preserved_continuity(modification, &edge, face);
        explorer.next().unwrap();
        count += 1;
    }
    count
}

fn check_edge_geometry(modification: &BRepOffsetSimpleOffset, edge: &Edge) {
    let mut result = NewCurve::default();
    assert!(modification.new_curve(edge, &mut result).unwrap());
    assert!(result.location.is_identity());
    assert_eq!(result.tolerance, 1e-7);
    let (source, first, last) = BRepTool::curve(edge).unwrap();
    for parameter in [first, last] {
        let point = source.value(parameter).unwrap();
        let expected = Point3::new(point.x(), point.y(), 10.0);
        assert!(result.curve.value(parameter).unwrap().distance(&expected) < 1e-12);
    }
}

fn check_preserved_pcurve(modification: &BRepOffsetSimpleOffset, edge: &Edge, face: &Face) {
    let mut result = Handle::default();
    let mut tolerance = -1.0;
    assert!(
        modification
            .new_curve2d(
                edge,
                face,
                &Edge::default(),
                &Face::default(),
                &mut result,
                &mut tolerance
            )
            .unwrap()
    );
    let source = BRepTool::curve_on_face(edge, face).unwrap();
    assert!(result == source.pcurve);
    assert_eq!(tolerance, 1e-7);
}

fn check_preserved_continuity(modification: &BRepOffsetSimpleOffset, edge: &Edge, face: &Face) {
    let continuity = modification
        .continuity(
            edge,
            face,
            face,
            &Edge::default(),
            &Face::default(),
            &Face::default(),
        )
        .unwrap();
    assert_eq!(
        continuity,
        BRepTool::continuity_on_faces(edge, face, face).unwrap()
    );
}

fn check_preserved_parameter(modification: &BRepOffsetSimpleOffset, edge: &Edge) {
    let (vertex, _) = topexp::vertices(edge, false);
    let mut parameter = -42.0;
    let mut tolerance = -1.0;
    assert!(
        modification
            .new_parameter(&vertex, edge, &mut parameter, &mut tolerance)
            .unwrap()
    );
    assert_eq!(parameter, BRepTool::parameter(&vertex, edge).unwrap());
    let mut point = Point3::default();
    let mut expected = BRepTool::vertex_tolerance(&vertex).unwrap();
    modification
        .new_point(&vertex, &mut point, &mut expected)
        .unwrap();
    assert_eq!(tolerance, expected);
}

fn check_vertex_callback(modification: &BRepOffsetSimpleOffset, vertex: &Vertex) {
    let original = BRepTool::pnt(vertex).unwrap();
    let mut point = Point3::default();
    let mut tolerance = -1.0;
    assert!(
        modification
            .new_point(vertex, &mut point, &mut tolerance)
            .unwrap()
    );
    assert_eq!(point, Point3::new(original.x(), original.y(), 10.0));
    assert_eq!(tolerance, 1e-7 * 1.001);
}

fn plane_face() -> Face {
    BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (-1.0, 1.0, -1.0, 1.0))
        .unwrap()
        .face()
        .unwrap()
}
