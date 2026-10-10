//! Adaptive counts and ranking branch order, independent of oracle fixtures.
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkv3d::select_mgr::{SelectMgrSortCriterion, SelectMgrToleranceMap};
#[test]
fn tolerance_defaults_and_custom_offset_are_distinct() {
    let mut map = SelectMgrToleranceMap::new();
    assert_eq!(
        (
            map.tolerance(),
            map.custom_tolerance(),
            map.is_custom_tol_set()
        ),
        (2, -1, false)
    );
    map.set_custom_tolerance(9);
    assert_eq!(map.tolerance(), 2);
    map.add(4);
    assert_eq!(map.tolerance(), 13);
    map.set_custom_tolerance(0);
    assert_eq!(map.tolerance(), 4);
    assert!(!map.is_custom_tol_set());
    map.reset_defaults();
    assert_eq!((map.tolerance(), map.custom_tolerance()), (4, -1));
}
#[test]
fn tolerance_frequencies_preserve_inactive_keys_and_reactivation() {
    let mut map = SelectMgrToleranceMap::default();
    for key in [4, 8, 8, 2] {
        map.add(key);
    }
    map.decrement(8).unwrap();
    assert_eq!(map.tolerance(), 8);
    map.decrement(8).unwrap();
    assert_eq!(map.tolerance(), 4);
    let failure = map.decrement(8).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ProgramError");
    assert!(failure.to_string().contains("8"));
    assert_eq!(map.tolerance(), 4);
    map.add(8);
    assert_eq!(map.tolerance(), 8);
    map.decrement(99).unwrap();
    assert_eq!(map.tolerance(), 8);
}
#[test]
fn negative_and_zero_tolerances_keep_the_native_default_rule() {
    let mut map = SelectMgrToleranceMap::new();
    map.add(-5);
    map.add(-4);
    assert_eq!(map.tolerance(), 2);
    map.decrement(-4).unwrap();
    map.add(0);
    assert_eq!(map.tolerance(), 0);
    map.set_custom_tolerance(3);
    assert_eq!(map.tolerance(), 3);
    map.decrement(0).unwrap();
    assert_eq!(map.tolerance(), 2);
    map.add(-4);
    assert_eq!(map.tolerance(), 2);
}
#[test]
fn ranking_defaults_clone_public_fields_and_ignore_preference_flag() {
    let mut a = SelectMgrSortCriterion::new();
    let b = a.clone();
    assert!(a.entity.is_null());
    assert_eq!(a.point.x(), 0.);
    assert_eq!(a.normal, Vec3::new(0., 0., 0.));
    assert_eq!((a.depth, a.min_dist, a.tolerance), (0., 0., 0.));
    assert_eq!(
        (
            a.selection_priority,
            a.display_priority,
            a.z_layer_position,
            a.nb_owner_matches
        ),
        (0, 0, 0, 0)
    );
    assert!(!a.is_closer_depth(&b).unwrap() && !a.is_higher_priority(&b));
    a.is_prefer_priority = true;
    a.nb_owner_matches = 100;
    assert!(!a.is_closer_depth(&b).unwrap() && !a.is_higher_priority(&b));
}
#[test]
fn layer_and_raw_depth_precede_normal_and_priority() {
    let mut a = SelectMgrSortCriterion {
        depth: 100.,
        z_layer_position: 1,
        ..Default::default()
    };
    let b = SelectMgrSortCriterion {
        selection_priority: 100,
        ..Default::default()
    };
    assert!(a.is_closer_depth(&b).unwrap() && a.is_higher_priority(&b));
    a.z_layer_position = 0;
    assert!(!a.is_closer_depth(&b).unwrap());
    a.depth = -100.;
    assert!(a.is_closer_depth(&b).unwrap());
    assert!(!a.is_higher_priority(&b));
}
#[test]
fn tolerance_adjusted_depth_depends_on_surface_angle() {
    let mut a = SelectMgrSortCriterion {
        depth: 5.,
        tolerance: 2.,
        normal: Vec3::new(0., 0., 1.),
        ..Default::default()
    };
    let b = SelectMgrSortCriterion {
        depth: 4.,
        tolerance: 2.,
        normal: Vec3::new(0., 0., 1.),
        selection_priority: 1,
        ..Default::default()
    };
    assert!(!a.is_closer_depth(&b).unwrap());
    a.tolerance = 4.;
    assert!(a.is_closer_depth(&b).unwrap());
    a.normal = Vec3::new(1., 0., 0.);
    assert!(!a.is_closer_depth(&b).unwrap());
    a.normal = Vec3::new(0., 0., 0.);
    assert!(a.is_closer_depth(&b).unwrap());
}
#[test]
fn depth_priority_fallback_is_not_a_lexicographic_order() {
    let a = SelectMgrSortCriterion {
        selection_priority: 1,
        display_priority: 10,
        ..Default::default()
    };
    let b = SelectMgrSortCriterion {
        selection_priority: 2,
        display_priority: 0,
        ..Default::default()
    };
    assert!(a.is_closer_depth(&b).unwrap() && b.is_closer_depth(&a).unwrap());
    assert!(!a.is_higher_priority(&b) && b.is_higher_priority(&a));
}
#[test]
fn older_priority_uses_confusion_before_mouse_distance() {
    let mut a = SelectMgrSortCriterion {
        depth: 0.00000005,
        min_dist: 1.,
        ..Default::default()
    };
    let b = SelectMgrSortCriterion {
        depth: 0.,
        min_dist: 2.,
        ..Default::default()
    };
    assert!(a.is_higher_priority(&b));
    a.depth = 0.0000002;
    assert!(!a.is_higher_priority(&b));
    a.display_priority = 1;
    assert!(a.is_higher_priority(&b));
    a.display_priority = 0;
    a.depth = 0.;
    assert!(a.is_closer_depth(&b).unwrap());
}
