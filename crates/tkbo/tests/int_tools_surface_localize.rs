//! Native sharing, assignment retention, strict frames, and independent cache states.
use rustycad_tkbo::int_tools::{SurfaceRangeLocalizeData as Cache, SurfaceRangeSample};
use rustycad_tkernel::ncollection::HashKey;
use rustycad_tkmath::{bnd::BoundingBox3, gp::Point3};
fn grid() -> Cache {
    let mut cache = Cache::new(2, 3, 0.1, 0.2);
    cache.set_range_u_grid(3).unwrap();
    cache.set_range_v_grid(3).unwrap();
    for index in 1..=3 {
        cache.set_u_param(index, f64::from(index)).unwrap();
        cache.set_v_param(index, -f64::from(index)).unwrap();
        cache
            .set_grid_point(index, index, Point3::new(f64::from(index), 2.0, 3.0))
            .unwrap();
    }
    cache
}
#[test]
fn settings_and_default_frame_match_native() {
    let cache = grid();
    assert_eq!((cache.get_nb_sample_u(), cache.get_nb_sample_v()), (2, 3));
    assert_eq!(
        (cache.get_min_range_u(), cache.get_min_range_v()),
        (0.1, 0.2)
    );
    let mut cache = Cache::default();
    cache.set_frame(0.0, 1.0, 0.0, 1.0);
    assert_eq!(
        (
            cache.get_nb_u_points_in_frame(),
            cache.get_nb_v_points_in_frame()
        ),
        (1, 1)
    );
    assert_eq!(cache.get_point_in_frame(1, 1).unwrap(), Point3::default());
    assert_eq!(cache.get_u_param_in_frame(1).unwrap(), 2e100);
    assert_eq!(cache.get_v_param_in_frame(1).unwrap(), 2e100);
}
#[test]
fn copies_share_grid_but_assignment_from_null_retains_arrays() {
    let cache = grid();
    let mut copy = cache.clone();
    copy.set_u_param(1, 99.0).unwrap();
    copy.set_v_param(1, 98.0).unwrap();
    copy.set_grid_point(1, 1, Point3::default()).unwrap();
    assert_eq!(cache.get_u_param(1).unwrap(), 99.0);
    assert_eq!(cache.get_v_param(1).unwrap(), 98.0);
    assert_eq!(cache.get_grid_point(1, 1).unwrap(), Point3::default());
    copy.assign(&Cache::default());
    assert_eq!((copy.get_range_u_grid(), copy.get_range_v_grid()), (3, 3));
    assert_eq!(copy.get_u_param(1).unwrap(), 99.0);
}
#[test]
fn frame_is_strict_and_can_have_negative_extent() {
    let mut cache = grid();
    cache.set_frame(1.0, 3.0, -4.0, 0.0);
    assert_eq!(cache.get_nb_u_points_in_frame(), 1);
    assert_eq!(cache.get_u_param_in_frame(1).unwrap(), 2.0);
    assert_eq!(cache.get_u_param_in_frame(2).unwrap(), 2e100);
    assert_eq!(
        cache.get_point_in_frame(1, 2).unwrap(),
        Point3::new(2.0, 2.0, 3.0)
    );
    cache.set_frame(4.0, 0.0, -1.0, -3.0);
    assert_eq!(cache.get_nb_u_points_in_frame(), -3);
    assert_eq!(cache.get_nb_v_points_in_frame(), -3);
}
#[test]
fn same_dimension_resets_frame_and_keeps_grid_storage() {
    let mut cache = grid();
    cache.set_frame(0.0, 4.0, -4.0, 0.0);
    cache.set_range_u_grid(3).unwrap();
    cache.set_range_v_grid(3).unwrap();
    assert_eq!(cache.get_nb_u_points_in_frame(), 1);
    assert_eq!(cache.get_u_param(2).unwrap(), 2.0);
    assert_eq!(
        cache.get_grid_point(2, 2).unwrap(),
        Point3::new(2.0, 2.0, 3.0)
    );
    cache.set_grid_deflection(-0.5);
    assert_eq!(cache.get_grid_deflection(), -0.5);
    cache.clear_grid();
    assert_eq!(cache.get_grid_deflection(), 0.0);
    assert_eq!((cache.get_range_u_grid(), cache.get_range_v_grid()), (0, 0));
}
#[test]
fn exclusion_keeps_boxes_and_listing_appends() {
    let mut cache = Cache::default();
    let key = SurfaceRangeSample::default();
    assert_eq!(key.occt_hash(), key.hash_code());
    let mut bounds = BoundingBox3::default();
    bounds.set_whole();
    assert!(!cache.find_box(&key, &mut bounds));
    assert!(bounds.is_whole());
    cache.add_box(&key, &bounds);
    cache.add_out_range(&key);
    assert!(cache.find_box(&key, &mut bounds));
    assert!(cache.is_range_out(&key));
    let mut ranges = vec![key];
    cache.list_range_out(&mut ranges);
    assert_eq!(ranges, vec![key, key]);
    cache.remove_range_out_all();
    assert!(!cache.is_range_out(&key));
    assert!(cache.find_box(&key, &mut bounds));
}
#[test]
fn changing_grid_size_preserves_other_axis_and_reinitializes_points() {
    let mut cache = grid();
    cache.set_range_u_grid(2).unwrap();
    assert_eq!(cache.get_grid_point(1, 1).unwrap(), Point3::default());
    assert_eq!(cache.get_v_param(2).unwrap(), -2.0);
    cache.set_range_v_grid(2).unwrap();
    assert_eq!(cache.get_grid_point(2, 2).unwrap(), Point3::default());
    assert!(cache.get_grid_point(3, 3).is_err());
    assert!(cache.get_u_param(0).is_err());
    assert!(cache.set_v_param(3, 0.0).is_err());
}
