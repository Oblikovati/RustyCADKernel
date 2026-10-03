use super::super::ShapeFixIntersectionTool;
use crate::shape_analysis::ShapeAnalysisEdge;
use crate::shape_build::{ShapeBuildEdge, ShapeBuildReShape};
use crate::shape_extend::ShapeExtendWireData;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::{Face, ShapeIterator};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane};
use rustycad_tkmath::gp::{Axis1, Axis2d, Axis3, Direction2, Direction3, Point2, Point3};
use rustycad_tkmath::toploc::Location;

#[test]
fn crossing_wire_edges_split_and_replace_the_face() {
    let mut face = planar_face();
    let first = make_wire(&face, [(-1.0, 0.0, 1.0, 0.0), (3.0, 3.0, 1.0, 0.0)]);
    let second = make_wire(&face, [(0.0, -1.0, 0.0, 1.0), (-4.0, -4.0, 0.0, 1.0)]);
    let builder = BRepBuilder::default();
    builder
        .add(&mut face, &first.wire().unwrap().into_shape())
        .unwrap();
    builder
        .add(&mut face, &second.wire().unwrap().into_shape())
        .unwrap();
    let old_face = face.clone().into_shape();
    let context = Handle::new(ShapeBuildReShape::new());
    let tool = ShapeFixIntersectionTool::new(&context, 1.0e-7);

    assert!(tool.fix_intersecting_wires(&mut face).unwrap());

    let applied = context
        .apply(&old_face, rustycad_tkg3d::top_abs::ShapeEnum::Shape)
        .unwrap();
    assert!(applied.is_same(&face.clone().into_shape()));
    let wires = direct_wires(&face);
    assert_eq!(wires.len(), 2);
    assert_eq!(wires[0].nb_children(), 3);
    assert_eq!(wires[1].nb_children(), 3);
    let ranges = wires
        .iter()
        .flat_map(|wire| direct_edge_ranges(wire, &face))
        .collect::<Vec<_>>();
    assert_eq!(
        ranges,
        [0.0, 1.0, 1.0, 2.0, 0.0, 2.0, 0.0, 1.0, 1.0, 2.0, 0.0, 2.0]
    );
}

#[test]
fn face_with_fewer_than_two_wires_is_unchanged() {
    let mut face = planar_face();
    let only_wire = make_wire(&face, [(-1.0, 0.0, 1.0, 0.0), (3.0, 3.0, 1.0, 0.0)]);
    BRepBuilder::default()
        .add(&mut face, &only_wire.wire().unwrap().into_shape())
        .unwrap();
    let old_face = face.clone().into_shape();
    let context = Handle::new(ShapeBuildReShape::new());
    let tool = ShapeFixIntersectionTool::new(&context, 1.0e-7);

    assert!(!tool.fix_intersecting_wires(&mut face).unwrap());
    assert!(old_face.is_same(&face.clone().into_shape()));
}

#[test]
fn middle_to_endpoint_crossing_reuses_the_other_wires_vertex() {
    let mut face = planar_face();
    let first = make_wire(&face, [(-1.0, 0.0, 1.0, 0.0), (3.0, 3.0, 1.0, 0.0)]);
    let second = make_wire(&face, [(0.0, 0.0, 0.0, 1.0), (-4.0, -4.0, 0.0, 1.0)]);
    let builder = BRepBuilder::default();
    builder
        .add(&mut face, &first.wire().unwrap().into_shape())
        .unwrap();
    builder
        .add(&mut face, &second.wire().unwrap().into_shape())
        .unwrap();
    let context = Handle::new(ShapeBuildReShape::new());
    let tool = ShapeFixIntersectionTool::new(&context, 1.0e-7);

    assert!(tool.fix_intersecting_wires(&mut face).unwrap());

    let wires = direct_wires(&face);
    assert_eq!(wires[0].nb_children(), 3);
    assert_eq!(wires[1].nb_children(), 2);
    let first_wire = crate::shape_extend::ShapeExtendWireData::from_wire(
        &rustycad_tkbrep::topods::wire(&wires[0]).unwrap(),
        true,
        true,
    )
    .unwrap();
    let second_wire = crate::shape_extend::ShapeExtendWireData::from_wire(
        &rustycad_tkbrep::topods::wire(&wires[1]).unwrap(),
        true,
        true,
    )
    .unwrap();
    let split_endpoint = ShapeAnalysisEdge::new().last_vertex(&first_wire.edge(1).unwrap());
    let reused_endpoint = ShapeAnalysisEdge::new().first_vertex(&second_wire.edge(1).unwrap());
    assert!(split_endpoint.is_same(&reused_endpoint));
}

