//! Native-backed EdgeFace crossings, coincidence and curve/surface tangencies.
use rustycad_tkbo::int_tools::{EdgeFace, Range};
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Plane3, Point3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace};

fn edge(first: Point3, last: Point3) -> Edge {
    BRepBuilderAPIMakeEdge::from_points(first, last)
        .unwrap()
        .edge()
        .unwrap()
}

fn face() -> Face {
    BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (-5., 5., -5., 5.))
        .unwrap()
        .face()
        .unwrap()
}

#[test]
fn crossing_line_reports_native_vertex_common_part() {
    let mut solver = EdgeFace::new();
    solver.set_edge(&edge(Point3::new(0., 0., -1.), Point3::new(0., 0., 1.)));
    solver.set_face(&face());
    solver.set_range(&Range::new(0., 1.));
    solver.perform().unwrap();

    assert!(solver.is_done());
    assert_eq!(solver.error_status(), 0);
    assert_eq!(solver.common_parts().length(), 1);
    let part = solver.common_parts().value(1).unwrap();
    assert_eq!(part.shape_type(), ShapeEnum::Vertex);
    let (first, last) = part.range1_bounds();
    assert!((first - 0.999_999_85).abs() < 1e-12);
    assert_eq!(last, 1.);
    assert_eq!(part.vertex_parameter1(), 1.);
    let (first_point, last_point) = part.bounding_points();
    assert!((first_point.z() + 1.5e-7).abs() < 1e-15);
    assert_eq!(last_point, Point3::new(0., 0., 0.));
    assert_eq!(solver.minimal_distance(), f64::MAX);
}

#[test]
fn quick_coincidence_returns_the_selected_edge_range() {
    let mut solver = EdgeFace::new();
    solver.set_edge(&edge(Point3::new(-1., 0., 0.), Point3::new(1., 0., 0.)));
    solver.set_face(&face());
    solver.set_range(&Range::new(0., 1.));
    solver.use_quick_coincidence_check(true);
    solver.perform().unwrap();

    assert!(solver.is_done());
    assert_eq!(solver.error_status(), 0);
    assert_eq!(solver.common_parts().length(), 1);
    let part = solver.common_parts().value(1).unwrap();
    assert_eq!(part.shape_type(), ShapeEnum::Edge);
    assert_eq!(part.range1_bounds(), (0., 1.));
    assert_eq!(
        part.bounding_points(),
        (Point3::default(), Point3::default())
    );
}

fn tangent_cylinder_pair() -> (Edge, Face) {
    use rustycad_tkmath::gp::{Axis3, Cylinder3};
    let cylinder = Cylinder3::new(Axis3::default(), 1.).unwrap();
    let face = BRepBuilderAPIMakeFace::from_cylinder_bounds(
        cylinder,
        (-std::f64::consts::PI, std::f64::consts::PI, -2., 2.),
    )
    .unwrap()
    .face()
    .unwrap();
    (
        edge(Point3::new(-2., 1., 0.), Point3::new(2., 1., 0.)),
        face,
    )
}

fn tangent_circle_pair() -> (Edge, Face) {
    use rustycad_tkmath::gp::{Axis2, Circle3, Direction3};
    let circle = Circle3::new(Axis2::default(), 1.).unwrap();
    let edge = BRepBuilderAPIMakeEdge::from_circ_params(circle, 0., std::f64::consts::TAU)
        .unwrap()
        .edge()
        .unwrap();
    let plane = Plane3::new(
        Point3::new(1., 0., 0.),
        Direction3::new(1., 0., 0.).unwrap(),
    )
    .unwrap();
    let face = BRepBuilderAPIMakeFace::from_plane_bounds(plane, (-2., 2., -2., 2.))
        .unwrap()
        .face()
        .unwrap();
    (edge, face)
}

fn tangent_snapshots(label: &str, edge: &Edge, face: &Face, last: f64) -> String {
    let mut solver = EdgeFace::new();
    solver.set_edge(edge);
    solver.set_face(face);
    solver.set_range(&Range::new(0., last));
    solver.perform().unwrap();
    let mut rows = format!(
        "{label} {} {} {} {}\n",
        i32::from(solver.is_done()),
        solver.error_status(),
        solver.common_parts().length(),
        solver.minimal_distance()
    );
    for part in solver.common_parts().iter() {
        rows.push_str(&common_part_snapshot(part));
    }
    rows
}

fn common_part_snapshot(part: &rustycad_tkbo::int_tools::CommonPrt) -> String {
    let (first, last) = part.range1_bounds();
    let (start, end) = part.bounding_points();
    let coordinates = [start.x(), start.y(), start.z(), end.x(), end.y(), end.z()];
    let mut row = format!(
        "part {} {first} {last} {}",
        part.shape_type() as i32,
        part.vertex_parameter1()
    );
    for coordinate in coordinates {
        row.push_str(&format!(" {coordinate}"));
    }
    row.push('\n');
    row
}

#[test]
fn tangent_curve_surface_cases_match_native_public_results() {
    let (line, cylinder) = tangent_cylinder_pair();
    let mut rows = tangent_snapshots("line-cylinder-touch", &line, &cylinder, 4.);
    let (circle, plane) = tangent_circle_pair();
    rows.push_str(&tangent_snapshots(
        "circle-plane-touch",
        &circle,
        &plane,
        std::f64::consts::TAU,
    ));
    let native = include_str!("../../../tools/oracle/tkbo_edge_face.expected");
    let expected = &native[native.find("line-cylinder-touch").unwrap()..];
    assert_native_tokens(&rows, expected);
}

fn assert_native_tokens(rows: &str, expected: &str) {
    let actual_tokens: Vec<_> = rows.split_whitespace().collect();
    let expected_tokens: Vec<_> = expected.split_whitespace().collect();
    assert_eq!(actual_tokens.len(), expected_tokens.len());
    for (actual, expected) in actual_tokens.iter().zip(expected_tokens) {
        if let Ok(reference) = expected.parse::<f64>() {
            assert!(
                (actual.parse::<f64>().unwrap() - reference).abs() <= 1e-12,
                "{actual} != {expected}"
            );
        } else {
            assert_eq!(*actual, expected);
        }
    }
}
