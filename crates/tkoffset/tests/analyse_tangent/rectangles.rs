use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Compound, Edge, Shape, Vertex, Wire},
};
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeVertex,
};
pub(super) fn pair(split: bool) -> Shape {
    let vertices: Vec<Vertex> = [
        (0., 0.),
        (1., 0.),
        (2., 0.),
        (0., 1.),
        (1., 1.),
        (2., 1.),
        (1., 0.5),
    ]
    .into_iter()
    .map(|(x, y)| {
        BRepBuilderAPIMakeVertex::new(Point3::new(x, y, 0.))
            .unwrap()
            .vertex()
            .unwrap()
    })
    .collect();
    let common: Vec<Edge> = if split {
        vec![edge(&vertices, 1, 6), edge(&vertices, 6, 4)]
    } else {
        vec![edge(&vertices, 1, 4)]
    };
    let mut left = vec![edge(&vertices, 0, 1).into()];
    left.extend(common.iter().cloned().map(Shape::from));
    left.extend([edge(&vertices, 4, 3).into(), edge(&vertices, 3, 0).into()]);
    let mut right = vec![
        edge(&vertices, 1, 2).into(),
        edge(&vertices, 2, 5).into(),
        edge(&vertices, 5, 4).into(),
    ];
    right.extend(common.iter().rev().map(|edge| edge.reversed()));
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    for edges in [left, right] {
        let mut wire = Wire::default();
        builder.make_wire(&mut wire);
        for edge in edges {
            builder.add(&mut wire, &edge).unwrap();
        }
        let face = BRepBuilderAPIMakeFace::from_wire(&wire, true)
            .unwrap()
            .face()
            .unwrap();
        builder.add(&mut compound, &face).unwrap();
    }
    compound.into()
}
fn edge(vertices: &[Vertex], first: usize, last: usize) -> Edge {
    BRepBuilderAPIMakeEdge::from_vertices(&vertices[first], &vertices[last])
        .unwrap()
        .edge()
        .unwrap()
}
