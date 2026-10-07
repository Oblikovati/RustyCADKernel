use super::*;
use rustycad_tkernel::standard::RuntimeTyped;

#[test]
fn copy_skips_the_curve_indices() {
    let mut source = SurfData::new();
    source.set_index_of_c1(4);
    source.change_index_of_s1(7);
    source.change_surf(9);
    source.set_twist_on_s2(true);
    let mut target = SurfData::new();
    target.copy_from(&source);
    assert!(!target.is_on_curve1());
    assert_eq!((target.index_of_s1(), target.surf()), (7, 9));
    assert!(target.twist_on_s2());
}

#[test]
fn curve_index_raises_standard_failure_when_not_on_curve() {
    let data = SurfData::new();
    let failure = data.index_of_c(2).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_Failure");
    assert_eq!(failure.message(), "Interference pas sur courbe");
}

#[test]
fn vertex_selection_follows_the_native_fallthrough() {
    let mut data = SurfData::new();
    data.change_vertex(false, 2).set_parameter(4.0);
    data.change_vertex(true, 2).set_parameter(2.0);
    assert_eq!(data.vertex(true, 3).parameter(), 4.0);
    assert_eq!(data.vertex(true, 2).parameter(), 2.0);
    assert_eq!(data.vertex_last_on_s2().parameter(), 4.0);
}

#[test]
fn two_d_points_follow_the_native_fallthrough() {
    let mut data = SurfData::new();
    let points = [
        Point2::new(1., 0.),
        Point2::new(2., 0.),
        Point2::new(3., 0.),
        Point2::new(4., 0.),
    ];
    data.set_2d_points(points);
    assert_eq!(data.get_2d_points(), points);
    assert_eq!(data.get_2d_point(true, 2).x(), 3.0);
    assert_eq!(data.get_2d_point(true, 3).x(), 4.0);
    assert_eq!(data.get_2d_point(false, 1).x(), 2.0);
}

#[test]
fn interference_and_index_select_the_second_face_by_default() {
    let mut data = SurfData::new();
    data.change_index_of_s2(5);
    data.change_interference(0).set_line_index(6);
    assert_eq!(data.index(0), 5);
    assert_eq!(data.interference_on_s2().line_index(), 6);
    assert_eq!(data.interference(1).line_index(), 0);
}

#[test]
fn spine_range_extension_and_simul_round_trip() {
    let mut data = SurfData::new();
    data.set_first_spine_param(0.5);
    data.set_last_spine_param(1.5);
    data.set_first_extension_value(0.1);
    data.set_last_extension_value(0.2);
    *data.change_orientation() = Orientation::Reversed;
    let simul = Handle::new(StandardTransient::default());
    data.set_simul(&simul);
    assert_eq!(
        (data.first_spine_param(), data.last_spine_param()),
        (0.5, 1.5)
    );
    assert_eq!(
        (data.first_extension_value(), data.last_extension_value()),
        (0.1, 0.2)
    );
    assert_eq!(data.orientation(), Orientation::Reversed);
    assert!(data.simul().is_some());
    data.reset_simul();
    assert!(data.simul().is_null());
}

#[test]
fn registered_with_its_native_name() {
    assert_eq!(SurfData::static_type().name(), "ChFiDS_SurfData");
}
