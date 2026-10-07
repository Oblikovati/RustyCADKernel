use super::*;
use crate::chfi_ds::State;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

fn segment(from: Point3, to: Point3) -> Edge {
    BRepBuilderAPIMakeEdge::from_points(from, to)
        .unwrap()
        .edge()
        .unwrap()
}

/// Two segments: (0,0,0)-(1,0,0) then (1,0,0)-(1,2,0); abscissa [1, 3].
fn loaded_spine() -> Spine {
    let mut spine = Spine::new();
    spine.set_edges(&segment(Point3::new(0., 0., 0.), Point3::new(1., 0., 0.)));
    spine.set_edges(&segment(Point3::new(1., 0., 0.), Point3::new(1., 2., 0.)));
    spine.load().unwrap();
    spine
}

fn close(first: Point3, second: Point3) -> bool {
    first.distance(&second) < 1e-12
}

#[test]
fn load_accumulates_the_edge_lengths() {
    let spine = loaded_spine();
    assert_eq!(spine.last_parameter().unwrap(), 3.0);
    assert_eq!(spine.first_parameter_of(2).unwrap(), 1.0);
    assert_eq!(spine.length(2).unwrap(), 2.0);
    assert_eq!(spine.get_tolesp(), rustycad_tkernel::precision::confusion());
    assert_eq!(spine.current_index_of_elementary_spine(), 1);
}

#[test]
fn load_lowers_the_tolerance_to_the_parameter_range() {
    let mut spine = Spine::with_tolerance(1.0);
    spine.set_edges(&segment(Point3::new(0., 0., 0.), Point3::new(2., 0., 0.)));
    spine.load().unwrap();
    assert_eq!(spine.get_tolesp(), 5.0e-5 * 2.0);
}

#[test]
fn abscissa_members_raise_before_load() {
    let spine = Spine::new();
    assert_eq!(
        spine.last_parameter().unwrap_err().exception_type(),
        "Standard_NullObject"
    );
    assert_eq!(spine.first_parameter_of(1).unwrap(), 0.0);
    assert!(spine.edges(1).is_err());
    assert!(spine.first_vertex().is_err());
}

#[test]
fn points_and_tangents_follow_the_abscissa() {
    let mut spine = loaded_spine();
    assert!(close(spine.value(0.5).unwrap(), Point3::new(0.5, 0., 0.)));
    assert!(close(spine.value(2.0).unwrap(), Point3::new(1., 1., 0.)));
    let (point, tangent) = spine.d1(2.0).unwrap();
    assert!(close(point, Point3::new(1., 1., 0.)));
    assert!(
        tangent
            .is_equal(Vector3::new(0., 1., 0.), 1e-12, 1e-12)
            .unwrap()
    );
    let (_, _, second) = spine.d2(0.5).unwrap();
    assert!(second.magnitude() < 1e-12);
}

#[test]
fn index_moves_at_junctions_by_direction() {
    let spine = loaded_spine();
    assert_eq!(spine.index_at(1.0, true).unwrap(), 2);
    assert_eq!(spine.index_at(1.0, false).unwrap(), 1);
    assert_eq!(spine.index_at(0.5, true).unwrap(), 1);
    assert_eq!(spine.index_at(10.0, true).unwrap(), 2);
}

#[test]
fn tangent_extensions_continue_the_ends_linearly() {
    let mut spine = loaded_spine();
    spine.set_first_tgt(0.0).unwrap();
    spine.set_last_tgt(3.0).unwrap();
    assert!(spine.has_first_tgt() && spine.has_last_tgt());
    assert!(close(spine.value(-1.0).unwrap(), Point3::new(-1., 0., 0.)));
    assert!(close(spine.value(4.0).unwrap(), Point3::new(1., 3., 0.)));
    let (_, tangent, second) = spine.d2(-1.0).unwrap();
    assert_eq!(second, Vector3::new(0., 0., 0.));
    assert!(
        tangent
            .is_equal(Vector3::new(1., 0., 0.), 1e-12, 1e-12)
            .unwrap()
    );
}

#[test]
fn periodic_queries_require_a_closed_first_state() {
    let mut spine = loaded_spine();
    assert_eq!(spine.period().unwrap_err().message(), "Non-periodic Spine");
    spine.set_first_status(State::Closed);
    assert_eq!(spine.period().unwrap(), 3.0);
    assert!(spine.set_first_tgt(0.0).is_err());
    assert_eq!(spine.index_at(3.0, true).unwrap(), 1);
}

#[test]
fn reversed_edges_mirror_parameters_and_tangents() {
    let mut spine = Spine::new();
    let mut reversed = segment(Point3::new(2., 0., 0.), Point3::new(0., 0., 0.));
    reversed.reverse();
    spine.set_edges(&reversed);
    spine.load().unwrap();
    let u = spine.parameter(0.5, true).unwrap();
    let raw = spine.parameter(0.5, false).unwrap();
    assert!((u + raw - 2.0).abs() < 1e-12);
    assert!(close(spine.value(0.5).unwrap(), Point3::new(0.5, 0., 0.)));
    let (_, tangent) = spine.d1(0.5).unwrap();
    assert!(
        tangent
            .is_equal(Vector3::new(1., 0., 0.), 1e-12, 1e-12)
            .unwrap()
    );
    assert!((spine.absc_on_edge(raw, 1).unwrap() - 0.5).abs() < 1e-12);
    let line = spine.line().unwrap();
    assert!(close(line.location(), Point3::new(0., 0., 0.)));
}

#[test]
fn vertices_and_edges_are_found_in_guideline_order() {
    let spine = loaded_spine();
    let first = spine.first_vertex().unwrap();
    let second_edge = spine.edges(2).unwrap().clone();
    assert_eq!(spine.absc_of_vertex(&first).unwrap(), 0.0);
    assert_eq!(spine.index_of_edge(&second_edge), 2);
    assert!(!spine.is_closed().unwrap());
    assert_eq!(spine.absc_of_vertex(&Default::default()).unwrap(), -1.0);
}

#[test]
fn reset_with_all_data_drops_the_extensions() {
    let mut spine = loaded_spine();
    spine.set_first_parameter(-1.0);
    spine.set_last_parameter(5.0);
    SpineObject::reset(&mut spine, false).unwrap();
    assert_eq!(spine.first_parameter(), -1.0);
    SpineObject::reset(&mut spine, true).unwrap();
    assert_eq!(spine.first_parameter(), 0.0);
    assert_eq!(spine.last_parameter().unwrap(), 3.0);
}

#[test]
fn reference_moves_the_junction_toward_it() {
    let mut spine = loaded_spine();
    spine.set_reference_on_edge(2).unwrap();
    let (point, _) = spine.d1(1.0).unwrap();
    assert!(close(point, Point3::new(1., 0., 0.)));
    spine.set_reference(-1.0).unwrap();
    spine.unset_reference();
    assert_eq!(spine.resolution(0.25), 0.25);
}
