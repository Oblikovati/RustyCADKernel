//! Pinned source behavior for ShapeFix_SplitCommonVertex.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Builder, Edge, Face, Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tkshhealing::shape_extend::{BasicMsgRegistrator, ShapeExtendMsgRegistrator};
use rustycad_tkshhealing::shape_fix::{FixRoot, ShapeFixSplitCommonVertex};

#[test]
fn splits_shared_equal_vertex_on_later_wire_and_warns_on_root_shape() {
    let (face, _, _) = two_wire_face(false);
    let registrator = Handle::new(ShapeExtendMsgRegistrator::new());
    let fixer = ShapeFixSplitCommonVertex::new();
    fixer.set_msg_registrator(&registrator.clone().cast::<dyn BasicMsgRegistrator>());
    fixer.init(&face.clone().into_shape()).unwrap();
    fixer.perform().unwrap();
    assert_face_endpoint_identity(&fixer.shape().unwrap(), false);
    assert_eq!(registrator.map_shape(|map| map.len()), 0);
}

#[test]
fn does_not_split_vertices_with_different_orientations() {
    let (face, _, _) = two_wire_face(true);
    let fixer = ShapeFixSplitCommonVertex::new();
    fixer.init(&face.into_shape()).unwrap();
    fixer.perform().unwrap();
    assert_face_endpoint_identity(&fixer.shape().unwrap(), true);
}

#[test]
fn init_discards_the_context_apply_result_until_perform() {
    let (source, _, _) = two_wire_face(false);
    let mut replacement = Face::default();
    BRepBuilder::default().make_face(&mut replacement);
    let context = rustycad_tkshhealing::shape_build::ShapeBuildReShape::new();
    ReShape::replace(
        &context,
        &source.clone().into_shape(),
        &replacement.clone().into_shape(),
    )
    .unwrap();
    let fixer = ShapeFixSplitCommonVertex::new();
    fixer.set_context(&Handle::new(context));
    fixer.init(&source.clone().into_shape()).unwrap();
    assert!(fixer.shape().unwrap().is_same(&source.clone().into_shape()));
    fixer.perform().unwrap();
    assert!(fixer.shape().unwrap().is_same(&replacement.into_shape()));
}

#[test]
fn replacements_are_reused_across_wire_pairs_on_one_face() {
    let face = three_wire_face();
    let fixer = ShapeFixSplitCommonVertex::new();
    fixer.init(&face.into_shape()).unwrap();
    fixer.perform().unwrap();
    let edges = face_edges(&fixer.shape().unwrap());
    let shared = edges.iter().map(vertex_at_shared_point).collect::<Vec<_>>();
    assert!(!shared[0].is_same(&shared[1]));
    assert!(shared[1].is_same(&shared[2]));
}

#[test]
fn compound_input_runs_face_search_and_root_shape_kinds_above_face_are_noop() {
    let (face, _, _) = two_wire_face(false);
    let mut compound = topods::Compound::default();
    Builder.make_compound(&mut compound);
    let mut compound_shape = compound.into_shape();
    Builder
        .add(&mut compound_shape, &face.clone().into_shape())
        .unwrap();
    let fixer = ShapeFixSplitCommonVertex::new();
    fixer.init(&compound_shape).unwrap();
    fixer.perform().unwrap();
    assert_face_endpoint_identity(&fixer.shape().unwrap(), false);

    let mut wire = Wire::default();
    Builder.make_wire(&mut wire);
    let no_op = ShapeFixSplitCommonVertex::new();
    no_op.init(&wire.clone().into_shape()).unwrap();
    no_op.perform().unwrap();
    assert!(no_op.shape().unwrap().is_same(&wire.into_shape()));
}

fn two_wire_face(reversed_second: bool) -> (Face, Edge, Edge) {
    let shared = make_vertex(0.0);
    let first = edge_from(&shared, &make_vertex(1.0), false);
    let second = edge_from(&shared, &make_vertex(2.0), reversed_second);
    let mut face = Face::default();
    let builder = BRepBuilder::default();
    builder.make_face(&mut face);
    let mut face_shape = face.into_shape();
    for edge in [&first, &second] {
        let mut wire = Wire::default();
        Builder.make_wire(&mut wire);
        let mut wire_shape = wire.into_shape();
        Builder
            .add(&mut wire_shape, &edge.clone().into_shape())
            .unwrap();
        Builder.add(&mut face_shape, &wire_shape).unwrap();
    }
    (topods::face(&face_shape).unwrap(), first, second)
}