#[test]
fn endpoint_to_endpoint_crossing_unions_the_wire_vertices() {
    let mut face = planar_face();
    let first = make_wire(&face, [(0.0, 0.0, 1.0, 0.0), (3.0, 3.0, 1.0, 0.0)]);
    let second = make_wire(&face, [(0.0, 0.0, 0.0, 1.0), (-4.0, -4.0, 0.0, 1.0)]);
    let builder = BRepBuilder::default();
    builder
        .add(&mut face, &first.wire().unwrap().into_shape())
        .unwrap();
    builder
        .add(&mut face, &second.wire().unwrap().into_shape())
        .unwrap();
    let context = Handle::new(ShapeBuildReShape::new());
    let tool = ShapeFixIntersectionTool::new(&context, 1.0e-7);

    assert!(tool.fix_intersecting_wires(&mut face).unwrap());

    let wires = direct_wires(&face);
    let first_wire = crate::shape_extend::ShapeExtendWireData::from_wire(
        &rustycad_tkbrep::topods::wire(&wires[0]).unwrap(),
        true,
        true,
    )
    .unwrap();
    let second_wire = crate::shape_extend::ShapeExtendWireData::from_wire(
        &rustycad_tkbrep::topods::wire(&wires[1]).unwrap(),
        true,
        true,
    )
    .unwrap();
    let first_vertex = ShapeAnalysisEdge::new().first_vertex(&first_wire.edge(1).unwrap());
    let second_vertex = ShapeAnalysisEdge::new().first_vertex(&second_wire.edge(1).unwrap());
    assert!(first_vertex.is_same(&second_vertex));
}

#[test]
fn short_cross_wire_overlap_splits_both_overlapping_edges() {
    let mut face = planar_face();
    let first = Handle::new(ShapeExtendWireData::new());
    first
        .add_edge(&line_edge_range(&face, 0.0, 0.0, 0.0, 1.0, 0.0, 10.0), 0)
        .unwrap();
    first
        .add_edge(&line_edge(&face, 30.0, 30.0, 1.0, 0.0), 0)
        .unwrap();
    let second = Handle::new(ShapeExtendWireData::new());
    second
        .add_edge(&line_edge_range(&face, 5.0, 0.0, 5.0e-8, 1.0, 0.0, 10.0), 0)
        .unwrap();
    second
        .add_edge(&line_edge(&face, -30.0, -30.0, 0.0, 1.0), 0)
        .unwrap();
    let builder = BRepBuilder::default();
    builder
        .add(&mut face, &first.wire().unwrap().into_shape())
        .unwrap();
    builder
        .add(&mut face, &second.wire().unwrap().into_shape())
        .unwrap();
    let context = Handle::new(ShapeBuildReShape::new());
    let tool = ShapeFixIntersectionTool::new(&context, 1.0e-7);

    assert!(tool.fix_intersecting_wires(&mut face).unwrap());

    let wires = direct_wires(&face);
    assert_eq!([wires[0].nb_children(), wires[1].nb_children()], [3, 3]);
    let ranges = wires
        .iter()
        .flat_map(|wire| direct_edge_ranges(wire, &face))
        .collect::<Vec<_>>();
    assert_eq!(
        ranges,
        [0.0, 7.5, 7.5, 10.0, 0.0, 2.0, 0.0, 2.5, 2.5, 10.0, 0.0, 2.0]
    );
}

#[test]
fn long_overlap_reuses_one_wire_segment_for_the_full_overlap() {
    assert_full_long_overlap_replacement(2.0, 1.0);
}

#[test]
fn reversed_long_overlap_reuses_one_wire_segment_for_the_full_overlap() {
    assert_full_long_overlap_replacement(8.0, -1.0);
}

