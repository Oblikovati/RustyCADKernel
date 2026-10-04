//! Native-backed bounded EdgeFace line/plane paths.
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