fn three_wire_face() -> Face {
    let shared = make_vertex(0.0);
    let mut face = Face::default();
    BRepBuilder::default().make_face(&mut face);
    let mut face_shape = face.into_shape();
    for endpoint in [1.0, 2.0, 3.0] {
        let edge = edge_from(&shared, &make_vertex(endpoint), false);
        let mut wire = Wire::default();
        Builder.make_wire(&mut wire);
        let mut wire_shape = wire.into_shape();
        Builder.add(&mut wire_shape, &edge.into_shape()).unwrap();
        Builder.add(&mut face_shape, &wire_shape).unwrap();
    }
    topods::face(&face_shape).unwrap()
}

fn make_vertex(x: f64) -> Vertex {
    let mut vertex = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut vertex, Point3::new(x, 0.0, 0.0), 1e-7)
        .unwrap();
    vertex
}

fn edge_from(first: &Vertex, last: &Vertex, reverse: bool) -> Edge {
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder.make_edge(&mut edge).unwrap();
    let mut edge_shape = edge.clone().into_shape();
    let oriented_first = first.oriented(rustycad_tkg3d::top_abs::Orientation::Forward);
    let oriented_last = last.oriented(rustycad_tkg3d::top_abs::Orientation::Reversed);
    Builder.add(&mut edge_shape, &oriented_first).unwrap();
    Builder.add(&mut edge_shape, &oriented_last).unwrap();
    edge = topods::edge(&edge_shape).unwrap();
    if reverse {
        edge.reverse();
    }
    edge
}

fn assert_face_endpoint_identity(shape: &rustycad_tkbrep::topods::Shape, should_match: bool) {
    let edges = face_edges(shape);
    let first_vertex = vertex_at_shared_point(&edges[0]);
    let second_vertex = vertex_at_shared_point(&edges[1]);
    assert_eq!(first_vertex.is_same(&second_vertex), should_match);
    assert_eq!(
        BRepTool::pnt(&first_vertex).unwrap(),
        BRepTool::pnt(&second_vertex).unwrap()
    );
}

fn face_edges(shape: &rustycad_tkbrep::topods::Shape) -> Vec<Shape> {
    let face = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)
        .unwrap()
        .current()
        .clone();
    ShapeIterator::new(&face, false, true)
        .filter(|child| child.shape_type() == Ok(ShapeEnum::Wire))
        .map(|wire| first_child(&wire))
        .collect()
}

fn first_child(shape: &rustycad_tkbrep::topods::Shape) -> Shape {
    ShapeIterator::new(shape, false, true)
        .value()
        .unwrap()
        .clone()
}

fn vertex_at_shared_point(edge: &Shape) -> Vertex {
    ShapeIterator::new(edge, false, true)
        .find(|child| {
            child.shape_type() == Ok(ShapeEnum::Vertex)
                && BRepTool::pnt(&topods::vertex(child).unwrap())
                    .is_ok_and(|point| point == Point3::new(0.0, 0.0, 0.0))
        })
        .map(|child| topods::vertex(&child).unwrap())
        .unwrap()
}

#[test]
fn native_handle_aliases_share_mutations_and_project_to_real_root() {
    use rustycad_tkshhealing::shape_fix::ShapeFixRoot;
    let (face, _, _) = two_wire_face(false);
    let fixer = Handle::new(ShapeFixSplitCommonVertex::new());
    let alias = fixer.clone();
    let root = fixer.cast::<ShapeFixRoot>();
    assert!(!root.is_null());
    fixer.init(&face.into_shape()).unwrap();
    assert_eq!(
        root.context().object_address(),
        fixer.context().object_address()
    );
    alias.perform().unwrap();
    assert_face_endpoint_identity(&fixer.shape().unwrap(), false);
    root.set_precision(0.25);
    assert_eq!(fixer.precision(), 0.25);
}