#[test]
fn contained_first_long_overlap_is_inserted_into_the_second_wire() {
    let mut face = planar_face();
    let first = Handle::new(ShapeExtendWireData::new());
    first
        .add_edge(&line_edge_range(&face, 2.0, 0.0, 0.0, 1.0, 0.0, 6.0), 0)
        .unwrap();
    first
        .add_edge(&line_edge(&face, 30.0, 30.0, 1.0, 0.0), 0)
        .unwrap();
    let second_edge = line_edge_range(&face, 0.0, 0.0, 0.0, 1.0, 0.0, 10.0);
    let second_endpoint = ShapeAnalysisEdge::new().first_vertex(&second_edge);
    BRepBuilder::default()
        .update_vertex_tolerance(&second_endpoint, 1.0e-5)
        .unwrap();
    let second = Handle::new(ShapeExtendWireData::new());
    second.add_edge(&second_edge, 0).unwrap();
    second
        .add_edge(&line_edge(&face, -30.0, -30.0, 0.0, 1.0), 0)
        .unwrap();
    add_wires(&mut face, &first, &second);
    let old_face = face.clone().into_shape();
    let context = Handle::new(ShapeBuildReShape::new());
    let tool = ShapeFixIntersectionTool::new(&context, 1.0e-7);

    assert!(tool.fix_intersecting_wires(&mut face).unwrap());

    let wires = direct_wires(&face);
    assert_eq!([wires[0].nb_children(), wires[1].nb_children()], [2, 4]);
    assert_eq!(
        wires
            .iter()
            .flat_map(|wire| direct_edge_ranges(wire, &face))
            .collect::<Vec<_>>(),
        [0.0, 6.0, 0.0, 2.0, 0.0, 2.0, 0.0, 6.0, 8.0, 10.0, 0.0, 2.0]
    );
    let first_data = ShapeExtendWireData::from_wire(
        &rustycad_tkbrep::topods::wire(&wires[0]).unwrap(),
        true,
        true,
    )
    .unwrap();
    let second_data = ShapeExtendWireData::from_wire(
        &rustycad_tkbrep::topods::wire(&wires[1]).unwrap(),
        true,
        true,
    )
    .unwrap();
    assert!(
        first_data
            .edge(1)
            .unwrap()
            .is_same(&second_data.edge(2).unwrap())
    );
    let applied = context
        .apply(&old_face, rustycad_tkg3d::top_abs::ShapeEnum::Shape)
        .unwrap();
    assert!(applied.is_same(&face.clone().into_shape()));
}

#[test]
fn partial_long_overlap_inserts_the_source_middle_and_keeps_target_tail() {
    let mut face = planar_face();
    let first = Handle::new(ShapeExtendWireData::new());
    first
        .add_edge(&line_edge_range(&face, 0.0, 0.0, 0.0, 1.0, 0.0, 10.0), 0)
        .unwrap();
    first
        .add_edge(&line_edge(&face, 30.0, 30.0, 1.0, 0.0), 0)
        .unwrap();
    let second_edge = line_edge_range(&face, 2.0, 0.0, 0.0, 1.0, 0.0, 10.0);
    let endpoint = ShapeAnalysisEdge::new().first_vertex(&second_edge);
    BRepBuilder::default()
        .update_vertex_tolerance(&endpoint, 1.0e-5)
        .unwrap();
    let second = Handle::new(ShapeExtendWireData::new());
    second.add_edge(&second_edge, 0).unwrap();
    second
        .add_edge(&line_edge(&face, -30.0, -30.0, 0.0, 1.0), 0)
        .unwrap();
    add_wires(&mut face, &first, &second);
    let old_face = face.clone().into_shape();
    let context = Handle::new(ShapeBuildReShape::new());
    let tool = ShapeFixIntersectionTool::new(&context, 1.0e-7);

    assert!(tool.fix_intersecting_wires(&mut face).unwrap());

    let wires = direct_wires(&face);
    assert_eq!([wires[0].nb_children(), wires[1].nb_children()], [3, 3]);
    assert_eq!(
        wires
            .iter()
            .flat_map(|wire| direct_edge_ranges(wire, &face))
            .collect::<Vec<_>>(),
        [
            0.0, 2.0, 2.0, 10.0, 0.0, 2.0, 2.0, 10.0, 8.0, 10.0, 0.0, 2.0
        ]
    );
    let first_data = ShapeExtendWireData::from_wire(
        &rustycad_tkbrep::topods::wire(&wires[0]).unwrap(),
        true,
        true,
    )
    .unwrap();
    let second_data = ShapeExtendWireData::from_wire(
        &rustycad_tkbrep::topods::wire(&wires[1]).unwrap(),
        true,
        true,
    )
    .unwrap();
    assert!(
        first_data
            .edge(2)
            .unwrap()
            .is_same(&second_data.edge(1).unwrap())
    );
    let applied = context
        .apply(&old_face, rustycad_tkg3d::top_abs::ShapeEnum::Shape)
        .unwrap();
    assert!(applied.is_same(&face.clone().into_shape()));
}

