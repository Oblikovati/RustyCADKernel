//! PrsDim package functions and dimensions: geometry extraction, values and validity.
use rustycad_tkbrep::topods::Edge;
use rustycad_tkmath::gp::{self, Axis3, Ellipse3, Plane3, Point3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
use rustycad_tkv3d::prs_dim::*;
use std::f64::consts::PI;

fn segment(a: Point3, b: Point3) -> Edge {
    BRepBuilderAPIMakeEdge::from_points(a, b).unwrap().edge().unwrap()
}

#[test]
fn package_projections_and_domains_match_the_native_formulas() {
    let plane = Plane3::new(Point3::new(0.0, 0.0, 2.0), gp::dz()).unwrap();
    let projected = PrsDim::project_point_on_plane(&Point3::new(1.0, 2.0, 7.0), &plane);
    assert_eq!(projected, Point3::new(1.0, 2.0, 2.0));
    let line = rustycad_tkmath::gp::Line3::new(gp::origin(), gp::dx());
    assert_eq!(PrsDim::project_point_on_line(&Point3::new(3.0, 4.0, 0.0), &line), Point3::new(3.0, 0.0, 0.0));
    assert!(PrsDim::in_domain(0.5, 1.5, 1.0));
    assert!(!PrsDim::in_domain(0.5, 1.5, 2.0));
    // A wrapping arc from 3/2 pi to pi/2 contains zero.
    assert!(PrsDim::in_domain(1.5 * PI, 0.5 * PI, 0.1));
    let ellipse = Ellipse3::new(gp::xoy(), 3.0, 1.0).unwrap();
    let apex = Point3::new(3.0, 0.0, 0.0);
    assert_eq!(PrsDim::distance_from_apex(&ellipse, &apex, 0.5).unwrap(), 0.5);
    assert_eq!(PrsDim::distance_from_apex(&ellipse, &apex, 1.5 * PI).unwrap(), 0.5 * PI);
}

#[test]
fn edge_geometry_reports_lines_and_their_ends() {
    let edge = segment(Point3::new(1.0, 0.0, 0.0), Point3::new(4.0, 0.0, 0.0));
    let (_, ends) = PrsDim::compute_edge_geometry(&edge).unwrap().unwrap();
    assert_eq!(ends, (Point3::new(1.0, 0.0, 0.0), Point3::new(4.0, 0.0, 0.0)));
    let (_, ends, infinite) = PrsDim::compute_edge_geometry_infinite(&edge).unwrap().unwrap();
    assert!(!infinite);
    assert_eq!(ends.1, Point3::new(4.0, 0.0, 0.0));
}

#[test]
fn circular_dimensions_measure_radius_and_diameter() {
    use rustycad_tkmath::gp::{Axis2, Circle3};
    let circle = Circle3::new(Axis2::new(Point3::new(1.0, 2.0, 0.0), gp::dz()).unwrap(), 3.0).unwrap();
    let radius = PrsDimRadiusDimension::from_circle(&circle).unwrap();
    assert!(radius.is_valid());
    assert_eq!(radius.get_value(), 3.0);
    assert_eq!(radius.anchor_point(), Point3::new(4.0, 2.0, 0.0));
    assert_eq!(radius.special_symbol(), u16::from(b'R'));
    assert_eq!(radius.display_special_symbol(), PrsDimDisplaySpecialSymbol::Before);
    // An anchor at the center is invalid.
    let centered = PrsDimRadiusDimension::from_circle_anchor(&circle, &circle.location()).unwrap();
    assert!(!centered.is_valid());
    let diameter = PrsDimDiameterDimension::from_circle(&circle).unwrap();
    assert_eq!(diameter.get_value(), 6.0);
    assert_eq!(diameter.special_symbol(), 0x00D8);
    let (first, second) = diameter.compute_side_points(&circle);
    assert_eq!((first, second), (Point3::new(4.0, 2.0, 0.0), Point3::new(-2.0, 2.0, 0.0)));
    // A plane missing the center is invalid.
    let off = Plane3::new(Point3::new(0.0, 0.0, 5.0), gp::dz()).unwrap();
    let off_plane = PrsDimDiameterDimension::from_circle_plane(&circle, &off).unwrap();
    assert!(!off_plane.is_valid());
}

#[test]
fn length_dimension_measures_points_and_parallel_edges() {
    let xoy = Plane3::from_position(Axis3::from_axis2(gp::xoy()));
    let points = PrsDimLengthDimension::from_points(&gp::origin(), &Point3::new(3.0, 4.0, 0.0), &xoy).unwrap();
    assert!(points.is_valid());
    assert_eq!(points.get_value(), 5.0);
    assert_eq!(points.get_flyout(), 15.0);
    assert_eq!(points.kind_of_dimension(), PrsDimKindOfDimension::Length);
    points.set_direction(&gp::dx(), true);
    assert_eq!(points.get_value(), 3.0);
    points.set_custom_value(7.5);
    assert_eq!(points.get_value(), 7.5);
    let same = PrsDimLengthDimension::from_points(&gp::origin(), &gp::origin(), &xoy).unwrap();
    assert!(!same.is_valid());
    assert_eq!(same.get_value(), 0.0);
    let edges = PrsDimLengthDimension::new();
    edges
        .set_measured_shapes(
            &segment(gp::origin(), Point3::new(10.0, 0.0, 0.0)).into(),
            &segment(Point3::new(2.0, 5.0, 0.0), Point3::new(12.0, 5.0, 0.0)).into(),
        )
        .unwrap();
    assert!(edges.is_valid());
    assert_eq!(edges.get_value(), 5.0);
    assert_eq!(edges.get_geometry_type(), PrsDimGeometryType::Edges as i32);
}
