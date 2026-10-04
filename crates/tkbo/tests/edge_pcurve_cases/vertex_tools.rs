use super::*;
use rustycad_tkbo::bop_tools;
use rustycad_tkbrep::{brep::BRepTool, topods::Vertex};
use rustycad_tkernel::ncollection::List;
use rustycad_tkmath::gp::Point3;
fn vertex(point: Point3, tolerance: f64) -> Vertex {
    let mut result = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut result, point, tolerance)
        .unwrap();
    result
}
fn sentinel() -> Vertex {
    let mut result = vertex(Point3::new(9., 8., 7.), 0.25);
    result.set_orientation(Orientation::Reversed);
    result
}
fn fields(line: &str) -> Vec<f64> {
    line.split_whitespace()
        .map(|s| s.parse().unwrap())
        .collect()
}
fn assert_vertex(line: &str, output: &Vertex, initial: &Vertex) {
    let point = BRepTool::pnt(output).unwrap();
    let expected = fields(line);
    let actual = [
        point.x(),
        point.y(),
        point.z(),
        BRepTool::vertex_tolerance(output).unwrap(),
        f64::from(output.is_same(initial)),
        output.orientation() as u8 as f64,
    ];
    for (actual, expected) in actual.iter().zip(&expected[1..]) {
        assert!((actual - expected).abs() < 1e-12, "{line}: actual {actual}");
    }
}
#[test]
fn native_vertex_tolerance_sphere_intersections_match() {
    for line in include_str!("../../../../tools/oracle/tkbo_vertex_compare.expected").lines() {
        let row = fields(line);
        let tolerance = [1e-7, 0.01, 1.][row[0] as usize];
        let fuzz: f64 = [-1., 0., 0.1][row[1] as usize];
        let sum = tolerance + 2. * tolerance + fuzz.max(1e-7);
        let distance = [0., sum - 1e-12, sum, sum + 1e-12][row[2] as usize];
        let first = vertex(Point3::default(), tolerance);
        let point = Point3::new(distance, 0., 0.);
        let second = vertex(point, 2. * tolerance);
        assert_eq!(
            bop_tools::compute_vertices(&first, &second, fuzz).unwrap(),
            row[3] as i32,
            "{line}"
        );
        assert_eq!(
            bop_tools::compute_vertex_point(&first, point, 2. * tolerance).unwrap(),
            row[4] as i32,
            "{line}"
        );
        assert_eq!(bop_tools::d_tolerance(), row[5]);
    }
}
#[test]
fn native_bounding_vertex_empty_singleton_and_lists_match() {
    for line in include_str!("../../../../tools/oracle/tkbo_vertex_list.expected").lines() {
        let count = fields(line)[0] as usize;
        let mut vertices = List::default();
        for index in 0..count {
            let i = index as f64;
            let mut item = vertex(Point3::new(i, (index % 2) as f64, -i), 0.1 * (i + 1.));
            if index == 0 {
                item.set_orientation(Orientation::Reversed);
            }
            vertices.append(item.into_shape());
        }
        let initial = sentinel();
        let mut output = initial.clone();
        bop_tools::make_vertex(&vertices, &mut output).unwrap();
        assert_vertex(line, &output, &initial);
        if count == 1 {
            assert!(output.is_same(&Vertex::unchecked(vertices.first().unwrap().clone())));
        }
    }
}
#[test]
fn native_new_vertex_overloads_match() {
    for line in include_str!("../../../../tools/oracle/tkbo_vertex_pair.expected").lines() {
        let kind = fields(line)[0] as usize;
        let initial = sentinel();
        let mut output = initial.clone();
        construct_vertex(kind, &mut output);
        assert_vertex(line, &output, &initial);
    }
}
fn construct_vertex(kind: usize, output: &mut Vertex) {
    let (fixture, edge) = edge_fixture(kind % 5);
    let (other_fixture, other) = edge_fixture((kind + 1) % 5);
    match kind {
        0..5 => bop_tools::make_new_vertex_edges(
            &edge,
            fixture.range.0,
            &other,
            other_fixture.range.1,
            output,
        ),
        5..10 => {
            bop_tools::make_new_vertex_edge_face(&edge, fixture.range.0, &fixture.face, output)
        }
        10 => bop_tools::make_new_vertex_pair(
            &vertex(Point3::new(-1., 2., 3.), 0.1),
            &vertex(Point3::new(4., -2., 1.), 0.3),
            output,
        ),
        _ => bop_tools::make_new_vertex_point(Point3::new(-1., 2., 3.), 0.1, output),
    }
    .unwrap();
}