fn assert_full_long_overlap_replacement(second_start: f64, second_direction: f64) {
    let mut face = planar_face();
    let first_edge = line_edge_range(&face, 0.0, 0.0, 0.0, 1.0, 0.0, 10.0);
    let first_vertex = ShapeAnalysisEdge::new().first_vertex(&first_edge);
    BRepBuilder::default()
        .update_vertex_tolerance(&first_vertex, 1.0e-5)
        .unwrap();
    let first = Handle::new(ShapeExtendWireData::new());
    first.add_edge(&first_edge, 0).unwrap();
    first
        .add_edge(&line_edge(&face, 30.0, 30.0, 1.0, 0.0), 0)
        .unwrap();
    let second = Handle::new(ShapeExtendWireData::new());
    second
        .add_edge(
            &line_edge_range(&face, second_start, 0.0, 0.0, second_direction, 0.0, 6.0),
            0,
        )
        .unwrap();
    second
        .add_edge(&line_edge(&face, -30.0, -30.0, 0.0, 1.0), 0)
        .unwrap();
    add_wires(&mut face, &first, &second);
    let old_face = face.clone().into_shape();
    let context = Handle::new(ShapeBuildReShape::new());
    let tool = ShapeFixIntersectionTool::new(&context, 1.0e-7);

    assert!(tool.fix_intersecting_wires(&mut face).unwrap());

    let wires = direct_wires(&face);
    assert_eq!([wires[0].nb_children(), wires[1].nb_children()], [4, 2]);
    let first_data = ShapeExtendWireData::from_wire(
        &rustycad_tkbrep::topods::wire(&wires[0]).unwrap(),
        true,
        true,
    )
    .unwrap();
    let second_data = ShapeExtendWireData::from_wire(
        &rustycad_tkbrep::topods::wire(&wires[1]).unwrap(),
        true,
        true,
    )
    .unwrap();
    assert!(
        first_data
            .edge(2)
            .unwrap()
            .is_same(&second_data.edge(1).unwrap())
    );
    let applied = context
        .apply(&old_face, rustycad_tkg3d::top_abs::ShapeEnum::Shape)
        .unwrap();
    assert!(applied.is_same(&face.clone().into_shape()));
    assert_eq!(
        wires
            .iter()
            .flat_map(|wire| direct_edge_ranges(wire, &face))
            .collect::<Vec<_>>(),
        [0.0, 2.0, 2.0, 8.0, 8.0, 10.0, 0.0, 2.0, 2.0, 8.0, 0.0, 2.0]
    );
}

