//! Native occurrence set behavior, including asymmetric duplicate equality.
use rustycad_tkbo::bop_tools::ShapeSet;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topexp::Explorer,
    topods::{Compound, Edge, Shape, Vertex},
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Point3;
struct Members {
    first: Vertex,
    second: Vertex,
    edge: Edge,
}
fn members() -> Members {
    let builder = BRepBuilder::default();
    let mut first = Vertex::default();
    let mut second = Vertex::default();
    let mut edge = Edge::default();
    builder
        .make_vertex_at(&mut first, Point3::default(), 1e-7)
        .unwrap();
    builder
        .make_vertex_at(&mut second, Point3::new(1., 0., 0.), 1e-7)
        .unwrap();
    builder.make_edge(&mut edge).unwrap();
    builder.degenerated(&edge, true).unwrap();
    builder.add(&mut edge, &first).unwrap();
    Members {
        first,
        second,
        edge,
    }
}
fn fixture(mode: usize, members: &Members) -> Shape {
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    let shapes: Vec<Shape> = match mode {
        0 => vec![
            members.first.clone().into_shape(),
            members.second.clone().into_shape(),
        ],
        1 => vec![members.first.clone().into_shape(); 2],
        2 => vec![
            members.second.clone().into_shape(),
            members.first.clone().into_shape(),
        ],
        3 => vec![members.first.oriented(Orientation::Internal)],
        4 => vec![members.edge.clone().into_shape()],
        5 => vec![
            members.edge.clone().into_shape(),
            members.edge.oriented(Orientation::Internal),
        ],
        _ => vec![],
    };
    for shape in shapes {
        builder.add(&mut compound, &shape).unwrap();
    }
    compound.into_shape()
}
#[test]
fn native_shape_set_occurrence_counts_and_equality_match() {
    let fixture = include_str!("../../../tools/oracle/tkbo_shape_set.expected");
    assert_eq!(fixture.lines().count(), 392);
    for line in fixture.lines() {
        let row: Vec<usize> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        assert_eq!(observe(&row), row[4..], "{line}");
    }
}
fn observe(row: &[usize]) -> Vec<usize> {
    let members = members();
    let mut first = fixture(row[0], &members);
    let second = fixture(row[1], &members);
    first.set_orientation(
        [
            Orientation::Forward,
            Orientation::Reversed,
            Orientation::Internal,
            Orientation::External,
        ][row[3]],
    );
    let kind = if row[2] == 6 {
        ShapeEnum::Edge
    } else {
        ShapeEnum::Vertex
    };
    let (mut one, mut two) = (ShapeSet::new(), ShapeSet::new());
    one.add(&first, kind).unwrap();
    two.add(&second, kind).unwrap();
    let copied = one.clone();
    let mut assigned = ShapeSet::new();
    assigned.assign(&two);
    let mut actual = vec![
        one.nb_shapes() as usize,
        two.nb_shapes() as usize,
        usize::from(one.is_equal(&two)),
        usize::from(two.is_equal(&one)),
    ];
    actual.extend([
        usize::from(one.get_sum() == expected_sum(&first, kind)),
        usize::from(two.get_sum() == expected_sum(&second, kind)),
        usize::from(one.shape().is_equal(&first)),
        usize::from(copied.is_equal(&one)),
        usize::from(assigned.is_equal(&two)),
    ]);
    one.clear();
    actual.extend([
        one.nb_shapes() as usize,
        one.get_sum(),
        usize::from(one.shape().is_equal(&first)),
        copied.nb_shapes() as usize,
    ]);
    actual
}
fn expected_sum(shape: &Shape, kind: ShapeEnum) -> usize {
    Explorer::new(shape, kind, ShapeEnum::Shape)
        .unwrap()
        .filter_map(|shape| {
            if kind == ShapeEnum::Edge
                && BRepTool::degenerated(&Edge::unchecked(shape.clone())).unwrap()
            {
                return None;
            }
            Some(
                (shape.hash_code() % 432123 + 1)
                    * if shape.orientation() == Orientation::Internal {
                        2
                    } else {
                        1
                    },
            )
        })
        .sum()
}
