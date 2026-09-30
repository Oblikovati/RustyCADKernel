//! State regressions for IntTools_EdgeEdge at OCCT 3d097a0.
use rustycad_tkbo::int_tools::{EdgeEdge, Range};
use rustycad_tkbrep::{brep::BRepBuilder, topods::Edge};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::{
    geom::{GeomCurve, GeomLine},
    top_abs::ShapeEnum,
};
use rustycad_tkmath::gp::{Direction3, Point3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
fn axis_edge(direction: Direction3) -> Result<Edge, Failure> {
    let curve: Handle<dyn GeomCurve> =
        Handle::new(GeomLine::from_point(Point3::default(), direction)).into_cast();
    BRepBuilderAPIMakeEdge::from_curve_params(&curve, -1., 1.)?.edge()
}
#[test]
fn empty_status_is_done_until_validation_and_a_valid_edge_does_not_reset_error()
-> Result<(), Failure> {
    let mut intersection = EdgeEdge::new();
    assert!(intersection.is_done());
    assert!(intersection.common_parts().is_empty());
    intersection.perform()?;
    assert!(!intersection.is_done());
    let edge = axis_edge(Direction3::X)?;
    intersection.set_edge1(&edge);
    intersection.set_edge2(&edge);
    intersection.perform()?;
    assert!(!intersection.is_done());
    assert!(intersection.common_parts().is_empty());
    Ok(())
}
#[test]
fn repeated_collinear_intersection_appends_copy_constructed_common_parts() -> Result<(), Failure> {
    let first = axis_edge(Direction3::X)?;
    let second = axis_edge(Direction3::X)?;
    let mut intersection = EdgeEdge::with_edges(&first, &second);
    for count in 1..=2 {
        intersection.perform()?;
        assert!(intersection.is_done());
        assert_eq!(intersection.common_parts().len(), count);
        let part = &intersection.common_parts()[count - 1];
        assert_eq!(part.shape_type(), ShapeEnum::Edge);
        assert!(!part.all_null_flag());
        assert_eq!(part.range1().range(), (-1., 1.));
        assert!(part.edge1().is_same(&first));
        assert!(part.edge2().is_same(&second));
    }
    Ok(())
}
#[test]
fn crossing_uses_vertex_ranges_and_retains_results_after_later_bad_input() -> Result<(), Failure> {
    let first = axis_edge(Direction3::X)?;
    let second = axis_edge(Direction3::Y)?;
    let mut intersection = EdgeEdge::with_ranges(&first, -1., 1., &second, -1., 1.);
    intersection.perform()?;
    assert_eq!(intersection.common_parts().len(), 1);
    assert_eq!(
        intersection.common_parts()[0].shape_type(),
        ShapeEnum::Vertex
    );
    assert_eq!(intersection.common_parts()[0].vertex_parameter1(), 0.);
    intersection.set_edge2(&Edge::default());
    intersection.perform()?;
    assert!(!intersection.is_done());
    assert_eq!(intersection.common_parts().len(), 1);
    Ok(())
}
#[test]
fn fuzzy_maximum_keeps_nan_order_and_setters_preserve_status() {
    let mut intersection = EdgeEdge::new();
    intersection.set_fuzzy_value(-1.);
    assert_eq!(intersection.fuzzy_value(), precision::confusion());
    intersection.set_fuzzy_value(f64::NAN);
    assert!(intersection.fuzzy_value().is_nan());
    intersection.use_quick_coincidence_check(true);
    assert!(intersection.is_coincidence_checked_quickly());
    intersection.set_range1(Range::new(2., 3.));
    intersection.set_range2(Range::new(4., 5.));
    intersection.set_range2_bounds(4., 5.);
    assert!(intersection.is_done());
}
#[test]
fn degenerate_and_nongeometric_validation_errors_survive_edge_replacement() -> Result<(), Failure> {
    let valid = axis_edge(Direction3::X)?;
    for degenerate in [false, true] {
        let mut invalid = Edge::default();
        BRepBuilder::default().make_edge(&mut invalid)?;
        BRepBuilder::default().degenerated(&invalid, degenerate)?;
        let mut intersection = EdgeEdge::with_edges(&invalid, &valid);
        intersection.perform()?;
        assert!(!intersection.is_done());
        intersection.set_edge1_range(&valid, -1., 1.);
        intersection.set_edge2_range(&valid, -1., 1.);
        intersection.perform()?;
        assert!(!intersection.is_done());
    }
    Ok(())
}