#[test]
fn one_sided_endpoint_cut_trims_edge_without_replacing_face() {
    let mut face = planar_face();
    let long_edge = line_edge_range(&face, 0.0, 0.0, 0.0, 1.0, 0.0, 1000.0);
    let endpoint = ShapeAnalysisEdge::new().first_vertex(&long_edge);
    BRepBuilder::default()
        .update_vertex_tolerance(&endpoint, 2.0)
        .unwrap();
    let first = Handle::new(ShapeExtendWireData::new());
    first.add_edge(&long_edge, 0).unwrap();
    first
        .add_edge(&line_edge(&face, 3000.0, 3000.0, 1.0, 0.0), 0)
        .unwrap();
    let second = Handle::new(ShapeExtendWireData::new());
    second
        .add_edge(&line_edge_range(&face, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0), 0)
        .unwrap();
    second
        .add_edge(&line_edge(&face, -3000.0, -3000.0, 0.0, 1.0), 0)
        .unwrap();
    add_wires(&mut face, &first, &second);
    let context = Handle::new(ShapeBuildReShape::new());
    let tool = ShapeFixIntersectionTool::new(&context, 1.0e-7);

    assert!(!tool.fix_intersecting_wires(&mut face).unwrap());

    let wires = direct_wires(&face);
    assert_eq!([wires[0].nb_children(), wires[1].nb_children()], [2, 2]);
    assert_eq!(
        wires
            .iter()
            .flat_map(|wire| direct_edge_ranges(wire, &face))
            .collect::<Vec<_>>(),
        [1.0, 1000.0, 0.0, 2.0, 0.0, 1.0, 0.0, 2.0]
    );
}

fn add_wires(face: &mut Face, first: &ShapeExtendWireData, second: &ShapeExtendWireData) {
    let builder = BRepBuilder::default();
    builder
        .add(face, &first.wire().unwrap().into_shape())
        .unwrap();
    builder
        .add(face, &second.wire().unwrap().into_shape())
        .unwrap();
}

fn planar_face() -> Face {
    let surface: Handle<dyn rustycad_tkg3d::geom::GeomSurface> =
        Handle::new(GeomPlane::new(Axis3::default())).cast();
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1.0e-7)
        .unwrap();
    face
}

fn make_wire(face: &Face, lines: [(f64, f64, f64, f64); 2]) -> Handle<ShapeExtendWireData> {
    let wire = Handle::new(ShapeExtendWireData::new());
    for (x, y, dx, dy) in lines {
        let edge = line_edge(face, x, y, dx, dy);
        wire.add_edge(&edge, 0).unwrap();
    }
    wire
}

fn line_edge(face: &Face, x: f64, y: f64, dx: f64, dy: f64) -> rustycad_tkbrep::topods::Edge {
    line_edge_range(face, x, y, 0.0, dx, dy, 2.0)
}

fn line_edge_range(
    face: &Face,
    x: f64,
    y: f64,
    z: f64,
    dx: f64,
    dy: f64,
    range: f64,
) -> rustycad_tkbrep::topods::Edge {
    let direction = Direction3::new(dx, dy, 0.0).unwrap();
    let curve: Handle<dyn GeomCurve> =
        Handle::new(GeomLine::new(Axis1::new(Point3::new(x, y, z), direction))).cast();
    let mut edge = rustycad_tkbrep::topods::Edge::default();
    ShapeBuildEdge
        .make_edge_on_curve_range(&mut edge, &curve, &Location::default(), 0.0, range)
        .unwrap();
    let pcurve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::new(
        Point2::new(x, y),
        Direction2::new(dx, dy).unwrap(),
    )))
    .cast();
    let builder = BRepBuilder::default();
    builder
        .update_edge_pcurve_on_face(&edge, &pcurve, face, 1.0e-7)
        .unwrap();
    builder.range_on_face(&edge, face, 0.0, range).unwrap();
    builder.same_range(&edge, true).unwrap();
    builder.same_parameter(&edge, true).unwrap();
    edge
}

fn direct_wires(face: &Face) -> Vec<rustycad_tkbrep::topods::Shape> {
    let mut iterator = ShapeIterator::new(&face.clone().into_shape(), false, true);
    let mut wires = Vec::new();
    while iterator.more() {
        let child = iterator.value().unwrap().clone();
        iterator.next();
        if child.shape_type().unwrap() == rustycad_tkg3d::top_abs::ShapeEnum::Wire {
            wires.push(child);
        }
    }
    wires
}

fn direct_edge_ranges(wire: &rustycad_tkbrep::topods::Shape, face: &Face) -> Vec<f64> {
    let mut iterator = ShapeIterator::new(wire, false, true);
    let mut ranges = Vec::new();
    while iterator.more() {
        let edge = rustycad_tkbrep::topods::edge(iterator.value().unwrap()).unwrap();
        let (first, last) = BRepTool::range_on_face(&edge, face).unwrap();
        ranges.extend([first, last]);
        iterator.next();
    }
    ranges
}
