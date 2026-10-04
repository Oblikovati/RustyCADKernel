use super::*;
use rustycad_tkbo::bop_tools;
use rustycad_tkbrep::topods::{Shape, Vertex};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
#[test]
fn native_empty_container_replacement_matches() {
    let kinds = [
        ShapeEnum::Compound,
        ShapeEnum::CompSolid,
        ShapeEnum::Solid,
        ShapeEnum::Shell,
        ShapeEnum::Face,
        ShapeEnum::Wire,
        ShapeEnum::Edge,
        ShapeEnum::Vertex,
        ShapeEnum::Shape,
    ];
    for line in include_str!("../../../../tools/oracle/tkbo_construction.expected").lines() {
        let row: Vec<usize> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        let mut shape = container_initial(row[1] != 0);
        let original = shape.clone();
        bop_tools::make_container(kinds[row[0]], &mut shape);
        let mut actual = vec![
            usize::from(shape.is_null()),
            usize::from(shape.is_same(&original)),
        ];
        if !shape.is_null() {
            actual.extend([
                shape.shape_type().unwrap() as usize,
                shape.orientation() as usize,
            ]);
        }
        assert_eq!(actual, row[2..], "{line}");
    }
}
fn container_initial(present: bool) -> Shape {
    if !present {
        return Shape::default();
    }
    let mut vertex = Vertex::default();
    BRepBuilder::default().make_vertex(&mut vertex);
    vertex.set_orientation(Orientation::Reversed);
    vertex.into_shape()
}
#[test]
fn native_edge_point_evaluation_matches() {
    for line in include_str!("../../../../tools/oracle/tkbo_point_on_edge.expected").lines() {
        let row: Vec<f64> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        let (fixture, edge) = edge_fixture(row[0] as usize);
        let parameter = fixture.range.0 + (fixture.range.1 - fixture.range.0) * row[1] / 2.;
        let mut point = Point3::new(9., 8., 7.);
        bop_tools::point_on_edge(&edge, parameter, &mut point).unwrap();
        for (actual, expected) in [point.x(), point.y(), point.z()].iter().zip(&row[2..]) {
            assert!((actual - expected).abs() < 1e-12, "{line}; actual {actual}");
        }
    }
}
