//! Differential replay of BRepOffset_Analyse box, face and spline incidence.
mod spline_fixture;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topexp::Explorer,
    topods::{self, Compound, Shape},
};
use rustycad_tkfillet::chfi_ds::TypeOfConcavity;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tkoffset::brep_offset::BRepOffsetAnalyse;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeShape};
#[test]
fn native_analyse_incidence_and_classifications_match() {
    let shapes = fixture_shapes();
    let analyses: Vec<BRepOffsetAnalyse> = shapes
        .iter()
        .map(|s| BRepOffsetAnalyse::with_shape(s, 0.01).unwrap())
        .collect();
    for row in include_str!("../../../tools/oracle/tkoffset_analyse.expected").lines() {
        let fields: Vec<&str> = row.split_whitespace().collect();
        let kind: usize = fields[1].parse().unwrap();
        replay_row(&fields, &shapes[kind], &analyses[kind]);
    }
}
fn fixture_shapes() -> Vec<Shape> {
    let shape = BRepPrimAPIMakeBox::from_sizes(1., 2., 3.)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    let face =
        Iterator::next(&mut Explorer::new(&shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap())
            .unwrap();
    let edge = BRepBuilderAPIMakeEdge::from_points(Point3::default(), Point3::new(2., 0., 0.))
        .unwrap()
        .edge()
        .unwrap();
    let mut empty = Compound::default();
    BRepBuilder::default().make_compound(&mut empty);
    vec![
        shape.clone(),
        shape.reversed(),
        face,
        edge.into_shape(),
        empty.into_shape(),
        spline_fixture::pair(0.),
        spline_fixture::pair(1.),
        spline_fixture::pair(3.),
    ]
}
fn replay_row(fields: &[&str], shape: &Shape, analysis: &BRepOffsetAnalyse) {
    match fields[0] {
        "state" => {
            assert_eq!(analysis.is_done(), fields[2] == "1");
            assert_eq!(analysis.new_faces().len(), number(fields[3]));
        }
        "edge" => replay_edge(fields, shape, analysis),
        "vertex" => replay_vertex(fields, shape, analysis),
        "face" => replay_face(fields, shape, analysis),
        "tangent" => replay_tangent(fields, shape, analysis),
        "clear" => {
            let mut analysis = BRepOffsetAnalyse::with_shape(shape, 0.01).unwrap();
            analysis.clear();
            assert!(!analysis.is_done());
            assert!(!analysis.has_ancestor(shape));
        }
        other => panic!("record {other}; expected state, edge, vertex or clear"),
    }
}
fn replay_edge(fields: &[&str], shape: &Shape, analysis: &BRepOffsetAnalyse) {
    let edge = Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)
        .unwrap()
        .nth(number(fields[2]))
        .unwrap();
    let edge = topods::edge(&edge).unwrap();
    let intervals = analysis.type_of(&edge).unwrap();
    assert_eq!(analysis.ancestors(&edge).unwrap().len(), number(fields[3]));
    assert_eq!(intervals.len(), number(fields[4]));
    for (index, interval) in intervals.iter().enumerate() {
        assert_eq!(
            interval.first().unwrap(),
            fields[5 + index * 3].parse::<f64>().unwrap()
        );
        assert_eq!(
            interval.last().unwrap(),
            fields[6 + index * 3].parse::<f64>().unwrap()
        );
        assert_eq!(
            interval.get_type().unwrap() as usize,
            number(fields[7 + index * 3])
        );
    }
    assert_eq!(
        analysis.has_generated(&edge),
        *fields.last().unwrap() == "1"
    );
}
fn replay_vertex(fields: &[&str], shape: &Shape, analysis: &BRepOffsetAnalyse) {
    let vertex = Explorer::new(shape, ShapeEnum::Vertex, ShapeEnum::Shape)
        .unwrap()
        .nth(number(fields[2]))
        .unwrap();
    let vertex = topods::vertex(&vertex).unwrap();
    assert_eq!(
        analysis.ancestors(&vertex).unwrap().len(),
        number(fields[3])
    );
    let types = concavity_types();
    for (index, kind) in types.into_iter().enumerate() {
        let mut edges = vec![Shape::default()];
        analysis.edges_on_vertex(&vertex, kind, &mut edges).unwrap();
        assert_eq!(edges.len(), number(fields[4 + index]), "{fields:?}");
    }
}
fn number(value: &str) -> usize {
    value.parse().unwrap()
}

fn replay_face(fields: &[&str], shape: &Shape, analysis: &BRepOffsetAnalyse) {
    let face = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)
        .unwrap()
        .nth(number(fields[2]))
        .unwrap();
    let face = topods::face(&face).unwrap();
    for (index, kind) in concavity_types().into_iter().enumerate() {
        let mut edges = vec![Shape::default()];
        analysis.edges_on_face(&face, kind, &mut edges).unwrap();
        assert_eq!(edges.len(), number(fields[3 + index]));
    }
}
fn replay_tangent(fields: &[&str], shape: &Shape, analysis: &BRepOffsetAnalyse) {
    let edge = Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)
        .unwrap()
        .nth(number(fields[2]))
        .unwrap();
    let edge = topods::edge(&edge).unwrap();
    let vertex = rustycad_tkbrep::topexp::first_vertex(&edge, false);
    let mut edges = vec![edge.clone().into_shape()];
    analysis.tangent_edges(&edge, &vertex, &mut edges).unwrap();
    assert_eq!(edges.len(), number(fields[3]));
}
fn concavity_types() -> [TypeOfConcavity; 6] {
    [
        TypeOfConcavity::Concave,
        TypeOfConcavity::Convex,
        TypeOfConcavity::Tangential,
        TypeOfConcavity::FreeBound,
        TypeOfConcavity::Other,
        TypeOfConcavity::Mixed,
    ]
}
