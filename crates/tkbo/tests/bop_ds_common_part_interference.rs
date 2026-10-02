//! Regression tests for edge/edge and edge/face BOPDS common-part records.
use rustycad_tkbo::bop_ds::{InterfEE, InterfEF};
use rustycad_tkbo::int_tools::{CommonPrt, Range};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;

#[test]
fn edge_edge_assignment_copies_state_and_appends_native_range_sequence() {
    let mut first = common_part(ShapeEnum::Edge, (1.0, 2.0), &[(3.0, 4.0)], true);
    first.set_bounding_points(Point3::new(1.0, 2.0, 3.0), Point3::new(4.0, 5.0, 6.0));
    let mut interference = InterfEE::default();
    assert_eq!(interference.indices(), (-1, -1));
    assert_eq!(interference.common_part().shape_type(), ShapeEnum::Shape);
    interference.set_common_part(&first);
    assert_common_part(
        interference.common_part(),
        ShapeEnum::Edge,
        (1.0, 2.0),
        &[(3.0, 4.0)],
        true,
    );
    assert_eq!(
        interference.common_part().bounding_points(),
        (Point3::new(1.0, 2.0, 3.0), Point3::new(4.0, 5.0, 6.0))
    );

    let second = common_part(ShapeEnum::Vertex, (-1.0, -2.0), &[(7.0, 8.0)], false);
    interference.set_common_part(&second);
    assert_common_part(
        interference.common_part(),
        ShapeEnum::Vertex,
        (-1.0, -2.0),
        &[(3.0, 4.0), (7.0, 8.0)],
        false,
    );
}

#[test]
fn edge_face_assignment_keeps_its_common_part_and_base_indices() {
    let part = common_part(ShapeEnum::Edge, (0.25, 0.75), &[(-4.0, 8.0)], true);
    let mut interference = InterfEF::default();
    interference.set_indices(11, 19);
    interference.set_common_part(&part);
    assert_eq!(interference.indices(), (11, 19));
    assert_common_part(
        interference.common_part(),
        ShapeEnum::Edge,
        (0.25, 0.75),
        &[(-4.0, 8.0)],
        true,
    );
}

fn common_part(
    shape_type: ShapeEnum,
    range1: (f64, f64),
    ranges2: &[(f64, f64)],
    all_null: bool,
) -> CommonPrt {
    let mut part = CommonPrt::default();
    part.set_type(shape_type);
    part.set_range1(Range::new(range1.0, range1.1));
    part.set_vertex_parameter1(1.25);
    part.set_vertex_parameter2(-2.5);
    part.set_all_null_flag(all_null);
    for (first, last) in ranges2 {
        part.append_range2_bounds(*first, *last);
    }
    part.set_bounding_points(Point3::new(1.0, 2.0, 3.0), Point3::new(4.0, 5.0, 6.0));
    part
}

fn assert_common_part(
    part: &CommonPrt,
    shape_type: ShapeEnum,
    range1: (f64, f64),
    ranges2: &[(f64, f64)],
    all_null: bool,
) {
    assert_eq!(part.shape_type(), shape_type);
    assert_eq!(part.range1_bounds(), range1);
    assert_eq!(part.vertex_parameter1(), 1.25);
    assert_eq!(part.vertex_parameter2(), -2.5);
    assert_eq!(part.all_null_flag(), all_null);
    assert_eq!(
        part.ranges2().iter().map(Range::range).collect::<Vec<_>>(),
        ranges2
    );
}
