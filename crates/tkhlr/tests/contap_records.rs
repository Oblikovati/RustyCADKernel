//! Focused contracts supplement the native differential record corpus.
use rustycad_tkernel::standard::Handle;
use rustycad_tkgeomalgo::int_surf::{
    IntSurfLineOn2S, IntSurfPntOn2S, IntSurfTransition, IntSurfTypeTrans,
};
use rustycad_tkhlr::contap::{ContapIType, ContapLine, ContapPoint, ContapTFunction};
use rustycad_tkmath::gp::{Axis2, Circle3, Direction3, Line3, Point3};

#[test]
fn contour_enumerators_match_native_values() {
    assert_eq!(
        [
            ContapIType::Lin as i32,
            ContapIType::Circle as i32,
            ContapIType::Walking as i32,
            ContapIType::Restriction as i32
        ],
        [0, 1, 2, 3]
    );
}
#[test]
fn contour_function_enumerators_match_native_values() {
    assert_eq!(
        [
            ContapTFunction::ContourStd as i32,
            ContapTFunction::ContourPrs as i32,
            ContapTFunction::DraftStd as i32,
            ContapTFunction::DraftPrs as i32
        ],
        [0, 1, 2, 3]
    );
}
#[test]
fn point_flags_do_not_depend_on_null_handles() {
    let mut point = ContapPoint::new();
    let transition = IntSurfTransition::with_type(true, IntSurfTypeTrans::Out);
    point.set_arc(&Handle::default(), 2.5, transition, transition);
    point.set_vertex(&Handle::default());
    assert!(point.is_on_arc() && point.is_vertex());
    assert!(point.arc().unwrap().is_null() && point.vertex().unwrap().is_null());
    assert_eq!(point.parameter_on_arc().unwrap(), 2.5);
    assert_eq!(point.transition_on_line().unwrap(), transition);
    assert_eq!(point.transition_on_arc().unwrap(), transition);
}
#[test]
fn set_value_resets_flags_but_retains_line_parameter() {
    let mut point = flagged_point();
    let position = Point3::new(1.0, 2.0, 3.0);
    point.set_value(position, 4.0, 5.0);
    assert_eq!(point.value(), position);
    assert_eq!(point.parameters(), (4.0, 5.0));
    assert_eq!(point.parameter_on_line(), 12.0);
    assert!(
        !point.is_multiple() && !point.is_internal() && !point.is_on_arc() && !point.is_vertex()
    );
    assert_point_domains(&point);
}
fn ordered_line() -> ContapLine {
    let line = ContapLine::default();
    for (identity, parameter) in [3.0, 1.0, 1.0, 2.0].into_iter().enumerate() {
        let mut vertex = ContapPoint::with_value(Point3::new(identity as f64, 0.0, 0.0), 0.0, 0.0);
        vertex.set_parameter(parameter);
        line.add_vertex(&vertex);
    }
    line
}
#[test]
fn equal_parameters_precede_existing_vertices_and_share_edits() {
    let line = ordered_line();
    let copy = line.clone();
    let identities: Vec<f64> = (1..=line.nb_vertex())
        .map(|i| line.vertex(i).unwrap().value().x())
        .collect();
    assert_eq!(identities, [2.0, 1.0, 3.0, 0.0]);
    copy.change_vertex(1, ContapPoint::set_multiple).unwrap();
    assert!(line.vertex(1).unwrap().is_multiple());
    for index in [-1, 0, 5] {
        assert_eq!(
            line.vertex(index).err().unwrap().exception_type(),
            "Standard_OutOfRange"
        );
    }
}
#[test]
fn reset_detaches_vertices_and_clear_mutates_shared_walking_points() {
    let mut line = ordered_line();
    let points = Handle::new(IntSurfLineOn2S::new(None));
    line.set_line_on_2s(points.clone());
    line.add_point(&IntSurfPntOn2S::new()).unwrap();
    let mut copy = line.clone();
    assert_eq!(copy.nb_pnts().unwrap(), 1);
    assert_eq!(copy.point(1).unwrap().value(), Point3::default());
    copy.reset_seq_of_vertex();
    assert_eq!(copy.nb_vertex(), 0);
    assert_eq!(line.nb_vertex(), 4);
    copy.clear();
    assert_eq!(points.nb_points(), 0);
    assert_eq!(line.nb_vertex(), 4);
    assert_eq!(copy.type_contour(), ContapIType::Walking);
}
#[test]
fn geometry_switches_enforce_accessor_domains() {
    let mut line = ContapLine::new();
    let straight = Line3::new(Point3::default(), Direction3::X);
    line.set_line(straight);
    assert_eq!(line.line().unwrap().direction(), Direction3::X);
    assert!(line.circle().is_err() && line.arc().is_err());
    assert!(line.nb_pnts().is_err() && line.point(1).is_err());
    check_alternate_geometry(&mut line);
    check_null_walking(&mut line);
}
fn check_alternate_geometry(line: &mut ContapLine) {
    line.set_circle(Circle3::new(Axis2::default(), 5.0).unwrap());
    assert_eq!(line.circle().unwrap().radius(), 5.0);
    line.set_arc(&Handle::default());
    assert!(line.arc().unwrap().is_null());
    assert!(line.line().is_err());
    line.set_transition_on_s(IntSurfTypeTrans::Touch);
    assert_eq!(line.transition_on_s(), IntSurfTypeTrans::Touch);
}
fn check_null_walking(line: &mut ContapLine) {
    assert_eq!(
        line.add_point(&IntSurfPntOn2S::new())
            .err()
            .unwrap()
            .exception_type(),
        "Standard_NullObject"
    );
    line.clear();
    assert!(line.line_on_2s().is_null());
    assert!(line.nb_pnts().is_err() && line.point(1).is_err());
}

fn flagged_point() -> ContapPoint {
    let mut point = ContapPoint::new();
    point.set_parameter(12.0);
    point.set_multiple();
    point.set_internal();
    point.set_arc(
        &Handle::default(),
        1.0,
        IntSurfTransition::new(),
        IntSurfTransition::new(),
    );
    point.set_vertex(&Handle::default());
    point
}

fn assert_point_domains(point: &ContapPoint) {
    assert_eq!(
        point.arc().err().unwrap().exception_type(),
        "Standard_DomainError"
    );
    assert_eq!(
        point.vertex().err().unwrap().exception_type(),
        "Standard_DomainError"
    );
    assert!(
        point.parameter_on_arc().is_err()
            && point.transition_on_line().is_err()
            && point.transition_on_arc().is_err()
    );
}
