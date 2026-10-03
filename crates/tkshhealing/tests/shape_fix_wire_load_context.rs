//! Context-aware ShapeFixWire loading from a wire and shared wire data.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{self, Builder, Compound, Edge, Vertex, Wire};
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::Point3;
use rustycad_tkshhealing::shape_build::ShapeBuildReShape;
use rustycad_tkshhealing::shape_extend::ShapeExtendWireData;
use rustycad_tkshhealing::shape_fix::{FixRoot, ShapeFixWire};

const NATIVE_RESULT: &str = include_str!("../../../tools/oracle/tkshhealing_wire_load.expected");

#[test]
fn wire_load_applies_context_replacement_before_analyzer_load() {
    let (wire, old_edge, first, second, context) = context_replacement();
    let fixer = ShapeFixWire::new();
    fixer.set_context(&context);
    fixer.load(&wire).unwrap();
    let data = fixer.wire_data().unwrap();
    assert_eq!(data.nb_edges(), 2);
    assert!(data.edge(1).unwrap().is_same(&first));
    assert!(data.edge(2).unwrap().is_same(&second));
    assert!(!data.edge(1).unwrap().is_same(&old_edge));
    assert!(fixer.is_loaded());
    assert!(
        NATIVE_RESULT
            .lines()
            .any(|line| line == "wire_count=2 wire_first=1 wire_second=1")
    );
}

#[test]
fn wire_data_load_splices_context_edges_into_the_shared_sequence() {
    let (_, old_edge, first, second, context) = context_replacement();
    let data = Handle::new(ShapeExtendWireData::new());
    data.add_edge(&old_edge, 0).unwrap();
    let fixer = ShapeFixWire::new();
    fixer.set_context(&context);
    fixer.load_wire_data(&data).unwrap();
    assert_eq!(data.nb_edges(), 2);
    assert!(data.edge(1).unwrap().is_same(&first));
    assert!(data.edge(2).unwrap().is_same(&second));
    assert!(
        NATIVE_RESULT
            .lines()
            .any(|line| line == "data_count=2 data_first=1 data_second=1")
    );
}

fn context_replacement() -> (Wire, Edge, Edge, Edge, Handle<ShapeBuildReShape>) {
    let old_edge = edge_between(0.0, 10.0);
    let first = edge_between(0.0, 5.0);
    let second = edge_between(5.0, 10.0);
    let wire = wire_containing(&old_edge);
    let compound = compound_containing(&first, &second);
    let context = Handle::new(ShapeBuildReShape::new());
    rustycad_tkbrep::brep_tools::ReShape::replace(
        context.get().unwrap(),
        &old_edge.clone().into_shape(),
        &compound.into_shape(),
    )
    .unwrap();
    (wire, old_edge, first, second, context)
}

fn edge_between(first_x: f64, last_x: f64) -> Edge {
    let builder = BRepBuilder::default();
    let mut first = Vertex::default();
    let mut last = Vertex::default();
    builder
        .make_vertex_at(&mut first, Point3::new(first_x, 0.0, 0.0), 1e-7)
        .unwrap();
    builder
        .make_vertex_at(&mut last, Point3::new(last_x, 0.0, 0.0), 1e-7)
        .unwrap();
    let mut edge = Edge::default();
    builder.make_edge(&mut edge).unwrap();
    let mut shape = edge.clone().into_shape();
    let topology = Builder;
    topology
        .add(&mut shape, &first.clone().into_shape())
        .unwrap();
    topology
        .add(&mut shape, &last.clone().into_shape().reversed())
        .unwrap();
    topods::edge(&shape).unwrap()
}

fn wire_containing(edge: &Edge) -> Wire {
    let topology = Builder;
    let mut wire = Wire::default();
    topology.make_wire(&mut wire);
    let mut shape = wire.into_shape();
    topology
        .add(&mut shape, &edge.clone().into_shape())
        .unwrap();
    topods::wire(&shape).unwrap()
}

fn compound_containing(first: &Edge, second: &Edge) -> Compound {
    let topology = Builder;
    let mut compound = Compound::default();
    topology.make_compound(&mut compound);
    let mut shape = compound.into_shape();
    topology
        .add(&mut shape, &first.clone().into_shape())
        .unwrap();
    topology
        .add(&mut shape, &second.clone().into_shape())
        .unwrap();
    topods::compound(&shape).unwrap()
}
