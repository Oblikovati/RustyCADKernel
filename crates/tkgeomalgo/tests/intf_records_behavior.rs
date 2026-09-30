//! Behavior of the Intf records (section points, lines, tangent zones and the
//! Interference insertions) and of Intf_Tool, beyond the upstream suite; the bits are
//! compared with OCCT by the numeric probe.
use rustycad_tkgeomalgo::intf::{
    Interference, PIType, SectionLine, SectionPoint, TangentZone, Tool,
};
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::{Direction2, Line2, Point2, Point3};

/// A point on edge `index` of both arguments, at `x` along the first.
fn edge_point(index: i32, x: f64) -> SectionPoint {
    SectionPoint::with_data(
        Point3::new(x, 0.0, 0.0),
        PIType::Edge,
        0,
        index,
        0.5,
        PIType::Edge,
        0,
        index,
        0.5,
        1.0,
    )
}

#[test]
fn section_point_merge_keeps_the_higher_dimension_on_each_side() {
    let mut vertex = SectionPoint::with_data_2d(
        Point2::new(1.0, 2.0),
        PIType::Vertex,
        3,
        0.0,
        PIType::Face,
        4,
        0.0,
        1.0,
    );
    let mut edge = SectionPoint::with_data_2d(
        Point2::new(5.0, 6.0),
        PIType::Edge,
        7,
        0.5,
        PIType::Edge,
        8,
        0.25,
        1.0,
    );
    vertex.merge(&mut edge);
    assert_eq!(edge.pnt(), Point3::new(1.0, 2.0, 0.0));
    assert_eq!(
        (edge.info_first_address(), vertex.info_second_address()),
        ((PIType::Vertex, 3, 0.0), (PIType::Edge, 8, 0.25))
    );
    assert!(vertex.is_equal(&edge));
}

#[test]
fn section_line_is_end_returns_the_length_for_the_last_point() {
    let mut line = SectionLine::new();
    assert_eq!(
        line.is_closed().unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
    (0..3).for_each(|i| line.append(edge_point(i, f64::from(i))));
    assert_eq!(
        (
            line.is_end(&edge_point(0, 9.0)).unwrap(),
            line.is_end(&edge_point(2, 9.0)).unwrap()
        ),
        (1, 3)
    );
    let mut text = String::new();
    line.dump(1, &mut text).unwrap();
    assert_eq!(text, " LS Open :\n");
}

#[test]
fn section_line_append_line_empties_the_argument_and_copies_drop_the_flag() {
    let (mut first, mut second) = (SectionLine::new(), SectionLine::new());
    first.append(edge_point(0, 0.0));
    second.append(edge_point(1, 1.0));
    first.append_line(&mut second);
    assert_eq!(
        (first.number_of_points(), second.number_of_points()),
        (2, 0)
    );
    first.close();
    assert!(first.clone() == first);
}

#[test]
fn tangent_zone_polygon_insert_puts_inner_points_first() {
    let mut zone = TangentZone::new();
    zone.polygon_insert(edge_point(1, 0.0));
    zone.polygon_insert(edge_point(3, 0.0));
    zone.polygon_insert(edge_point(2, 0.0));
    let first: i32 = zone.get_point(1).unwrap().info_first_address().1;
    assert_eq!((zone.number_of_points(), first), (3, 2));
    assert_eq!(zone.param_on_first(), (0.5, 2.5));
    assert_eq!(zone.info_first(), (0, 0.5, 2, 0.5));
    assert!(!zone.insert(&edge_point(4, 0.0)));
    assert_eq!(
        zone.insert_before(0, edge_point(4, 0.0))
            .unwrap_err()
            .exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn interference_joins_section_segments_into_one_line() {
    let mut interference = Interference::new(false);
    let points: Vec<SectionPoint> = (0..4).map(|i| edge_point(i, f64::from(i))).collect();
    interference.insert_points(&points[0], &points[1]).unwrap();
    interference.insert_points(&points[2], &points[3]).unwrap();
    interference.insert_points(&points[3], &points[1]).unwrap();
    assert_eq!(interference.nb_section_lines(), 1);
    assert_eq!(interference.line_value(1).unwrap().number_of_points(), 4);
    assert!(interference.contains(&points[2]));
}

#[test]
fn interference_insert_zone_needs_a_known_zone_sharing_a_point() {
    let mut interference = Interference::new(false);
    let mut zone = TangentZone::new();
    zone.append(edge_point(0, 0.0));
    zone.append(edge_point(1, 1.0));
    assert!(!interference.insert_zone(&zone).unwrap());
    interference.tangent_zones_mut().append(zone.clone());
    let mut other = TangentZone::new();
    other.append(edge_point(1, 1.0));
    other.append(edge_point(2, 2.0));
    assert!(interference.insert_zone(&other).unwrap());
    assert_eq!(interference.nb_tangent_zones(), 1);
    // The shared point is inserted again when the walk comes back to it, as natively.
    assert_eq!(interference.zone_value(1).unwrap().number_of_points(), 5);
}

#[test]
fn tool_clips_a_diagonal_line_to_the_domain() {
    let mut domain = BoundingBox2::default();
    domain.update_bounds([0.0, 0.0], [2.0, 1.0]);
    let mut tool = Tool::new();
    let mut clipped = BoundingBox2::default();
    let line = Line2::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 1.0).unwrap());
    tool.lin2d_box(line, &domain, &mut clipped);
    let [x0, y0, x1, y1] = clipped.get().unwrap();
    assert_eq!(tool.nb_segments(), 1);
    assert!((x0, y0) == (0.0, 0.0) && (x1 - 1.0).abs() < 1e-15 && (y1 - 1.0).abs() < 1e-15);
    assert_eq!(
        tool.end_param(2).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
}
