//! Native sample counts and the concrete adaptor forwarding contract.
use super::{Tool, fixtures};
use rustycad_tkmath::{geom_abs::SurfaceType, gp::Point3};
const SAMPLE_COUNTS: [(i32, i32, i32); 12] = [
    (0, 2, 2),
    (1, 10, 15),
    (2, 10, 15),
    (3, 10, 15),
    (4, 20, 15),
    (5, 12, 12),
    (6, 12, 12),
    (7, 21, 21),
    (8, 21, 21),
    (9, 10, 15),
    (10, 10, 15),
    (11, 10, 10),
];
#[test]
fn sampling_counts_follow_surface_kind() {
    for (kind, u, v) in SAMPLE_COUNTS {
        let s = fixtures::surface(kind, 0.).unwrap();
        assert_eq!(Tool::nb_samples_u(&s).unwrap(), u);
        assert_eq!(Tool::nb_samples_v(&s).unwrap(), v);
    }
}
#[test]
fn forward_and_reversed_ranges_retain_native_sign() {
    let s = fixtures::surface(7, 0.).unwrap();
    assert_eq!(Tool::nb_samples_u_range(&s, 0., 6.).unwrap(), 5);
    assert_eq!(Tool::nb_samples_v_range(&s, 6., 0.).unwrap(), 21);
    let plane = fixtures::surface(0, 0.).unwrap();
    assert_eq!(Tool::nb_samples_u_range(&plane, 1., -2.).unwrap(), 2);
    assert_eq!(Tool::nb_samples_v_range(&plane, 1., -2.).unwrap(), 2);
}
#[test]
fn located_plane_evaluations_and_wrong_kind_errors() {
    let s = fixtures::surface(0, 2.).unwrap();
    assert_eq!(Tool::get_type(&s).unwrap(), SurfaceType::Plane);
    assert_eq!(
        Tool::value(&s, 0.5, 0.25).unwrap(),
        Point3::new(2.5, 4.25, -2.)
    );
    assert_eq!(
        Tool::d0(&s, 0.5, 0.25).unwrap(),
        Tool::d3(&s, 0.5, 0.25).unwrap().point
    );
    assert_eq!(
        Tool::bezier(&s).err().unwrap().exception_type(),
        "Standard_NoSuchObject"
    );
}
#[test]
fn all_declarations_and_fixture_helpers_execute() {
    for kind in 0..12 {
        let s = fixtures::surface(kind, 2.).unwrap();
        assert!(Tool::value(&s, 0.3, 0.7).is_ok());
        super::case(&[f64::from(kind), 2., 0.3, 0.7, 0., 1.]);
    }
}
